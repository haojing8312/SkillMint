# Rust Employee Agents Service Follow-up Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Shrink `apps/runtime/src-tauri/src/commands/employee_agents/service.rs` by extracting the remaining group-run execution and session orchestration into one focused child module while preserving existing service entrypoints for `group_run_entry.rs`.

**Architecture:** Introduce `group_run_execution_service.rs` as a child module under `employee_agents/`. Keep `service.rs` as the stable aggregation layer by re-exporting the moved functions, and avoid any repo/API contract changes during this step.

**Tech Stack:** Rust, sqlx, SQLite, Tauri runtime tests

---

### Task 1: Create the execution child module skeleton

**Files:**
- Create: `apps/runtime/src-tauri/src/commands/employee_agents/group_run_execution_service.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/service.rs`

**Step 1: Add the module declaration**

- Add `#[path = "group_run_execution_service.rs"] mod group_run_execution_service;`
- Re-export the moved functions from `service.rs`

**Step 2: Move the first low-risk helper**

- Move `append_group_run_assistant_message_with_pool`
- Keep visibility compatible with `group_run_entry.rs`

**Step 3: Compile-check through focused tests**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib review_group_run_step_requires_valid_action -- --nocapture
```

Expected: PASS

### Task 2: Move session bootstrap helpers

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/service.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/group_run_execution_service.rs`

**Step 1: Move**

- `ensure_group_run_session_with_pool`
- `ensure_group_step_session_with_pool`

**Step 2: Keep service re-export stable**

- `group_run_entry.rs` should not need direct edits

**Step 3: Verify**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib ensure_employee_sessions_for_event_returns_empty_when_no_employee_matches -- --nocapture
```

Expected: PASS

### Task 3: Move employee-context execution helper

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/service.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/group_run_execution_service.rs`

**Step 1: Move**

- `execute_group_step_in_employee_context_with_pool`

**Step 2: Preserve helper imports**

- keep `extract_assistant_text_content`, `AgentExecutor`, tool registry, and permission behavior unchanged

**Step 3: Verify**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib review_group_run_step_requires_valid_action -- --nocapture
```

Expected: PASS

### Task 4: Move group-run start bootstrap

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/service.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/group_run_execution_service.rs`

**Step 1: Move**

- `start_employee_group_run_internal_with_pool`

**Step 2: Keep dependent helpers reachable**

- continue using stable service-level exports for any callers outside the module

**Step 3: Add or keep one focused regression test**

- prefer a guard on missing run/session inputs rather than a broad integration rewrite

### Task 5: Verify the full follow-up

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/service.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/group_run_execution_service.rs`

**Step 1: Run focused cargo tests**

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib load_group_run_continue_state_requires_run_id -- --nocapture
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib review_group_run_step_requires_valid_action -- --nocapture
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib ensure_employee_sessions_for_event_returns_empty_when_no_employee_matches -- --nocapture
```

**Step 2: Run WorkClaw Rust fast verification**

```bash
pnpm test:rust-fast
```

**Step 3: Review residual warnings**

- note any remaining compatibility re-export warnings separately from behavior regressions
