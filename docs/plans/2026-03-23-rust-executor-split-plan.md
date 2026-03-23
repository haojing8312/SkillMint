# Rust Executor Split Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Split `apps/runtime/src-tauri/src/agent/executor.rs` into focused execution-boundary modules while keeping `AgentExecutor`, event names, approval semantics, cancel behavior, and stop reasons stable.

**Architecture:** Move pure helpers out first, then separate approval flow and event bridging, and only after that move the main turn loop into a thin orchestration module. Keep `apps/runtime/src-tauri/src/agent/mod.rs` as the public re-export surface and preserve the current behavior contract throughout the migration.

**Tech Stack:** Rust, Tauri, Tokio, SQLx, Serde, chrono, uuid, `pnpm test:rust-fast`

---

### Task 1: Extract Executor Event Types

**Files:**
- Create: `apps/runtime/src-tauri/src/agent/types.rs`
- Modify: `apps/runtime/src-tauri/src/agent/executor.rs`
- Modify: `apps/runtime/src-tauri/src/agent/mod.rs`

**Step 1: Write the failing test**

Add a small Rust test that imports `ToolCallEvent` and `AgentStateEvent` through the public `agent` module and asserts their serialized shape remains unchanged.

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib tool_call_event_serializes_expected_shape -- --nocapture`
Expected: FAIL until the new module is wired up.

**Step 3: Write minimal implementation**

Move the two event structs into `types.rs` and re-export them from `agent/mod.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib tool_call_event_serializes_expected_shape -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/types.rs apps/runtime/src-tauri/src/agent/executor.rs apps/runtime/src-tauri/src/agent/mod.rs
git commit -m "refactor(runtime): extract executor event types"
```

### Task 2: Extract Tool Context Setup

**Files:**
- Create: `apps/runtime/src-tauri/src/agent/context.rs`
- Modify: `apps/runtime/src-tauri/src/agent/executor.rs`
- Modify: `apps/runtime/src-tauri/src/agent/mod.rs`
- Test: `apps/runtime/src-tauri/src/agent/executor.rs`

**Step 1: Write the failing test**

Add a test that verifies `build_tool_context` still populates session ID, allowed tools, task temp dir, execution caps, and leaves `file_task_caps` empty.

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib tool_context_construction_includes_p0_metadata_slots -- --nocapture`
Expected: FAIL until the helper moves.

**Step 3: Write minimal implementation**

Move `build_tool_context` and `build_task_temp_dir` into `context.rs`, then import them back into `executor.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib tool_context_construction_includes_p0_metadata_slots -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/context.rs apps/runtime/src-tauri/src/agent/executor.rs apps/runtime/src-tauri/src/agent/mod.rs
git commit -m "refactor(runtime): extract executor context setup"
```

### Task 3: Extract Safety Classification

**Files:**
- Create: `apps/runtime/src-tauri/src/agent/safety.rs`
- Modify: `apps/runtime/src-tauri/src/agent/executor.rs`
- Modify: `apps/runtime/src-tauri/src/agent/mod.rs`
- Test: `apps/runtime/src-tauri/src/agent/executor.rs`

**Step 1: Write the failing test**

Add tests that cover `wait_for_tool_confirmation`, `classify_policy_blocked_tool_error`, and the delete-target summary behavior.

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib workspace_boundary_error_maps_to_policy_blocked -- --nocapture`
Expected: FAIL until the helpers move.

**Step 3: Write minimal implementation**

Move the destructive-action helper block into `safety.rs` and keep the same user-facing messages.

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib workspace_boundary_error_maps_to_policy_blocked -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/safety.rs apps/runtime/src-tauri/src/agent/executor.rs apps/runtime/src-tauri/src/agent/mod.rs
git commit -m "refactor(runtime): extract executor safety checks"
```

### Task 4: Extract Approval Flow

**Files:**
- Create: `apps/runtime/src-tauri/src/agent/approval_flow.rs`
- Modify: `apps/runtime/src-tauri/src/agent/executor.rs`
- Modify: `apps/runtime/src-tauri/src/agent/mod.rs`
- Test: `apps/runtime/src-tauri/src/agent/executor.rs`

**Step 1: Write the failing test**

Add a focused approval test that exercises `request_tool_approval_and_wait` with the existing approval bus behavior.

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib approval_bus_blocks_file_delete_until_resolved -- --nocapture`
Expected: FAIL until the approval-flow module is introduced.

**Step 3: Write minimal implementation**

Move `ToolConfirmationDecision`, `ApprovalWaitRuntime`, `resolve_approval_wait_runtime`, `wait_for_tool_confirmation`, and `request_tool_approval_and_wait` into `approval_flow.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib approval_bus_blocks_file_delete_until_resolved -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/approval_flow.rs apps/runtime/src-tauri/src/agent/executor.rs apps/runtime/src-tauri/src/agent/mod.rs
git commit -m "refactor(runtime): extract executor approval flow"
```

### Task 5: Extract Event Bridge

**Files:**
- Create: `apps/runtime/src-tauri/src/agent/event_bridge.rs`
- Modify: `apps/runtime/src-tauri/src/agent/executor.rs`
- Modify: `apps/runtime/src-tauri/src/agent/mod.rs`
- Test: `apps/runtime/src-tauri/src/agent/executor.rs`

**Step 1: Write the failing test**

Add tests around `append_tool_run_event`, `append_run_guard_warning_event`, and `build_skill_route_event` so the expected payloads stay stable.

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib skill_allowlist_error_is_not_policy_blocked -- --nocapture`
Expected: FAIL if helper wiring is broken.

**Step 3: Write minimal implementation**

Move the run-journal and skill-route helpers into `event_bridge.rs` and keep `executor.rs` using them as utilities.

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib skill_allowlist_error_is_not_policy_blocked -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/event_bridge.rs apps/runtime/src-tauri/src/agent/executor.rs apps/runtime/src-tauri/src/agent/mod.rs
git commit -m "refactor(runtime): extract executor event bridge"
```

### Task 6: Extract Progress Helpers

**Files:**
- Create: `apps/runtime/src-tauri/src/agent/progress.rs`
- Modify: `apps/runtime/src-tauri/src/agent/executor.rs`
- Modify: `apps/runtime/src-tauri/src/agent/mod.rs`

**Step 1: Write the failing test**

Add a small test that locks `text_progress_signature` and `json_progress_signature` to their current deterministic output behavior.

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib tool_context_reuses_task_temp_dir_for_same_session -- --nocapture`
Expected: FAIL if the helper module is not wired correctly.

**Step 3: Write minimal implementation**

Move the progress-signature helpers into `progress.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib tool_context_reuses_task_temp_dir_for_same_session -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/progress.rs apps/runtime/src-tauri/src/agent/executor.rs apps/runtime/src-tauri/src/agent/mod.rs
git commit -m "refactor(runtime): extract executor progress helpers"
```

### Task 7: Move The Core Turn Loop

**Files:**
- Create: `apps/runtime/src-tauri/src/agent/turn_executor.rs`
- Modify: `apps/runtime/src-tauri/src/agent/executor.rs`
- Modify: `apps/runtime/src-tauri/src/agent/mod.rs`

**Step 1: Write the failing test**

Add one integration-style lib test that exercises `AgentExecutor::execute_turn` on a simple text-only response and one tool-call response path.

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib tool_confirmation_false_is_rejected -- --nocapture`
Expected: FAIL if the loop wiring is incomplete.

**Step 3: Write minimal implementation**

Move the body of `execute_turn` into `turn_executor.rs`, keeping `AgentExecutor` as the public entrypoint in `executor.rs`.

**Step 4: Run test to verify it passes**

Run: `pnpm test:rust-fast`
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/turn_executor.rs apps/runtime/src-tauri/src/agent/executor.rs apps/runtime/src-tauri/src/agent/mod.rs
git commit -m "refactor(runtime): split agent executor turn loop"
```

### Task 8: Move Executor Tests Out Of The Root File

**Files:**
- Create: `apps/runtime/src-tauri/src/agent/tests.rs`
- Modify: `apps/runtime/src-tauri/src/agent/executor.rs`
- Modify: `apps/runtime/src-tauri/src/agent/mod.rs`

**Step 1: Write the failing test**

No new behavior test is needed here; this task is about relocation only. Make sure the existing executor tests still compile once imported from `tests.rs`.

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib approval_bus_blocks_file_delete_until_resolved -- --nocapture`
Expected: FAIL if the module path or imports are wrong.

**Step 3: Write minimal implementation**

Move the root-file test module into `tests.rs` and keep the module imports explicit.

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib approval_bus_blocks_file_delete_until_resolved -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/tests.rs apps/runtime/src-tauri/src/agent/executor.rs apps/runtime/src-tauri/src/agent/mod.rs
git commit -m "refactor(runtime): move executor tests into module"
```

### Task 9: Final Verification Sweep

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/mod.rs`
- Verify: all files touched in earlier tasks

**Step 1: Run the focused Rust fast path**

Run: `pnpm test:rust-fast`
Expected: PASS.

**Step 2: Run the executor-specific lib tests**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib tool_confirmation_timeout_is_treated_as_rejection -- --nocapture`
Expected: PASS.

**Step 3: Run the adjacent runtime tests if any imports moved**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_feishu_gateway -- --nocapture`
Expected: PASS.

**Step 4: Commit the cleanup if needed**

```bash
git add apps/runtime/src-tauri/src/agent/mod.rs
git commit -m "refactor(runtime): finalize executor split"
```
