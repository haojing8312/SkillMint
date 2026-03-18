# Chat Reasoning Layout Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make reasoning and tool execution UI feel like one stable assistant response by removing nested card chrome and keeping internal content aligned to a single message width.

**Architecture:** Keep the existing assistant bubble contract in `ChatView` and narrow the change to internal message presentation. Update `ThinkingBlock` and `ToolIsland` so they inherit the assistant message rail instead of defining competing visual containers or width models, and prove the behavior with focused component tests first.

**Tech Stack:** React, TypeScript, Tailwind utility classes, Framer Motion, Vitest, Testing Library

---

### Task 1: Lock the expected layout behavior with tests

**Files:**
- Modify: `apps/runtime/src/components/__tests__/ToolIsland.test.tsx`
- Modify: `apps/runtime/src/components/__tests__/ChatView.thinking-block.test.tsx`

**Step 1: Write the failing tests**

- Add a `ToolIsland` test that asserts the summary container uses full-width message alignment instead of a centered fixed-width island.
- Add a `ChatView` or thinking-block integration assertion that reasoning remains rendered inside the assistant response flow without introducing a second bordered card treatment expectation.

**Step 2: Run test to verify it fails**

Run: `pnpm vitest apps/runtime/src/components/__tests__/ToolIsland.test.tsx apps/runtime/src/components/__tests__/ChatView.thinking-block.test.tsx`

Expected: FAIL because current classes still reflect the old centered island and bordered reasoning treatment.

**Step 3: Write minimal implementation**

- Update `ThinkingBlock.tsx` styles to remove border-heavy treatment and use soft inset styling.
- Update `ToolIsland.tsx` wrapper and summary/detail styles so the component uses the parent message width rail.

**Step 4: Run test to verify it passes**

Run: `pnpm vitest apps/runtime/src/components/__tests__/ToolIsland.test.tsx apps/runtime/src/components/__tests__/ChatView.thinking-block.test.tsx`

Expected: PASS

### Task 2: Refine assistant message composition

**Files:**
- Modify: `apps/runtime/src/components/ThinkingBlock.tsx`
- Modify: `apps/runtime/src/components/ToolIsland.tsx`
- Modify: `apps/runtime/src/components/ChatView.tsx`

**Step 1: Write the failing test**

- If needed after Task 1, extend `ChatView` coverage to assert tool history and reasoning align inside the assistant message bubble under both historical and streaming rendering paths.

**Step 2: Run test to verify it fails**

Run: `pnpm vitest apps/runtime/src/components/__tests__/ChatView.thinking-block.test.tsx apps/runtime/src/components/__tests__/ToolIsland.test.tsx`

Expected: FAIL on the new assertions if composition still uses the older layout assumptions.

**Step 3: Write minimal implementation**

- Keep the assistant bubble width contract intact.
- Ensure internal modules use `w-full` or parent-driven layout instead of fixed max widths or centering that conflict with the message rail.
- Preserve current interaction logic and only soften visuals plus stabilize width behavior.

**Step 4: Run test to verify it passes**

Run: `pnpm vitest apps/runtime/src/components/__tests__/ChatView.thinking-block.test.tsx apps/runtime/src/components/__tests__/ToolIsland.test.tsx`

Expected: PASS

### Task 3: Verify the touched runtime surface honestly

**Files:**
- Read: `apps/runtime/src/components/ThinkingBlock.tsx`
- Read: `apps/runtime/src/components/ToolIsland.tsx`
- Read: `apps/runtime/src/components/ChatView.tsx`
- Test: `apps/runtime/src/components/__tests__/ToolIsland.test.tsx`
- Test: `apps/runtime/src/components/__tests__/ChatView.thinking-block.test.tsx`

**Step 1: Run targeted runtime tests**

Run: `pnpm vitest apps/runtime/src/components/__tests__/ToolIsland.test.tsx apps/runtime/src/components/__tests__/ChatView.thinking-block.test.tsx`

Expected: PASS

**Step 2: Run the repo-level runtime verification command for this UI surface**

Run: `pnpm test:e2e:runtime`

Expected: PASS if the local environment supports the runtime E2E suite. If not practical in this session, report that gap explicitly instead of overstating verification.

**Step 3: Summarize coverage**

- Report which assistant chat UI surfaces were covered by targeted tests.
- Report whether broader runtime E2E coverage was executed or remains unverified.
