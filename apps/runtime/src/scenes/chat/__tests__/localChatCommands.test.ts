import { describe, expect, it, vi } from "vitest";

import {
  parseClawhubCheckUpdateCommand,
  parseClawhubInstallCommand,
  parseLocalChatCommand,
  parseClawhubSearchCommand,
  parseClawhubUpdateCommand,
  parseStatusCommand,
  tryHandleLocalChatCommand,
} from "../localChatCommands";

describe("localChatCommands", () => {
  it("parses clawhub and skillhub command phrases with natural-language prefix", () => {
    expect(parseClawhubInstallCommand("帮我安装skill：clawhub install AI Recruiting Engine")).toEqual({
      query: "AI Recruiting Engine",
    });
    expect(parseClawhubInstallCommand("帮我安装skill：skillhub install AI Recruiting Engine")).toEqual({
      query: "AI Recruiting Engine",
    });
    expect(parseClawhubSearchCommand("帮我搜索技能：clawhub search recruiting")).toEqual({
      query: "recruiting",
    });
    expect(parseClawhubSearchCommand("帮我搜索技能：skillhub search recruiting")).toEqual({
      query: "recruiting",
    });
    expect(parseClawhubCheckUpdateCommand("帮我检查更新：clawhub check-update clawhub-self-improving-agent")).toEqual({
      skillId: "clawhub-self-improving-agent",
    });
    expect(parseClawhubCheckUpdateCommand("帮我检查更新：skillhub check-update clawhub-self-improving-agent")).toEqual({
      skillId: "clawhub-self-improving-agent",
    });
    expect(parseClawhubUpdateCommand("帮我更新技能：clawhub update clawhub-self-improving-agent")).toEqual({
      skillId: "clawhub-self-improving-agent",
    });
    expect(parseClawhubUpdateCommand("帮我更新技能：skillhub update clawhub-self-improving-agent")).toEqual({
      skillId: "clawhub-self-improving-agent",
    });
    expect(parseLocalChatCommand("clawhub search recruiting")).toEqual({
      commandName: "clawhub.search",
      query: "recruiting",
    });
    expect(parseLocalChatCommand("skillhub search recruiting")).toEqual({
      commandName: "clawhub.search",
      query: "recruiting",
    });
    expect(parseStatusCommand("/status")).toEqual({});
    expect(parseClawhubInstallCommand("随便聊聊")).toBeNull();
  });

  it("handles exact clawhub install commands locally", async () => {
    const setInstallError = vi.fn();
    const setMessages = vi.fn((updater) =>
      typeof updater === "function" ? updater([]) : updater,
    );
    const onSkillInstalled = vi.fn();
    const searchClawhubSkills = vi.fn().mockResolvedValue([
      {
        slug: "ai-recruiting-engine",
        name: "AI Recruiting Engine",
        description: "Automates recruiting workflows",
        stars: 42,
        github_url: "https://github.com/example/ai-recruiting-engine",
      },
    ]);
    const recommendClawhubSkills = vi.fn().mockResolvedValue([]);
    const installClawhubSkill = vi.fn().mockResolvedValue({
      manifest: { id: "clawhub-ai-recruiting-engine" },
    });

    const handled = await tryHandleLocalChatCommand(
      {
        sessionId: "session-1",
        parts: [{ type: "text", text: "clawhub install AI Recruiting Engine" }],
      },
      {
        setInstallError,
        setMessages,
        onSkillInstalled,
        searchClawhubSkills,
        recommendClawhubSkills,
        installClawhubSkill,
        checkClawhubSkillUpdate: vi.fn(),
        updateClawhubSkill: vi.fn(),
        buildStatusSummary: () => "status",
      },
    );

    expect(handled).toEqual({
      kind: "handled",
      commandName: "clawhub.install",
      outcome: "completed",
    });
    expect(searchClawhubSkills).toHaveBeenCalledWith("AI Recruiting Engine");
    expect(recommendClawhubSkills).not.toHaveBeenCalled();
    expect(installClawhubSkill).toHaveBeenCalledWith(
      expect.objectContaining({
        slug: "ai-recruiting-engine",
        name: "AI Recruiting Engine",
      }),
    );
    expect(onSkillInstalled).toHaveBeenCalledWith("clawhub-ai-recruiting-engine");
    expect(setMessages).toHaveBeenCalledTimes(1);
  });

  it("handles exact skillhub install commands through the same local flow", async () => {
    const setInstallError = vi.fn();
    const setMessages = vi.fn((updater) =>
      typeof updater === "function" ? updater([]) : updater,
    );
    const onSkillInstalled = vi.fn();
    const searchClawhubSkills = vi.fn().mockResolvedValue([
      {
        slug: "ai-recruiting-engine",
        name: "AI Recruiting Engine",
        description: "Automates recruiting workflows",
        stars: 42,
        github_url: "https://github.com/example/ai-recruiting-engine",
      },
    ]);
    const recommendClawhubSkills = vi.fn().mockResolvedValue([]);
    const installClawhubSkill = vi.fn().mockResolvedValue({
      manifest: { id: "clawhub-ai-recruiting-engine" },
    });

    const handled = await tryHandleLocalChatCommand(
      {
        sessionId: "session-1",
        parts: [{ type: "text", text: "skillhub install AI Recruiting Engine" }],
      },
      {
        setInstallError,
        setMessages,
        onSkillInstalled,
        searchClawhubSkills,
        recommendClawhubSkills,
        installClawhubSkill,
        checkClawhubSkillUpdate: vi.fn(),
        updateClawhubSkill: vi.fn(),
        buildStatusSummary: () => "status",
      },
    );

    expect(handled).toEqual({
      kind: "handled",
      commandName: "clawhub.install",
      outcome: "completed",
    });
    expect(searchClawhubSkills).toHaveBeenCalledWith("AI Recruiting Engine");
    expect(installClawhubSkill).toHaveBeenCalledTimes(1);
    expect(onSkillInstalled).toHaveBeenCalledWith("clawhub-ai-recruiting-engine");
    expect(setMessages).toHaveBeenCalledTimes(1);
  });

  it("returns local candidates when no exact install match exists", async () => {
    const setInstallError = vi.fn();
    const setMessages = vi.fn((updater) =>
      typeof updater === "function" ? updater([]) : updater,
    );
    const searchClawhubSkills = vi.fn().mockResolvedValue([]);
    const recommendClawhubSkills = vi.fn().mockResolvedValue([
      {
        slug: "recruiting-copilot",
        name: "Recruiting Copilot",
        description: "Candidate screening assistant",
        stars: 10,
      },
    ]);
    const installClawhubSkill = vi.fn();

    const handled = await tryHandleLocalChatCommand(
      {
        sessionId: "session-1",
        parts: [{ type: "text", text: "clawhub install AI Recruiting Engine" }],
      },
      {
        setInstallError,
        setMessages,
        searchClawhubSkills,
        recommendClawhubSkills,
        installClawhubSkill,
        checkClawhubSkillUpdate: vi.fn(),
        updateClawhubSkill: vi.fn(),
        buildStatusSummary: () => "status",
      },
    );

    expect(handled).toEqual({
      kind: "handled",
      commandName: "clawhub.install",
      outcome: "presented_candidates",
    });
    expect(searchClawhubSkills).toHaveBeenCalledWith("AI Recruiting Engine");
    expect(recommendClawhubSkills).toHaveBeenCalledWith("AI Recruiting Engine");
    expect(installClawhubSkill).not.toHaveBeenCalled();
    expect(setMessages).toHaveBeenCalledTimes(1);
  });

  it("handles clawhub search locally without installing", async () => {
    const setMessages = vi.fn((updater) => (typeof updater === "function" ? updater([]) : updater));
    const searchClawhubSkills = vi.fn().mockResolvedValue([
      {
        slug: "recruiting-copilot",
        name: "Recruiting Copilot",
        description: "Candidate screening assistant",
        stars: 10,
      },
    ]);

    const handled = await tryHandleLocalChatCommand(
      {
        sessionId: "session-1",
        parts: [{ type: "text", text: "clawhub search recruiting" }],
      },
      {
        setInstallError: vi.fn(),
        setMessages,
        searchClawhubSkills,
        recommendClawhubSkills: vi.fn().mockResolvedValue([]),
        installClawhubSkill: vi.fn(),
        checkClawhubSkillUpdate: vi.fn(),
        updateClawhubSkill: vi.fn(),
        buildStatusSummary: () => "status",
      },
    );

    expect(handled).toEqual({
      kind: "handled",
      commandName: "clawhub.search",
      outcome: "presented_candidates",
    });
    expect(searchClawhubSkills).toHaveBeenCalledWith("recruiting");
    expect(setMessages).toHaveBeenCalledTimes(1);
  });

  it("updates a clawhub skill locally when a newer version exists", async () => {
    const onSkillInstalled = vi.fn();
    const checkClawhubSkillUpdate = vi.fn().mockResolvedValue({
      has_update: true,
      message: "发现新版本",
    });
    const updateClawhubSkill = vi.fn().mockResolvedValue({
      manifest: { id: "clawhub-self-improving-agent" },
    });
    const setMessages = vi.fn((updater) => (typeof updater === "function" ? updater([]) : updater));

    const handled = await tryHandleLocalChatCommand(
      {
        sessionId: "session-1",
        parts: [{ type: "text", text: "clawhub update clawhub-self-improving-agent" }],
      },
      {
        setInstallError: vi.fn(),
        setMessages,
        installedSkillIds: ["clawhub-self-improving-agent"],
        onSkillInstalled,
        searchClawhubSkills: vi.fn(),
        recommendClawhubSkills: vi.fn(),
        installClawhubSkill: vi.fn(),
        checkClawhubSkillUpdate,
        updateClawhubSkill,
        buildStatusSummary: () => "status",
      },
    );

    expect(handled).toEqual({
      kind: "handled",
      commandName: "clawhub.update",
      outcome: "completed",
    });
    expect(checkClawhubSkillUpdate).toHaveBeenCalledWith("clawhub-self-improving-agent");
    expect(updateClawhubSkill).toHaveBeenCalledWith("clawhub-self-improving-agent");
    expect(onSkillInstalled).toHaveBeenCalledWith("clawhub-self-improving-agent");
    expect(setMessages).toHaveBeenCalledTimes(1);
  });

  it("checks clawhub skill update locally without performing update", async () => {
    const checkClawhubSkillUpdate = vi.fn().mockResolvedValue({
      has_update: true,
      message: "发现新版本：1.0.0 -> 1.1.0",
    });
    const updateClawhubSkill = vi.fn();
    const setMessages = vi.fn((updater) => (typeof updater === "function" ? updater([]) : updater));

    const handled = await tryHandleLocalChatCommand(
      {
        sessionId: "session-1",
        parts: [{ type: "text", text: "clawhub check-update clawhub-self-improving-agent" }],
      },
      {
        setInstallError: vi.fn(),
        setMessages,
        installedSkillIds: ["clawhub-self-improving-agent"],
        searchClawhubSkills: vi.fn(),
        recommendClawhubSkills: vi.fn(),
        installClawhubSkill: vi.fn(),
        checkClawhubSkillUpdate,
        updateClawhubSkill,
        buildStatusSummary: () => "status",
      },
    );

    expect(handled).toEqual({
      kind: "handled",
      commandName: "clawhub.check-update",
      outcome: "completed",
    });
    expect(checkClawhubSkillUpdate).toHaveBeenCalledWith("clawhub-self-improving-agent");
    expect(updateClawhubSkill).not.toHaveBeenCalled();
    expect(setMessages).toHaveBeenCalledTimes(1);
  });

  it("handles /status locally", async () => {
    const setMessages = vi.fn((updater) => (typeof updater === "function" ? updater([]) : updater));

    const handled = await tryHandleLocalChatCommand(
      {
        sessionId: "session-1",
        parts: [{ type: "text", text: "/status" }],
      },
      {
        setInstallError: vi.fn(),
        setMessages,
        searchClawhubSkills: vi.fn(),
        recommendClawhubSkills: vi.fn(),
        installClawhubSkill: vi.fn(),
        checkClawhubSkillUpdate: vi.fn(),
        updateClawhubSkill: vi.fn(),
        buildStatusSummary: () => "当前会话状态：\n- 模型：Model A",
      },
    );

    expect(handled).toEqual({
      kind: "handled",
      commandName: "status",
      outcome: "completed",
    });
    expect(setMessages).toHaveBeenCalledTimes(1);
  });
});
