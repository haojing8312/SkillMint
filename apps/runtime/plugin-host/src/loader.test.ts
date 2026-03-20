import fs from "node:fs/promises";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { loadPluginModule, rewritePluginModuleImports } from "./loader";

const tempRoots: string[] = [];
const tempBaseDir = path.join(process.cwd(), "plugin-host", ".tmp-tests");

async function createTempPluginRoot(): Promise<string> {
  await fs.mkdir(tempBaseDir, { recursive: true });
  const root = await fs.mkdtemp(path.join(tempBaseDir, "workclaw-plugin-loader-"));
  tempRoots.push(root);
  return root;
}

afterEach(async () => {
  await Promise.all(
    tempRoots.splice(0, tempRoots.length).map((root) => fs.rm(root, { recursive: true, force: true })),
  );
});

describe("plugin loader", () => {
  it("loads the main entry in full mode", async () => {
    const root = await createTempPluginRoot();
    await fs.writeFile(
      path.join(root, "index.mjs"),
      'export const marker = "full-entry"; export default { id: "demo-plugin" };',
      "utf8",
    );

    const loaded = await loadPluginModule({
      rootDir: root,
      entrypoints: ["./index.mjs"],
      registrationMode: "full",
    });

    expect(loaded.entryPath).toBe(path.join(root, "index.mjs"));
    expect(loaded.importPath).toContain(".workclaw-plugin-host");
    expect(loaded.module.marker).toBe("full-entry");
  });

  it("prefers setupEntry in setup-only mode", async () => {
    const root = await createTempPluginRoot();
    await fs.writeFile(path.join(root, "index.mjs"), 'export const marker = "full-entry";', "utf8");
    await fs.writeFile(path.join(root, "setup.mjs"), 'export const marker = "setup-entry";', "utf8");

    const loaded = await loadPluginModule({
      rootDir: root,
      entrypoints: ["./index.mjs"],
      setupEntry: "./setup.mjs",
      registrationMode: "setup-only",
    });

    expect(loaded.entryPath).toBe(path.join(root, "setup.mjs"));
    expect(loaded.module.marker).toBe("setup-entry");
  });

  it("prefers setupEntry in setup-runtime mode", async () => {
    const root = await createTempPluginRoot();
    await fs.writeFile(path.join(root, "index.mjs"), 'export const marker = "full-entry";', "utf8");
    await fs.writeFile(path.join(root, "setup.mjs"), 'export const marker = "setup-runtime-entry";', "utf8");

    const loaded = await loadPluginModule({
      rootDir: root,
      entrypoints: ["./index.mjs"],
      setupEntry: "./setup.mjs",
      registrationMode: "setup-runtime",
    });

    expect(loaded.entryPath).toBe(path.join(root, "setup.mjs"));
    expect(loaded.module.marker).toBe("setup-runtime-entry");
  });

  it("falls back to the first extension entry when setupEntry is absent", async () => {
    const root = await createTempPluginRoot();
    await fs.writeFile(
      path.join(root, "entry-one.mjs"),
      'export const marker = "fallback-entry";',
      "utf8",
    );

    const loaded = await loadPluginModule({
      rootDir: root,
      entrypoints: ["./entry-one.mjs", "./entry-two.mjs"],
      registrationMode: "setup-only",
    });

    expect(loaded.entryPath).toBe(path.join(root, "entry-one.mjs"));
    expect(loaded.module.marker).toBe("fallback-entry");
  });

  it("rewrites openclaw plugin-sdk bare imports to the local shim", () => {
    const rewritten = rewritePluginModuleImports(
      "import { emptyPluginConfigSchema } from 'openclaw/plugin-sdk';\nimport 'openclaw/plugin-sdk/compat';\nimport \"openclaw/plugin-sdk/feishu\";",
      "file:///D:/code/WorkClaw/apps/runtime/plugin-host/openclaw/package.json",
      undefined,
      "node",
    );

    expect(rewritten).toContain("file:///D:/code/WorkClaw/apps/runtime/plugin-host/openclaw/plugin-sdk/index.js");
    expect(rewritten).toContain("file:///D:/code/WorkClaw/apps/runtime/plugin-host/openclaw/plugin-sdk/compat.js");
    expect(rewritten).toContain("file:///D:/code/WorkClaw/apps/runtime/plugin-host/openclaw/plugin-sdk/feishu.js");
  });

  it("rewrites relative imports to absolute file urls", () => {
    const rewritten = rewritePluginModuleImports(
      "import { feishuPlugin } from './src/channel/plugin.js';\nexport { registerSyntheticTool } from '../tools/register.js';",
      "file:///D:/code/WorkClaw/apps/runtime/plugin-host/openclaw/package.json",
      "D:/tmp/plugin/index.js",
      "node",
    );

    expect(rewritten).toContain("file:///D:/tmp/plugin/src/channel/plugin.js");
    expect(rewritten).toContain("file:///D:/tmp/tools/register.js");
  });

  it("rewrites relative imports so entry modules can load sibling files", async () => {
    const root = await createTempPluginRoot();
    await fs.mkdir(path.join(root, "src", "channel"), { recursive: true });
    await fs.mkdir(path.join(root, "src", "tools"), { recursive: true });
    await fs.writeFile(
      path.join(root, "src", "channel", "plugin.js"),
      "export const feishuPlugin = { id: 'feishu' };",
      "utf8",
    );
    await fs.writeFile(
      path.join(root, "src", "tools", "register.js"),
      "export function registerSyntheticTool(api) { api.registerTool({ name: 'synthetic_tool' }); }",
      "utf8",
    );
    await fs.writeFile(
      path.join(root, "index.js"),
      [
        "import { feishuPlugin } from './src/channel/plugin.js';",
        "import { registerSyntheticTool } from './src/tools/register.js';",
        "export default {",
        "  register(api) {",
        "    api.registerChannel({ plugin: feishuPlugin });",
        "    registerSyntheticTool(api);",
        "  }",
        "};",
      ].join("\n"),
      "utf8",
    );

    const loaded = await loadPluginModule({
      rootDir: root,
      entrypoints: ["./index.js"],
      registrationMode: "full",
    });

    expect(typeof loaded.module.default).toBe("object");
  });
});
