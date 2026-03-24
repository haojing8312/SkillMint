# Chat View Split Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Split `apps/runtime/src/components/ChatView.tsx` into stable shell, controller, and presentation boundaries without changing current user-visible behavior.

**Architecture:** Keep `ChatView.tsx` as the page entry shell while extracting chat-local service wrappers, session/runtime orchestration, stream-event orchestration, group-run orchestration, and composer behavior into focused chat modules. Preserve current props, Tauri command contracts, stream event semantics, and IM/group-run behavior.

**Tech Stack:** React, TypeScript, Vitest, existing Tauri `invoke(...)` APIs, chat stream event helpers, frontend large-file guardrails.

---

### Task 1: Freeze the shell contract and extract chat shell primitives

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Create: `apps/runtime/src/components/chat/ChatShell.tsx`
- Create: `apps/runtime/src/components/chat/ChatHeader.tsx`
- Create: `apps/runtime/src/components/chat/ChatExecutionContextBar.tsx`
- Test: `apps/runtime/src/components/__tests__/ChatView.theme.test.tsx`

**Step 1: Confirm shell-level coverage**

Use the existing `ChatView.theme.test.tsx` coverage as the shell guardrail. If a critical shell contract is missing, extend only the exact missing assertion.

**Step 2: Run the focused shell test**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/ChatView.theme.test.tsx
```

Expected: PASS before refactor

**Step 3: Extract shell-only components**

Move pure shell surfaces into:

- `ChatShell.tsx`
- `ChatHeader.tsx`
- `ChatExecutionContextBar.tsx`

Do not move session/runtime orchestration yet.

**Step 4: Re-run the focused shell test**

Run the same command from Step 2.

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/ChatView.tsx apps/runtime/src/components/chat/ChatShell.tsx apps/runtime/src/components/chat/ChatHeader.tsx apps/runtime/src/components/chat/ChatExecutionContextBar.tsx apps/runtime/src/components/__tests__/ChatView.theme.test.tsx
git commit -m "refactor(ui): extract chat shell primitives"
```

### Task 2: Centralize chat-local service wrappers

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Create: `apps/runtime/src/services/chat/chatSessionService.ts`
- Create: `apps/runtime/src/services/chat/chatApprovalService.ts`
- Create: `apps/runtime/src/services/chat/chatGroupRunService.ts`
- Test: `apps/runtime/src/components/__tests__/ChatView.session-resilience.test.tsx`
- Test: `apps/runtime/src/components/__tests__/ChatView.run-guardrails.test.tsx`

**Step 1: Confirm service-sensitive chat coverage**

Use:

- `ChatView.session-resilience.test.tsx`
- `ChatView.run-guardrails.test.tsx`

as the first service-layer guardrail because they touch message/run loading and send/cancel recovery semantics.

**Step 2: Run the focused tests**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/ChatView.session-resilience.test.tsx src/components/__tests__/ChatView.run-guardrails.test.tsx
```

Expected: PASS before refactor

**Step 3: Extract service wrappers**

Move chat-local `invoke(...)` clusters into service modules. Keep command names and payloads identical.

**Step 4: Rewire `ChatView.tsx` to use the service modules**

Do not move orchestration yet beyond using service wrappers.

**Step 5: Re-run the focused tests**

Run the same command from Step 2.

Expected: PASS

**Step 6: Commit**

```bash
git add apps/runtime/src/components/ChatView.tsx apps/runtime/src/services/chat/chatSessionService.ts apps/runtime/src/services/chat/chatApprovalService.ts apps/runtime/src/services/chat/chatGroupRunService.ts apps/runtime/src/components/__tests__/ChatView.session-resilience.test.tsx apps/runtime/src/components/__tests__/ChatView.run-guardrails.test.tsx
git commit -m "refactor(ui): centralize chat service wrappers"
```

### Task 3: Extract session/runtime orchestration

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Create: `apps/runtime/src/scenes/chat/useChatSessionController.ts`
- Test: `apps/runtime/src/components/__tests__/ChatView.session-resilience.test.tsx`
- Test: `apps/runtime/src/components/__tests__/ChatView.side-panel-redesign.test.tsx`

**Step 1: Confirm session-level guardrails**

Use:

- `ChatView.session-resilience.test.tsx`
- `ChatView.side-panel-redesign.test.tsx`

because this step touches session switching, recovery state, side-panel source data, and persisted runtime state.

**Step 2: Run the focused tests**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/ChatView.session-resilience.test.tsx src/components/__tests__/ChatView.side-panel-redesign.test.tsx
```

Expected: PASS before refactor

**Step 3: Create `useChatSessionController.ts`**

Move:

- message loading
- session run loading
- pending approval loading
- session focus handling
- persisted runtime-state hydration/publication

into the controller.

**Step 4: Rewire `ChatView.tsx`**

Keep `ChatView` as the shell, but consume narrower controller outputs instead of owning the whole session/runtime lifecycle directly.

**Step 5: Re-run the focused tests**

Run the same command from Step 2.

Expected: PASS

**Step 6: Commit**

```bash
git add apps/runtime/src/components/ChatView.tsx apps/runtime/src/scenes/chat/useChatSessionController.ts apps/runtime/src/components/__tests__/ChatView.session-resilience.test.tsx apps/runtime/src/components/__tests__/ChatView.side-panel-redesign.test.tsx
git commit -m "refactor(ui): extract chat session controller"
```

### Task 4: Extract stream-event orchestration

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Create: `apps/runtime/src/scenes/chat/useChatStreamController.ts`
- Test: `apps/runtime/src/components/__tests__/ChatView.thinking-block.test.tsx`
- Test: `apps/runtime/src/components/__tests__/ChatView.run-guardrails.test.tsx`

**Step 1: Confirm stream-event guardrails**

Use:

- `ChatView.thinking-block.test.tsx`
- `ChatView.run-guardrails.test.tsx`

because they cover streaming output, reasoning, and blocked-stop display behavior.

**Step 2: Run the focused tests**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/ChatView.thinking-block.test.tsx src/components/__tests__/ChatView.run-guardrails.test.tsx
```

Expected: PASS before refactor

**Step 3: Create `useChatStreamController.ts`**

Move:

- stream event subscriptions
- tool-call event handling
- ask-user event handling
- reasoning delta/completed/interrupted handling
- stream item and agent-state mutation

into the controller.

**Step 4: Rewire `ChatView.tsx`**

Keep the shell behavior and render order unchanged.

**Step 5: Re-run the focused tests**

Run the same command from Step 2.

Expected: PASS

**Step 6: Commit**

```bash
git add apps/runtime/src/components/ChatView.tsx apps/runtime/src/scenes/chat/useChatStreamController.ts apps/runtime/src/components/__tests__/ChatView.thinking-block.test.tsx apps/runtime/src/components/__tests__/ChatView.run-guardrails.test.tsx
git commit -m "refactor(ui): extract chat stream controller"
```

### Task 5: Extract IM/group-run orchestration

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Create: `apps/runtime/src/scenes/chat/useChatGroupRunController.ts`
- Create: `apps/runtime/src/components/chat/group-run/ChatGroupRunBoard.tsx`
- Test: `apps/runtime/src/components/__tests__/ChatView.im-routing-panel.test.tsx`

**Step 1: Use the IM/group-run suite as the hard guardrail**

This step must use the existing `ChatView.im-routing-panel.test.tsx` suite because it already protects the highest-risk orchestration board behaviors.

**Step 2: Run the focused IM/group-run suite**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/ChatView.im-routing-panel.test.tsx
```

Expected: PASS before refactor

**Step 3: Extract `useChatGroupRunController.ts`**

Move:

- group snapshot loading and refresh
- review, pause, resume, retry, and reassign handlers
- delegation-card derivation
- group-run step focus state

into the controller.

**Step 4: Extract `ChatGroupRunBoard.tsx`**

Move the orchestration-board JSX and view-only rendering there.

**Step 5: Re-run the focused suite**

Run the same command from Step 2.

Expected: PASS

**Step 6: Commit**

```bash
git add apps/runtime/src/components/ChatView.tsx apps/runtime/src/scenes/chat/useChatGroupRunController.ts apps/runtime/src/components/chat/group-run/ChatGroupRunBoard.tsx apps/runtime/src/components/__tests__/ChatView.im-routing-panel.test.tsx
git commit -m "refactor(ui): extract chat group run domain"
```

### Task 6: Extract the message rail and pending-action dialogs

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Create: `apps/runtime/src/components/chat/ChatMessageRail.tsx`
- Create: `apps/runtime/src/components/chat/ChatPendingActionDialogs.tsx`
- Test: `apps/runtime/src/components/__tests__/ChatView.find-skills-install.test.tsx`
- Test: `apps/runtime/src/components/__tests__/ChatView.thinking-block.test.tsx`

**Step 1: Confirm message-rail guardrails**

Use:

- `ChatView.find-skills-install.test.tsx`
- `ChatView.thinking-block.test.tsx`

because this step affects streaming output, install prompts, message rendering, and modal/dialog surfaces.

**Step 2: Run the focused tests**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/ChatView.find-skills-install.test.tsx src/components/__tests__/ChatView.thinking-block.test.tsx
```

Expected: PASS before refactor

**Step 3: Extract `ChatMessageRail.tsx`**

Move:

- historical message rendering
- recovered assistant message rendering
- streaming assistant bubble rendering
- failed-run cards
- install-candidate presentation

into the message rail component.

**Step 4: Extract `ChatPendingActionDialogs.tsx`**

Move ask-user and confirm-dialog rendering out of the root shell.

**Step 5: Re-run the focused tests**

Run the same command from Step 2.

Expected: PASS

**Step 6: Commit**

```bash
git add apps/runtime/src/components/ChatView.tsx apps/runtime/src/components/chat/ChatMessageRail.tsx apps/runtime/src/components/chat/ChatPendingActionDialogs.tsx apps/runtime/src/components/__tests__/ChatView.find-skills-install.test.tsx apps/runtime/src/components/__tests__/ChatView.thinking-block.test.tsx
git commit -m "refactor(ui): extract chat message rail"
```

### Task 7: Extract composer behavior and stabilize the root shell

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Create: `apps/runtime/src/scenes/chat/useChatComposerController.ts`
- Create: `apps/runtime/src/components/chat/ChatComposer.tsx`
- Test: `apps/runtime/src/components/__tests__/ChatView.side-panel-redesign.test.tsx`
- Test: `apps/runtime/src/components/__tests__/ChatView.session-resilience.test.tsx`

**Step 1: Confirm composer guardrails**

Use:

- `ChatView.side-panel-redesign.test.tsx`
- `ChatView.session-resilience.test.tsx`

and add a narrow composer-focused assertion only if the current coverage misses a key send/attachment/workdir behavior.

**Step 2: Run the focused tests**

Run:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/ChatView.side-panel-redesign.test.tsx src/components/__tests__/ChatView.session-resilience.test.tsx
```

Expected: PASS before refactor

**Step 3: Extract `useChatComposerController.ts` and `ChatComposer.tsx`**

Move:

- draft persistence
- attachment selection/removal
- workdir picker behavior
- send/stop controls
- quick prompt handling

into the composer domain.

**Step 4: Re-run the focused tests**

Run the same command from Step 2.

Expected: PASS

**Step 5: Re-run the large-file report**

Run:

```bash
pnpm report:frontend-large-files
```

Expected:

- `ChatView.tsx` falls materially
- no obvious new giant-file regression appears without being called out

**Step 6: Commit**

```bash
git add apps/runtime/src/components/ChatView.tsx apps/runtime/src/scenes/chat/useChatComposerController.ts apps/runtime/src/components/chat/ChatComposer.tsx
git commit -m "refactor(ui): split chat composer domain"
```

## Verification Checkpoint

Before calling the full split ready for merge, re-run the smallest honest verification set across the changed chat surfaces:

```bash
pnpm --dir apps/runtime exec vitest run src/components/__tests__/ChatView.theme.test.tsx src/components/__tests__/ChatView.session-resilience.test.tsx src/components/__tests__/ChatView.run-guardrails.test.tsx src/components/__tests__/ChatView.im-routing-panel.test.tsx src/components/__tests__/ChatView.side-panel-redesign.test.tsx src/components/__tests__/ChatView.find-skills-install.test.tsx src/components/__tests__/ChatView.thinking-block.test.tsx
pnpm report:frontend-large-files
git diff --check
```

## Expected Outcome

After these tasks:

- `ChatView.tsx` should be a shell and composition entry rather than the owner of every chat concern
- chat service wrappers should own most `invoke(...)` usage
- session, stream, group-run, and composer behavior should each have a clear controller boundary
- large presentation blocks should live in focused chat components instead of the root
- the repo should gain a second strong frontend reference pattern after `SettingsView`
