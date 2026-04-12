import { TaskTabStrip, type TaskTabStripItem } from "./TaskTabStrip";
import { AppMainSceneContent } from "./AppMainSceneContent";
import type { AppMainContentProps } from "./AppMainContent.types";

export type { AppMainContentProps } from "./AppMainContent.types";

export function AppMainContent(props: AppMainContentProps) {
  const { showSettings, activeMainView, taskTabs, activeTabId, onSelectTab, onCreateTab, onCloseTab } = props;

  return (
    <div className="flex-1 overflow-hidden flex flex-col">
      {!showSettings && activeMainView === "start-task" ? (
        <TaskTabStrip
          tabs={taskTabs}
          activeTabId={activeTabId}
          onSelectTab={onSelectTab}
          onCreateTab={onCreateTab}
          onCloseTab={onCloseTab}
        />
      ) : null}
      <div className="flex-1 overflow-hidden">
        <AppMainSceneContent {...props} />
      </div>
    </div>
  );
}
