# Remove Manual Chat Compaction Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove manual chat compaction entry points from the main chat UI while preserving existing automatic compaction behavior.

**Architecture:** Keep backend compaction support unchanged for now, but stop exposing manual compaction through the composer UI. Update ChatView tests first to lock the intended UX, then remove the button state/handler and the `/compact` shortcut path.

**Tech Stack:** React, TypeScript, Vitest, Testing Library, Tauri invoke bridge

---

### Task 1: Lock the intended chat UX with tests

**Files:**
- Modify: `apps/runtime/src/components/__tests__/ChatView.theme.test.tsx`

**Step 1: Write the failing tests**

- Add a test asserting the composer does not render a `压缩` button.
- Add a test asserting sending `/compact` goes through the normal `send_message` path instead of calling `compact_context`.

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime test -- ChatView.theme.test.tsx`

Expected: FAIL because the current UI still renders the button and special-cases `/compact`.

### Task 2: Remove manual compaction from ChatView

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`

**Step 1: Write minimal implementation**

- Remove manual compaction state and handler.
- Remove `/compact` special handling from send flow.
- Remove the composer `压缩` button.

**Step 2: Run test to verify it passes**

Run: `pnpm --dir apps/runtime test -- ChatView.theme.test.tsx`

Expected: PASS.

### Task 3: Verify no regression in adjacent chat behavior

**Files:**
- Verify: `apps/runtime/src/components/__tests__/ChatView.theme.test.tsx`

**Step 1: Run targeted verification**

Run: `pnpm --dir apps/runtime test -- ChatView.theme.test.tsx`

Expected: PASS with no manual compaction entry points remaining in the tested composer flow.
