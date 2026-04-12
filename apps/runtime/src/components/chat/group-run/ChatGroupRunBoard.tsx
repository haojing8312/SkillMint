import { ChatGroupRunActionPanel } from "./ChatGroupRunActionPanel";
import { ChatGroupRunStepList } from "./ChatGroupRunStepList";
import type { ChatGroupRunBoardProps } from "./chatGroupRunBoardShared";

export function ChatGroupRunBoard({
  groupPhaseLabel,
  groupRound,
  groupReviewRound,
  groupWaitingLabel,
  groupStatusReason,
  groupRunSnapshot,
  onApproveGroupRunReview,
  onRejectGroupRunReview,
  onPauseGroupRun,
  onResumeGroupRun,
  onRetryFailedGroupRunSteps,
  onReassignFailedGroupRunStep,
  groupRunActionLoading,
  canPauseGroupRun,
  canResumeGroupRun,
  canRetryFailedGroupRunSteps,
  canReassignFailedGroupRunStep,
  failedGroupRunReassignOptions,
  groupMemberStates,
  recentGroupEvents,
  groupRunExecuteStepCards,
  highlightedGroupRunStepId,
  highlightedGroupRunStepEventId,
  expandedGroupRunStepIds,
  groupRunStepElementRefs,
  groupRunStepEventElementRefs,
  onToggleGroupRunStepDetails,
  onOpenSession,
}: ChatGroupRunBoardProps) {
  if (!groupPhaseLabel && groupMemberStates.length === 0 && !groupRunSnapshot) {
    return null;
  }

  return (
    <div
      data-testid="group-orchestration-board"
      className="sticky top-0 z-10 max-w-[80%] rounded-xl border border-indigo-200 bg-indigo-50 px-4 py-2 text-xs text-indigo-900"
    >
      <div className="font-medium">{`阶段：${groupPhaseLabel || "计划"}`}</div>
      <div className="mt-1">{`轮次：第 ${groupRound || 1} 轮`}</div>
      {groupReviewRound > 0 && <div className="mt-1">{`审议轮次：${groupReviewRound}`}</div>}
      {groupWaitingLabel && <div className="mt-1">{`等待：${groupWaitingLabel}`}</div>}
      {groupStatusReason && <div className="mt-1 text-amber-700">{groupStatusReason}</div>}
      <ChatGroupRunActionPanel
        groupRunSnapshot={groupRunSnapshot}
        onApproveGroupRunReview={onApproveGroupRunReview}
        onRejectGroupRunReview={onRejectGroupRunReview}
        onPauseGroupRun={onPauseGroupRun}
        onResumeGroupRun={onResumeGroupRun}
        onRetryFailedGroupRunSteps={onRetryFailedGroupRunSteps}
        onReassignFailedGroupRunStep={onReassignFailedGroupRunStep}
        groupRunActionLoading={groupRunActionLoading}
        canPauseGroupRun={canPauseGroupRun}
        canResumeGroupRun={canResumeGroupRun}
        canRetryFailedGroupRunSteps={canRetryFailedGroupRunSteps}
        canReassignFailedGroupRunStep={canReassignFailedGroupRunStep}
        failedGroupRunReassignOptions={failedGroupRunReassignOptions}
        groupMemberStates={groupMemberStates}
      />
      <ChatGroupRunStepList
        groupRunExecuteStepCards={groupRunExecuteStepCards}
        highlightedGroupRunStepId={highlightedGroupRunStepId}
        highlightedGroupRunStepEventId={highlightedGroupRunStepEventId}
        expandedGroupRunStepIds={expandedGroupRunStepIds}
        groupRunStepElementRefs={groupRunStepElementRefs}
        groupRunStepEventElementRefs={groupRunStepEventElementRefs}
        onToggleGroupRunStepDetails={onToggleGroupRunStepDetails}
        onOpenSession={onOpenSession}
      />
      {recentGroupEvents.length > 0 && (
        <div className="mt-2 border-t border-indigo-100 pt-2">
          <div className="text-[11px] font-medium text-indigo-800">最近事件</div>
          <div className="mt-1 space-y-1">
            {recentGroupEvents.map((event) => (
              <div key={event.id} className="text-[11px] text-indigo-800">
                {event.label}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
