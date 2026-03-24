# Rust Employee Agents Test Visibility Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a narrow public integration-test support surface for `employee_agents` so `test_im_employee_agents` can compile through an intentional contract and the later large-test-file split can proceed safely.

**Architecture:** Keep the normal `employee_agents` root API small. Introduce a dedicated `test_support` child module that re-exports only the currently private helpers needed by the integration test binary. Then update the test binary to import those helpers from the new surface instead of from accidental private root re-exports.

**Tech Stack:** Rust, Tauri runtime library crate, Tokio integration tests, sqlx, SQLite

---

### Task 1: Create the test-support surface

**Files:**
- Create: `apps/runtime/src-tauri/src/commands/employee_agents/test_support.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents.rs`

**Step 1: Add the dedicated module**

- create `test_support.rs`
- re-export only the private helpers currently needed by `test_im_employee_agents`

**Step 2: Expose it intentionally**

- add `#[doc(hidden)] pub mod test_support;` from the root module
- do not move business logic into this file

**Step 3: Verify library compile**

Run:

```bash
cargo check --manifest-path apps/runtime/src-tauri/Cargo.toml -q
```

Expected: PASS

### Task 2: Repoint the integration test imports

**Files:**
- Modify: `apps/runtime/src-tauri/tests/test_im_employee_agents.rs`

**Step 1: Keep current public-root imports for already-public helpers**

- leave normal public helpers on `runtime_lib::commands::employee_agents::*`

**Step 2: Move private-helper imports to test support**

- import private group-management and group-run-entry helpers from:
  - `runtime_lib::commands::employee_agents::test_support::*`

**Step 3: Avoid semantic edits**

- do not rewrite test bodies
- do not rename tests
- only update the import boundary

### Task 3: Verify the binary compiles through the new contract

**Files:**
- Modify: `apps/runtime/src-tauri/tests/test_im_employee_agents.rs`

**Step 1: Run the focused target**

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_im_employee_agents employee_config_and_im_session_mapping_work -- --nocapture
```

Expected: compile succeeds and the targeted test runs

**Step 2: Run the whole binary if the focused test compiles**

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_im_employee_agents -- --nocapture
```

Expected: PASS, or at minimum failures are now real behavior failures rather than private-import compile errors

### Task 4: Resume the giant test-file split

**Files:**
- Modify: `docs/plans/2026-03-24-rust-test-im-employee-agents-split-design.md`
- Modify: `docs/plans/2026-03-24-rust-test-im-employee-agents-split-plan.md`

**Step 1: Mark the prerequisite complete**

- note that `employee_agents` now has a stable integration-test support surface

**Step 2: Restart the split from `im_routing.rs`**

- move the first routing cluster only
- verify the binary again before moving the next scenario family
