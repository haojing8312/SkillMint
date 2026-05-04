import type {
  ChatRuntimeAgentState,
  ChatRuntimeCompactionStatus,
  SendMessageRequest,
  StreamItem,
} from "../../types";
import type { ChatStreamReasoningState } from "../../scenes/chat/chatStreamControllerTypes";

export function buildScrollJumpViewModel({
  hasScrollableContent,
  isNearBottom,
  isNearTop,
}: {
  hasScrollableContent: boolean;
  isNearBottom: boolean;
  isNearTop: boolean;
}) {
  return {
    showScrollJump: hasScrollableContent || !isNearBottom,
    scrollJumpLabel: isNearBottom ? "跳转到顶部" : "跳转到底部",
    scrollJumpHint: isNearBottom ? (isNearTop ? "当前已在顶部" : "返回顶部") : "回到底部并继续跟随",
  };
}

export function buildStreamingAssistantViewModel({
  streamReasoning,
  agentState,
  thinkingIndicator,
  streamItems,
  subAgentBuffer,
}: {
  streamReasoning: ChatStreamReasoningState;
  agentState: ChatRuntimeAgentState | null;
  thinkingIndicator: boolean;
  streamItems: StreamItem[];
  subAgentBuffer: string;
}) {
  const showStreamingThinkingState =
    Boolean(streamReasoning) || (agentState?.state === "thinking" && thinkingIndicator);
  return {
    showStreamingThinkingState,
    showStreamingAssistantBubble:
      showStreamingThinkingState || streamItems.length > 0 || subAgentBuffer.length > 0,
  };
}

export function getLiveBlockingStatus({
  pendingApprovalCount,
  agentState,
  streamReasoning,
  compactionStatus,
  streaming,
  streamItems,
  subAgentBuffer,
}: {
  pendingApprovalCount: number;
  agentState: ChatRuntimeAgentState | null;
  streamReasoning: ChatStreamReasoningState;
  compactionStatus: ChatRuntimeCompactionStatus | null;
  streaming: boolean;
  streamItems: StreamItem[];
  subAgentBuffer: string;
}) {
  if (pendingApprovalCount > 0 || agentState?.state === "waiting_approval") {
    return "waiting_approval";
  }
  if (agentState?.state === "thinking" || streamReasoning?.status === "thinking") {
    return "thinking";
  }
  if (agentState?.state === "tool_calling") {
    return "tool_calling";
  }
  if (compactionStatus?.phase === "started" || compactionStatus?.phase === "completed") {
    return "thinking";
  }
  if (streaming || streamItems.length > 0 || subAgentBuffer.trim()) {
    return "running";
  }
  return null;
}

export function shouldShowTeamEntryEmptyState({
  isTeamEntrySession,
  initialMessage,
  messageCount,
  streamItemCount,
  subAgentBuffer,
  streaming,
  hasGroupRunSnapshot,
}: {
  isTeamEntrySession: boolean;
  initialMessage?: string;
  messageCount: number;
  streamItemCount: number;
  subAgentBuffer: string;
  streaming: boolean;
  hasGroupRunSnapshot: boolean;
}) {
  return (
    isTeamEntrySession &&
    !initialMessage?.trim() &&
    messageCount === 0 &&
    streamItemCount === 0 &&
    !subAgentBuffer.trim() &&
    !streaming &&
    !hasGroupRunSnapshot
  );
}

export function shouldGrantContinuationBudgetRequest(
  request: SendMessageRequest,
  hasLatestMaxTurnsRun: boolean,
) {
  if (!hasLatestMaxTurnsRun) return false;
  if (request.parts.length !== 1) return false;
  const [part] = request.parts;
  if (part.type !== "text") return false;
  const normalized = part.text.trim().toLowerCase();
  return normalized === "继续" || normalized === "继续执行" || normalized === "continue";
}
