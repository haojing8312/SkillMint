type ChatSessionTimelineItem = {
  eventId?: string;
  label: string;
  createdAt?: string;
};

type ChatSessionExecutionContext = {
  sourceSessionId: string;
  sourceStepId: string;
  sourceEmployeeId?: string;
  assigneeEmployeeId?: string;
  sourceStepTimeline?: ChatSessionTimelineItem[];
};

type ChatExecutionContextBarProps = {
  sessionExecutionContext: ChatSessionExecutionContext;
  onOpenSession?: (
    sessionId: string,
    options?: {
      groupRunStepFocusId?: string;
      groupRunEventFocusId?: string;
    },
  ) => Promise<void> | void;
  onReturnToSourceSession?: (sessionId: string) => Promise<void> | void;
};

export function ChatExecutionContextBar({
  sessionExecutionContext,
  onOpenSession,
  onReturnToSourceSession,
}: ChatExecutionContextBarProps) {
  return (
    <div data-testid="chat-session-execution-context-bar" className="border-b border-sky-100 bg-sky-50/80 text-[11px] text-sky-900">
      <div className="mx-auto flex w-full max-w-[76rem] flex-wrap items-center justify-between gap-2 px-5 py-2 lg:px-8">
        <div className="flex min-w-0 flex-1 flex-col gap-1">
          <div className="flex flex-wrap items-center gap-3">
            <span>{`来源 step：${sessionExecutionContext.sourceStepId}`}</span>
            {sessionExecutionContext.sourceEmployeeId && <span>{`来源员工：${sessionExecutionContext.sourceEmployeeId}`}</span>}
            {sessionExecutionContext.assigneeEmployeeId && <span>{`当前负责人：${sessionExecutionContext.assigneeEmployeeId}`}</span>}
          </div>
          {(sessionExecutionContext.sourceStepTimeline || []).length > 0 && (
            <div data-testid="chat-session-execution-context-timeline" className="space-y-1 text-[10px] text-sky-800/90">
              {(sessionExecutionContext.sourceStepTimeline || []).map((item, index) => {
                const label = item.createdAt ? `${item.label} · ${item.createdAt}` : item.label;
                return onOpenSession ? (
                  <button
                    key={`${item.label}-${item.createdAt || index}`}
                    type="button"
                    data-testid={`chat-session-execution-context-timeline-item-${index}`}
                    onClick={() =>
                      void onOpenSession(sessionExecutionContext.sourceSessionId, {
                        groupRunStepFocusId: sessionExecutionContext.sourceStepId,
                        groupRunEventFocusId: item.eventId,
                      })
                    }
                    className="block text-left underline underline-offset-2 hover:text-sky-900"
                  >
                    {label}
                  </button>
                ) : (
                  <div key={`${item.label}-${item.createdAt || index}`} data-testid={`chat-session-execution-context-timeline-item-${index}`}>
                    {label}
                  </div>
                );
              })}
            </div>
          )}
        </div>
        <button
          type="button"
          data-testid="chat-session-execution-context-back"
          onClick={() => void onReturnToSourceSession?.(sessionExecutionContext.sourceSessionId)}
          className="text-[11px] font-medium text-sky-700 underline underline-offset-2 hover:text-sky-800"
        >
          返回协作看板
        </button>
      </div>
    </div>
  );
}
