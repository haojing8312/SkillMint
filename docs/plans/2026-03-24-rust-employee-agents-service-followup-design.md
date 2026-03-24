# Rust Employee Agents Service Follow-up Design

**Goal:** Finish the next safe shrink step for `apps/runtime/src-tauri/src/commands/employee_agents/service.rs` by moving the remaining group-run execution and session orchestration into a focused child module without changing the `group_run_entry.rs -> super::service::*` call surface.

## Why This Follow-up Exists

`employee_agents.rs` and `employee_agents/repo.rs` are already back under the governance line, but `service.rs` is still above the target and still owns the heaviest execution path:

- group-run session bootstrap
- group-step session reuse and creation
- assistant message append helpers
- employee-context execution orchestration
- group-run start bootstrap

Those functions are cohesive enough to move together, and they already sit on the same call path used by `group_run_entry.rs`.

## Recommended Split

Create one new child module:

- `group_run_execution_service.rs`

Move these functions into it:

- `execute_group_step_in_employee_context_with_pool`
- `ensure_group_run_session_with_pool`
- `append_group_run_assistant_message_with_pool`
- `ensure_group_step_session_with_pool`
- `start_employee_group_run_internal_with_pool`

Keep these in `service.rs`:

- thin `pub(super)` re-exports for the moved functions
- existing lower-risk group-run progress helpers already moved to `group_run_progress_service.rs`
- existing status/update helpers such as `mark_group_run_step_*`

## Why One Module First

Two-module variants such as `group_run_session_service.rs + group_run_execution_service.rs` are possible, but they would create more churn right now because:

- `start_employee_group_run_internal_with_pool` depends on both session bootstrap and execution helper composition
- `group_run_entry.rs` already treats these helpers as one execution lane
- the immediate goal is to bring `service.rs` under the governance threshold, not to maximize fragmentation

So the smallest safe move is one execution-focused child module first.

## Compatibility Rules

- `group_run_entry.rs` must keep calling `super::service::*`
- SQL access must stay in repo modules; this follow-up should not pull SQL back into service
- existing user-visible behavior, group-run state transitions, and session reuse behavior must stay unchanged
- no Tauri command signatures change

## Test Strategy

Preserve existing focused coverage and add at least one test that exercises the newly extracted module through the stable service entrypoint. Good candidates:

- `load_group_run_continue_state_requires_run_id`
- `review_group_run_step_requires_valid_action`
- one execution/session-specific guard such as empty preferred session handling or assistant append behavior

## Success Criteria

- `service.rs` drops below the `<= 800` governance target if practical, or at minimum meaningfully closer without creating a new giant child file
- `group_run_entry.rs` stays unchanged at the call boundary
- focused cargo tests stay green
- `pnpm test:rust-fast` stays green
