import { describe, expect, it } from "vitest";
import { createPluginRuntime } from "./runtime";

describe("plugin runtime", () => {
  it("provides logger hierarchy, config loader, and channel compatibility helpers", () => {
    const runtime = createPluginRuntime({
      config: {
        channels: {
          feishu: {
            enabled: true,
            requireMention: true,
            groupPolicy: "allowlist",
            allowFrom: ["ou_owner"],
          },
        },
      },
    });

    const child = runtime.logging.getChildLogger({ scope: "feishu" });
    child.info?.("hello");
    runtime.system.enqueueSystemEvent("inbound", { sessionKey: "agent:main:direct:user" });

    expect(runtime.channel.text.chunkMarkdownText("abcdef", 3)).toEqual(["abc", "def"]);
    expect(runtime.channel.text.convertMarkdownTables("|a|", "bullets")).toBe("|a|");
    expect(runtime.channel.groups.resolveRequireMention({})).toBe(true);
    expect(runtime.channel.groups.resolveGroupPolicy({})).toBe("allowlist");
    expect(
      runtime.channel.pairing.buildPairingReply({
        channel: "feishu",
        idLine: "ou_sender",
        code: "PAIR123",
      }),
    ).toContain("openclaw pairing approve feishu PAIR123");
    expect(runtime.channel.commands.shouldComputeCommandAuthorized("/help", {})).toBe(true);
    expect(
      runtime.channel.routing.resolveAgentRoute({
        channel: "feishu",
        peer: { kind: "direct", id: "ou_user" },
      }).sessionKey,
    ).toContain("feishu:direct:ou_user");
    expect(runtime.config.loadConfig()).toEqual({
      channels: {
        feishu: {
          enabled: true,
          requireMention: true,
          groupPolicy: "allowlist",
          allowFrom: ["ou_owner"],
        },
      },
    });
    expect(runtime.logging.records).toHaveLength(1);
    expect(runtime.logging.records[0]?.scope).toBe("feishu");
    expect(runtime.system.records).toHaveLength(1);
  });

  it("captures dispatch requests from the official reply bridge", async () => {
    const runtime = createPluginRuntime({ config: {} });

    await runtime.channel.reply.dispatchReplyFromConfig({
      ctx: {
        AccountId: "default",
        SenderId: "ou_sender",
        MessageSid: "om_123",
        RawBody: "你好",
        ChatType: "direct",
        ChatId: "oc_chat_123",
        To: "user:ou_sender",
        From: "feishu:ou_sender",
      },
    });

    expect(runtime.system.dispatchRequests).toEqual([
      {
        accountId: "default",
        chatId: "oc_chat_123",
        threadId: "oc_chat_123",
        senderId: "ou_sender",
        messageId: "om_123",
        text: "你好",
        chatType: "direct",
      },
    ]);
  });

  it("matches the reply dispatcher shape expected by the official feishu plugin", async () => {
    const runtime = createPluginRuntime({ config: {} });
    const result = runtime.channel.reply.createReplyDispatcherWithTyping();

    expect(result.replyOptions).toEqual({});
    expect(typeof result.markDispatchIdle).toBe("function");
    expect(typeof result.markRunComplete).toBe("function");
    expect(typeof result.dispatcher.sendToolResult).toBe("function");
    expect(typeof result.dispatcher.sendBlockReply).toBe("function");
    expect(typeof result.dispatcher.sendFinalReply).toBe("function");
    expect(typeof result.dispatcher.waitForIdle).toBe("function");
    expect(result.dispatcher.getQueuedCounts()).toEqual({
      tool: 0,
      block: 0,
      final: 0,
    });

    await runtime.channel.reply.withReplyDispatcher({
      dispatcher: result.dispatcher,
      run: async () => {
        result.dispatcher.sendFinalReply({ text: "ok" });
      },
    });

    await expect(result.dispatcher.waitForIdle()).resolves.toBeUndefined();
  });
});
