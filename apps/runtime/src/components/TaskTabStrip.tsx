import { Plus, X } from "lucide-react";

export type TaskTabStripItem = {
  id: string;
  kind: "start-task" | "session";
  title: string;
  runtimeStatus?: string | null;
};

type Props = {
  tabs: TaskTabStripItem[];
  activeTabId: string | null;
  onSelectTab: (tabId: string) => void;
  onCreateTab: () => void;
  onCloseTab: (tabId: string) => void;
};

function getRuntimeBadge(status?: string | null) {
  const normalized = (status || "").trim().toLowerCase();
  if (normalized === "thinking") {
    return {
      label: "思考中",
      dotClassName: "bg-violet-500",
    };
  }
  if (normalized === "running") {
    return {
      label: "执行中",
      dotClassName: "bg-sky-500",
    };
  }
  if (normalized === "tool_calling") {
    return {
      label: "处理中",
      dotClassName: "bg-cyan-500",
    };
  }
  if (normalized === "waiting_approval") {
    return {
      label: "待确认",
      dotClassName: "bg-amber-500",
    };
  }
  if (normalized === "completed" || normalized === "done") {
    return {
      label: "已完成",
      dotClassName: "bg-emerald-500",
    };
  }
  if (normalized === "failed" || normalized === "error" || normalized === "cancelled") {
    return {
      label: "异常",
      dotClassName: "bg-rose-500",
    };
  }
  return null;
}

export function TaskTabStrip({
  tabs,
  activeTabId,
  onSelectTab,
  onCreateTab,
  onCloseTab,
}: Props) {
  return (
    <div className="border-b border-[var(--sm-border)] bg-white">
      <div className="flex items-stretch overflow-x-auto" role="tablist" aria-label="任务标签">
        {tabs.map((tab) => {
          const active = tab.id === activeTabId;
          const runtimeBadge = getRuntimeBadge(tab.runtimeStatus);
          return (
            <div
              key={tab.id}
              className={
                "group inline-flex min-w-0 max-w-[240px] items-center border-r border-[var(--sm-border)] text-sm transition-colors " +
                (active
                  ? "bg-white text-[var(--sm-text)]"
                  : "bg-[#fbfbfc] text-[var(--sm-text-muted)] hover:bg-[var(--sm-surface-soft)] hover:text-[var(--sm-text)]")
              }
            >
              <button
                type="button"
                role="tab"
                aria-label={tab.title}
                aria-selected={active}
                className="flex min-w-0 flex-1 items-center gap-2 px-5 py-4 text-left outline-none"
                onClick={() => onSelectTab(tab.id)}
              >
                <span
                  aria-hidden="true"
                  className={
                    "h-2.5 w-2.5 flex-shrink-0 rounded-full " +
                    (runtimeBadge?.dotClassName || (active ? "bg-[var(--sm-primary)]" : "bg-[var(--sm-border)]")) +
                    (runtimeBadge && ["thinking", "running", "tool_calling"].includes((tab.runtimeStatus || "").trim().toLowerCase())
                      ? " animate-pulse"
                      : "")
                  }
                />
                <span className="truncate font-medium">{tab.title}</span>
                {runtimeBadge ? <span className="sr-only">{runtimeBadge.label}</span> : null}
              </button>
              <button
                type="button"
                className="sm-btn sm-btn-ghost mr-3 h-7 w-7 flex-shrink-0 rounded-md text-[var(--sm-text-muted)] opacity-80 transition-colors hover:bg-[var(--sm-surface-soft)] hover:text-[var(--sm-text)] focus-visible:text-[var(--sm-text)]"
                aria-label={`关闭标签 ${tab.title}`}
                onClick={(event) => {
                  event.stopPropagation();
                  onCloseTab(tab.id);
                }}
              >
                <X className="h-3.5 w-3.5" />
              </button>
            </div>
          );
        })}
        <button
          type="button"
          className="sm-btn sm-btn-ghost h-auto min-h-[56px] min-w-[56px] flex-shrink-0 rounded-none border-r border-[var(--sm-border)] bg-white px-4 text-[var(--sm-text-muted)] transition-colors hover:bg-[var(--sm-surface-soft)] hover:text-[var(--sm-text)]"
          aria-label="新建任务标签"
          onClick={onCreateTab}
        >
          <Plus className="h-4 w-4" />
        </button>
      </div>
    </div>
  );
}
