# Chat View Phase 2 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Finish a safe root-thinning pass on `apps/runtime/src/components/ChatView.tsx` by moving remaining root-only presentation helpers and orchestration-board rendering into focused chat modules without changing current behavior.

**Architecture:** Keep the phase 1 chat split intact. `ChatView.tsx` remains the page entry and composition shell, while render-only helpers, board presentation, and any remaining pending dialog surfaces move closer to the rail or collaboration board. Preserve Tauri command contracts, stream event semantics, and visible chat UX.

**Tech Stack:** React, TypeScript, Vitest, existing chat controllers and service wrappers, frontend large-file guardrails.

---

### Task 1: Freeze current chat root behavior and report baseline

**Files:**
- Read: `apps/runtime/src/components/ChatView.tsx`
- Read: `apps/runtime/src/components/chat/ChatMessageRail.tsx`
- Read: `apps/runtime/src/scenes/chat/useChatStreamController.ts`
- Test: `apps/runtime/src/components/__tests__/ChatView.run-guardrails.test.tsx`
- Test: `apps/runtime/src/components/__tests__/ChatView.thinking-block.test.tsx`

**Step 1: Confirm the render-heavy guardrail suites**

Use:

- `ChatView.run-guardrails.test.tsx`
- `ChatView.thinking-block.test.tsx`

as the initial safety net for stream and render behavior.

**Step 2: Run the focused suites**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/ChatView.run-guardrails.test.tsx src/components/__tests__/ChatView.thinking-block.test.tsx
pnpm report:frontend-large-files
```

Expected: PASS and current `ChatView.tsx` baseline recorded before phase 2 edits.

### Task 2: Extract markdown and render-only helper surfaces

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Create: `apps/runtime/src/components/chat/chatMarkdownComponents.tsx`
- Create or modify: `apps/runtime/src/components/chat/chatMessageRailHelpers.tsx`
- Test: `apps/runtime/src/components/__tests__/ChatView.thinking-block.test.tsx`

**Step 1: Identify render-only helper clusters**

Target helpers such as:

- `markdownComponents`
- render-only formatting glue
- rail-local helper surfaces that do not own orchestration

**Step 2: Move the helpers into focused chat modules**

Keep them narrow and presentation-only.

**Step 3: Re-run the focused render suite**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/ChatView.thinking-block.test.tsx
```

Expected: PASS

**Step 4: Commit**

```bash
git add apps/runtime/src/components/ChatView.tsx apps/runtime/src/components/chat/chatMarkdownComponents.tsx apps/runtime/src/components/chat/chatMessageRailHelpers.tsx
git commit -m "refactor(ui): extract chat render helpers"
```

### Task 3: Move stream-item and failed-run presentation fully behind the rail

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Modify: `apps/runtime/src/components/chat/ChatMessageRail.tsx`
- Modify or create: `apps/runtime/src/components/chat/chatMessageRailHelpers.tsx`
- Test: `apps/runtime/src/components/__tests__/ChatView.run-guardrails.test.tsx`
- Test: `apps/runtime/src/components/__tests__/ChatView.find-skills-install.test.tsx`

**Step 1: Use rail-sensitive coverage**

Use:

- `ChatView.run-guardrails.test.tsx`
- `ChatView.find-skills-install.test.tsx`

because they exercise failed-run cards, install prompts, and streamed output.

**Step 2: Run the focused suites**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/ChatView.run-guardrails.test.tsx src/components/__tests__/ChatView.find-skills-install.test.tsx
```

Expected: PASS before refactor

**Step 3: Move root-only rail presentation helpers**

Pull `renderStreamItems`, `renderRunFailureCard`, and similar rail-only helpers into the rail surface.

**Step 4: Re-run the focused suites**

Run the same command from Step 2.

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/ChatView.tsx apps/runtime/src/components/chat/ChatMessageRail.tsx apps/runtime/src/components/chat/chatMessageRailHelpers.tsx
git commit -m "refactor(ui): thin chat root rail helpers"
```

### Task 4: Extract the group orchestration board presentation

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Create or modify: `apps/runtime/src/components/chat/group-run/ChatGroupRunBoard.tsx`
- Create or modify: `apps/runtime/src/components/chat/group-run/groupRunBoardHelpers.tsx`
- Test: `apps/runtime/src/components/__tests__/ChatView.im-routing-panel.test.tsx`

**Step 1: Use the collaboration suite as the hard guardrail**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/ChatView.im-routing-panel.test.tsx
```

Expected: PASS before refactor

**Step 2: Move board-only presentation**

Keep state ownership and mutations in `useChatCollaborationController.ts`, but move board rendering and board-only summaries into the board module.

**Step 3: Re-run the collaboration suite**

Run the same command from Step 1.

Expected: PASS

**Step 4: Commit**

```bash
git add apps/runtime/src/components/ChatView.tsx apps/runtime/src/components/chat/group-run/ChatGroupRunBoard.tsx apps/runtime/src/components/chat/group-run/groupRunBoardHelpers.tsx
git commit -m "refactor(ui): extract chat group board presentation"
```

### Task 5: Shrink any remaining pending dialog surfaces

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Create or modify: `apps/runtime/src/components/chat/ChatPendingActionDialogs.tsx`
- Test: `apps/runtime/src/components/__tests__/ChatView.find-skills-install.test.tsx`
- Test: `apps/runtime/src/components/__tests__/ChatView.session-resilience.test.tsx`

**Step 1: Confirm remaining modal/dialog ownership**

Only do this task if the root still contains meaningful dialog-specific JSX and local modal glue after Tasks 2-4.

**Step 2: Run the focused suites**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/ChatView.find-skills-install.test.tsx src/components/__tests__/ChatView.session-resilience.test.tsx
```

Expected: PASS before refactor

**Step 3: Move dialog rendering into a focused chat dialog surface**

Keep the root responsible only for passing the needed state and actions.

**Step 4: Re-run the focused suites**

Run the same command from Step 2.

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/ChatView.tsx apps/runtime/src/components/chat/ChatPendingActionDialogs.tsx
git commit -m "refactor(ui): extract chat pending dialogs"
```

### Task 6: Reassess controller growth and finish the root-thinning pass

**Files:**
- Modify only if needed:
  - `apps/runtime/src/scenes/chat/useChatStreamController.ts`
  - `apps/runtime/src/scenes/chat/useChatCollaborationController.ts`
- Test:
  - `apps/runtime/src/components/__tests__/ChatView.run-guardrails.test.tsx`
  - `apps/runtime/src/components/__tests__/ChatView.im-routing-panel.test.tsx`

**Step 1: Inspect controller size and responsibility**

Only split a controller further if there is a clearly pure helper or selector boundary. Do not split purely for line count.

**Step 2: If needed, extract pure helpers**

Allowed targets:

- event normalization helpers
- board-only summary helpers
- stable no-state selectors

**Step 3: Re-run the affected focused suites**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/ChatView.run-guardrails.test.tsx src/components/__tests__/ChatView.im-routing-panel.test.tsx
```

Expected: PASS

**Step 4: Commit**

```bash
git add apps/runtime/src/scenes/chat/useChatStreamController.ts apps/runtime/src/scenes/chat/useChatCollaborationController.ts
git commit -m "refactor(ui): stabilize chat phase 2 helpers"
```

### Verification Checkpoint

Before calling phase 2 ready for review, run the smallest honest verification set for the touched chat surfaces:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/ChatView.run-guardrails.test.tsx src/components/__tests__/ChatView.thinking-block.test.tsx src/components/__tests__/ChatView.find-skills-install.test.tsx src/components/__tests__/ChatView.im-routing-panel.test.tsx src/components/__tests__/ChatView.session-resilience.test.tsx
pnpm report:frontend-large-files
git diff --check
```

## Expected Outcome

After these tasks:

- `ChatView.tsx` should remain the page shell, but lose most remaining large render-only helper clusters
- `ChatMessageRail.tsx` and the group board should own more of their presentation details
- any remaining pending dialog surfaces should be off the root when that reduces real root weight
- the chat controllers should stay visible and bounded instead of turning into the next hidden giant files
- the repo should have a clear `ChatView` phase 2 follow-up path that continues the frontend guardrail pattern without changing chat behavior
