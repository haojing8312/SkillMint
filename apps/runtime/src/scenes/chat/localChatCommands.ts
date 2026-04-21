import type { Dispatch, SetStateAction } from "react";

import type { Message, SendMessageRequest, StreamItem } from "../../types";

export type ClawhubInstallCommand = {
  query: string;
};

export type ClawhubSearchCommand = {
  query: string;
};

export type ClawhubUpdateCommand = {
  skillId: string;
};

export type ClawhubCheckUpdateCommand = {
  skillId: string;
};

export type StatusCommand = Record<string, never>;

export type ClawhubSkillSearchItem = {
  name: string;
  slug: string;
  description: string;
  stars: number;
  github_url?: string | null;
  source_url?: string | null;
};

export type LocalChatCommandName =
  | "status"
  | "clawhub.install"
  | "clawhub.search"
  | "clawhub.check-update"
  | "clawhub.update";

export type ParsedLocalChatCommand =
  | { commandName: "status" }
  | { commandName: "clawhub.install"; query: string }
  | { commandName: "clawhub.search"; query: string }
  | { commandName: "clawhub.check-update"; skillId: string }
  | { commandName: "clawhub.update"; skillId: string };

export type LocalChatCommandResult =
  | { kind: "not_handled" }
  | {
      kind: "handled";
      commandName: LocalChatCommandName;
      outcome: "completed" | "presented_candidates" | "rejected" | "failed";
    };

const LOCAL_SKILL_CATALOG_LABEL = "SkillHub";
const SKILL_MARKETPLACE_COMMAND_PREFIX = "(?:clawhub|skillhub)";

type LocalChatCommandDependencies = {
  setInstallError: (value: string | null) => void;
  setMessages: Dispatch<SetStateAction<Message[]>>;
  installedSkillIds?: string[];
  onSkillInstalled?: (skillId: string) => Promise<void> | void;
  searchClawhubSkills: (query: string) => Promise<ClawhubSkillSearchItem[]>;
  recommendClawhubSkills: (query: string) => Promise<ClawhubSkillSearchItem[]>;
  installClawhubSkill: (candidate: ClawhubSkillSearchItem) => Promise<{ manifest?: { id?: string | null } | null }>;
  checkClawhubSkillUpdate: (skillId: string) => Promise<{ has_update: boolean; message: string }>;
  updateClawhubSkill: (skillId: string) => Promise<{ manifest?: { id?: string | null } | null }>;
  buildStatusSummary: () => string;
};

const CLAWHUB_INSTALL_PREFIX =
  new RegExp(
    `^(?:(?:请)?(?:帮我)?(?:安装|安装skill|安装技能|帮我安装skill|帮我安装技能|安装 skill|安装 技能)[:：]?\\s*)?${SKILL_MARKETPLACE_COMMAND_PREFIX}\\s+install\\s+(.+)$`,
    "i",
  );
const CLAWHUB_SEARCH_PREFIX =
  new RegExp(
    `^(?:(?:请)?(?:帮我)?(?:搜索|查找|找|搜)技能[:：]?\\s*)?${SKILL_MARKETPLACE_COMMAND_PREFIX}\\s+search\\s+(.+)$`,
    "i",
  );
const CLAWHUB_CHECK_UPDATE_PREFIX =
  new RegExp(
    `^(?:(?:请)?(?:帮我)?(?:检查|查看|查询)(?:技能)?更新[:：]?\\s*)?${SKILL_MARKETPLACE_COMMAND_PREFIX}\\s+check-update\\s+(.+)$`,
    "i",
  );
const CLAWHUB_UPDATE_PREFIX =
  new RegExp(
    `^(?:(?:请)?(?:帮我)?(?:更新|升级)技能[:：]?\\s*)?${SKILL_MARKETPLACE_COMMAND_PREFIX}\\s+update\\s+(.+)$`,
    "i",
  );
const STATUS_PREFIX = /^\/status$/i;

export function parseClawhubInstallCommand(text: string): ClawhubInstallCommand | null {
  const match = text.trim().match(CLAWHUB_INSTALL_PREFIX);
  const query = match?.[1]?.trim();
  if (!query) {
    return null;
  }
  return { query };
}

export function parseClawhubSearchCommand(text: string): ClawhubSearchCommand | null {
  const match = text.trim().match(CLAWHUB_SEARCH_PREFIX);
  const query = match?.[1]?.trim();
  if (!query) {
    return null;
  }
  return { query };
}

export function parseClawhubUpdateCommand(text: string): ClawhubUpdateCommand | null {
  const match = text.trim().match(CLAWHUB_UPDATE_PREFIX);
  const skillId = match?.[1]?.trim();
  if (!skillId) {
    return null;
  }
  return { skillId };
}

export function parseClawhubCheckUpdateCommand(text: string): ClawhubCheckUpdateCommand | null {
  const match = text.trim().match(CLAWHUB_CHECK_UPDATE_PREFIX);
  const skillId = match?.[1]?.trim();
  if (!skillId) {
    return null;
  }
  return { skillId };
}

export function parseStatusCommand(text: string): StatusCommand | null {
  return STATUS_PREFIX.test(text.trim()) ? {} : null;
}

export function normalizeClawhubCommandLookupKey(text: string): string {
  return text
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

export function isExactClawhubCandidateMatch(
  query: string,
  candidate: { name: string; slug: string },
): boolean {
  const normalizedQuery = normalizeClawhubCommandLookupKey(query);
  if (!normalizedQuery) {
    return false;
  }
  return (
    normalizeClawhubCommandLookupKey(candidate.slug) === normalizedQuery ||
    normalizeClawhubCommandLookupKey(candidate.name) === normalizedQuery
  );
}

export function buildClawhubToolStreamItem(
  toolName: "clawhub_search" | "clawhub_recommend",
  query: string,
  items: ClawhubSkillSearchItem[],
): StreamItem {
  return {
    type: "tool_call",
    toolCall: {
      id: `local-${toolName}-${Date.now()}`,
      name: toolName,
      input: { query },
      output: JSON.stringify({
        source: "skillhub",
        query,
        items,
      }),
      status: "completed",
    },
  };
}

export function parseLocalChatCommand(text: string): ParsedLocalChatCommand | null {
  const statusCommand = parseStatusCommand(text);
  if (statusCommand) {
    return { commandName: "status" };
  }

  const checkUpdateCommand = parseClawhubCheckUpdateCommand(text);
  if (checkUpdateCommand) {
    return { commandName: "clawhub.check-update", skillId: checkUpdateCommand.skillId };
  }

  const updateCommand = parseClawhubUpdateCommand(text);
  if (updateCommand) {
    return { commandName: "clawhub.update", skillId: updateCommand.skillId };
  }

  const searchCommand = parseClawhubSearchCommand(text);
  if (searchCommand) {
    return { commandName: "clawhub.search", query: searchCommand.query };
  }

  const installCommand = parseClawhubInstallCommand(text);
  if (installCommand) {
    return { commandName: "clawhub.install", query: installCommand.query };
  }

  return null;
}

function appendAssistantMessage(
  setMessages: Dispatch<SetStateAction<Message[]>>,
  {
    content,
    createdAt,
    streamItems,
  }: {
    content: string;
    createdAt: string;
    streamItems?: StreamItem[];
  },
): void {
  setMessages((prev) => [
    ...prev,
    {
      role: "assistant",
      content,
      created_at: createdAt,
      streamItems: streamItems ?? [],
    },
  ]);
}

async function handleClawhubUpdateCommand(
  command: Extract<ParsedLocalChatCommand, { commandName: "clawhub.check-update" | "clawhub.update" }>,
  {
    setMessages,
    installedSkillIds = [],
    onSkillInstalled,
    checkClawhubSkillUpdate,
    updateClawhubSkill,
  }: LocalChatCommandDependencies,
  createdAt: string,
): Promise<LocalChatCommandResult> {
  const normalizedSkillId = command.skillId.trim();
  if (!normalizedSkillId.startsWith("clawhub-")) {
    appendAssistantMessage(setMessages, {
      content: "本地技能库更新命令目前只支持明确 skillId，例如：`skillhub update clawhub-self-improving-agent` 或 `clawhub update clawhub-self-improving-agent`。",
      createdAt,
    });
    return { kind: "handled", commandName: command.commandName, outcome: "rejected" };
  }
  if (installedSkillIds.length > 0 && !installedSkillIds.includes(normalizedSkillId)) {
    appendAssistantMessage(setMessages, {
      content: `当前未安装技能「${normalizedSkillId}」，请先确认 skillId 是否正确。`,
      createdAt,
    });
    return { kind: "handled", commandName: command.commandName, outcome: "rejected" };
  }

  const checkResult = await checkClawhubSkillUpdate(normalizedSkillId);
  if (command.commandName === "clawhub.check-update") {
    appendAssistantMessage(setMessages, {
      content: checkResult.message || (checkResult.has_update
        ? `技能「${normalizedSkillId}」有可用更新。`
        : `技能「${normalizedSkillId}」已经是最新版本。`),
      createdAt,
    });
    return { kind: "handled", commandName: command.commandName, outcome: "completed" };
  }

  if (!checkResult.has_update) {
    appendAssistantMessage(setMessages, {
      content: checkResult.message || `技能「${normalizedSkillId}」已经是最新版本。`,
      createdAt,
    });
    return { kind: "handled", commandName: command.commandName, outcome: "rejected" };
  }

  const updateResult = await updateClawhubSkill(normalizedSkillId);
  const updatedSkillId = updateResult?.manifest?.id?.trim() || normalizedSkillId;
  await onSkillInstalled?.(updatedSkillId);
  appendAssistantMessage(setMessages, {
    content: `已更新技能「${updatedSkillId}」。`,
    createdAt,
  });
  return { kind: "handled", commandName: command.commandName, outcome: "completed" };
}

async function handleClawhubLookupCommand(
  command: Extract<ParsedLocalChatCommand, { commandName: "clawhub.install" | "clawhub.search" }>,
  {
    setMessages,
    onSkillInstalled,
    searchClawhubSkills,
    recommendClawhubSkills,
    installClawhubSkill,
  }: LocalChatCommandDependencies,
  createdAt: string,
): Promise<LocalChatCommandResult> {
  let toolName: "clawhub_search" | "clawhub_recommend" = "clawhub_search";
  let toolItems = await searchClawhubSkills(command.query);
  if (toolItems.length === 0) {
    toolName = "clawhub_recommend";
    toolItems = await recommendClawhubSkills(command.query);
  }

  const exactMatches = toolItems.filter((item) => isExactClawhubCandidateMatch(command.query, item));
  if (command.commandName === "clawhub.install" && exactMatches.length === 1) {
    const target = exactMatches[0];
    const result = await installClawhubSkill(target);
    const installedSkillId = result?.manifest?.id?.trim();
    if (installedSkillId) {
      await onSkillInstalled?.(installedSkillId);
    }
    appendAssistantMessage(setMessages, {
      content: `已安装技能「${target.name}」。`,
      createdAt,
      streamItems: [buildClawhubToolStreamItem(toolName, command.query, toolItems)],
    });
    return { kind: "handled", commandName: command.commandName, outcome: "completed" };
  }

  const content =
    command.commandName === "clawhub.search"
      ? toolItems.length > 0
        ? `已在 ${LOCAL_SKILL_CATALOG_LABEL} 找到与「${command.query}」相关的技能候选。`
        : `没有在 ${LOCAL_SKILL_CATALOG_LABEL} 找到与「${command.query}」相关的技能候选。你可以换个关键词再试试。`
      : toolItems.length > 0
        ? `已在 ${LOCAL_SKILL_CATALOG_LABEL} 找到与「${command.query}」相关的技能候选，请从下方卡片确认安装。`
        : `没有在 ${LOCAL_SKILL_CATALOG_LABEL} 找到可直接安装的技能「${command.query}」。你可以换个关键词，或先在技能库里搜索更通用的词。`;
  appendAssistantMessage(setMessages, {
    content,
    createdAt,
    streamItems: toolItems.length > 0 ? [buildClawhubToolStreamItem(toolName, command.query, toolItems)] : [],
  });
  return { kind: "handled", commandName: command.commandName, outcome: "presented_candidates" };
}

function handleStatusCommand(
  command: Extract<ParsedLocalChatCommand, { commandName: "status" }>,
  { setMessages, buildStatusSummary }: LocalChatCommandDependencies,
  createdAt: string,
): LocalChatCommandResult {
  appendAssistantMessage(setMessages, {
    content: buildStatusSummary(),
    createdAt,
  });
  return { kind: "handled", commandName: command.commandName, outcome: "completed" };
}

export async function tryHandleLocalChatCommand(
  request: SendMessageRequest,
  dependencies: LocalChatCommandDependencies,
): Promise<LocalChatCommandResult> {
  const {
    setInstallError,
  } = dependencies;
  if (request.parts.length !== 1 || request.parts[0]?.type !== "text") {
    return { kind: "not_handled" };
  }

  const parsedCommand = parseLocalChatCommand(request.parts[0].text);
  if (!parsedCommand) {
    return { kind: "not_handled" };
  }

  setInstallError(null);
  const createdAt = new Date().toISOString();
  switch (parsedCommand.commandName) {
    case "status":
      return handleStatusCommand(parsedCommand, dependencies, createdAt);
    case "clawhub.check-update":
    case "clawhub.update":
      return handleClawhubUpdateCommand(parsedCommand, dependencies, createdAt);
    case "clawhub.install":
    case "clawhub.search":
      return handleClawhubLookupCommand(parsedCommand, dependencies, createdAt);
  }
}
