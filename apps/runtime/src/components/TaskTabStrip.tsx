import { LoaderCircle, Plus, ShieldAlert, X } from "lucide-react";

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
  if (normalized === "running") {
    return {
      label: "运行中",
      className: "text-blue-600",
      icon: <LoaderCircle className="h-3.5 w-3.5 animate-spin" />,
    };
  }
  if (normalized === "waiting_approval") {
    return {
      label: "待确认",
      className: "text-amber-600",
      icon: <ShieldAlert className="h-3.5 w-3.5" />,
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
    <div className="border-b border-[var(--sm-border)] bg-[var(--sm-surface)] px-3 py-2">
      <div className="flex items-center gap-2 overflow-x-auto" role="tablist" aria-label="任务标签">
        {tabs.map((tab) => {
          const active = tab.id === activeTabId;
          const runtimeBadge = getRuntimeBadge(tab.runtimeStatus);
          return (
            <div
              key={tab.id}
              className={
                "group inline-flex min-w-0 max-w-[240px] items-center gap-2 rounded-xl border px-3 py-2 text-sm transition-colors " +
                (active
                  ? "border-[var(--sm-primary)] bg-[var(--sm-primary-soft)] text-[var(--sm-primary-strong)]"
                  : "border-[var(--sm-border)] bg-white text-[var(--sm-text-muted)] hover:border-[var(--sm-primary)] hover:bg-[var(--sm-surface-soft)]")
              }
            >
              <button
                type="button"
                role="tab"
                aria-selected={active}
                className="flex min-w-0 flex-1 items-center gap-2 text-left"
                onClick={() => onSelectTab(tab.id)}
              >
                {runtimeBadge ? (
                  <span className={`inline-flex flex-shrink-0 ${runtimeBadge.className}`}>
                    {runtimeBadge.icon}
                  </span>
                ) : null}
                <span className="truncate">{tab.title}</span>
              </button>
              <button
                type="button"
                className="sm-btn sm-btn-ghost h-6 w-6 flex-shrink-0 rounded-md opacity-70 hover:opacity-100"
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
          className="sm-btn sm-btn-secondary h-10 w-10 flex-shrink-0 rounded-xl"
          aria-label="新建任务标签"
          onClick={onCreateTab}
        >
          <Plus className="h-4 w-4" />
        </button>
      </div>
    </div>
  );
}
