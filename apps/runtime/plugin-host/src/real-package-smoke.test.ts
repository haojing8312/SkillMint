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

function rewritePluginSdkImportsInFixture(rootDir: string): void {
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
      const rewritten = fs
        .readFileSync(entryPath, "utf8")
        .replaceAll(
          /(['"])openclaw\/plugin-sdk\/compat\1/g,
          (_match, quote) => `${quote}${relativeShimRoot}/compat.js${quote}`,
        )
        .replaceAll(
          /(['"])openclaw\/plugin-sdk\/feishu\1/g,
          (_match, quote) => `${quote}${relativeShimRoot}/feishu.js${quote}`,
        )
        .replaceAll(
          /(['"])openclaw\/plugin-sdk\1/g,
          (_match, quote) => `${quote}${relativeShimRoot}/index.js${quote}`,
        );
      fs.writeFileSync(entryPath, rewritten, "utf8");
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
