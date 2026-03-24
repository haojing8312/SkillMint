import type { ReactNode } from "react";

type ChatShellProps = {
  header: ReactNode;
  executionContextBar?: ReactNode;
  mainContent: ReactNode;
  sidePanel: ReactNode;
  composer: ReactNode;
};

export function ChatShell({ header, executionContextBar, mainContent, sidePanel, composer }: ChatShellProps) {
  return (
    <div className="flex h-full flex-col">
      {header}
      {executionContextBar}
      <div className="flex flex-1 overflow-hidden bg-[#f7f7f4]">
        {mainContent}
        {sidePanel}
      </div>
      {composer}
    </div>
  );
}
