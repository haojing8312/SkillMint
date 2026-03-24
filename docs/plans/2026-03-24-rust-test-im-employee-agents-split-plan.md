# Rust IM Employee Agents Test Split Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Split `apps/runtime/src-tauri/tests/test_im_employee_agents.rs` into scenario-focused child modules while preserving integration-test semantics and keeping production code untouched.

**Architecture:** Turn the root integration test file into a thin shell that keeps `mod helpers;` and registers child modules under `tests/test_im_employee_agents/`. Move scenario families one cluster at a time, starting with IM routing and session bridge coverage because it is already cohesive and lowest-risk.

**Tech Stack:** Rust integration tests, Tokio, sqlx, SQLite fixtures, WorkClaw runtime integration test harness

---

### Task 1: Register the child test module

**Files:**
- Modify: `apps/runtime/src-tauri/tests/test_im_employee_agents.rs`
- Create: `apps/runtime/src-tauri/tests/test_im_employee_agents/im_routing.rs`

**Step 1: Add the module declaration**

- Keep `mod helpers;` in the root file
- Add `#[path = "test_im_employee_agents/im_routing.rs"] mod im_routing;`

**Step 2: Move the first IM routing test**

- Move `employee_config_and_im_session_mapping_work`

**Step 3: Compile through the integration test binary**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_im_employee_agents employee_config_and_im_session_mapping_work -- --nocapture
```

Expected: PASS

**Status update**

- The visibility blocker has been resolved through a narrow `employee_agents::test_support` surface.
- The first split slice has started: three IM routing tests have moved into `im_routing.rs`.

### Task 2: Move the remaining routing and binding tests

**Files:**
- Modify: `apps/runtime/src-tauri/tests/test_im_employee_agents.rs`
- Modify: `apps/runtime/src-tauri/tests/test_im_employee_agents/im_routing.rs`

**Step 1: Move**

- `group_message_without_mention_routes_to_main_employee`
- `group_message_with_mention_routes_to_target_employee`
- `save_feishu_employee_association_replaces_default_binding_and_updates_scope`
- `save_feishu_employee_association_rolls_back_scope_update_when_binding_insert_fails`
- `wecom_event_prefers_wecom_scoped_employee_and_creates_session`
- `group_message_with_text_mention_routes_to_target_employee_when_role_id_missing`
- `ensure_employee_sessions_for_event_prefers_team_entry_employee_when_binding_team_id_matches`

**Step 2: Keep imports local to the child module**

- prefer importing runtime commands directly inside `im_routing.rs`
- do not create a second helper layer

**Step 3: Remove now-unused root imports**

- shrink the root test file rather than leaving a stale giant import block

### Task 3: Verify the first split milestone

**Files:**
- Modify: `apps/runtime/src-tauri/tests/test_im_employee_agents.rs`
- Modify: `apps/runtime/src-tauri/tests/test_im_employee_agents/im_routing.rs`

**Step 1: Run the full integration test binary**

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_im_employee_agents -- --nocapture
```

Expected: PASS

**Current decision**

- Continue implementation from `im_routing.rs`.
- Keep the binary green after each small routing cluster move.
- Do not jump to `group_run.rs` until the remaining routing and binding cases are fully moved and reverified.

**Step 2: Review remaining root size**

- confirm the root file is materially smaller
- confirm the moved module boundary is scenario-based, not helper-based

### Task 4: Prepare the next split wave

**Files:**
- Modify: `docs/plans/2026-03-24-rust-test-im-employee-agents-split-design.md`
- Modify: `docs/plans/2026-03-24-rust-test-im-employee-agents-split-plan.md`

**Step 1: Record the first milestone**

- note that `im_routing.rs` is complete

**Step 2: Queue the next files**

- `group_management.rs`
- `group_run.rs`
- `team_entry.rs`

**Step 3: Keep legacy-schema coverage as a future bucket only if new cohesive tests are added**

- do not force a fake `legacy_schema.rs` module before there is enough real test content for it

### Task 5: Finish the remaining IM routing cases

**Files:**
- Modify: `apps/runtime/src-tauri/tests/test_im_employee_agents.rs`
- Modify: `apps/runtime/src-tauri/tests/test_im_employee_agents/im_routing.rs`

**Step 1: Move the remaining routing and binding tests**

- `save_feishu_employee_association_replaces_default_binding_and_updates_scope`
- `save_feishu_employee_association_rolls_back_scope_update_when_binding_insert_fails`
- `wecom_event_prefers_wecom_scoped_employee_and_creates_session`
- `group_message_with_text_mention_routes_to_target_employee_when_role_id_missing`
- `ensure_employee_sessions_for_event_prefers_team_entry_employee_when_binding_team_id_matches`

**Step 2: Run the full binary again**

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_im_employee_agents -- --nocapture
```

Expected: PASS

### Task 6: Add the missing prerequisite before resuming future scenario waves

**Files:**
- Out of scope for this plan; requires a follow-up design on the `employee_agents` test visibility surface

**Step 1: Define the integration-test callable surface**

- decide which `employee_agents` helpers should be reachable from integration tests
- avoid relying on accidental private imports

**Step 2: Resume this split plan only after that boundary is explicit**

- restart from `im_routing.rs`
- rerun the targeted binary command before moving additional scenario families
