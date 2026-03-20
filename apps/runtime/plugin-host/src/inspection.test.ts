import { describe, expect, it } from "vitest";
import { createPluginRegistry } from "./registry";
import { summarizeRegistry } from "./inspection";

describe("inspection", () => {
  it("summarizes channel, tool, command, cli, and hook registrations", () => {
    const registry = createPluginRegistry();
    registry.channels.push({
      id: "feishu",
      meta: {
        label: "Feishu",
        docsPath: "/channels/feishu",
        aliases: ["lark"],
      },
      capabilities: {
        threads: true,
        reactions: true,
      },
      reload: {
        configPrefixes: ["channels.feishu"],
      },
      pairing: {},
      setup: {},
      onboarding: {},
      directory: {},
      outbound: {},
      threading: {},
      actions: {},
      status: {},
      messaging: {
        targetResolver: {
          hint: "<chatId|user:openId>",
        },
      },
    });
    registry.tools.push({
      id: "feishu_doc.create",
      title: "Create Feishu Doc",
      description: "creates a doc",
    });
    registry.commands.push({
      name: "/feishu",
    });
    registry.cliEntries.push({
      registration: {
        commands: ["feishu-diagnose"],
      },
    });
    registry.gatewayMethods["feishu.send"] = () => {};
    registry.hooks.before_tool_call.push(() => {});

    const summary = summarizeRegistry(registry);

    expect(summary.channels).toEqual([
      expect.objectContaining({
        id: "feishu",
        reloadConfigPrefixes: ["channels.feishu"],
        hasPairing: true,
        hasSetup: true,
        hasOnboarding: true,
        hasDirectory: true,
        hasOutbound: true,
        hasThreading: true,
        hasActions: true,
        hasStatus: true,
        targetHint: "<chatId|user:openId>",
      }),
    ]);
    expect(summary.tools).toEqual([
      expect.objectContaining({
        id: "feishu_doc.create",
        title: "Create Feishu Doc",
      }),
    ]);
    expect(summary.commandNames).toEqual(["/feishu"]);
    expect(summary.cliCommandNames).toEqual(["feishu-diagnose"]);
    expect(summary.gatewayMethods).toEqual(["feishu.send"]);
    expect(summary.hookCounts.before_tool_call).toBe(1);
    expect(summary.hookCounts.after_tool_call).toBe(0);
  });
});
