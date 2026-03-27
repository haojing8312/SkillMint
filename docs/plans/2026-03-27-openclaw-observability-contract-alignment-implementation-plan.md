# OpenClaw Observability And Contract Alignment Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Refactor WorkClaw's runtime trace/eval foundation into an OpenClaw-style hidden observability and contract-regression system without exposing a default trace UI.

**Architecture:** Add a runtime observability module that owns recent-event buffering and aggregate snapshot reporting, wire that data into diagnostics export, and add a reusable runtime contract test harness plus fixture-backed contract coverage. Preserve the existing persisted event model and reuse the current trace builder as the structured export/read-model layer.

**Tech Stack:** Rust, Tauri, sqlx, serde/serde_json, Vitest, cargo test, WorkClaw desktop diagnostics pipeline

---

### Task 0: Create Isolated Worktree

**Files:**
- Verify: `.gitignore`
- Create workspace: `.worktrees/openclaw-observability-contract`

**Step 1: Verify project-local worktree directory is ignored**

Run: `git check-ignore -q .worktrees`
Expected: exit code `0`

**Step 2: Create the worktree**

Run: `git worktree add .worktrees/openclaw-observability-contract -b feat/openclaw-observability-contract`
Expected: new branch and isolated workspace created

**Step 3: Verify clean baseline in the worktree**

Run: `git status --short --branch`
Expected: clean branch status

**Step 4: Commit**

Do not commit in this task. This task only establishes the isolated workspace.

### Task 1: Add Runtime Observability Module Skeleton

**Files:**
- Create: `apps/runtime/src-tauri/src/agent/runtime/observability.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/mod.rs`
- Test: `apps/runtime/src-tauri/src/agent/runtime/observability.rs`

**Step 1: Write the failing unit tests**

Add Rust tests for:

- appending recent runtime events trims to max size
- snapshot counters start at zero
- latency stats update after completed/failed runs
- hidden-event buffer and counters serialize predictably

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib observability --manifest-path apps/runtime/src-tauri/Cargo.toml -- --nocapture`
Expected: FAIL because the new module does not exist yet

**Step 3: Write minimal implementation**

Implement:

- bounded recent-event storage
- counter fields for admission conflicts, loop guard warnings, approvals, child sessions, compactions, failover/error kinds
- latency tracking for completed runs
- serializable snapshot structs

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib observability --manifest-path apps/runtime/src-tauri/Cargo.toml -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/runtime/observability.rs apps/runtime/src-tauri/src/agent/runtime/mod.rs
git commit -m "feat(runtime): add observability snapshot foundation"
```

### Task 2: Feed Runtime Observability From Existing Runtime Paths

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/runtime/session_runtime.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/run_guard.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/admission_gate.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/child_session_runtime.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/compaction_pipeline.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/failover.rs`
- Modify: `apps/runtime/src-tauri/src/lib.rs`
- Test: touched Rust unit tests near each affected module

**Step 1: Write the failing tests**

Add focused tests proving:

- admission conflict increments observability counters
- guard interception increments warning/interception counters
- child-session creation increments child-session counters
- compaction success increments compaction counters
- failover classifications increment error/failover counters

**Step 2: Run targeted tests to verify failure**

Run:
- `cargo test --lib admission_gate --manifest-path apps/runtime/src-tauri/Cargo.toml -- --nocapture`
- `cargo test --lib run_guard --manifest-path apps/runtime/src-tauri/Cargo.toml -- --nocapture`

Expected: FAIL for new snapshot expectations

**Step 3: Write minimal implementation**

Thread the observability state through existing runtime initialization and update counters only at already-known transition points.

**Step 4: Run targeted tests to verify pass**

Run the same commands again.
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/runtime/session_runtime.rs apps/runtime/src-tauri/src/agent/runtime/run_guard.rs apps/runtime/src-tauri/src/agent/runtime/admission_gate.rs apps/runtime/src-tauri/src/agent/runtime/child_session_runtime.rs apps/runtime/src-tauri/src/agent/runtime/compaction_pipeline.rs apps/runtime/src-tauri/src/agent/runtime/failover.rs apps/runtime/src-tauri/src/lib.rs
git commit -m "feat(runtime): record observability counters from runtime paths"
```

### Task 3: Add Diagnostics-Facing Snapshot And Recent Event Export

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/desktop_lifecycle/types.rs`
- Modify: `apps/runtime/src-tauri/src/commands/desktop_lifecycle/diagnostics_service.rs`
- Modify: `apps/runtime/src-tauri/src/commands/desktop_lifecycle.rs`
- Modify: `apps/runtime/src-tauri/src/commands/session_runs.rs`
- Test: `apps/runtime/src-tauri/src/commands/desktop_lifecycle.rs`
- Test: `apps/runtime/src-tauri/src/commands/session_runs.rs`

**Step 1: Write the failing tests**

Add tests proving:

- diagnostics payload includes observability snapshot JSON
- diagnostics payload includes recent runtime events JSON
- exports remain stable when recent-event buffer is empty

**Step 2: Run tests to verify failure**

Run:
- `cargo test --lib desktop_lifecycle --manifest-path apps/runtime/src-tauri/Cargo.toml -- --nocapture`
- `cargo test --lib session_runs --manifest-path apps/runtime/src-tauri/Cargo.toml -- --nocapture`

Expected: FAIL for missing payload fields or snapshot content

**Step 3: Write minimal implementation**

Add new diagnostics payload fields and feed them from the observability module while preserving current session-run trace export behavior.

**Step 4: Run tests to verify pass**

Run the same commands again.
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/desktop_lifecycle/types.rs apps/runtime/src-tauri/src/commands/desktop_lifecycle/diagnostics_service.rs apps/runtime/src-tauri/src/commands/desktop_lifecycle.rs apps/runtime/src-tauri/src/commands/session_runs.rs
git commit -m "feat(runtime): export observability snapshot in diagnostics"
```

### Task 4: Add Runtime Contract Testkit

**Files:**
- Create: `apps/runtime/src-tauri/tests/support/runtime_contract_testkit.rs`
- Create or modify: targeted contract test files under `apps/runtime/src-tauri/tests/`
- Test: new contract tests

**Step 1: Write the failing contract tests**

Create a reusable harness that can:

- seed runtime state
- execute contract scenarios
- capture emitted outcomes
- assert stable success/failure semantics

Add first contract scenarios for:

- successful run
- admission conflict
- loop interception
- approval resume
- child-session success
- child-session failure

**Step 2: Run contract tests to verify failure**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml runtime_contract -- --nocapture`
Expected: FAIL because the harness and/or scenarios do not exist yet

**Step 3: Write minimal implementation**

Build the harness around existing session-run state, event exports, and runtime helpers. Keep the contract layer readably small and behavior-focused.

**Step 4: Run tests to verify pass**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml runtime_contract -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/tests/support/runtime_contract_testkit.rs apps/runtime/src-tauri/tests
git commit -m "test(runtime): add runtime contract harness"
```

### Task 5: Upgrade Trace Fixtures Into First-Class Contract Outputs

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/runtime/trace_builder.rs`
- Modify: `apps/runtime/src-tauri/tests/fixtures/run_traces/*.json`
- Test: `apps/runtime/src-tauri/src/agent/runtime/trace_builder.rs`

**Step 1: Write the failing fixture assertions**

Add or extend fixture-driven cases to validate:

- snapshot-aware trace exports remain normalized
- parse warnings remain stable
- dynamic timestamps and hidden session identifiers remain normalized

**Step 2: Run tests to verify failure**

Run: `cargo test --lib trace_fixture --manifest-path apps/runtime/src-tauri/Cargo.toml -- --nocapture`
Expected: FAIL if fixture outputs are not updated to the new contract shape

**Step 3: Write minimal implementation**

Update normalization only where necessary and keep fixture shapes readable for long-term maintenance.

**Step 4: Run tests to verify pass**

Run: `cargo test --lib trace_fixture --manifest-path apps/runtime/src-tauri/Cargo.toml -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/runtime/trace_builder.rs apps/runtime/src-tauri/tests/fixtures/run_traces
git commit -m "test(runtime): align trace fixtures with observability contracts"
```

### Task 6: Final Verification

**Files:**
- Verify only

**Step 1: Run Rust fast path**

Run: `pnpm test:rust-fast`
Expected: PASS

**Step 2: Run targeted runtime unit coverage**

Run:
- `cargo test --lib observability --manifest-path apps/runtime/src-tauri/Cargo.toml -- --nocapture`
- `cargo test --lib session_runs --manifest-path apps/runtime/src-tauri/Cargo.toml -- --nocapture`
- `cargo test --lib desktop_lifecycle --manifest-path apps/runtime/src-tauri/Cargo.toml -- --nocapture`
- `cargo test --lib trace_fixture --manifest-path apps/runtime/src-tauri/Cargo.toml -- --nocapture`

Expected: PASS

**Step 3: Run contract coverage**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml runtime_contract -- --nocapture`
Expected: PASS

**Step 4: Run desktop packaging check**

Run: `pnpm build:runtime`
Expected: PASS

**Step 5: Commit**

Do not create a new commit unless code changed during verification fixes.

### Task 7: Finish Branch

**Files:**
- Verify only

**Step 1: Inspect final branch status**

Run:
- `git status --short --branch`
- `git log --oneline --decorate -8`

Expected: clean feature branch with the planned commit sequence

**Step 2: Prepare completion summary**

Summarize:

- OpenClaw-aligned observability modules added
- diagnostics export additions
- runtime contract harness coverage
- fixture contract coverage
- exact verification commands and results

**Step 3: Finish with `finishing-a-development-branch`**

Present the standard integration options after verification passes.
