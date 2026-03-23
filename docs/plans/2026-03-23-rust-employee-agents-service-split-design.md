# Rust Employee Agents Service Split Design

**Goal:** Reduce the responsibility of [employee_agents.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/employee_agents.rs) by extracting employee profile CRUD logic into a reusable service and repository layer without changing the Tauri command contract.

## Scope

- Extract employee profile listing, create/update, delete, default-employee switching, and skill binding persistence
- Preserve current Tauri command names and response payloads
- Keep existing database schema unchanged in this phase
- Keep Feishu reconcile side effects intact at the command boundary

## Current Problem

[employee_agents.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/employee_agents.rs) is currently 4745 lines and mixes:

- Tauri command entrypoints
- employee validation and normalization
- employee CRUD business rules
- SQLite query logic
- skill binding persistence
- downstream side effects after mutation

This makes the employee domain hard to test, hard to extend, and too expensive to modify safely.

## Recommended Design

### 1. Split employee profile management into a submodule

Introduce a dedicated module under `apps/runtime/src-tauri/src/commands/employee_agents/`:

- `mod.rs` or the existing root file keeps command entrypoints and shared types
- `service.rs` owns employee profile business logic
- `repo.rs` owns `agent_employees` and `agent_employee_skills` queries

The first cut only covers employee profile management, not all employee-domain features.

### 2. Keep commands thin

Tauri commands should only:

- receive command input
- call the employee profile service
- preserve existing post-write side effects like Feishu reconciliation
- return the same shapes as today

This keeps the external contract stable while making the internal structure maintainable.

### 3. Move SQL into repository functions

The repository layer should own:

- listing employees and attached skills
- checking duplicate `employee_id`
- clearing and rewriting skill bindings
- deleting employee rows and direct dependent rows already handled today
- enforcing single-default behavior through the same SQL updates currently in use

This makes later Rust-side regression tests much easier to add.

## First-Cut Boundary

### In scope for the first split

- `list_agent_employees_with_pool`
- `upsert_agent_employee_with_pool`
- `delete_agent_employee_with_pool`
- helper logic directly tied to employee profile persistence
- skill binding read/write used by employee CRUD

### Out of scope for the first split

- `save_feishu_employee_association_with_pool`
- IM routing / thread session orchestration
- employee group / team / run logic
- employee memory export / clear / stats logic
- event routing and session bootstrapping

## Responsibility Split

### Command layer

- `list_agent_employees`
- `upsert_agent_employee`
- `delete_agent_employee`
- preserve Feishu reconcile call after upsert/delete

### Service layer

- validate input
- normalize employee identity and workdir defaults
- decide when to clear other defaults
- prepare skill binding updates
- orchestrate repository calls

### Repository layer

- execute SQL
- map rows into domain structs
- persist employee and skill binding rows

## Risks

- Losing mutation side effects after moving logic out of commands
- Accidentally changing default employee semantics
- Accidentally changing ordering or payload shaping in employee list results

## Smallest Safe Path

1. Create service/repo files for employee profile management only
2. Move read path first (`list_agent_employees_with_pool`)
3. Move write path second (`upsert_agent_employee_with_pool`)
4. Move delete path third (`delete_agent_employee_with_pool`)
5. Keep command entrypoints and Feishu reconcile flow unchanged

## Success Criteria

- [employee_agents.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/employee_agents.rs) is materially smaller
- employee profile CRUD no longer lives directly inside the command giant file
- Tauri command names and payloads remain unchanged
- existing employee UI flows continue to pass verification
- the same command/service/repo pattern can be reused for later employee-domain splits
