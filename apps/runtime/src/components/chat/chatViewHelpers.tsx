import type { ReactNode } from "react";
import type { ModelConfig, SessionRunProjection, StreamItem } from "../../types";
import type { PendingApprovalView } from "../../scenes/chat/useChatSessionController";
import type { TaskJourneyViewModel } from "../chat-side-panel/view-model";
import {
  getModelErrorDisplay,
  inferModelErrorKindFromMessage,
  isModelErrorKind,
} from "../../lib/model-error-display";

export type ClawhubInstallCandidate = {
  slug: string;
  name: string;
  description?: string;
  stars?: number;
  githubUrl?: string | null;
  sourceUrl?: string | null;
};

export const TOOL_ACTION_LABELS: Record<string, string> = {
  file_delete: "删除文件",
  write_file: "写入文件",
  edit: "编辑文件",
  bash: "执行命令",
  web_search: "网页搜索",
  web_fetch: "获取网页",
};

export function buildApprovalReasonText(
  approval: PendingApprovalView | null,
  toolLabel: string,
  readOnly: boolean,
  destructive: boolean,
  requiresApproval: boolean,
): string | undefined {
  if (!approval) return undefined;
  if (approval.irreversible || destructive || approval.toolName === "file_delete") {
    return `原因：这是不可逆的${toolLabel}操作，确认后会立即执行一次。`;
  }
  if (requiresApproval || approval.toolName === "bash") {
    return `原因：这是${readOnly ? "读取环境" : "会修改环境的"}${toolLabel}操作，确认后才会继续。`;
  }
  if (!readOnly) {
    return `原因：这是会修改环境的${toolLabel}操作。`;
  }
  return undefined;
}

export function buildApprovalImpactText(
  approval: PendingApprovalView | null,
  readOnly: boolean,
  destructive: boolean,
): string | undefined {
  if (approval?.impact?.trim()) return approval.impact;
  if (approval?.irreversible || destructive || approval?.toolName === "file_delete") {
    return "这类操作可能直接删除或覆盖本地内容。";
  }
  if (readOnly) {
    return "这类操作通常只读取信息，不会直接修改本地内容。";
  }
  return "这类操作可能修改本地文件、命令环境或会话状态。";
}

export function shouldRenderCompletedJourneySummary(model: TaskJourneyViewModel) {
  if (model.deliverables.length === 0) return false;
  return model.status === "completed" || model.status === "partial";
}

export function CopyActionIcon({ copied }: { copied: boolean }) {
  if (copied) {
    return (
      <svg aria-hidden="true" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
      </svg>
    );
  }

  return (
    <svg aria-hidden="true" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
      <rect x="9" y="9" width="10" height="10" rx="2" />
      <path strokeLinecap="round" strokeLinejoin="round" d="M15 9V7a2 2 0 00-2-2H7a2 2 0 00-2 2v6a2 2 0 002 2h2" />
    </svg>
  );
}

export function getThinkingSupport(model: ModelConfig | null): {
  indicator: boolean;
  reasoning: boolean;
} {
  if (!model) {
    return { indicator: true, reasoning: false };
  }

  if (model.api_format === "openai") {
    return { indicator: true, reasoning: true };
  }

  if (model.api_format === "anthropic") {
    const normalizedBaseUrl = model.base_url.trim().toLowerCase();
    const normalizedModelName = model.model_name.trim().toLowerCase();
    const supportsExtendedAnthropicReasoning =
      normalizedBaseUrl.includes("api.anthropic.com/v1") &&
      (normalizedModelName.startsWith("claude-sonnet-4") || normalizedModelName.startsWith("claude-opus-4"));

    return {
      indicator: true,
      reasoning: supportsExtendedAnthropicReasoning,
    };
  }

  return { indicator: false, reasoning: false };
}

function parseClawhubCandidatesFromOutput(output?: string): ClawhubInstallCandidate[] {
  if (!output) return [];
  try {
    const parsed = JSON.parse(output);
    if (parsed?.source !== "clawhub" || !Array.isArray(parsed?.items)) return [];
    return parsed.items
      .map((item: any) => {
        const slug = typeof item?.slug === "string" ? item.slug.trim() : "";
        const name = typeof item?.name === "string" ? item.name.trim() : "";
        if (!slug || !name) return null;
        return {
          slug,
          name,
          description: typeof item?.description === "string" ? item.description : "",
          stars: typeof item?.stars === "number" ? item.stars : undefined,
          githubUrl: typeof item?.github_url === "string" ? item.github_url : null,
          sourceUrl: typeof item?.source_url === "string" ? item.source_url : null,
        } as ClawhubInstallCandidate;
      })
      .filter(Boolean) as ClawhubInstallCandidate[];
  } catch {
    return [];
  }
}

function mergeInstallCandidate(map: Map<string, ClawhubInstallCandidate>, candidate: ClawhubInstallCandidate) {
  const key = `${candidate.slug}:${candidate.githubUrl ?? ""}`;
  const exists = map.get(key);
  if (!exists) {
    map.set(key, candidate);
    return;
  }
  const existingLen = exists.description?.length ?? 0;
  const currentLen = candidate.description?.length ?? 0;
  if (currentLen > existingLen || (candidate.stars ?? 0) > (exists.stars ?? 0)) {
    map.set(key, candidate);
  }
}

export function extractInstallCandidates(items: StreamItem[] | undefined): ClawhubInstallCandidate[] {
  const map = new Map<string, ClawhubInstallCandidate>();
  if (items && items.length > 0) {
    for (const item of items) {
      if (item.type !== "tool_call" || !item.toolCall) continue;
      const name = item.toolCall.name;
      if (name !== "clawhub_search" && name !== "clawhub_recommend") continue;
      const parsed = parseClawhubCandidatesFromOutput(item.toolCall.output);
      for (const candidate of parsed) {
        mergeInstallCandidate(map, candidate);
      }
    }
  }
  return Array.from(map.values());
}

export function extractInstallCandidatesWithContent(
  items: StreamItem[] | undefined,
  _content?: string,
): ClawhubInstallCandidate[] {
  return extractInstallCandidates(items);
}

export function formatCompactionBoundaryGuidance(run: SessionRunProjection) {
  const boundary = run.turn_state?.compaction_boundary;
  if (!boundary) return "";

  const lines = [`最近一次上下文压缩：${boundary.original_tokens} -> ${boundary.compacted_tokens}`];
  if ((boundary.summary || "").trim()) {
    lines.push(`压缩摘要：${boundary.summary.trim()}`);
  }
  if (typeof run.turn_state?.reconstructed_history_len === "number") {
    lines.push(`重建历史消息数：${run.turn_state.reconstructed_history_len}`);
  }
  lines.push("继续执行会从压缩后的上下文继续。");
  return lines.join("\n");
}

export function getRunFailureDisplay(run: SessionRunProjection) {
  const networkRecoverySuffix =
    "\n已经保留当前任务的历史消息和部分输出。网络恢复后可直接输入“继续”，从当前上下文继续完成任务。";

  if (run.error_kind === "cancelled") {
    return {
      title: "任务已取消",
      message: run.error_message || "",
      rawMessage: null as string | null,
    };
  }

  if (run.error_kind === "max_turns") {
    const compactionGuidance = formatCompactionBoundaryGuidance(run);
    const baseMessage =
      run.error_message || "已达到执行步数上限，系统已自动停止。\n你可以点击下方“继续执行”，或直接发送“继续”来再追加 100 步预算。";
    return {
      title: "任务达到执行步数上限",
      message: compactionGuidance ? `${baseMessage}\n${compactionGuidance}` : baseMessage,
      rawMessage: null as string | null,
    };
  }

  if (run.error_kind === "loop_detected") {
    return {
      title: "任务疑似卡住，已自动停止",
      message: run.error_message || "系统检测到重复执行模式，已自动停止本轮任务。",
      rawMessage: null as string | null,
    };
  }

  if (run.error_kind === "no_progress") {
    return {
      title: "任务长时间没有进展",
      message: run.error_message || "系统检测到任务在多轮执行后没有明显进展，已自动停止。",
      rawMessage: null as string | null,
    };
  }

  if (run.error_kind === "policy_blocked") {
    return {
      title: "当前任务无法继续执行",
      message: run.error_message || "本次请求触发了安全或工作区限制，系统已停止继续尝试。",
      rawMessage: null as string | null,
    };
  }

  if (isModelErrorKind(run.error_kind)) {
    const display = getModelErrorDisplay(run.error_kind);
    return {
      title: display.title,
      message: run.error_kind === "network" ? `${display.message}${networkRecoverySuffix}` : display.message,
      rawMessage:
        run.error_kind === "unknown" &&
        run.error_message &&
        run.error_message !== display.title &&
        run.error_message !== display.message
          ? run.error_message
          : null,
    };
  }

  const inferredModelErrorKind = run.error_message ? inferModelErrorKindFromMessage(run.error_message) : null;
  if (inferredModelErrorKind) {
    const display = getModelErrorDisplay(inferredModelErrorKind);
    return {
      title: display.title,
      message:
        inferredModelErrorKind === "network"
          ? `${display.message}${networkRecoverySuffix}`
          : display.message,
      rawMessage: null as string | null,
    };
  }

  return {
    title: run.error_message || "本轮执行失败",
    message: "",
    rawMessage: null as string | null,
  };
}

export function renderAgentStateIndicator(agentState: {
  state: string;
} | null): ReactNode {
  if (!agentState) return null;
  if (agentState.state === "stopped") {
    return <span className="inline-flex h-3 w-3 rounded-full bg-amber-400" />;
  }
  if (agentState.state === "error") {
    return <span className="inline-flex h-3 w-3 rounded-full bg-red-400" />;
  }
  return <span className="animate-spin h-3 w-3 border-2 border-blue-400 border-t-transparent rounded-full" />;
}

export function renderAgentStateSecondaryText(agentState: {
  state: string;
  stopReasonMessage?: string | null;
  stopReasonTitle?: string | null;
  detail?: string | null;
  stopReasonLastCompletedStep?: string | null;
} | null) {
  if (!agentState || agentState.state !== "stopped") {
    return null;
  }

  const secondaryLines: string[] = [];
  if (agentState.stopReasonMessage && agentState.stopReasonMessage !== agentState.stopReasonTitle) {
    secondaryLines.push(agentState.stopReasonMessage);
  }
  if (
    agentState.detail &&
    agentState.detail !== agentState.stopReasonTitle &&
    agentState.detail !== agentState.stopReasonMessage
  ) {
    secondaryLines.push(agentState.detail);
  }
  if (agentState.stopReasonLastCompletedStep) {
    secondaryLines.push(`最后完成步骤：${agentState.stopReasonLastCompletedStep}`);
  }
  if (secondaryLines.length === 0) {
    return null;
  }

  return (
    <div className="flex min-w-0 flex-col gap-0.5 text-[11px] text-amber-700">
      {secondaryLines.map((line) => (
        <span key={line} className="whitespace-pre-wrap">
          {line}
        </span>
      ))}
    </div>
  );
}
