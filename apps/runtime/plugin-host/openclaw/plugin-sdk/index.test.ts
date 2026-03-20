import { describe, expect, it } from "vitest";

describe("openclaw plugin-sdk shim", () => {
  it("exports the root plugin-sdk surface", async () => {
    const mod = await import("../plugin-sdk/index");

    expect(mod.DEFAULT_ACCOUNT_ID).toBe("default");
    expect(mod.emptyPluginConfigSchema()).toEqual({
      type: "object",
      additionalProperties: false,
      properties: {},
    });
    expect(mod.normalizeAccountId("  account-a  ")).toBe("account-a");
    expect(
      mod.resolveThreadSessionKeys({
        baseSessionKey: "agent:main:feishu:group:chat-1",
        threadId: "omt-thread",
      }),
    ).toEqual({
      sessionKey: "agent:main:feishu:group:chat-1:thread:omt-thread",
      parentSessionKey: undefined,
    });
  });

  it("exports compat and feishu subpaths", async () => {
    const compat = await import("../plugin-sdk/compat");
    const feishu = await import("../plugin-sdk/feishu");

    expect(compat.SILENT_REPLY_TOKEN).toBe("NO_REPLY");
    expect(feishu.PAIRING_APPROVED_MESSAGE).toContain("Pairing approved");
  });

  it("matches command-auth and history helper shapes expected by the feishu plugin", async () => {
    const mod = await import("../plugin-sdk/index");
    const historyMap = new Map<string, Array<{ sender: string; body: string }>>();
    mod.recordPendingHistoryEntryIfEnabled({
      historyMap,
      historyKey: "group-1",
      entry: { sender: "alice", body: "hello" },
      limit: 10,
    });

    expect(
      mod.buildPendingHistoryContextFromMap({
        historyMap,
        historyKey: "group-1",
        limit: 10,
        currentMessage: "current",
        formatEntry: (entry: { sender: string; body: string }) => `${entry.sender}: ${entry.body}`,
      }),
    ).toContain("alice: hello");

    await expect(
      mod.resolveSenderCommandAuthorization({
        rawBody: "/help",
        cfg: {},
        isGroup: false,
        dmPolicy: "allowlist",
        configuredAllowFrom: ["ou_owner"],
        senderId: "ou_owner",
        isSenderAllowed: (senderId: string, allowFrom: string[]) => allowFrom.includes(senderId),
        readAllowFromStore: async () => [],
        shouldComputeCommandAuthorized: () => true,
        resolveCommandAuthorizedFromAuthorizers: ({ authorizers }: { authorizers: Array<{ allowed: boolean }> }) =>
          authorizers.some((entry) => entry.allowed),
      }),
    ).resolves.toMatchObject({
      shouldComputeAuth: true,
      senderAllowedForCommands: true,
      commandAuthorized: true,
    });
  });
});
