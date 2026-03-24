# Rust Employee Agents Repo Split Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Split `apps/runtime/src-tauri/src/commands/employee_agents/repo.rs` into a thin aggregation shell plus focused child repos for `group_run`, `session`, and `feishu_binding` persistence.

**Architecture:** Keep `repo.rs` as the stable import surface for the rest of the employee-agents module while moving SQL and row structs into persistence-specific child repos. Extract the largest lane first (`group_run_repo.rs`), then the session/thread lane, then the Feishu binding lane.

**Tech Stack:** Rust, sqlx, SQLite, Tauri runtime tests

---

## Guardrails

- Preserve all current SQL semantics, null/default handling, and result ordering.
- Keep `repo.rs` import compatibility by re-exporting child functions during the transition.
- Do not move profile CRUD out of `profile_repo.rs`; that lane is already in the right place.
- Avoid creating a new giant child repo file by extracting the largest lane first and checking size after each task.

## Task 1: Extract `group_run_repo.rs`

**Files:**
- Create: `apps/runtime/src-tauri/src/commands/employee_agents/group_run_repo.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/repo.rs`

**Step 1: Move group-run rows and queries**

- Move the row structs and SQL helpers that operate on:
  - `group_runs`
  - `group_run_steps`
  - `group_run_events`
- Include read-model helpers, retry/reassign/review state transitions, and snapshot queries.

**Step 2: Keep the root import surface stable**

- Re-export the moved functions and row structs from `repo.rs`.
- Do not force the rest of the employee-agents module to update imports yet.

**Step 3: Verify**

Run:
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib resume_employee_group_run_requires_paused_state -- --nocapture`
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib get_group_run_session_id_returns_not_found_for_missing_run -- --nocapture`
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib retry_employee_group_run_failed_steps_requires_failed_rows -- --nocapture`

Expected:
- PASS

## Task 2: Extract `session_repo.rs`

**Files:**
- Create: `apps/runtime/src-tauri/src/commands/employee_agents/session_repo.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/repo.rs`

**Step 1: Move session/thread persistence**

- Move:
  - session seed helpers
  - `im_thread_sessions` lookup and upsert helpers
  - inbound event link persistence
  - session message insert/list helpers
  - the row structs used only by this lane

**Step 2: Re-export from root**

- Re-export the moved session helpers from `repo.rs`.
- Preserve current callers and behavior.

**Step 3: Verify**

Run:
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib ensure_employee_sessions_for_event_returns_empty_when_no_employee_matches -- --nocapture`
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib resolve_target_employees_for_event_prefers_explicit_role_match -- --nocapture`
- `pnpm test:rust-fast`

Expected:
- PASS

## Task 3: Extract `feishu_binding_repo.rs`

**Files:**
- Create: `apps/runtime/src-tauri/src/commands/employee_agents/feishu_binding_repo.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/repo.rs`

**Step 1: Move Feishu routing binding persistence**

- Move:
  - employee association rows used only for Feishu binding updates
  - binding count/list helpers
  - displaced-binding cleanup
  - binding insert helpers
  - any remaining `im_routing_bindings` SQL owned by employee-agents

**Step 2: Re-export from root**

- Re-export the moved helpers from `repo.rs`.
- Keep behavior and call sites stable.

**Step 3: Verify**

Run:
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib save_feishu_employee_association_rejects_invalid_mode -- --nocapture`
- `pnpm test:rust-fast`

Expected:
- PASS

## Task 4: Thin the root repo shell

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/repo.rs`

**Step 1: Reduce root `repo.rs` to module declarations and re-exports**

- Keep:
  - child module declarations
  - re-exports
  - only the smallest compatibility glue if still necessary
- Remove row structs that now belong in child repos.

**Step 2: Keep or move the current repo test**

- If the existing list-ordering test naturally belongs in `profile_repo.rs`, move it there.
- Otherwise keep it temporarily only if it still matches the root file’s reduced role.

**Step 3: Verify**

Run:
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib list_agent_employee_rows_orders_default_before_recent_updates -- --nocapture`
- `pnpm test:rust-fast`

Expected:
- PASS

## Task 5: Update local Rust governance docs if needed

**Files:**
- Modify: `apps/runtime/src-tauri/AGENTS.md`
- Modify: `docs/plans/2026-03-23-rust-large-file-backlog.md`

**Step 1: Record the child-repo governance rule**

- Add short guidance that:
  - `group_run_repo.rs` owns group run persistence
  - `session_repo.rs` owns thread/session persistence
  - `feishu_binding_repo.rs` owns Feishu routing persistence

**Step 2: Verify**

- No code verification required if this step is docs-only.

Plan complete and saved to `docs/plans/2026-03-24-rust-employee-agents-repo-split-plan.md`. Two execution options:

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

**Which approach?**
