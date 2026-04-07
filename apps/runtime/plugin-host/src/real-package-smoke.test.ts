import fs from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { createPluginApi } from "./api";
import { createPluginRegistry } from "./registry";
import { createPluginRuntime } from "./runtime";
import { loadPluginModule } from "./loader";

async function expectStageToResolve<T>(label: string, promise: Promise<T>, timeoutMs = 20000): Promise<T> {
  return await Promise.race([
    promise,
    new Promise<T>((_resolve, reject) => {
      setTimeout(() => reject(new Error(`${label} timed out after ${timeoutMs}ms`)), timeoutMs);
    }),
  ]);
}

const realPackageSmoke = process.env.WORKCLAW_REAL_OPENCLAW_PLUGIN_SMOKE === "1" ? it : it.skip;

const unpackedPackageSourceRoot = path.join(
  process.env.TEMP ?? "",
  "workclaw-openclaw-lark-inspect",
  "package",
);
const localFixtureRoot = path.join(
  process.cwd(),
  ".workclaw-plugin-host-fixtures",
  "openclaw-lark-package",
);
const shimPluginSdkRoot = path.join(
  process.cwd(),
  "plugin-host",
  "openclaw",
  "plugin-sdk",
);
const shimPluginSdkCjsRoot = path.join(
  process.cwd(),
  "plugin-host",
  "openclaw",
  "plugin-sdk-cjs",
);

function rewritePluginSdkImportsInFixture(rootDir: string): void {
  const stack = [rootDir];
  const importMetaCompatBinding =
    "const __workclawImportMetaUrl = require('node:url').pathToFileURL(__filename).href;";

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
        );
      const needsImportMetaCompat =
        rewritten.includes("import.meta.url") &&
        ["module.exports", "exports.", "Object.defineProperty(exports"].some((marker) =>
          rewritten.includes(marker),
        );
      let normalized = needsImportMetaCompat
        ? rewritten
            .replaceAll(
              /const __filename = .*?import\.meta\.url.*?;/g,
              "const __filenameCompat = __filename;",
            )
            .replaceAll("dirname(__filename)", "dirname(__filenameCompat)")
            .replaceAll(".dirname)(__filename)", ".dirname)(__filenameCompat)")
            .replaceAll("import.meta.url", "__workclawImportMetaUrl")
        : rewritten;
      if (
        needsImportMetaCompat &&
        normalized.includes("__workclawImportMetaUrl") &&
        !normalized.includes(importMetaCompatBinding)
      ) {
        const strictDirectiveMatch = normalized.match(/^(["'])use strict\1;\r?\n?/);
        normalized = strictDirectiveMatch
          ? `${strictDirectiveMatch[0]}${importMetaCompatBinding}\n${normalized.slice(strictDirectiveMatch[0].length)}`
          : `${importMetaCompatBinding}\n${normalized}`;
      }
      fs.writeFileSync(entryPath, normalized, "utf8");
    }
  }
}

describe("real package smoke", () => {
  realPackageSmoke("loads the published @larksuite/openclaw-lark package and executes register()", async () => {
    if (!process.env.TEMP || !fs.existsSync(unpackedPackageSourceRoot)) {
      throw new Error("published package fixture is missing under TEMP");
    }

    fs.rmSync(localFixtureRoot, { recursive: true, force: true });
    fs.mkdirSync(path.dirname(localFixtureRoot), { recursive: true });
    fs.cpSync(unpackedPackageSourceRoot, localFixtureRoot, { recursive: true });
    rewritePluginSdkImportsInFixture(localFixtureRoot);

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
    const shimPackageUrl = "file:///D:/code/WorkClaw/apps/runtime/plugin-host/openclaw/package.json";
    const registry = createPluginRegistry();
    const runtime = createPluginRuntime({ config });
    const logger = runtime.logging.getChildLogger({ scope: "plugin-host-test" });
    const api = createPluginApi(registry, {
      runtime,
      logger,
      config,
      registrationMode: "full",
      createCliContext() {
        return {
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
        };
      },
    });

    const loaded = await expectStageToResolve(
      "loadPluginModule",
      loadPluginModule({
        rootDir: localFixtureRoot,
        entrypoints: ["./index.js"],
        registrationMode: "full",
        shimPackageUrl,
      }),
    );

    const plugin = (loaded.module.default ?? loaded.module) as {
      register?: (api: typeof api) => void | Promise<void>;
    };

    if (typeof plugin.register !== "function") {
      throw new Error("plugin module must export a register(api) function");
    }

    await expectStageToResolve("plugin.register", plugin.register(api), 20000);

    expect(registry.channels.length).toBeGreaterThan(0);
    expect(registry.tools.length).toBeGreaterThan(0);
    expect(registry.commands.length).toBeGreaterThan(0);
    expect(registry.cliEntries.length).toBeGreaterThan(0);
  }, 120000);
});
