import type { ComponentProps, UIEventHandler, RefObject } from "react";

import { ChatActionDialogs } from "./ChatActionDialogs";
import { ChatAgentStateBanner } from "./ChatAgentStateBanner";
import { ChatCollaborationStatusPanel } from "./ChatCollaborationStatusPanel";
import { ChatEmployeeAssistantContext } from "./ChatEmployeeAssistantContext";
import { ChatLinkToast } from "./ChatLinkToast";
import { ChatMessageRail } from "./ChatMessageRail";
import { ChatScrollJumpButton } from "./ChatScrollJumpButton";
import { ChatGroupRunSection } from "./group-run/ChatGroupRunSection";

type ChatMainContentProps = {
  scrollRegionRef: RefObject<HTMLDivElement>;
  bottomRef: RefObject<HTMLDivElement>;
  onScroll: UIEventHandler<HTMLDivElement>;
  employeeAssistantContext: ComponentProps<typeof ChatEmployeeAssistantContext>["employeeAssistantContext"];
  agentBanner: ComponentProps<typeof ChatAgentStateBanner>;
  collaborationStatusPanel: ComponentProps<typeof ChatCollaborationStatusPanel>;
  groupRunSection: ComponentProps<typeof ChatGroupRunSection>;
  messageRail: ComponentProps<typeof ChatMessageRail>;
  linkToast: ComponentProps<typeof ChatLinkToast>;
  actionDialogs: ComponentProps<typeof ChatActionDialogs>;
  scrollJump: ComponentProps<typeof ChatScrollJumpButton>;
};

export function ChatMainContent({
  scrollRegionRef,
  bottomRef,
  onScroll,
  employeeAssistantContext,
  agentBanner,
  collaborationStatusPanel,
  groupRunSection,
  messageRail,
  linkToast,
  actionDialogs,
  scrollJump,
}: ChatMainContentProps) {
  return (
    <div className="relative flex-1 bg-[#f7f7f4]">
      <div
        ref={scrollRegionRef}
        data-testid="chat-scroll-region"
        onScroll={onScroll}
        className="h-full overflow-y-auto bg-transparent px-4 py-6 sm:px-6 xl:px-8"
      >
        <div data-testid="chat-content-rail" className="mx-auto flex w-full max-w-[76rem] flex-col gap-5">
          <ChatEmployeeAssistantContext employeeAssistantContext={employeeAssistantContext} />
          <ChatAgentStateBanner {...agentBanner} />
          <ChatCollaborationStatusPanel {...collaborationStatusPanel} />
          <ChatGroupRunSection {...groupRunSection} />
          <ChatMessageRail {...messageRail} />
          <ChatLinkToast {...linkToast} />
          <ChatActionDialogs {...actionDialogs} />
          <div ref={bottomRef} />
        </div>
      </div>
      <ChatScrollJumpButton {...scrollJump} />
    </div>
  );
}
