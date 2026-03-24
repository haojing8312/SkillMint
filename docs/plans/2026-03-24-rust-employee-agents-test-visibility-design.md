# Rust Employee Agents Test Visibility Design

**Goal:** Define a stable, narrow visibility surface for `apps/runtime/src-tauri/tests/test_im_employee_agents.rs` so the integration test binary can compile and future test-file splitting can proceed without exposing the full `employee_agents` internal API.

## Problem Summary

`test_im_employee_agents.rs` currently imports a mixed set of `_with_pool` helpers from `runtime_lib::commands::employee_agents`.

Some of these helpers are already public:

- `list_agent_employees_with_pool`
- `save_feishu_employee_association_with_pool`
- `upsert_agent_employee_with_pool`
- `review_group_run_step_with_pool`
- `pause_employee_group_run_with_pool`
- `resume_employee_group_run_with_pool`
- `reassign_group_run_step_with_pool`
- `get_employee_group_run_snapshot_with_pool`
- `cancel_employee_group_run_with_pool`
- `retry_employee_group_run_failed_steps_with_pool`
- `ensure_employee_sessions_for_event_with_pool`
- `link_inbound_event_to_session_with_pool`

But a second set still comes from private or `pub(crate)` re-exports:

- `clone_employee_group_template_with_pool`
- `create_employee_group_with_pool`
- `create_employee_team_with_pool`
- `delete_employee_group_with_pool`
- `list_employee_group_rules_with_pool`
- `list_employee_groups_with_pool`
- `continue_employee_group_run_with_pool`
- `run_group_step_with_pool`
- `start_employee_group_run_with_pool`
- `maybe_handle_team_entry_session_message_with_pool`

That leaves the integration test binary depending on an accidental visibility shape rather than an intentional contract.

## Why `#[cfg(test)]` Is Not The Right Fix

`test_im_employee_agents.rs` is an integration test, not a unit test inside the library crate.

That means:

- `#[cfg(test)]` on library-only exports will not provide a stable surface to this binary
- the library must expose a callable surface through normal compilation if integration tests are expected to import it

So the right design is not a hidden `cfg(test)` escape hatch inside `employee_agents.rs`. The right design is a deliberate, narrow public test-support surface.

## Approaches Considered

### 1. Recommended: narrow public test-support surface

Add an explicit `employee_agents::test_support` surface that re-exports only the helpers needed by integration tests.

Example direction:

- `runtime_lib::commands::employee_agents::test_support::*`

Characteristics:

- public enough for integration tests to compile
- narrow enough to avoid promoting every helper into the main production API
- easy to audit because all integration-test-facing helpers live in one place

Recommended refinements:

- mark the module `#[doc(hidden)]` to avoid treating it as a normal app-facing API
- keep all re-exports grouped and commented as integration-test support only
- let `test_im_employee_agents.rs` import from `test_support` instead of from the root module

### 2. Make all current helpers public at the root

This is the fastest implementation but the worst long-term contract.

Problems:

- expands the normal `employee_agents` API surface
- makes internal orchestration helpers harder to refactor later
- keeps tests coupled to the root module rather than an intentional boundary

### 3. Rewrite integration tests to avoid helper imports

This is the cleanest conceptual boundary, but not the smallest safe next step.

Problems:

- would require reworking a large existing test binary
- would delay the current giant-test-file governance work
- would blur whether failures come from behavior changes or from test harness rewrites

## Recommended Design

Use **Approach 1**: add a narrow public `test_support` surface under `employee_agents`.

### Proposed shape

- Root module keeps normal app-facing exports unchanged
- Add a dedicated child module, for example:
  - `apps/runtime/src-tauri/src/commands/employee_agents/test_support.rs`
- The root module exposes it as:
  - `#[doc(hidden)] pub mod test_support;`

### What belongs in `test_support`

Only helpers currently needed by integration tests and not already part of the normal public surface:

- group management helpers
- group-run entry helpers
- team-entry helpers

More concretely, the first export set should include:

- `clone_employee_group_template_with_pool`
- `create_employee_group_with_pool`
- `create_employee_team_with_pool`
- `delete_employee_group_with_pool`
- `list_employee_group_rules_with_pool`
- `list_employee_groups_with_pool`
- `continue_employee_group_run_with_pool`
- `run_group_step_with_pool`
- `start_employee_group_run_with_pool`
- `maybe_handle_team_entry_session_message_with_pool`

Helpers that are already public can stay on the root surface and do not need to be duplicated unless consistency becomes more important than minimal change.

## Import Strategy For Tests

After this change:

- `test_im_employee_agents.rs` should import existing public functions from the root module as before
- private-helper callers should switch to:
  - `runtime_lib::commands::employee_agents::test_support::*`

This keeps the migration incremental and avoids a giant import rewrite.

## Compatibility Rules

- Do not change helper behavior as part of the visibility change
- Do not rename test functions
- Do not move production logic between modules in the same step
- Do not widen the root `employee_agents` API just to satisfy tests

## Success Criteria

- `test_im_employee_agents` compiles without relying on accidental private imports
- `employee_agents` keeps a narrow app-facing public surface
- integration-test-only helper exposure is centralized and auditable
- the later `test_im_employee_agents.rs` split can resume from a stable contract
