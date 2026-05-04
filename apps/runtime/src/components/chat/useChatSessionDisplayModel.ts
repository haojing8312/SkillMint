import { useMemo } from "react";

import type { ModelConfig, SkillManifest } from "../../types";
import { getDefaultModel } from "../../lib/default-model";
import { getThinkingSupport } from "./chatViewHelpers";

export function useChatSessionDisplayModel({
  skill,
  models,
  sessionModelId,
  workspace,
  sessionTitle,
  sessionMode,
  sessionEmployeeName,
  sessionSourceChannel,
  sessionSourceLabel,
  operationPermissionMode,
}: {
  skill: SkillManifest;
  models: ModelConfig[];
  sessionModelId?: string;
  workspace: string;
  sessionTitle?: string;
  sessionMode?: string;
  sessionEmployeeName?: string;
  sessionSourceChannel?: string;
  sessionSourceLabel?: string;
  operationPermissionMode: string;
}) {
  const currentModel = useMemo(
    () => models.find((model) => model.id === sessionModelId) ?? getDefaultModel(models),
    [models, sessionModelId],
  );
  const thinkingSupport = useMemo(() => getThinkingSupport(currentModel), [currentModel]);
  const normalizedSessionMode = (sessionMode || "").trim().toLowerCase();
  const isTeamEntrySession = normalizedSessionMode === "team_entry";
  const isEmployeeDirectSession = normalizedSessionMode === "employee_direct";
  const normalizedSessionTitle = (sessionTitle || "").trim();
  const normalizedSessionEmployeeName = (sessionEmployeeName || "").trim();
  const sessionDisplayTitle = isTeamEntrySession
    ? "团队协作"
    : isEmployeeDirectSession
    ? normalizedSessionEmployeeName || normalizedSessionTitle || skill.name
    : skill.name;
  const sessionDisplaySubtitle = isTeamEntrySession ? normalizedSessionTitle || "团队已连接" : "";
  const normalizedSessionSourceChannel = (sessionSourceChannel || "").trim().toLowerCase();
  const isImSource = normalizedSessionSourceChannel.length > 0 && normalizedSessionSourceChannel !== "local";
  const sessionSourceBadgeText =
    (sessionSourceLabel || "").trim() ||
    (normalizedSessionSourceChannel ? `${normalizedSessionSourceChannel} 同步` : "IM 同步");
  const displayWorkDirLabel = (workspace || "").trim() || "选择工作目录";
  const localStatusSummary = useMemo(() => {
    const lines = [
      "当前会话状态：",
      `- 模型：${currentModel?.name || "未配置"}`,
      `- 工作目录：${(workspace || "").trim() || "未设置"}`,
      `- 会话类型：${normalizedSessionMode || "general"}`,
      `- 来源：${isImSource ? sessionSourceBadgeText : "本地"}`,
      `- 权限模式：${operationPermissionMode}`,
    ];
    if (sessionDisplayTitle.trim()) {
      lines.push(`- 标题：${sessionDisplayTitle.trim()}`);
    }
    if (sessionDisplaySubtitle.trim()) {
      lines.push(`- 副标题：${sessionDisplaySubtitle.trim()}`);
    }
    return lines.join("\n");
  }, [
    currentModel?.name,
    workspace,
    normalizedSessionMode,
    isImSource,
    sessionSourceBadgeText,
    operationPermissionMode,
    sessionDisplayTitle,
    sessionDisplaySubtitle,
  ]);

  return {
    currentModel,
    thinkingSupport,
    normalizedSessionMode,
    isTeamEntrySession,
    sessionDisplayTitle,
    sessionDisplaySubtitle,
    isImSource,
    sessionSourceBadgeText,
    displayWorkDirLabel,
    localStatusSummary,
  };
}
