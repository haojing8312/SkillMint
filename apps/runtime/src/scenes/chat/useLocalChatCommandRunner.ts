import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

import type { Dispatch, SetStateAction } from "react";

import type { Message, SendMessageRequest } from "../../types";
import {
  parseClawhubCheckUpdateCommand,
  parseClawhubInstallCommand,
  parseClawhubSearchCommand,
  parseClawhubUpdateCommand,
  parseStatusCommand,
  tryHandleLocalChatCommand,
  type LocalChatCommandName,
  type LocalChatCommandResult,
} from "./localChatCommands";

type UseLocalChatCommandRunnerArgs = {
  hasAttachments: boolean;
  installedSkillIds: string[];
  onSkillInstalled?: (skillId: string) => Promise<void> | void;
  setInstallError: (value: string | null) => void;
  setMessages: Dispatch<SetStateAction<Message[]>>;
  parseDuplicateSkillName: (error: unknown) => string | null;
  buildStatusSummary: () => string;
};

function inferLocalCommandName(rawText: string): LocalChatCommandName {
  if (parseStatusCommand(rawText)) {
    return "status";
  }
  if (parseClawhubCheckUpdateCommand(rawText)) {
    return "clawhub.check-update";
  }
  if (parseClawhubUpdateCommand(rawText)) {
    return "clawhub.update";
  }
  if (parseClawhubSearchCommand(rawText)) {
    return "clawhub.search";
  }
  if (parseClawhubInstallCommand(rawText)) {
    return "clawhub.install";
  }
  return "clawhub.install";
}

export function useLocalChatCommandRunner({
  hasAttachments,
  installedSkillIds,
  onSkillInstalled,
  setInstallError,
  setMessages,
  parseDuplicateSkillName,
  buildStatusSummary,
}: UseLocalChatCommandRunnerArgs) {
  return useCallback(async (request: SendMessageRequest): Promise<LocalChatCommandResult> => {
    if (hasAttachments) {
      return { kind: "not_handled" };
    }

    try {
      return await tryHandleLocalChatCommand(request, {
        setInstallError,
        setMessages,
        installedSkillIds,
        onSkillInstalled,
        searchClawhubSkills: async (query) => {
          const result = await invoke<Array<{
            name: string;
            slug: string;
            description: string;
            stars: number;
            github_url?: string | null;
            source_url?: string | null;
          }>>("search_clawhub_skills", {
            query,
            page: 1,
            limit: 10,
          });
          return Array.isArray(result) ? result : [];
        },
        recommendClawhubSkills: async (query) => {
          const result = await invoke<Array<{
            name: string;
            slug: string;
            description: string;
            stars: number;
            github_url?: string | null;
            source_url?: string | null;
          }>>("recommend_clawhub_skills", {
            query,
            limit: 5,
          });
          return Array.isArray(result) ? result : [];
        },
        installClawhubSkill: (candidate) =>
          invoke<{ manifest?: { id?: string | null } | null }>("install_clawhub_skill", {
            slug: candidate.slug,
            githubUrl: candidate.github_url ?? candidate.source_url ?? null,
          }),
        checkClawhubSkillUpdate: (skillId) =>
          invoke<{ has_update: boolean; message: string }>("check_clawhub_skill_update", {
            skillId,
          }),
        updateClawhubSkill: (skillId) =>
          invoke<{ manifest?: { id?: string | null } | null }>("update_clawhub_skill", {
            skillId,
          }),
        buildStatusSummary,
      });
    } catch (error) {
      const rawText = request.parts.find((part) => part.type === "text")?.text ?? "";
      const duplicateName = parseDuplicateSkillName(error);
      const content = duplicateName
        ? `技能名称冲突：已存在「${duplicateName}」，请先重命名后再安装。`
        : `执行本地技能库命令失败：${String(error ?? "未知错误")}`;
      setMessages((prev) => [
        ...prev,
        {
          role: "assistant",
          content,
          created_at: new Date().toISOString(),
        },
      ]);
      return {
        kind: "handled",
        commandName: inferLocalCommandName(rawText),
        outcome: "failed",
      };
    }
  }, [
    hasAttachments,
    installedSkillIds,
    onSkillInstalled,
    parseDuplicateSkillName,
    buildStatusSummary,
    setInstallError,
    setMessages,
  ]);
}
