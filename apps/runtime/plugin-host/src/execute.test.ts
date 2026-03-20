import fs from "node:fs/promises";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { executePluginRegistration } from "./execute";

const tempRoots: string[] = [];
const tempBaseDir = path.join(process.cwd(), "plugin-host", ".tmp-tests");

async function createTempPluginRoot(): Promise<string> {
  await fs.mkdir(tempBaseDir, { recursive: true });
  const root = await fs.mkdtemp(path.join(tempBaseDir, "workclaw-plugin-execute-"));
  tempRoots.push(root);
  return root;
}

afterEach(async () => {
  await Promise.all(
    tempRoots.splice(0, tempRoots.length).map((root) => fs.rm(root, { recursive: true, force: true })),
  );
});

describe("plugin execution", () => {
  it("loads and executes a plugin register function against the host api", async () => {
    const root = await createTempPluginRoot();
    await fs.writeFile(
      path.join(root, "index.mjs"),
      [
        "import { emptyPluginConfigSchema } from 'openclaw/plugin-sdk';",
        "export default {",
        "  id: 'synthetic-feishu',",
        "  configSchema: emptyPluginConfigSchema(),",
        "  register(api) {",
        "    const child = api.runtime.logging.getChildLogger({ scope: 'synthetic' });",
        "    child.info?.('registering synthetic plugin');",
        "    api.registerChannel({ plugin: { id: 'feishu' } });",
        "    api.registerTool({ name: 'feishu_search' });",
        "    api.registerCli((ctx) => { ctx.program.command('feishu-diagnose'); }, { commands: ['feishu-diagnose'] });",
        "    api.registerCommand({ name: 'feishu_help' });",
        "    api.registerGatewayMethod('feishu.sync', () => ({ ok: true }));",
        "    api.on('before_tool_call', () => undefined);",
        "  }",
        "};",
      ].join("\n"),
      "utf8",
    );

    const result = await executePluginRegistration({
      rootDir: root,
      entrypoints: ["./index.mjs"],
      shimPackageUrl: "file:///D:/code/WorkClaw/apps/runtime/plugin-host/openclaw/package.json",
      config: {
        channels: {
          feishu: {
            enabled: true,
          },
        },
      },
    });

    expect(result.registry.channels).toHaveLength(1);
    expect(result.registry.tools).toHaveLength(1);
    expect(result.registry.cliEntries).toHaveLength(1);
    expect(result.registry.commands).toHaveLength(1);
    expect(Object.keys(result.registry.gatewayMethods)).toEqual(["feishu.sync"]);
    expect(result.registry.hooks.before_tool_call).toHaveLength(1);
    expect(result.runtime.logging.records[0]?.scope).toBe("synthetic");
  }, 15000);
});
