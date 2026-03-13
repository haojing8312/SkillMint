# Chat Delivery Files Entry Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the transcript task progress summary with a single finished-state card that opens the workspace files panel.

**Architecture:** Keep the existing journey view model and transcript anchor point, but narrow the render condition to completed states and swap the summary UI for a compact delivery-files entry card. Reuse existing side-panel state transitions so the new card only changes presentation, not file browsing data flow.

**Tech Stack:** React, TypeScript, Vitest, Testing Library, Tailwind utility classes

---

### Task 1: Lock the new transcript behavior with tests

**Files:**
- Modify: `apps/runtime/src/components/__tests__/ChatView.side-panel-redesign.test.tsx`

**Step 1: Write the failing test**

Add assertions that:
- the transcript shows `查看此任务中的所有文件` for completed delivery output
- clicking it opens the panel and selects `文件`
- partial completion also shows the card
- running and failed journeys do not show the card

**Step 2: Run test to verify it fails**

Run: `pnpm --filter runtime test -- --run src/components/__tests__/ChatView.side-panel-redesign.test.tsx`
Expected: FAIL because the old transcript still renders `任务进度` / `交付结果` and does not match the new card expectations.

**Step 3: Write minimal implementation**

Do not implement in this task.

**Step 4: Run test to verify it still captures the intended failure**

Run the same targeted test command and confirm the failure is due to the missing new card behavior.

**Step 5: Commit**

Do not commit yet. This task only establishes the red state.

### Task 2: Replace transcript summary UI with the compact files-entry card

**Files:**
- Modify: `apps/runtime/src/components/chat-journey/TaskJourneySummary.tsx`
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Optional cleanup: `apps/runtime/src/components/chat-journey/DeliverySummaryCard.tsx`
- Optional cleanup: `apps/runtime/src/components/chat-journey/TaskJourneyTimeline.tsx`

**Step 1: Write the failing test**

Use the red tests from Task 1. Do not add production code before those tests are failing for the expected reason.

**Step 2: Run test to verify it fails**

Run: `pnpm --filter runtime test -- --run src/components/__tests__/ChatView.side-panel-redesign.test.tsx`
Expected: FAIL with missing `查看此任务中的所有文件` or unexpected old summary content.

**Step 3: Write minimal implementation**

- Change the transcript render guard to only mount the summary for `completed` or `partial`.
- Simplify `TaskJourneySummary` to render one button/card when `model.deliverables.length > 0`.
- Wire the card click to `onViewFiles`.

**Step 4: Run test to verify it passes**

Run: `pnpm --filter runtime test -- --run src/components/__tests__/ChatView.side-panel-redesign.test.tsx`
Expected: PASS for the updated transcript summary assertions.

**Step 5: Commit**

```bash
git add docs/plans/2026-03-13-chat-delivery-files-entry-design.md docs/plans/2026-03-13-chat-delivery-files-entry.md apps/runtime/src/components/ChatView.tsx apps/runtime/src/components/chat-journey/TaskJourneySummary.tsx apps/runtime/src/components/__tests__/ChatView.side-panel-redesign.test.tsx
git commit -m "feat(ui): simplify chat delivery files entry"
```

### Task 3: Verify no regressions in adjacent side-panel behavior

**Files:**
- Verify: `apps/runtime/src/components/chat-side-panel/ChatWorkspaceSidePanel.tsx`
- Verify: `apps/runtime/src/components/chat-side-panel/WorkspaceFilesPanel.tsx`

**Step 1: Write the failing test**

If the targeted transcript test does not already cover the panel switch, add one minimal assertion in the existing test file instead of creating a new suite.

**Step 2: Run test to verify it fails**

Run the same targeted test command if additional assertions were added.
Expected: FAIL until the implementation correctly opens the panel and selects `文件`.

**Step 3: Write minimal implementation**

Reuse `handleViewFilesFromDelivery()` without changing side-panel component behavior.

**Step 4: Run test to verify it passes**

Run: `pnpm --filter runtime test -- --run src/components/__tests__/ChatView.side-panel-redesign.test.tsx`
Expected: PASS

**Step 5: Commit**

Do not create a separate commit if Task 2 already contains this change.
