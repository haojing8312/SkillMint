import { memo, useMemo } from "react";
import { motion } from "framer-motion";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

import { ThinkingBlock } from "../ThinkingBlock";
import type { SessionToolManifestEntry, StreamItem } from "../../types";
import { createChatMarkdownComponents } from "./chatMarkdownComponents";
import { extractPlainTextFromStreamItems, renderChatStreamItems } from "./chatMessageRailHelpers";

type ChatStreamingAssistantBubbleProps = {
  showStreamingThinkingState: boolean;
  streamReasoning:
    | {
        status: "thinking" | "completed" | "interrupted";
        content: string;
        durationMs?: number;
      }
    | null;
  expandedThinkingKeys: string[];
  onToggleThinkingBlock: (key: string) => void;
  streamItems: StreamItem[];
  toolManifest: SessionToolManifestEntry[];
  subAgentBuffer: string;
  subAgentRoleName: string;
  copiedAssistantMessageKey: string | null;
  onCopyAssistantMessage: (messageKey: string, content: string) => Promise<void> | void;
  CopyActionIcon: (props: { copied: boolean }) => React.ReactNode;
  onOpenExternalLink?: (url: string) => Promise<void> | void;
};

function ChatStreamingAssistantBubbleImpl({
  showStreamingThinkingState,
  streamReasoning,
  expandedThinkingKeys,
  onToggleThinkingBlock,
  streamItems,
  toolManifest,
  subAgentBuffer,
  subAgentRoleName,
  copiedAssistantMessageKey,
  onCopyAssistantMessage,
  CopyActionIcon,
  onOpenExternalLink,
}: ChatStreamingAssistantBubbleProps) {
  const markdownComponents = useMemo(() => createChatMarkdownComponents(onOpenExternalLink), [onOpenExternalLink]);
  const streamText = useMemo(() => extractPlainTextFromStreamItems(streamItems), [streamItems]);

  return (
    <motion.div initial={{ opacity: 0, x: -20 }} animate={{ opacity: 1, x: 0 }} className="flex justify-start">
      <div
        data-testid="chat-streaming-bubble"
        className="w-full max-w-[92%] px-0 py-1 text-sm text-slate-800 sm:max-w-[88%] md:max-w-[48rem] xl:max-w-[52rem]"
      >
        {showStreamingThinkingState && (
          <ThinkingBlock
            status={streamReasoning?.status || "thinking"}
            content={streamReasoning?.content || ""}
            durationMs={streamReasoning?.durationMs}
            expanded={expandedThinkingKeys.includes("stream")}
            onToggle={(streamReasoning?.content || "").trim() ? () => onToggleThinkingBlock("stream") : undefined}
          />
        )}
        {streamItems.length > 0 &&
          renderChatStreamItems({
            items: streamItems,
            subAgentBuffer,
            markdownComponents,
            toolManifest,
          })}
        {subAgentBuffer && (
          <div
            data-testid="sub-agent-stream-buffer"
            className="mt-2 rounded-xl border border-slate-200/80 bg-slate-50/80 px-3 py-2"
          >
            <div className="mb-1 text-[11px] font-semibold text-slate-600">
              {subAgentRoleName ? `子员工 · ${subAgentRoleName}` : "子员工"}
            </div>
            <div className="prose prose-xs max-w-none text-slate-700">
              <ReactMarkdown remarkPlugins={[remarkGfm]} components={markdownComponents}>
                {subAgentBuffer}
              </ReactMarkdown>
              <span className="animate-pulse text-slate-400">|</span>
            </div>
          </div>
        )}
        {streamItems.length > 0 && (
          <span className="inline-block h-4 w-0.5 animate-[blink_1s_infinite] align-middle bg-blue-400 ml-0.5" />
        )}
        {streamText.trim() && (
          <div className="mt-3 flex items-center justify-end gap-2">
            <button
              type="button"
              data-testid="assistant-copy-action-stream"
              aria-label="复制回答"
              title={copiedAssistantMessageKey === "stream" ? "已复制" : "复制回答"}
              onClick={() => void onCopyAssistantMessage("stream", streamText)}
              className="inline-flex h-9 w-9 items-center justify-center rounded-full text-slate-400 transition-colors hover:bg-slate-100 hover:text-slate-600 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-slate-300 focus-visible:ring-offset-2"
            >
              <CopyActionIcon copied={copiedAssistantMessageKey === "stream"} />
            </button>
          </div>
        )}
      </div>
    </motion.div>
  );
}

export const ChatStreamingAssistantBubble = memo(ChatStreamingAssistantBubbleImpl);
