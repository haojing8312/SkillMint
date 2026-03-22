# App Employee Hub Scene Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Extract the employee-center workflow from `apps/runtime/src/App.tsx` into a dedicated scene container while preserving user-visible behavior.

**Architecture:** Keep `App.tsx` as the app shell, move employee-specific orchestration into `EmployeeHubScene`, and centralize employee-scene Tauri calls in `employeeHubApi.ts`. Reuse `EmployeeHubView` as the presentation layer and avoid backend or protocol changes.

**Tech Stack:** React 18, TypeScript, Tauri invoke API, Vitest/Testing Library

---

### Task 1: Map the employee-center boundary

**Files:**
- Modify: `apps/runtime/src/App.tsx`
- Reference: `apps/runtime/src/components/employees/EmployeeHubView.tsx`
- Reference: `apps/runtime/src/types.ts`

**Step 1: Identify the exact employee-center state and handlers**

List and group:

- employee data state
- employee-group data state
- employee-center highlight / initial-tab state
- employee-center action handlers
- shell-owned dependencies that must stay in `App.tsx`

**Step 2: Mark shell-owned dependencies**

Keep only true app-shell concerns in `App.tsx`, especially:

- active main view navigation
- top-level session opening
- app-wide dialogs or settings openers

**Step 3: Define the future scene props**

Write the target prop contract for `EmployeeHubScene`, keeping it small and explicit.

**Step 4: Commit**

```bash
git add apps/runtime/src/App.tsx
git commit -m "refactor: define employee hub scene boundary"
```

### Task 2: Create employee-scene API wrapper

**Files:**
- Create: `apps/runtime/src/scenes/employees/employeeHubApi.ts`
- Test: `apps/runtime/src/components/employees/__tests__/EmployeeHubView.overview-home.test.tsx`

**Step 1: Add named API functions**

Create small wrappers around the employee-center `invoke(...)` calls, such as:

- `listAgentEmployees`
- `listEmployeeGroups`
- `upsertAgentEmployee`
- `deleteAgentEmployee`

Only include functions actually needed by the employee scene in this phase.

**Step 2: Keep behavior unchanged**

Do not rename Tauri commands or reshape returned data unless already required by current UI.

**Step 3: Update one low-risk call site first**

Switch one employee-related `invoke(...)` call to use the wrapper and confirm typing still works.

**Step 4: Run a focused frontend test**

Run: `pnpm --dir apps/runtime test -- EmployeeHubView.overview-home`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/scenes/employees/employeeHubApi.ts apps/runtime/src/App.tsx
git commit -m "refactor: add employee hub api wrapper"
```

### Task 3: Add the employee scene container

**Files:**
- Create: `apps/runtime/src/scenes/employees/EmployeeHubScene.tsx`
- Modify: `apps/runtime/src/App.tsx`
- Reference: `apps/runtime/src/components/employees/EmployeeHubView.tsx`

**Step 1: Create the container skeleton**

Render `EmployeeHubView` from the new scene with props passed through from `App.tsx`.

**Step 2: Keep the first version dumb**

Do not move logic yet. First make `App.tsx -> EmployeeHubScene -> EmployeeHubView` render cleanly.

**Step 3: Replace direct `EmployeeHubView` usage in `App.tsx`**

Switch the employee main-view branch to render `EmployeeHubScene`.

**Step 4: Run focused employee UI tests**

Run: `pnpm --dir apps/runtime test -- EmployeeHubView`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/scenes/employees/EmployeeHubScene.tsx apps/runtime/src/App.tsx
git commit -m "refactor: introduce employee hub scene container"
```

### Task 4: Move employee-scene state into the new container

**Files:**
- Modify: `apps/runtime/src/scenes/employees/EmployeeHubScene.tsx`
- Modify: `apps/runtime/src/App.tsx`

**Step 1: Move employee list and group list state**

Transfer the employee-center data state and refresh functions from `App.tsx` into `EmployeeHubScene`.

**Step 2: Move employee-center local UI orchestration**

Transfer:

- initial tab
- employee highlight state
- employee-center refresh and mutation status

Only keep shell-level navigation in `App.tsx`.

**Step 3: Keep cross-scene actions as callbacks**

If employee actions still need shell behavior, pass them explicitly as callbacks rather than letting the scene reach into app-shell internals.

**Step 4: Run focused tests**

Run: `pnpm --dir apps/runtime test -- EmployeeHubView EmployeeHub`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/scenes/employees/EmployeeHubScene.tsx apps/runtime/src/App.tsx
git commit -m "refactor: move employee hub state into scene"
```

### Task 5: Move employee mutation handlers into the scene

**Files:**
- Modify: `apps/runtime/src/scenes/employees/EmployeeHubScene.tsx`
- Modify: `apps/runtime/src/App.tsx`
- Reference: `apps/runtime/src/components/employees/EmployeeHubView.tsx`

**Step 1: Move save/delete/set-main/start-task handlers**

Transfer employee-specific handlers into the scene, using `employeeHubApi.ts` where possible.

**Step 2: Keep session-opening helpers minimal**

If task launch still depends on shell-level session helpers, leave only the final shell handoff in `App.tsx`.

**Step 3: Verify prop surface shrinks**

Reduce the number of employee-only props flowing through `App.tsx`.

**Step 4: Run tests**

Run: `pnpm --dir apps/runtime test -- EmployeeHubView EmployeeHub App.employee`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/scenes/employees/EmployeeHubScene.tsx apps/runtime/src/App.tsx apps/runtime/src/scenes/employees/employeeHubApi.ts
git commit -m "refactor: move employee hub handlers into scene"
```

### Task 6: Clean up App shell responsibilities

**Files:**
- Modify: `apps/runtime/src/App.tsx`

**Step 1: Remove dead employee-only state and helpers**

Delete employee-center code that is no longer used by the app shell.

**Step 2: Re-check imports and typing**

Trim unused imports, helper functions, and stale type aliases in `App.tsx`.

**Step 3: Confirm shell readability**

Make sure the employee main-view branch is short and scene-oriented.

**Step 4: Run targeted tests**

Run: `pnpm --dir apps/runtime test -- App.employee EmployeeHubView`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/App.tsx
git commit -m "refactor: slim app shell after employee hub extraction"
```

### Task 7: Final verification

**Files:**
- Verify: `apps/runtime/src/App.tsx`
- Verify: `apps/runtime/src/scenes/employees/EmployeeHubScene.tsx`
- Verify: `apps/runtime/src/scenes/employees/employeeHubApi.ts`

**Step 1: Run runtime frontend tests**

Run: `pnpm --dir apps/runtime test`

Expected: PASS

**Step 2: Run a narrower employee test subset if needed for debugging**

Run: `pnpm --dir apps/runtime test -- EmployeeHubView App.employee`

Expected: PASS

**Step 3: Review the diff**

Confirm:

- no user-visible behavior changes were introduced
- `App.tsx` is materially smaller
- employee-center logic now has a clear home

**Step 4: Commit**

```bash
git add apps/runtime/src/App.tsx apps/runtime/src/scenes/employees/EmployeeHubScene.tsx apps/runtime/src/scenes/employees/employeeHubApi.ts
git commit -m "refactor: extract employee hub scene from app shell"
```
