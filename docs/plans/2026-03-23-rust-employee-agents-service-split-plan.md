# Rust Employee Agents Service Split Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Extract employee profile CRUD from `apps/runtime/src-tauri/src/commands/employee_agents.rs` into command, service, and repository layers while preserving current Tauri behavior.

**Architecture:** Keep the existing Tauri command interface stable and move only employee profile management into a new `employee_agents` submodule. The command layer remains the mutation entrypoint and still performs Feishu reconciliation, while service and repo layers absorb validation, orchestration, and SQL.

**Tech Stack:** Rust, Tauri commands, sqlx, SQLite, WorkClaw runtime tests

---

### Task 1: Create the employee profile module skeleton

**Files:**
- Create: `apps/runtime/src-tauri/src/commands/employee_agents/service.rs`
- Create: `apps/runtime/src-tauri/src/commands/employee_agents/repo.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents.rs`

**Step 1: Add module declarations**

- Add a local `mod service;`
- Add a local `mod repo;`
- Re-export only the functions the root command file needs

**Step 2: Compile with placeholder functions**

Run: `cargo check -p runtime`
Expected: PASS with no behavior change

**Step 3: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/employee_agents.rs apps/runtime/src-tauri/src/commands/employee_agents/service.rs apps/runtime/src-tauri/src/commands/employee_agents/repo.rs
git commit -m "refactor(runtime): add employee agents service skeleton"
```

### Task 2: Move employee list read path into repository and service

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/service.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/repo.rs`

**Step 1: Move row-loading SQL into repo**

- Extract the `agent_employees` list query and skill-binding query into repo helpers
- Keep result ordering identical to today

**Step 2: Move employee shaping into service**

- Build the `AgentEmployee` output in service
- Preserve fallback behavior for empty `employee_id`

**Step 3: Rewire command root**

- Make `list_agent_employees_with_pool` a thin wrapper or remove it if the root command can call service directly

**Step 4: Verify**

Run: `pnpm test:rust-fast`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/employee_agents.rs apps/runtime/src-tauri/src/commands/employee_agents/service.rs apps/runtime/src-tauri/src/commands/employee_agents/repo.rs
git commit -m "refactor(runtime): extract employee list service"
```

### Task 3: Move employee upsert path into service and repository

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/service.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/repo.rs`
- Test: `apps/runtime/src-tauri/tests/` related employee tests or add a new focused Rust test file if coverage is missing

**Step 1: Add a focused failing Rust test**

- Cover one create/update path with:
  - employee creation
  - single-default behavior
  - skill binding persistence

**Step 2: Run the new test and confirm failure**

Run the narrowest rust test command for the new file or case
Expected: FAIL before implementation is complete

**Step 3: Move validation/orchestration into service**

- Normalize `employee_id`
- Preserve duplicate checks
- Preserve workdir creation behavior
- Preserve `is_default` behavior

**Step 4: Move SQL writes into repo**

- employee row upsert
- default reset update
- skill binding clear/reinsert

**Step 5: Keep command-layer reconcile intact**

- `upsert_agent_employee` still calls the same Feishu reconcile function after successful write

**Step 6: Verify**

Run: `pnpm test:rust-fast`
Expected: PASS

**Step 7: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/employee_agents.rs apps/runtime/src-tauri/src/commands/employee_agents/service.rs apps/runtime/src-tauri/src/commands/employee_agents/repo.rs apps/runtime/src-tauri/tests
git commit -m "refactor(runtime): extract employee upsert service"
```

### Task 4: Move delete path into service and repository

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/service.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/repo.rs`
- Test: Rust employee deletion coverage

**Step 1: Add failing delete coverage**

- Cover employee delete plus dependent cleanup already performed today

**Step 2: Move delete SQL into repo**

- delete employee skill bindings
- delete IM thread session references already handled in current logic
- delete employee row

**Step 3: Move orchestration into service**

- Keep the same error behavior

**Step 4: Keep command-layer reconcile intact**

- `delete_agent_employee` still reconciles Feishu state after successful delete

**Step 5: Verify**

Run: `pnpm test:rust-fast`
Expected: PASS

**Step 6: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/employee_agents.rs apps/runtime/src-tauri/src/commands/employee_agents/service.rs apps/runtime/src-tauri/src/commands/employee_agents/repo.rs apps/runtime/src-tauri/tests
git commit -m "refactor(runtime): extract employee delete service"
```

### Task 5: Shrink the command root and review boundaries

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/service.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/repo.rs`

**Step 1: Remove duplicated helpers left in the root file**

- Delete moved validation or SQL helpers that are no longer needed

**Step 2: Check public API boundaries**

- Ensure only employee profile CRUD moved
- Confirm group-run, IM, Feishu-association, and memory logic stayed untouched

**Step 3: Run verification**

Run:

```bash
pnpm test:rust-fast
pnpm --dir apps/runtime test -- App.employee-chat-entry
```

Expected: PASS

**Step 4: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/employee_agents.rs apps/runtime/src-tauri/src/commands/employee_agents/service.rs apps/runtime/src-tauri/src/commands/employee_agents/repo.rs
git commit -m "refactor(runtime): thin employee agent commands"
```
