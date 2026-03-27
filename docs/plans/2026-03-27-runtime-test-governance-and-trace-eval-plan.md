# Runtime Test Governance And Trace/Eval Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Stabilize grouped runtime/frontend regression tests and add an internal trace/eval foundation on top of the existing session run event pipeline.

**Architecture:** First harden the Vitest environment so grouped `EmployeeHub` and `Settings` suites stop leaking state across files. Then extend the existing `session_run_events` and journal infrastructure with structured trace query/export capabilities and cover them with normalized golden-fixture tests.

**Tech Stack:** Vitest, React Testing Library, Tauri commands, Rust `sqlx`, Rust serde/JSON, existing WorkClaw diagnostics and session journal infrastructure.

---

### Task 0: Create A Dedicated Worktree For Implementation

**Files:**
- Modify: none

**Step 1: Create an isolated branch/worktree**

Run:

```powershell
git worktree add .worktrees/runtime-test-trace -b feat/runtime-test-trace
```

Expected: a new worktree is created at `.worktrees/runtime-test-trace`.

**Step 2: Switch implementation work to that worktree**

Run:

```powershell
git -C .worktrees/runtime-test-trace status --short --branch
```

Expected: clean branch state on `feat/runtime-test-trace`.

**Step 3: Commit**

No commit required yet. This is environment preparation only.

### Task 1: Establish Global Runtime Test Cleanup

**Files:**
- Modify: `apps/runtime/src/test/setup.ts`
- Modify: `apps/runtime/vitest.config.ts`
- Test: grouped runtime Vitest commands below

**Step 1: Write the failing grouped regression command into the implementation notes**

Target grouped commands:

```powershell
pnpm --dir apps/runtime exec vitest run src/components/employees/__tests__/EmployeeHubView.group-orchestrator.test.tsx src/components/employees/__tests__/EmployeeHubView.team-template.test.tsx --pool forks --poolOptions.forks.singleFork
```

```powershell
pnpm --dir apps/runtime exec vitest run src/components/settings/feishu/__tests__/useFeishuInstallerSessionController.test.tsx src/components/settings/feishu/__tests__/useFeishuRuntimeStatusController.test.tsx src/components/settings/feishu/__tests__/useFeishuSetupProgressController.test.tsx src/components/settings/feishu/__tests__/useFeishuSettingsController.test.tsx --pool forks --poolOptions.forks.singleFork
```

Expected before stabilization: at least one grouped path is flaky or fails due to leaked state.

**Step 2: Add global cleanup**

Update `apps/runtime/src/test/setup.ts` to:

- import `cleanup` from `@testing-library/react`
- register global `afterEach`
- call `cleanup()`
- call `vi.clearAllMocks()`
- call `vi.restoreAllMocks()`
- call `vi.useRealTimers()`
- clear `localStorage` and `sessionStorage`
- reset `window.location.hash`

**Step 3: Add config defaults if needed**

Update `apps/runtime/vitest.config.ts` only if the current config still leaves mock-reset behavior inconsistent. Prefer the smallest config change that aligns with the new setup behavior.

**Step 4: Run the grouped commands**

Run the two grouped commands above.

Expected: either they pass immediately, or failures become smaller/localized.

**Step 5: Commit**

```powershell
git add apps/runtime/src/test/setup.ts apps/runtime/vitest.config.ts
git commit -m "test(runtime): centralize vitest cleanup baseline"
```

### Task 2: Stabilize The Remaining EmployeeHub Grouped Tests

**Files:**
- Modify: `apps/runtime/src/components/employees/__tests__/EmployeeHubView.group-orchestrator.test.tsx`
- Modify: `apps/runtime/src/components/employees/__tests__/EmployeeHubView.team-template.test.tsx`

**Step 1: Reproduce grouped failure after Task 1**

Run:

```powershell
pnpm --dir apps/runtime exec vitest run src/components/employees/__tests__/EmployeeHubView.group-orchestrator.test.tsx src/components/employees/__tests__/EmployeeHubView.team-template.test.tsx --pool forks --poolOptions.forks.singleFork
```

Expected: identify the remaining shared-state leak, if any.

**Step 2: Apply the smallest test-only fix**

Typical fixes may include:

- explicit `cleanup()` removal if already covered globally
- ensuring local mocks are reset in `beforeEach`
- avoiding accidental reliance on stale DOM state
- making fake timer usage fully owned by the file

Do not change production `EmployeeHubView` behavior unless a real product bug is proven.

**Step 3: Re-run grouped employee tests**

Run the grouped employee command again.

Expected: pass in a single worker.

**Step 4: Commit**

```powershell
git add apps/runtime/src/components/employees/__tests__/EmployeeHubView.group-orchestrator.test.tsx apps/runtime/src/components/employees/__tests__/EmployeeHubView.team-template.test.tsx
git commit -m "test(runtime): stabilize grouped employee hub suites"
```

### Task 3: Stabilize The Remaining Feishu Settings Grouped Tests

**Files:**
- Modify: `apps/runtime/src/components/settings/feishu/__tests__/useFeishuInstallerSessionController.test.tsx`
- Modify: `apps/runtime/src/components/settings/feishu/__tests__/useFeishuRuntimeStatusController.test.tsx`
- Modify: `apps/runtime/src/components/settings/feishu/__tests__/useFeishuSetupProgressController.test.tsx`
- Modify: `apps/runtime/src/components/settings/feishu/__tests__/useFeishuSettingsController.test.tsx`

**Step 1: Reproduce grouped settings failure after Task 1**

Run:

```powershell
pnpm --dir apps/runtime exec vitest run src/components/settings/feishu/__tests__/useFeishuInstallerSessionController.test.tsx src/components/settings/feishu/__tests__/useFeishuRuntimeStatusController.test.tsx src/components/settings/feishu/__tests__/useFeishuSetupProgressController.test.tsx src/components/settings/feishu/__tests__/useFeishuSettingsController.test.tsx --pool forks --poolOptions.forks.singleFork
```

**Step 2: Apply minimal test-local fixes**

Typical fixes may include:

- moving fake timer activation into each test or `beforeEach`
- removing duplicate teardown now covered globally
- ensuring `invokeMock` resets and timer cleanup are consistent

Do not widen scope beyond the grouped-run contamination issue.

**Step 3: Re-run grouped settings command**

Expected: pass in a single worker.

**Step 4: Commit**

```powershell
git add apps/runtime/src/components/settings/feishu/__tests__/useFeishuInstallerSessionController.test.tsx apps/runtime/src/components/settings/feishu/__tests__/useFeishuRuntimeStatusController.test.tsx apps/runtime/src/components/settings/feishu/__tests__/useFeishuSetupProgressController.test.tsx apps/runtime/src/components/settings/feishu/__tests__/useFeishuSettingsController.test.tsx
git commit -m "test(runtime): stabilize grouped feishu settings suites"
```

### Task 4: Add Structured Session Run Event Query Support

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/session_runs.rs`
- Modify: `apps/runtime/src-tauri/src/lib.rs`
- Test: Rust tests near `session_runs.rs`

**Step 1: Add a failing Rust test for event querying**

Cover:

- list all events for a session
- filter by `run_id`
- limit result size
- preserve event ordering

**Step 2: Implement a structured query command**

Add a new command shaped like:

- `list_session_run_events(session_id, run_id?, limit?)`

Return structured summaries rather than unbounded raw payload dumps.

Suggested fields:

- `session_id`
- `run_id`
- `event_type`
- `created_at`
- key normalized summary fields derived from payload

**Step 3: Register the command in `lib.rs`**

**Step 4: Run focused Rust tests**

Run:

```powershell
cargo test --lib session_runs --manifest-path apps/runtime/src-tauri/Cargo.toml -- --nocapture
```

Expected: new and existing nearby tests pass.

**Step 5: Commit**

```powershell
git add apps/runtime/src-tauri/src/commands/session_runs.rs apps/runtime/src-tauri/src/lib.rs
git commit -m "feat(runtime): add session run event query command"
```

### Task 5: Add Trace Builder And Export Path

**Files:**
- Create: `apps/runtime/src-tauri/src/agent/runtime/trace_builder.rs`
- Modify: `apps/runtime/src-tauri/src/commands/session_runs.rs`
- Modify: `apps/runtime/src-tauri/src/commands/desktop_lifecycle/diagnostics_service.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/mod.rs`
- Test: new Rust tests for trace builder/export

**Step 1: Write failing Rust tests for normalized trace summaries**

Cover at least:

- success flow
- guard-warning flow
- approval flow
- cancellation/failure flow
- malformed event payload degradation

**Step 2: Implement `trace_builder.rs`**

The builder should:

- take ordered session run events
- build normalized lifecycle and tool summaries
- collect parse warnings instead of hard-failing on bad payloads

**Step 3: Add an export command or export helper**

Prefer reusing existing diagnostics/export conventions. The export should produce a structured JSON trace artifact for one run.

**Step 4: Run focused Rust tests**

Run:

```powershell
cargo test --lib trace_builder --manifest-path apps/runtime/src-tauri/Cargo.toml -- --nocapture
```

Then:

```powershell
pnpm test:rust-fast
```

**Step 5: Commit**

```powershell
git add apps/runtime/src-tauri/src/agent/runtime/trace_builder.rs apps/runtime/src-tauri/src/agent/runtime/mod.rs apps/runtime/src-tauri/src/commands/session_runs.rs apps/runtime/src-tauri/src/commands/desktop_lifecycle/diagnostics_service.rs
git commit -m "feat(runtime): add run trace builder and export path"
```

### Task 6: Add Golden Trace Fixtures As Minimal Eval Coverage

**Files:**
- Create: `apps/runtime/src-tauri/tests/fixtures/run_traces/*.json`
- Create or modify: Rust tests near trace builder/export coverage

**Step 1: Create normalized fixture cases**

Start with:

- `success.json`
- `loop_intercepted.json`
- `admission_conflict.json`
- `approval_resume.json`
- `child_session_success.json`
- `child_session_failure.json`

**Step 2: Add fixture-driven Rust tests**

Each test should:

- load the fixture event sequence
- run the trace builder
- compare against normalized expected output

**Step 3: Run the targeted fixture tests**

Run:

```powershell
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml trace_fixture -- --nocapture
```

**Step 4: Commit**

```powershell
git add apps/runtime/src-tauri/tests/fixtures/run_traces apps/runtime/src-tauri/src/agent/runtime/trace_builder.rs
git commit -m "test(runtime): add golden trace fixture coverage"
```

### Task 7: Final Verification

**Files:**
- Modify: none

**Step 1: Run grouped frontend regression commands**

Run:

```powershell
pnpm --dir apps/runtime exec vitest run src/components/employees/__tests__/EmployeeHubView.group-orchestrator.test.tsx src/components/employees/__tests__/EmployeeHubView.team-template.test.tsx --pool forks --poolOptions.forks.singleFork
```

```powershell
pnpm --dir apps/runtime exec vitest run src/components/settings/feishu/__tests__/useFeishuInstallerSessionController.test.tsx src/components/settings/feishu/__tests__/useFeishuRuntimeStatusController.test.tsx src/components/settings/feishu/__tests__/useFeishuSetupProgressController.test.tsx src/components/settings/feishu/__tests__/useFeishuSettingsController.test.tsx --pool forks --poolOptions.forks.singleFork
```

**Step 2: Run broader runtime verification**

Run:

```powershell
pnpm test:rust-fast
```

```powershell
pnpm --dir apps/runtime build
```

```powershell
git diff --check
```

**Step 3: Commit any final small cleanup if verification forced it**

If nothing changed, no extra commit is needed.

### Task 8: Integration And Cleanup

**Files:**
- Modify: none unless merge fixes are required

**Step 1: Review commit stack**

Run:

```powershell
git log --oneline --decorate -10
```

**Step 2: Decide finish path**

Use the existing branch-finishing workflow:

- merge locally
- or push and create PR

**Step 3: Preserve the verification summary**

Record:

- grouped test commands run
- Rust verification commands run
- build result
- any still-unverified areas

**Step 4: Final branch cleanup**

Only after merge/push confirmation, remove the dedicated worktree and delete the feature branch.
