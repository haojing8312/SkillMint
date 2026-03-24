# Rust IM Employee Agents Test Split Design

**Goal:** Shrink `apps/runtime/src-tauri/tests/test_im_employee_agents.rs` into a thin integration-test shell that delegates to scenario-focused child modules, without changing test semantics or production code.

## Why This File Needs Governance

`test_im_employee_agents.rs` is now one of the largest Rust integration test files in the repository. It has grown into a mixed surface that covers:

- employee config and IM session mapping
- IM routing and binding behavior across Feishu and WeCom
- employee group and team management
- group-run lifecycle, review, retry, reassign, and snapshots
- team-entry chat session behavior

That makes it expensive to read, expensive to review, and increasingly risky to extend because unrelated scenario families sit in one file and share one import surface.

## Observed Scenario Families

The current file clusters naturally into these scenario families:

1. **IM routing and session bridge**
   - employee config and session mapping
   - group mention routing
   - Feishu or WeCom binding behavior
   - route-to-session persistence

2. **Employee group and team management**
   - create/list/delete group behavior
   - review rule listing
   - team runtime config creation
   - template clone behavior

3. **Group run, review, retry, and reassign**
   - run start and snapshot behavior
   - execute-step context
   - pause/resume/cancel
   - review approve/reject loops
   - retry and reassign edge cases

4. **Team-entry orchestration**
   - entry-session filtering
   - reuse of existing chat session during group run startup

## Important Constraint

The suggested `legacy schema / migration compatibility` split direction is valid as a long-term test bucket, but this file does not currently contain a cohesive block of explicit legacy-schema regression tests. So the first safe implementation slice should not invent that bucket here. The first real split should follow the scenario family that already exists and is easiest to move without changing meaning.

## Current Blocker

An attempted first implementation slice showed that the `test_im_employee_agents` integration binary currently imports many private `_with_pool` functions from `runtime_lib::commands::employee_agents`.

This means the current obstacle is not the test-file layout itself. The real obstacle is a pre-existing visibility mismatch:

- the production module still keeps many helpers private or `pub(crate)`
- the integration test binary imports those helpers as if they were part of a stable external test surface

As long as that boundary stays unresolved, any partial file split will be difficult to verify honestly because the integration test binary already fails compilation on private imports.

## Status Update

This blocker has now been addressed by introducing a narrow `employee_agents::test_support` surface for the integration test binary.

That means the split can resume, but it should still proceed incrementally:

- first move the lowest-risk IM routing tests
- verify the whole binary
- then continue with the next scenario family

## Recommended Split

Create a sibling directory:

- `apps/runtime/src-tauri/tests/test_im_employee_agents/`

Then evolve the root test file into a shell that keeps:

- `mod helpers;`
- child-module declarations
- only the smallest shared import surface needed by multiple children, if any

Planned child modules:

- `im_routing.rs`
- `group_management.rs`
- `group_run.rs`
- `team_entry.rs`

## Prerequisite For Real Implementation

Before the first durable test-file split lands, the project should define a narrow visibility strategy for `employee_agents` integration tests.

That strategy should answer one question clearly:

- which `employee_agents` functions are intended to be callable from integration tests
- through which stable public or test-only surface they should be exposed

Reasonable options include:

- a deliberate re-export surface for integration tests
- test-only wrapper entry points around private helpers
- shifting some integration coverage to public command-level flows instead of private helper calls

Without that prerequisite, moving tests into child modules risks creating layout churn without a verifiable green path.

## First Safe Implementation Slice

Start with:

- `test_im_employee_agents/im_routing.rs`

Current milestone:

- `im_routing.rs` now owns the first three routing tests:
  - `employee_config_and_im_session_mapping_work`
  - `group_message_without_mention_routes_to_main_employee`
  - `group_message_with_mention_routes_to_target_employee`

Move only the already-cohesive IM routing and session bridge tests first. That cluster is the safest because:

- the tests are near the top of the file
- they mostly depend on the same routing-related commands and bindings
- they do not require redesigning the broader group-run import surface yet

Candidate tests for the first move:

- `employee_config_and_im_session_mapping_work`
- `group_message_without_mention_routes_to_main_employee`
- `group_message_with_mention_routes_to_target_employee`
- `save_feishu_employee_association_replaces_default_binding_and_updates_scope`
- `save_feishu_employee_association_rolls_back_scope_update_when_binding_insert_fails`
- `wecom_event_prefers_wecom_scoped_employee_and_creates_session`
- `group_message_with_text_mention_routes_to_target_employee_when_role_id_missing`
- `ensure_employee_sessions_for_event_prefers_team_entry_employee_when_binding_team_id_matches`

## Compatibility Rules

- Do not change test names.
- Do not change assertions or SQL fixture semantics.
- Do not move `helpers::setup_test_db()` into a new helper layer.
- Do not touch production code as part of this split.
- Keep the integration-test binary name stable as `test_im_employee_agents`.

## Verification Strategy

Because this is an integration test split, verification should focus on the touched binary:

- run `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_im_employee_agents -- --nocapture`

This gives honest coverage for:

- module registration
- test discovery
- retained setup behavior
- all moved tests plus untouched tests in the same binary

## Success Criteria

- the root `test_im_employee_agents.rs` becomes materially smaller
- the first child module represents a real scenario family, not a random helper bucket
- all moved tests keep the same semantics and names
- the integration test binary still passes unchanged
- the integration test binary compiles through an intentional visibility surface rather than accidental private imports
