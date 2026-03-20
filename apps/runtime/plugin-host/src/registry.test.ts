import { describe, expect, it } from "vitest";
import { createPluginApi } from "./api";
import { createPluginRegistry } from "./registry";

describe("plugin registry", () => {
  it("stores registered channels, tools, cli entries, gateway methods, and commands", () => {
    const registry = createPluginRegistry();
    const api = createPluginApi(registry);
    const channelPlugin = { id: "feishu", capabilities: {} };
    const tool = { name: "feishu_search" };
    const cli = { name: "openclaw-lark" };
    const gatewayMethod = (payload: unknown) => payload;
    const command = { id: "sync-feishu" };

    api.registerChannel({ plugin: channelPlugin });
    api.registerTool(tool);
    api.registerCli(cli);
    api.registerGatewayMethod("feishu.sync", gatewayMethod);
    api.registerCommand(command);

    expect(registry.channels).toEqual([channelPlugin]);
    expect(registry.tools).toEqual([tool]);
    expect(registry.cliEntries).toEqual([{ entry: cli, registration: undefined }]);
    expect(registry.gatewayMethods["feishu.sync"]).toBe(gatewayMethod);
    expect(registry.commands).toEqual([command]);
  });

  it("stores hook handlers by event name", async () => {
    const registry = createPluginRegistry();
    const api = createPluginApi(registry);
    const seen: string[] = [];

    api.on("before_tool_call", async () => {
      seen.push("before");
    });
    api.on("after_tool_call", async () => {
      seen.push("after");
    });

    expect(registry.hooks.before_tool_call).toHaveLength(1);
    expect(registry.hooks.after_tool_call).toHaveLength(1);

    await registry.hooks.before_tool_call[0]?.({});
    await registry.hooks.after_tool_call[0]?.({});

    expect(seen).toEqual(["before", "after"]);
  });
});
