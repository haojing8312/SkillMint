import type { MutableRefObject } from "react";

import type { GroupRunExecuteStepCard, GroupRunOpenSessionOptions } from "./chatGroupRunBoardShared";
import { formatGroupRunStepStatusLabel } from "./chatGroupRunBoardShared";

type ChatGroupRunStepListProps = {
  groupRunExecuteStepCards: GroupRunExecuteStepCard[];
  highlightedGroupRunStepId: string | null;
  highlightedGroupRunStepEventId: string | null;
  expandedGroupRunStepIds: string[];
  groupRunStepElementRefs: MutableRefObject<Record<string, HTMLDivElement | null>>;
  groupRunStepEventElementRefs: MutableRefObject<Record<string, HTMLDivElement | HTMLButtonElement | null>>;
  onToggleGroupRunStepDetails: (stepId: string) => void;
  onOpenSession?: (sessionId: string, options?: GroupRunOpenSessionOptions) => Promise<void> | void;
};

export function ChatGroupRunStepList({
  groupRunExecuteStepCards,
  highlightedGroupRunStepId,
  highlightedGroupRunStepEventId,
  expandedGroupRunStepIds,
  groupRunStepElementRefs,
  groupRunStepEventElementRefs,
  onToggleGroupRunStepDetails,
  onOpenSession,
}: ChatGroupRunStepListProps) {
  if (groupRunExecuteStepCards.length === 0) {
    return null;
  }

  return (
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
                            "data-group-run-step-event-linkable": linkedSessionId && onOpenSession ? "true" : "false",
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
  );
}
