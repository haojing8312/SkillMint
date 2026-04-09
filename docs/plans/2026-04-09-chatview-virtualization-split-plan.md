# ChatView Virtualization Split Plan

## Goal
- Reduce long-session chat rendering cost in `apps/runtime/src/components/ChatView.tsx` by rendering only the visible message window plus overscan.

## Why This Stays Small
- Keep `ChatView` as the owner of scroll metrics and viewport state.
- Move virtualization math into a focused pure helper under `components/chat/`.
- Keep `ChatMessageRail` mostly presentation-only by passing a precomputed visible slice and spacer heights.

## Planned Changes
- Add a pure helper to compute virtual window start/end and spacer heights from:
  - total item count
  - scroll offset
  - viewport height
  - estimated row heights
  - overscan
- Update `ChatView` to track scroll position and viewport height for the chat rail.
- Update `ChatMessageRail` to render:
  - top spacer
  - visible message slice
  - bottom spacer

## Guardrails
- Only enable virtualization above a message-count threshold.
- Preserve message order, existing message IDs, focus highlighting, and streaming behavior.
- Avoid changing composer, send, approval, or session recovery contracts.
