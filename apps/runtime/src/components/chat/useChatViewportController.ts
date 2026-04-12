import { useEffect, useRef, useState } from "react";

import type { Message, StreamItem, EmployeeGroupRunSnapshot } from "../../types";
import type { PendingApprovalView } from "../../scenes/chat/useChatSessionController";

const CHAT_SCROLL_EDGE_THRESHOLD = 48;

type UseChatViewportControllerArgs = {
  sessionId: string;
  messages: Message[];
  streamItems: StreamItem[];
  streamReasoning:
    | {
        status: "thinking" | "completed" | "interrupted";
        content: string;
        durationMs?: number;
      }
    | null;
  askUserQuestion: string | null;
  pendingApprovals: PendingApprovalView[];
  sessionFocusRequest?: { nonce: number; snippet: string };
  groupRunStepFocusRequest?: { nonce: number; stepId: string; eventId?: string };
  groupRunSnapshot: EmployeeGroupRunSnapshot | null;
  expandedGroupRunStepIds: string[];
  onExpandGroupRunStep: (stepId: string) => void;
};

export function useChatViewportController({
  sessionId,
  messages,
  streamItems,
  streamReasoning,
  askUserQuestion,
  pendingApprovals,
  sessionFocusRequest,
  groupRunStepFocusRequest,
  groupRunSnapshot,
  expandedGroupRunStepIds,
  onExpandGroupRunStep,
}: UseChatViewportControllerArgs) {
  const [highlightedMessageIndex, setHighlightedMessageIndex] = useState<number | null>(null);
  const [highlightedGroupRunStepId, setHighlightedGroupRunStepId] = useState<string | null>(null);
  const [highlightedGroupRunStepEventId, setHighlightedGroupRunStepEventId] = useState<string | null>(null);
  const [isNearTop, setIsNearTop] = useState(true);
  const [isNearBottom, setIsNearBottom] = useState(true);
  const [hasScrollableContent, setHasScrollableContent] = useState(false);
  const [scrollTop, setScrollTop] = useState(0);
  const [viewportHeight, setViewportHeight] = useState(0);

  const bottomRef = useRef<HTMLDivElement>(null);
  const scrollRegionRef = useRef<HTMLDivElement>(null);
  const autoFollowScrollRef = useRef(true);
  const scrollAnimationFrameRef = useRef<number | null>(null);
  const scrollAnimationTargetRef = useRef<"top" | "bottom" | null>(null);
  const lastHandledSessionFocusNonceRef = useRef<number | null>(null);
  const messageElementRefs = useRef<Record<number, HTMLDivElement | null>>({});
  const lastHandledGroupRunStepFocusNonceRef = useRef<number | null>(null);
  const groupRunStepElementRefs = useRef<Record<string, HTMLDivElement | null>>({});
  const groupRunStepEventElementRefs = useRef<Record<string, HTMLDivElement | HTMLButtonElement | null>>({});

  const syncScrollMetrics = (element: HTMLDivElement | null) => {
    if (!element) {
      return;
    }
    const distanceFromBottom = Math.max(0, element.scrollHeight - element.scrollTop - element.clientHeight);
    const nextNearBottom = distanceFromBottom <= CHAT_SCROLL_EDGE_THRESHOLD;
    const nextNearTop = element.scrollTop <= CHAT_SCROLL_EDGE_THRESHOLD;
    const keepFollowingBottom = scrollAnimationTargetRef.current === "bottom";
    setIsNearBottom(nextNearBottom);
    setIsNearTop(nextNearTop);
    setHasScrollableContent(element.scrollHeight > element.clientHeight + 4);
    setScrollTop(element.scrollTop);
    setViewportHeight(element.clientHeight);
    autoFollowScrollRef.current = keepFollowingBottom || nextNearBottom;
  };

  const stopScrollAnimation = () => {
    if (scrollAnimationFrameRef.current !== null) {
      cancelAnimationFrame(scrollAnimationFrameRef.current);
      scrollAnimationFrameRef.current = null;
    }
    scrollAnimationTargetRef.current = null;
  };

  const setScrollRegionTop = (scrollRegion: HTMLDivElement, top: number) => {
    if (typeof scrollRegion.scrollTo === "function") {
      scrollRegion.scrollTo({ top });
      return;
    }
    scrollRegion.scrollTop = top;
  };

  const animateScrollRegionTo = (targetTop: number, durationMs = 1000, target: "top" | "bottom" | null = null) => {
    const scrollRegion = scrollRegionRef.current;
    if (!scrollRegion) {
      return;
    }

    stopScrollAnimation();
    scrollAnimationTargetRef.current = target;

    const maxTop = Math.max(0, scrollRegion.scrollHeight - scrollRegion.clientHeight);
    const startTop = scrollRegion.scrollTop;
    const clampedTargetTop = Math.max(0, Math.min(targetTop, maxTop));
    const distance = clampedTargetTop - startTop;

    if (Math.abs(distance) < 1) {
      setScrollRegionTop(scrollRegion, clampedTargetTop);
      syncScrollMetrics(scrollRegion);
      if (target !== "bottom") {
        scrollAnimationTargetRef.current = null;
      }
      return;
    }

    const easeOutCubic = (t: number) => 1 - Math.pow(1 - t, 3);
    const initialTop = startTop + distance * 0.22;
    setScrollRegionTop(scrollRegion, initialTop);
    syncScrollMetrics(scrollRegion);
    let startTime: number | null = null;

    const step = (timestamp: number) => {
      if (startTime === null) {
        startTime = timestamp;
      }
      const progress = Math.min((timestamp - startTime) / durationMs, 1);
      const nextTop = startTop + distance * easeOutCubic(progress);
      setScrollRegionTop(scrollRegion, nextTop);
      syncScrollMetrics(scrollRegion);

      if (progress < 1) {
        scrollAnimationFrameRef.current = requestAnimationFrame(step);
        return;
      }

      scrollRegion.scrollTo({ top: clampedTargetTop });
      syncScrollMetrics(scrollRegion);
      scrollAnimationFrameRef.current = null;
      if (target !== "bottom") {
        scrollAnimationTargetRef.current = null;
      }
    };

    scrollAnimationFrameRef.current = requestAnimationFrame(step);
  };

  const handleScrollRegionScroll = () => {
    syncScrollMetrics(scrollRegionRef.current);
  };

  const handleScrollJump = () => {
    const scrollRegion = scrollRegionRef.current;
    if (!scrollRegion) {
      return;
    }

    if (isNearBottom) {
      autoFollowScrollRef.current = false;
      setIsNearBottom(false);
      setIsNearTop(true);
      animateScrollRegionTo(0, 1000, "top");
      return;
    }

    autoFollowScrollRef.current = true;
    setIsNearBottom(true);
    setIsNearTop(false);
    animateScrollRegionTo(scrollRegion.scrollHeight - scrollRegion.clientHeight, 1000, "bottom");
  };

  useEffect(() => {
    autoFollowScrollRef.current = true;
    setIsNearTop(true);
    setIsNearBottom(true);
    setHasScrollableContent(false);
  }, [sessionId]);

  useEffect(() => {
    if (autoFollowScrollRef.current) {
      bottomRef.current?.scrollIntoView({ behavior: "smooth" });
      return;
    }
    syncScrollMetrics(scrollRegionRef.current);
  }, [messages, streamItems, streamReasoning, askUserQuestion, pendingApprovals]);

  useEffect(() => {
    syncScrollMetrics(scrollRegionRef.current);
  }, []);

  useEffect(() => stopScrollAnimation, []);

  useEffect(() => {
    if (!sessionFocusRequest || !sessionFocusRequest.snippet.trim()) {
      return;
    }
    if (messages.length === 0) {
      return;
    }
    if (lastHandledSessionFocusNonceRef.current === sessionFocusRequest.nonce) {
      return;
    }

    const normalize = (value: string) => value.replace(/\s+/g, " ").trim().toLowerCase();
    const normalizedSnippet = normalize(sessionFocusRequest.snippet);
    const fallbackSnippet = normalizedSnippet.slice(0, 16);
    const assistantMessageIndexes = messages
      .map((message, index) => ({ message, index }))
      .filter(({ message }) => message.role === "assistant");

    let matchedIndex = -1;
    for (let i = assistantMessageIndexes.length - 1; i >= 0; i -= 1) {
      const candidate = assistantMessageIndexes[i];
      const normalizedContent = normalize(candidate.message.content || "");
      if (!normalizedContent) continue;
      if (
        normalizedContent.includes(normalizedSnippet) ||
        normalizedSnippet.includes(normalizedContent) ||
        (fallbackSnippet.length > 0 && normalizedContent.includes(fallbackSnippet))
      ) {
        matchedIndex = candidate.index;
        break;
      }
    }
    if (matchedIndex < 0 && assistantMessageIndexes.length > 0) {
      matchedIndex = assistantMessageIndexes[assistantMessageIndexes.length - 1].index;
    }

    lastHandledSessionFocusNonceRef.current = sessionFocusRequest.nonce;
    if (matchedIndex < 0) {
      return;
    }

    setHighlightedMessageIndex(matchedIndex);
    messageElementRefs.current[matchedIndex]?.scrollIntoView({ behavior: "smooth", block: "center" });
    const timer = setTimeout(() => {
      setHighlightedMessageIndex((current) => (current === matchedIndex ? null : current));
    }, 2400);
    return () => clearTimeout(timer);
  }, [messages, sessionFocusRequest, sessionId]);

  useEffect(() => {
    const targetStepId = (groupRunStepFocusRequest?.stepId || "").trim();
    if (!targetStepId || !groupRunSnapshot) {
      return;
    }
    if (lastHandledGroupRunStepFocusNonceRef.current === groupRunStepFocusRequest?.nonce) {
      return;
    }
    const matchedStep = (groupRunSnapshot.steps || []).find((step) => (step.id || "").trim() === targetStepId);
    if (!matchedStep) {
      return;
    }
    const targetEventId = (groupRunStepFocusRequest?.eventId || "").trim();
    if (targetEventId && !expandedGroupRunStepIds.includes(targetStepId)) {
      onExpandGroupRunStep(targetStepId);
      return;
    }

    lastHandledGroupRunStepFocusNonceRef.current = groupRunStepFocusRequest?.nonce ?? null;
    setHighlightedGroupRunStepId(targetStepId);
    setHighlightedGroupRunStepEventId(targetEventId || null);
    const targetElement =
      (targetEventId ? groupRunStepEventElementRefs.current[targetEventId] : null) ||
      groupRunStepElementRefs.current[targetStepId];
    targetElement?.scrollIntoView({ behavior: "smooth", block: "center" });
    const timer = setTimeout(() => {
      setHighlightedGroupRunStepId((current) => (current === targetStepId ? null : current));
      setHighlightedGroupRunStepEventId((current) => (current === targetEventId ? null : current));
    }, 2400);
    return () => clearTimeout(timer);
  }, [
    expandedGroupRunStepIds,
    groupRunSnapshot,
    groupRunStepFocusRequest,
    onExpandGroupRunStep,
    sessionId,
  ]);

  return {
    highlightedMessageIndex,
    highlightedGroupRunStepId,
    highlightedGroupRunStepEventId,
    isNearTop,
    isNearBottom,
    setIsNearTop,
    setIsNearBottom,
    hasScrollableContent,
    scrollTop,
    viewportHeight,
    bottomRef,
    scrollRegionRef,
    autoFollowScrollRef,
    messageElementRefs,
    groupRunStepElementRefs,
    groupRunStepEventElementRefs,
    animateScrollRegionTo,
    handleScrollRegionScroll,
    handleScrollJump,
  };
}
