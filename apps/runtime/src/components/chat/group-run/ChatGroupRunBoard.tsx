import type { MutableRefObject } from "react";

import type { EmployeeGroupRunSnapshot } from "../../../types";

type GroupRunExecuteStepCard = {
  step: EmployeeGroupRunSnapshot["steps"][number];
  currentAssigneeEmployeeId: string;
  dispatchSourceEmployeeId: string;
  previousAssigneeEmployeeId: string;
  latestFailureSummary: string;
  attemptNo: number;
  detailSessionId: string;
  detailOutputSummary: string;
  latestEventCreatedAt: string;
  sourceStepTimeline: Array<{
    eventId?: string;
    linkedSessionId?: string;
    label: string;
    createdAt?: string;
    openSessionOptions?: {
      focusHint?: string;
      sourceSessionId?: string;
      sourceStepId?: string;
      sourceEmployeeId?: string;
      assigneeEmployeeId?: string;
      sourceStepTimeline?: Array<{
        eventId?: string;
        linkedSessionId?: string;
        label: string;
        createdAt?: string;
      }>;
    };
  }>;
  detailSessionOpenOptions?: {
    focusHint?: string;
    sourceSessionId?: string;
    sourceStepId?: string;
    sourceEmployeeId?: string;
    assigneeEmployeeId?: string;
    sourceStepTimeline?: Array<{
      eventId?: string;
      linkedSessionId?: string;
      label: string;
      createdAt?: string;
    }>;
  };
};

type GroupRunReassignOption = {
  step: EmployeeGroupRunSnapshot["steps"][number];
  candidateEmployeeIds: string[];
};

type ChatGroupRunBoardProps = {
  groupPhaseLabel: string | null;
  groupRound: number;
  groupReviewRound: number;
  groupWaitingLabel: string;
  groupStatusReason: string;
  groupRunSnapshot: EmployeeGroupRunSnapshot | null;
  onApproveGroupRunReview: () => void;
  onRejectGroupRunReview: () => void;
  onPauseGroupRun: () => void;
  onResumeGroupRun: () => void;
  onRetryFailedGroupRunSteps: () => void;
  onReassignFailedGroupRunStep: (stepId: string, employeeId: string) => void;
  groupRunActionLoading: "approve" | "reject" | "pause" | "resume" | "retry" | "reassign" | null;
  canPauseGroupRun: boolean;
  canResumeGroupRun: boolean;
  canRetryFailedGroupRunSteps: boolean;
  canReassignFailedGroupRunStep: boolean;
  failedGroupRunReassignOptions: GroupRunReassignOption[];
  groupMemberStates: Array<{
    role: string;
    status: string;
    stepType?: string;
  }>;
  recentGroupEvents: Array<{
    id: string;
    label: string;
  }>;
  groupRunExecuteStepCards: GroupRunExecuteStepCard[];
  highlightedGroupRunStepId: string | null;
  highlightedGroupRunStepEventId: string | null;
  expandedGroupRunStepIds: string[];
  groupRunStepElementRefs: MutableRefObject<Record<string, HTMLDivElement | null>>;
  groupRunStepEventElementRefs: MutableRefObject<Record<string, HTMLDivElement | HTMLButtonElement | null>>;
  onToggleGroupRunStepDetails: (stepId: string) => void;
  onOpenSession?: (
    sessionId: string,
    options?: {
      focusHint?: string;
      sourceSessionId?: string;
      sourceStepId?: string;
      sourceEmployeeId?: string;
      assigneeEmployeeId?: string;
      sourceStepTimeline?: Array<{
        eventId?: string;
        linkedSessionId?: string;
        label: string;
        createdAt?: string;
      }>;
      groupRunStepFocusId?: string;
      groupRunEventFocusId?: string;
    },
  ) => Promise<void> | void;
  sessionId: string;
};

function formatGroupRunStepStatusLabel(status?: string) {
  const normalized = (status || "").trim().toLowerCase();
  if (normalized === "completed" || normalized === "done") return "已完成";
  if (normalized === "failed") return "失败";
  if (normalized === "running" || normalized === "executing") return "执行中";
  if (normalized === "pending") return "待执行";
  if (normalized === "paused") return "已暂停";
  if (normalized === "cancelled") return "已取消";
  return status?.trim() || "待执行";
}

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
  sessionId,
}: ChatGroupRunBoardProps) {
  return (
    (groupPhaseLabel || groupMemberStates.length > 0 || groupRunSnapshot) ? (
      <div
        data-testid="group-orchestration-board"
        className="sticky top-0 z-10 max-w-[80%] rounded-xl border border-indigo-200 bg-indigo-50 px-4 py-2 text-xs text-indigo-900"
      >
          <div className="font-medium">{`阶段：${groupPhaseLabel || "计划"}`}</div>
          <div className="mt-1">{`轮次：第 ${groupRound || 1} 轮`}</div>
          {groupReviewRound > 0 && <div className="mt-1">{`审议轮次：${groupReviewRound}`}</div>}
          {groupWaitingLabel && <div className="mt-1">{`等待：${groupWaitingLabel}`}</div>}
          {groupStatusReason && <div className="mt-1 text-amber-700">{groupStatusReason}</div>}
          {groupRunSnapshot && (groupRunSnapshot.state || "").trim().toLowerCase() === "waiting_review" && (
            <div className="mt-2 flex items-center gap-2">
              <button
                type="button"
                data-testid="group-run-review-reject"
                onClick={() => void onRejectGroupRunReview()}
                disabled={groupRunActionLoading !== null}
                className="rounded bg-rose-600 px-2.5 py-1 text-[11px] text-white hover:bg-rose-700 disabled:bg-rose-300"
              >
                {groupRunActionLoading === "reject" ? "打回中..." : "打回重审"}
              </button>
              <button
                type="button"
                data-testid="group-run-review-approve"
                onClick={() => void onApproveGroupRunReview()}
                disabled={groupRunActionLoading !== null}
                className="rounded bg-emerald-600 px-2.5 py-1 text-[11px] text-white hover:bg-emerald-700 disabled:bg-emerald-300"
              >
                {groupRunActionLoading === "approve" ? "通过中..." : "通过审议"}
              </button>
            </div>
          )}
          {groupRunSnapshot && (
            <div className="mt-2 flex flex-wrap items-center gap-2">
              {canPauseGroupRun && (
                <button
                  type="button"
                  data-testid="group-run-pause"
                  onClick={() => void onPauseGroupRun()}
                  disabled={groupRunActionLoading !== null}
                  className="rounded bg-slate-600 px-2.5 py-1 text-[11px] text-white hover:bg-slate-700 disabled:bg-slate-300"
                >
                  {groupRunActionLoading === "pause" ? "暂停中..." : "暂停协作"}
                </button>
              )}
              {canResumeGroupRun && (
                <button
                  type="button"
                  data-testid="group-run-resume"
                  onClick={() => void onResumeGroupRun()}
                  disabled={groupRunActionLoading !== null}
                  className="rounded bg-sky-600 px-2.5 py-1 text-[11px] text-white hover:bg-sky-700 disabled:bg-sky-300"
                >
                  {groupRunActionLoading === "resume" ? "继续中..." : "继续协作"}
                </button>
              )}
              {canRetryFailedGroupRunSteps && (
                <button
                  type="button"
                  data-testid="group-run-retry-failed"
                  onClick={() => void onRetryFailedGroupRunSteps()}
                  disabled={groupRunActionLoading !== null}
                  className="rounded bg-amber-600 px-2.5 py-1 text-[11px] text-white hover:bg-amber-700 disabled:bg-amber-300"
                >
                  {groupRunActionLoading === "retry" ? "重试中..." : "重试失败步骤"}
                </button>
              )}
              {canReassignFailedGroupRunStep && (
                <div className="w-full space-y-1.5">
                  {failedGroupRunReassignOptions.map(({ step, candidateEmployeeIds }) => (
                    <div
                      key={step.id}
                      data-testid={`group-run-reassign-row-${step.id}`}
                      className="rounded border border-indigo-200 bg-white/70 px-2.5 py-2"
                    >
                      <div className="text-[11px] font-medium text-indigo-800">{`失败步骤：${step.assignee_employee_id || step.id}`}</div>
                      {(step.dispatch_source_employee_id || "").trim().length > 0 && (
                        <div className="mt-1 text-[10px] text-indigo-700/80">{`来源：${step.dispatch_source_employee_id}`}</div>
                      )}
                      {(step.output || "").trim().length > 0 && (
                        <div className="mt-1 text-[10px] text-indigo-700/80">{step.output}</div>
                      )}
                      <div className="mt-1.5 flex flex-wrap gap-2">
                        {candidateEmployeeIds.map((employeeId) => (
                          <button
                            key={`${step.id}-${employeeId}`}
                            type="button"
                            data-testid={`group-run-reassign-${step.id}-${employeeId}`}
                            onClick={() => void onReassignFailedGroupRunStep(step.id, employeeId)}
                            disabled={groupRunActionLoading !== null}
                            className="rounded bg-fuchsia-600 px-2.5 py-1 text-[11px] text-white hover:bg-fuchsia-700 disabled:bg-fuchsia-300"
                          >
                            {groupRunActionLoading === "reassign" ? "改派中..." : `改派给${employeeId}`}
                          </button>
                        ))}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
          {groupMemberStates.length > 0 && (
            <div className="mt-2 space-y-1">
              {groupMemberStates.map((member) => (
                <div key={member.role} className="text-[11px] text-indigo-800">
                  {member.role}
                  {member.stepType ? ` · ${member.stepType}` : ""}
                  {` · ${member.status}`}
                </div>
              ))}
            </div>
          )}
          {groupRunExecuteStepCards.length > 0 && (
            <div className="mt-2 border-t border-indigo-100 pt-2">
              <div className="text-[11px] font-medium text-indigo-800">步骤链路</div>
              <div className="mt-1 space-y-1.5">
                {groupRunExecuteStepCards.map(
                  ({
                    step,
                    currentAssigneeEmployeeId,
                    dispatchSourceEmployeeId,
                    previousAssigneeEmployeeId,
                    latestFailureSummary,
                    attemptNo,
                    detailSessionId,
                    detailSessionOpenOptions,
                    detailOutputSummary,
                    latestEventCreatedAt,
                    sourceStepTimeline,
                  }) => {
                    const isGroupRunStepFocusTarget = highlightedGroupRunStepId === step.id;
                    return (
                      <div
                        key={step.id}
                        ref={(node) => {
                          groupRunStepElementRefs.current[step.id] = node;
                        }}
                        data-testid={`group-run-step-card-${step.id}`}
                        data-group-run-step-highlighted={isGroupRunStepFocusTarget ? "true" : "false"}
                        className={
                          "rounded border border-indigo-200 bg-white/70 px-2.5 py-2 transition-all " +
                          (isGroupRunStepFocusTarget ? "ring-2 ring-amber-300 bg-amber-50/80 " : "")
                        }
                      >
                        <div className="text-[11px] font-medium text-indigo-800">{step.assignee_employee_id || step.id}</div>
                        <div className="mt-1 text-[10px] text-indigo-700/80">{`当前负责人：${currentAssigneeEmployeeId || "未分配"}`}</div>
                        <div className="mt-1 text-[10px] text-indigo-700/80">{`当前状态：${formatGroupRunStepStatusLabel(step.status)}`}</div>
                        <div className="mt-1 text-[10px] text-indigo-700/80">{`尝试次数：${attemptNo}`}</div>
                        {dispatchSourceEmployeeId && (
                          <div className="mt-1 text-[10px] text-indigo-700/80">{`来源人：${dispatchSourceEmployeeId}`}</div>
                        )}
                        {previousAssigneeEmployeeId &&
                          previousAssigneeEmployeeId.toLowerCase() !== currentAssigneeEmployeeId.toLowerCase() && (
                            <div className="mt-1 text-[10px] text-indigo-700/80">{`原负责人：${previousAssigneeEmployeeId}`}</div>
                          )}
                        {latestFailureSummary && (
                          <div className="mt-1 text-[10px] text-amber-700/90">{`最近失败：${latestFailureSummary}`}</div>
                        )}
                        <button
                          type="button"
                          data-testid={`group-run-step-card-${step.id}-toggle`}
                          onClick={() => onToggleGroupRunStepDetails(step.id)}
                          className="mt-2 text-[10px] text-indigo-700 underline underline-offset-2 hover:text-indigo-800"
                        >
                          {expandedGroupRunStepIds.includes(step.id) ? "收起详情" : "查看详情"}
                        </button>
                        {expandedGroupRunStepIds.includes(step.id) && (
                          <div
                            data-testid={`group-run-step-card-${step.id}-details`}
                            className="mt-2 space-y-1 rounded border border-indigo-100 bg-indigo-50/60 px-2 py-1.5 text-[10px] text-indigo-800"
                          >
                            <div>{`session_id：${detailSessionId || "暂无"}`}</div>
                            <div>{`输出摘要：${detailOutputSummary || "暂无"}`}</div>
                            <div>{`最近事件时间：${latestEventCreatedAt || "暂无"}`}</div>
                            {sourceStepTimeline.length > 0 && (
                              <div className="space-y-1">
                                <div className="font-medium text-indigo-800">步骤事件</div>
                                {sourceStepTimeline.map((item, index) => {
                                  const eventId = (item.eventId || "").trim();
                                  const linkedSessionId = (item.linkedSessionId || "").trim();
                                  const isGroupRunEventFocusTarget =
                                    eventId.length > 0 && highlightedGroupRunStepEventId === eventId;
                                  const eventLabel = item.createdAt ? `${item.label} · ${item.createdAt}` : item.label;
                                  const eventKey = `${eventId || item.label}-${item.createdAt || index}`;
                                  const commonProps = {
                                    ref: (node: HTMLDivElement | HTMLButtonElement | null) => {
                                      if (eventId) {
                                        groupRunStepEventElementRefs.current[eventId] = node as HTMLDivElement | null;
                                      }
                                    },
                                    "data-testid": `group-run-step-card-${step.id}-event-${eventId || index}`,
                                    "data-group-run-step-event-linkable":
                                      linkedSessionId && onOpenSession ? "true" : "false",
                                    "data-group-run-step-event-highlighted": isGroupRunEventFocusTarget ? "true" : "false",
                                    className:
                                      "rounded px-1.5 py-1 transition-all flex items-center justify-between gap-2 " +
                                      (isGroupRunEventFocusTarget ? "bg-amber-100 ring-1 ring-amber-300 " : "") +
                                      (linkedSessionId && onOpenSession
                                        ? " w-full text-left border border-sky-200 bg-white text-sky-900 underline underline-offset-2 hover:bg-sky-50"
                                        : " border border-indigo-100 bg-white/60 text-indigo-700/90"),
                                  } as const;
                                  return linkedSessionId && onOpenSession ? (
                                    <button
                                      key={eventKey}
                                      {...commonProps}
                                      type="button"
                                      onClick={() => void onOpenSession(linkedSessionId, item.openSessionOptions)}
                                    >
                                      <span className="min-w-0 flex-1 truncate">{eventLabel}</span>
                                      <span className="shrink-0 rounded bg-sky-100 px-1.5 py-0.5 text-[9px] font-medium text-sky-700">
                                        执行会话
                                      </span>
                                    </button>
                                  ) : (
                                    <div key={eventKey} {...commonProps}>
                                      <span className="min-w-0 flex-1 truncate">{eventLabel}</span>
                                      <span className="shrink-0 rounded bg-indigo-100 px-1.5 py-0.5 text-[9px] font-medium text-indigo-700">
                                        日志
                                      </span>
                                    </div>
                                  );
                                })}
                              </div>
                            )}
                            {onOpenSession && detailSessionId && (
                              <button
                                type="button"
                                data-testid={`group-run-step-card-${step.id}-open-session`}
                                onClick={() => void onOpenSession(detailSessionId, detailSessionOpenOptions)}
                                className="text-[10px] text-indigo-700 underline underline-offset-2 hover:text-indigo-800"
                              >
                                查看执行会话
                              </button>
                            )}
                          </div>
                        )}
                      </div>
                    );
                  },
                )}
              </div>
            </div>
          )}
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
    ) : null
  );
}
