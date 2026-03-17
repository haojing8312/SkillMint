# Parallel Task Tabs Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add lightweight parallel task tabs so users can start a new task without interrupting a currently running session.

**Architecture:** Keep backend sessions and sidebar history unchanged, and add a front-end work-tab layer in the runtime app shell. Tabs represent active working contexts, while the left sidebar remains the canonical historical session list.

**Tech Stack:** React 18, TypeScript, Vitest, Playwright, Tauri runtime shell

---

### Task 1: Add tab-state model in App shell

**Files:**
- Modify: `apps/runtime/src/App.tsx`
- Test: `apps/runtime/src/__tests__/App.chat-landing.test.tsx`

**Step 1: Write the failing test**

Add tests that describe:
- running session + `开始任务` opens a new start-task tab
- finished session + `开始任务` reuses current tab

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime test -- src/__tests__/App.chat-landing.test.tsx`
Expected: FAIL because the app still uses a single global session slot.

**Step 3: Write minimal implementation**

Add:
- `WorkTab` type
- `tabs`
- `activeTabId`
- helpers for replacing/creating start-task and session tabs

Keep `sessions` and backend loading unchanged.

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime test -- src/__tests__/App.chat-landing.test.tsx`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/App.tsx apps/runtime/src/__tests__/App.chat-landing.test.tsx
git commit -m "feat(runtime): add task tab shell state"
```

### Task 2: Add top tab-strip UI

**Files:**
- Create: `apps/runtime/src/components/TaskTabStrip.tsx`
- Modify: `apps/runtime/src/App.tsx`
- Test: `apps/runtime/src/components/__tests__/TaskTabStrip.test.tsx`

**Step 1: Write the failing test**

Add tests for:
- rendering tabs
- active tab highlighting
- plus button creating a tab
- close button removing a tab

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime test -- src/components/__tests__/TaskTabStrip.test.tsx`
Expected: FAIL because component does not exist.

**Step 3: Write minimal implementation**

Create a small presentational tab strip with:
- tab label
- optional runtime badge
- close action
- trailing plus button

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime test -- src/components/__tests__/TaskTabStrip.test.tsx`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/TaskTabStrip.tsx apps/runtime/src/components/__tests__/TaskTabStrip.test.tsx apps/runtime/src/App.tsx
git commit -m "feat(runtime): add parallel task tab strip"
```

### Task 3: Route start-task and chat through active tab

**Files:**
- Modify: `apps/runtime/src/App.tsx`
- Test: `apps/runtime/src/__tests__/App.chat-landing.test.tsx`
- Test: `apps/runtime/src/__tests__/App.session-create-flow.test.tsx`

**Step 1: Write the failing test**

Add tests that verify:
- creating a session consumes the current active start-task tab
- switching tabs changes the rendered chat/start-task surface

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime test -- src/__tests__/App.chat-landing.test.tsx src/__tests__/App.session-create-flow.test.tsx`
Expected: FAIL because rendering is still driven by global `selectedSessionId`.

**Step 3: Write minimal implementation**

Refactor render logic so:
- active start-task tab renders `NewSessionLanding`
- active session tab renders `ChatView`
- selected session helpers derive from active tab

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime test -- src/__tests__/App.chat-landing.test.tsx src/__tests__/App.session-create-flow.test.tsx`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/App.tsx apps/runtime/src/__tests__/App.chat-landing.test.tsx apps/runtime/src/__tests__/App.session-create-flow.test.tsx
git commit -m "feat(runtime): render work surface from active task tab"
```

### Task 4: Implement running-session auto-split rules

**Files:**
- Modify: `apps/runtime/src/App.tsx`
- Test: `apps/runtime/src/__tests__/App.chat-landing.test.tsx`

**Step 1: Write the failing test**

Add tests for:
- runtime status `running` or `waiting_approval` causes `开始任务` to open a new tab
- ended or idle session reuses current tab

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime test -- src/__tests__/App.chat-landing.test.tsx`
Expected: FAIL because `开始任务` does not inspect tab/session runtime state.

**Step 3: Write minimal implementation**

Add helper:
- `isSessionBlockingStartTaskReuse(session)`

Use it in:
- `handleOpenStartTask`
- `+` button behavior if needed

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime test -- src/__tests__/App.chat-landing.test.tsx`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/App.tsx apps/runtime/src/__tests__/App.chat-landing.test.tsx
git commit -m "feat(runtime): protect running sessions with new task tabs"
```

### Task 5: Keep sidebar history as source of truth

**Files:**
- Modify: `apps/runtime/src/components/Sidebar.tsx`
- Modify: `apps/runtime/src/App.tsx`
- Test: `apps/runtime/src/__tests__/App.sidebar-navigation-selected-session.test.tsx`

**Step 1: Write the failing test**

Add tests verifying:
- clicking a sidebar session opens it in the active tab
- open tabs do not remove sessions from history

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime test -- src/__tests__/App.sidebar-navigation-selected-session.test.tsx`
Expected: FAIL because sidebar still assumes a single global active session slot.

**Step 3: Write minimal implementation**

Keep sidebar list unchanged as data source, but adapt its selection handling to update the active tab context instead of a global single-session render slot.

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime test -- src/__tests__/App.sidebar-navigation-selected-session.test.tsx`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/Sidebar.tsx apps/runtime/src/App.tsx apps/runtime/src/__tests__/App.sidebar-navigation-selected-session.test.tsx
git commit -m "refactor(runtime): bind sidebar history to active task tab"
```

### Task 6: Handle tab close and deleted-session fallback

**Files:**
- Modify: `apps/runtime/src/App.tsx`
- Test: `apps/runtime/src/__tests__/App.chat-landing.test.tsx`

**Step 1: Write the failing test**

Add tests for:
- closing current tab focuses neighbor
- closing last tab creates a fresh start-task tab
- deleting an open session converts its tab into a start-task tab

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime test -- src/__tests__/App.chat-landing.test.tsx`
Expected: FAIL because tab lifecycle fallback rules do not exist.

**Step 3: Write minimal implementation**

Implement:
- `closeTab(tabId)`
- last-tab fallback
- deleted-session tab repair

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime test -- src/__tests__/App.chat-landing.test.tsx`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/App.tsx apps/runtime/src/__tests__/App.chat-landing.test.tsx
git commit -m "fix(runtime): add resilient task tab lifecycle rules"
```

### Task 7: Preserve tab-safe refresh behavior

**Files:**
- Modify: `apps/runtime/src/App.tsx`
- Test: `apps/runtime/src/__tests__/App.chat-landing.test.tsx`
- Test: `apps/runtime/e2e/smoke.navigation.spec.ts`

**Step 1: Write the failing test**

Add tests that verify:
- refresh restores the last active tab context safely
- session history still appears after refresh
- employees/settings navigation does not destroy task tabs

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/runtime test -- src/__tests__/App.chat-landing.test.tsx`
Run: `pnpm --dir apps/runtime test:e2e -- smoke.navigation.spec.ts`
Expected: FAIL because tab-aware recovery is not yet implemented.

**Step 3: Write minimal implementation**

Persist only:
- active tab kind
- active session id when relevant

Do not persist the full open-tab set in v1.

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/runtime test -- src/__tests__/App.chat-landing.test.tsx`
Run: `pnpm --dir apps/runtime test:e2e -- smoke.navigation.spec.ts`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/App.tsx apps/runtime/src/__tests__/App.chat-landing.test.tsx apps/runtime/e2e/smoke.navigation.spec.ts
git commit -m "fix(runtime): recover active task tab after refresh"
```

### Task 8: Final verification and packaging check

**Files:**
- Modify as needed from previous tasks only

**Step 1: Run focused runtime tests**

Run:

```bash
pnpm --dir apps/runtime test -- src/__tests__/App.chat-landing.test.tsx src/__tests__/App.session-create-flow.test.tsx src/__tests__/App.sidebar-navigation-selected-session.test.tsx
```

Expected: PASS

**Step 2: Run smoke E2E**

Run:

```bash
pnpm --dir apps/runtime test:e2e -- smoke.navigation.spec.ts
```

Expected: PASS

**Step 3: Run Rust fast verification**

Run:

```bash
pnpm test:rust-fast
```

Expected: PASS

**Step 4: Run packaging verification**

Run:

```bash
pnpm build:runtime
```

Expected: PASS and produce updated runtime bundles

**Step 5: Commit final polish**

```bash
git add apps/runtime
git commit -m "feat(runtime): add lightweight parallel task tabs"
```
