type ChatHeaderProps = {
  sessionDisplayTitle: string;
  sessionDisplaySubtitle: string;
  isImSource: boolean;
  sessionSourceBadgeText: string;
  sidePanelOpen: boolean;
  onToggleSidePanel: () => void;
};

export function ChatHeader({
  sessionDisplayTitle,
  sessionDisplaySubtitle,
  isImSource,
  sessionSourceBadgeText,
  sidePanelOpen,
  onToggleSidePanel,
}: ChatHeaderProps) {
  return (
    <div className="px-4 pt-4 sm:px-6 xl:px-8">
      <div className="flex w-full items-start justify-between gap-4">
        <div className="min-w-0">
          <div
            data-testid="chat-session-display-title"
            className="truncate text-[22px] font-semibold tracking-tight text-[var(--sm-text)]"
          >
            {sessionDisplayTitle}
          </div>
          {(sessionDisplaySubtitle || isImSource) && (
            <div className="mt-1 flex flex-wrap items-center gap-2 text-[11px] text-[var(--sm-text-muted)]">
              {sessionDisplaySubtitle ? (
                <div data-testid="chat-session-display-subtitle" className="truncate">
                  {sessionDisplaySubtitle}
                </div>
              ) : null}
              {isImSource && (
                <span
                  data-testid="chat-session-source-badge"
                  title={`该会话由${sessionSourceBadgeText}触发`}
                  className="inline-flex items-center rounded-full border border-[var(--sm-border)] bg-[var(--sm-surface)] px-2 py-0.5 font-medium text-[var(--sm-text-muted)]"
                >
                  {sessionSourceBadgeText}
                </span>
              )}
            </div>
          )}
        </div>
        <button
          type="button"
          aria-label="面板"
          title="面板"
          data-testid="chat-side-panel-trigger"
          aria-pressed={sidePanelOpen}
          onClick={onToggleSidePanel}
          className={
            "sm-btn ml-auto h-10 w-10 rounded-xl border transition-colors " +
            (sidePanelOpen
              ? "border-[var(--sm-primary-soft)] bg-[var(--sm-primary-soft)] text-[var(--sm-primary-strong)]"
              : "border-[var(--sm-border)] bg-[var(--sm-surface)] text-[var(--sm-text-muted)] hover:bg-[var(--sm-surface-soft)] hover:text-[var(--sm-text)]")
          }
        >
          <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M9 17V7m0 10a2 2 0 01-2 2H5a2 2 0 01-2-2V7a2 2 0 012-2h2a2 2 0 012 2m0 10a2 2 0 002 2h2a2 2 0 002-2M9 7a2 2 0 012-2h2a2 2 0 012 2m0 10V7m0 10a2 2 0 002 2h2a2 2 0 002-2V7a2 2 0 00-2-2h-2a2 2 0 00-2 2"
            />
          </svg>
        </button>
      </div>
    </div>
  );
}
