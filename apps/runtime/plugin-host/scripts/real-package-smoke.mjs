import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

const workspaceRuntimeDir = path.resolve(process.cwd(), "..");
const pluginHostDir = path.resolve(workspaceRuntimeDir, "plugin-host");
const unpackedPackageSourceRoot = path.join(
  process.env.TEMP ?? "",
  "workclaw-openclaw-lark-inspect",
  "package",
);
const localFixtureRoot = path.join(
  workspaceRuntimeDir,
  ".workclaw-plugin-host-fixtures",
  "openclaw-lark-package",
);
const shimPluginSdkRoot = path.join(pluginHostDir, "openclaw", "plugin-sdk");
const shimPluginSdkCjsRoot = path.join(pluginHostDir, "openclaw", "plugin-sdk-cjs");

function createPluginRegistry() {
  return {
    channels: [],
    tools: [],
    cliEntries: [],
    commands: [],
    gatewayMethods: {},
    hooks: {
      before_tool_call: [],
      after_tool_call: [],
    },
  };
}

function createPluginRuntime(config) {
  const records = [];

  function createLogger(scope) {
    return {
      debug: (...args) => records.push({ level: "debug", scope, args }),
      info: (...args) => records.push({ level: "info", scope, args }),
      warn: (...args) => records.push({ level: "warn", scope, args }),
      error: (...args) => records.push({ level: "error", scope, args }),
    };
  }

  return {
    config: {
      loadConfig: async () => config,
    },
    channel: {
      text: {
        chunkMarkdownText(text, limit) {
          if (limit <= 0) {
            return [text];
          }
          const chunks = [];
          for (let index = 0; index < text.length; index += limit) {
            chunks.push(text.slice(index, index + limit));
          }
          return chunks;
        },
        convertMarkdownTables(text) {
          return text;
        },
      },
    },
    logging: {
      records,
      getChildLogger({ scope }) {
        return createLogger(scope);
      },
    },
    log(...args) {
      records.push({ level: "info", scope: "runtime", args });
    },
    error(...args) {
      records.push({ level: "error", scope: "runtime", args });
    },
  };
}

function createPluginApi(registry, { runtime, logger, config, registrationMode }) {
  return {
    runtime,
    logger,
    config,
    registrationMode,
    registerChannel(input) {
      registry.channels.push(input.plugin);
    },
    registerTool(tool) {
      registry.tools.push(tool);
    },
    registerCli(cliEntry, registration) {
      if (typeof cliEntry === "function") {
        cliEntry({
          program: {
            commands: [],
            command() {
              const chain = {
                description() {
                  return chain;
                },
                option() {
                  return chain;
                },
                action() {
                  return chain;
                },
              };
              return chain;
            },
          },
          config,
          logger,
        });
      }
      registry.cliEntries.push({ entry: cliEntry, registration });
    },
    registerGatewayMethod(name, handler) {
      registry.gatewayMethods[name] = handler;
    },
    registerCommand(command) {
      registry.commands.push(command);
    },
    on(eventName, handler) {
      registry.hooks[eventName].push(handler);
    },
  };
}

function resolvePluginExport(loadedModule) {
  const candidates = [loadedModule, loadedModule?.default, loadedModule?.default?.default];
  for (const candidate of candidates) {
    if (candidate && typeof candidate.register === "function") {
      return candidate;
    }
  }
  return loadedModule?.default ?? loadedModule;
}

function rewritePluginSdkImportsInFixture(rootDir) {
  const stack = [rootDir];

  while (stack.length > 0) {
    const currentDir = stack.pop();
    if (!currentDir) {
      continue;
    }

    for (const entry of fs.readdirSync(currentDir, { withFileTypes: true })) {
      const entryPath = path.join(currentDir, entry.name);
      if (entry.isDirectory()) {
        stack.push(entryPath);
        continue;
      }

      if (!entry.isFile() || !entry.name.endsWith(".js")) {
        continue;
      }

      const relativeShimRoot = path
        .relative(path.dirname(entryPath), shimPluginSdkRoot)
        .replace(/\\/g, "/");
      const relativeShimCjsRoot = path
        .relative(path.dirname(entryPath), shimPluginSdkCjsRoot)
        .replace(/\\/g, "/");
      const normalizeRelativeImport = (rawSpecifier) => {
        const resolvedPath = path.resolve(path.dirname(entryPath), rawSpecifier);
        const fileCandidate = `${resolvedPath}.js`;
        const indexCandidate = path.join(resolvedPath, "index.js");
        let normalizedTarget = rawSpecifier;

        if (!path.extname(rawSpecifier)) {
          if (fs.existsSync(fileCandidate)) {
            normalizedTarget = `${rawSpecifier}.js`;
          } else if (fs.existsSync(indexCandidate)) {
            normalizedTarget = `${rawSpecifier}/index.js`;
          }
        }

        return normalizedTarget.replace(/\\/g, "/");
      };
      const rewritten = fs
        .readFileSync(entryPath, "utf8")
        .replaceAll(
          /require\((['"])openclaw\/plugin-sdk(?:\/[^'"]+)?\1\)/g,
          (_match, quote) => `require(${quote}${relativeShimCjsRoot}/index.cjs${quote})`,
        )
        .replaceAll(
          /from\s+(['"])openclaw\/plugin-sdk(?:\/[^'"]+)?\1/g,
          (_match, quote) => `from ${quote}${relativeShimRoot}/index.js${quote}`,
        )
        .replaceAll(
          /import\s+(['"])openclaw\/plugin-sdk(?:\/[^'"]+)?\1/g,
          (_match, quote) => `import ${quote}${relativeShimRoot}/index.js${quote}`,
        )
        .replaceAll(
          /from\s+(['"])(\.\.?\/[^'"]+)\1/g,
          (_match, quote, specifier) =>
            `from ${quote}${normalizeRelativeImport(specifier)}${quote}`,
        )
        .replaceAll(
          /import\s+(['"])(\.\.?\/[^'"]+)\1/g,
          (_match, quote, specifier) =>
            `import ${quote}${normalizeRelativeImport(specifier)}${quote}`,
        );
      const needsImportMetaCompat =
        rewritten.includes("import.meta.url") &&
        ["module.exports", "exports.", "Object.defineProperty(exports"].some((marker) =>
          rewritten.includes(marker),
        );
      const normalized = needsImportMetaCompat
        ? rewritten
            .replaceAll(
              /const __filename = .*?import\.meta\.url.*?;/g,
              "const __filenameCompat = __filename;",
            )
            .replaceAll("dirname(__filename)", "dirname(__filenameCompat)")
            .replaceAll(".dirname)(__filename)", ".dirname)(__filenameCompat)")
        : rewritten;
      fs.writeFileSync(entryPath, normalized, "utf8");
    }
  }
}

function ensureFixture() {
  if (!process.env.TEMP || !fs.existsSync(unpackedPackageSourceRoot)) {
    throw new Error("published package fixture is missing under TEMP");
  }

  fs.rmSync(localFixtureRoot, { recursive: true, force: true });
  fs.mkdirSync(path.dirname(localFixtureRoot), { recursive: true });
  fs.cpSync(unpackedPackageSourceRoot, localFixtureRoot, { recursive: true });
  rewritePluginSdkImportsInFixture(localFixtureRoot);
  return localFixtureRoot;
}

async function main() {
  const fixtureRoot = ensureFixture();
  const config = {
    channels: {
      feishu: {
        enabled: true,
        accounts: {
          default: {
            appId: "demo-app",
            appSecret: "demo-secret",
            enabled: true,
          },
        },
      },
    },
    tools: {
      profile: "default",
    },
    plugins: {
      entries: {},
    },
  };
  const registry = createPluginRegistry();
  const runtime = createPluginRuntime(config);
  const logger = runtime.logging.getChildLogger({ scope: "plugin-host-smoke" });
  const api = createPluginApi(registry, {
    runtime,
    logger,
    config,
    registrationMode: "full",
  });

  const entryPath = path.join(fixtureRoot, "index.js");
  const loadedModule = await import(pathToFileURL(entryPath).href);
  const plugin = resolvePluginExport(loadedModule);

  if (typeof plugin.register !== "function") {
    throw new Error("plugin module must export a register(api) function");
  }

  await plugin.register(api);

  console.log(
    JSON.stringify(
      {
        entryPath,
        channels: registry.channels.length,
        tools: registry.tools.length,
        commands: registry.commands.length,
        cliEntries: registry.cliEntries.length,
        gatewayMethods: Object.keys(registry.gatewayMethods),
        hookCounts: {
          before_tool_call: registry.hooks.before_tool_call.length,
          after_tool_call: registry.hooks.after_tool_call.length,
        },
        logRecords: runtime.logging.records.length,
      },
      null,
      2,
    ),
  );
}

main().catch((error) => {
  console.error("[real-package-smoke] failed");
  console.error(error);
  process.exitCode = 1;
});
