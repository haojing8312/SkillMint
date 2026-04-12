import { useMemo } from "react";

import type { Message, SessionRunProjection, StreamItem } from "../../types";
import { computeVirtualWindow } from "./chatVirtualization";
import {
  buildTaskPanelViewModel,
  buildWebSearchViewModel,
  extractSessionTouchedFiles,
} from "../chat-side-panel/view-model";

type UseChatDerivedViewModelsArgs = {
  messages: Message[];
  sessionRuns: SessionRunProjection[];
  streamItems: StreamItem[];
  highlightedMessageIndex: number | null;
  scrollTop: number;
  viewportHeight: number;
};

export function useChatDerivedViewModels({
  messages,
  sessionRuns,
  streamItems,
  highlightedMessageIndex,
  scrollTop,
  viewportHeight,
}: UseChatDerivedViewModelsArgs) {
  const recoverableSessionRun = useMemo(() => {
    return [...sessionRuns]
      .reverse()
      .find((run) => {
        const status = (run.status || "").trim().toLowerCase();
        const hasAssistantMessage = (run.assistant_message_id || "").trim().length > 0;
        const bufferedText = (run.buffered_text || "").trim();
        return (
          !hasAssistantMessage &&
          bufferedText.length > 0 &&
          ["thinking", "tool_calling", "waiting_approval"].includes(status)
        );
      }) ?? null;
  }, [sessionRuns]);

  const recoveredAssistantMessage = useMemo<Message | null>(() => {
    if (!recoverableSessionRun) return null;
    return {
      id: `recovered-run-${recoverableSessionRun.id}`,
      role: "assistant",
      content: recoverableSessionRun.buffered_text,
      created_at: recoverableSessionRun.updated_at || recoverableSessionRun.created_at,
      runId: recoverableSessionRun.id,
    };
  }, [recoverableSessionRun]);

  const renderedMessages = useMemo<Message[]>(
    () => (recoveredAssistantMessage ? [...messages, recoveredAssistantMessage] : messages),
    [messages, recoveredAssistantMessage],
  );

  const estimatedRenderedMessageHeights = useMemo(
    () =>
      renderedMessages.map((message) => {
        if (message.role === "user") {
          return message.contentParts?.length ? 140 : 92;
        }
        if (message.streamItems?.length) {
          return 280;
        }
        if (message.toolCalls?.length) {
          return 240;
        }
        if (message.reasoning?.content?.trim()) {
          return 260;
        }
        const lineCount = Math.max(1, Math.ceil((message.content || "").length / 48));
        return Math.min(360, 120 + lineCount * 22);
      }),
    [renderedMessages],
  );

  const virtualWindow = useMemo(
    () =>
      computeVirtualWindow({
        itemCount: renderedMessages.length,
        itemHeights: estimatedRenderedMessageHeights,
        scrollTop,
        viewportHeight,
        overscan: 6,
        minVirtualizeCount: 40,
        forceIncludeIndex: highlightedMessageIndex,
      }),
    [estimatedRenderedMessageHeights, highlightedMessageIndex, renderedMessages.length, scrollTop, viewportHeight],
  );

  const virtualizedRenderedMessages = useMemo(
    () => renderedMessages.slice(virtualWindow.startIndex, virtualWindow.endIndex),
    [renderedMessages, virtualWindow.endIndex, virtualWindow.startIndex],
  );

  const sidePanelMessages = useMemo<Message[]>(() => {
    if (streamItems.length === 0) return renderedMessages;
    return [
      ...renderedMessages,
      {
        role: "assistant",
        content: "",
        created_at: new Date().toISOString(),
        streamItems,
      },
    ];
  }, [renderedMessages, streamItems]);

  const taskPanelModel = useMemo(() => buildTaskPanelViewModel(sidePanelMessages), [sidePanelMessages]);
  const webSearchEntries = useMemo(() => buildWebSearchViewModel(sidePanelMessages), [sidePanelMessages]);
  const touchedFilePaths = useMemo(
    () => extractSessionTouchedFiles(sidePanelMessages).map((item) => item.path),
    [sidePanelMessages],
  );

  const failedSessionRuns = useMemo(
    () => sessionRuns.filter((run) => run.status === "failed" || run.status === "cancelled"),
    [sessionRuns],
  );

  const latestMaxTurnsRun = useMemo(
    () =>
      [...sessionRuns]
        .reverse()
        .find((run) => (run.error_kind || "").trim().toLowerCase() === "max_turns") ?? null,
    [sessionRuns],
  );

  const failedRunsByAssistantMessageId = useMemo(() => {
    const mapping = new Map<string, SessionRunProjection[]>();
    for (const run of failedSessionRuns) {
      const messageId = (run.assistant_message_id || "").trim();
      if (!messageId) continue;
      const current = mapping.get(messageId) ?? [];
      current.push(run);
      mapping.set(messageId, current);
    }
    return mapping;
  }, [failedSessionRuns]);

  const failedRunsByUserMessageId = useMemo(() => {
    const mapping = new Map<string, SessionRunProjection[]>();
    for (const run of failedSessionRuns) {
      if ((run.assistant_message_id || "").trim()) continue;
      const messageId = (run.user_message_id || "").trim();
      if (!messageId) continue;
      const current = mapping.get(messageId) ?? [];
      current.push(run);
      mapping.set(messageId, current);
    }
    return mapping;
  }, [failedSessionRuns]);

  const orphanFailedRuns = useMemo(() => {
    const anchoredMessageIds = new Set(
      messages.flatMap((message) => {
        const ids: string[] = [];
        const messageId = (message.id || "").trim();
        if (!messageId) return ids;
        ids.push(messageId);
        if ((message.runId || "").trim()) ids.push((message.runId || "").trim());
        return ids;
      }),
    );
    return failedSessionRuns.filter((run) => {
      const userMessageId = (run.user_message_id || "").trim();
      const assistantMessageId = (run.assistant_message_id || "").trim();
      return (
        (!userMessageId || !anchoredMessageIds.has(userMessageId)) &&
        (!assistantMessageId || !anchoredMessageIds.has(assistantMessageId))
      );
    });
  }, [failedSessionRuns, messages]);

  return {
    recoverableSessionRun,
    renderedMessages,
    virtualWindow,
    virtualizedRenderedMessages,
    sidePanelMessages,
    taskPanelModel,
    webSearchEntries,
    touchedFilePaths,
    failedSessionRuns,
    latestMaxTurnsRun,
    failedRunsByAssistantMessageId,
    failedRunsByUserMessageId,
    orphanFailedRuns,
  };
}
