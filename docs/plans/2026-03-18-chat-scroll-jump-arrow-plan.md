# Chat Scroll Jump Arrow Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a bidirectional floating jump arrow to the runtime chat view, while preventing streamed output from forcibly snapping the user back to the bottom after they scroll away.

**Architecture:** Keep the change local to `ChatView` by tracking the message scroller's position and deriving `isNearTop` / `isNearBottom` state from scroll metrics. Replace unconditional bottom auto-scroll with conditional follow behavior and render a floating arrow that jumps to the top when already at the bottom, or to the bottom otherwise.

**Tech Stack:** React, TypeScript, Framer Motion, Vitest, Testing Library

---

### Task 1: Lock desired scroll behavior with tests

**Files:**
- Modify: `apps/runtime/src/components/__tests__/ChatView.thinking-block.test.tsx`

**Step 1: Write the failing tests**

Add tests that verify:
- stream updates do not call `scrollIntoView` after the user scrolls away from the bottom
- the floating arrow shows `↓` away from the bottom and jumps to the latest content when clicked
- the floating arrow shows `↑` at the bottom and jumps to the top when clicked

**Step 2: Run test to verify it fails**

Run: `pnpm vitest run apps/runtime/src/components/__tests__/ChatView.thinking-block.test.tsx`

Expected: FAIL because `ChatView` does not yet expose the floating arrow or conditional scroll follow behavior.

### Task 2: Implement the smallest runtime change in ChatView

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`

**Step 1: Add scroll state tracking**

Introduce a scroll container ref plus near-top / near-bottom state derived from `scrollTop`, `scrollHeight`, and `clientHeight`.

**Step 2: Gate auto-follow on bottom proximity**

Only call `bottomRef.scrollIntoView(...)` when the user is already near the bottom. Preserve existing follow behavior for fresh streaming when the user has not scrolled away.

**Step 3: Add the floating jump arrow**

Render a lightweight floating arrow inside the message pane:
- `↑` when near the bottom, clicking scrolls to top
- `↓` otherwise, clicking scrolls to bottom and restores auto-follow

**Step 4: Keep interactions smooth**

Use smooth scroll behavior and small motion transitions for opacity / position so the arrow feels attached to the scrolling experience instead of appearing abruptly.

### Task 3: Verify the touched runtime surface

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Modify: `apps/runtime/src/components/__tests__/ChatView.thinking-block.test.tsx`

**Step 1: Run targeted runtime tests**

Run: `pnpm vitest run apps/runtime/src/components/__tests__/ChatView.thinking-block.test.tsx`

Expected: PASS

**Step 2: Run the repo-level runtime verification command for this surface**

Run: `pnpm test:e2e:runtime`

Expected: PASS if available in this environment; otherwise report why it could not be completed.
