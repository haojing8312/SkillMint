# src-tauri Phase 4A Chat Refactor Design

**Date:** 2026-03-11

## Goal

Create `packages/runtime-chat-app` and move the pre-execution chat orchestration out of `apps/runtime/src-tauri/src/commands/chat.rs` so the Tauri command layer becomes a thinner adapter.

## Problem

`chat.rs` currently mixes several responsibilities:

- Tauri command entrypoints and `State` extraction
- session and permission normalization
- capability inference
- route fallback parsing and retry policy decisions
- execution-time context preparation
- runtime wiring for `AgentExecutor`, event emission, ask-user flow, and employee/team integrations

This keeps high-churn orchestration logic trapped inside the heaviest crate in the workspace.

## Recommended Approach

Introduce a new lightweight crate: `packages/runtime-chat-app`.

This crate should own only the preparation stage that happens before `AgentExecutor` starts the main run loop.

`src-tauri` should keep:

- tauri command signatures
- `State` extraction
- event emission
- runtime wiring
- SQLx adapters
- employee/team runtime integration
- the actual `AgentExecutor` invocation

`runtime-chat-app` should own:

- permission mode normalization
- session mode normalization
- capability inference
- fallback chain parsing
- retry/error classification policy
- route and model preparation
- execution-preparation guidance/context aggregation

## Why This Cut Is Correct

This is the highest-value cut with the lowest architectural risk.

It continues the dependency direction established by `runtime-models-app`:

- command layer depends on app layer
- app layer depends on traits
- infra adapters stay in `src-tauri`

It does not prematurely move:

- Tauri-specific state
- app event emit logic
- long-running executor loop
- employee/team entry flow
- session/message persistence writes

## New Crate Boundary

Proposed crate:

- `packages/runtime-chat-app`

Initial files:

- `src/lib.rs`
- `src/types.rs`
- `src/traits.rs`
- `src/service.rs`

Initial exported service:

- `ChatPreparationService`

Initial core result type:

- `PreparedChatExecution`

This result should describe the normalized execution plan that `chat.rs` can hand off to the existing runtime.

## Traits

Phase 4A should start with narrow read-oriented traits.

### `ChatSettingsRepository`

Responsibilities:

- read routing settings
- read chat routing policy
- read capability routing policy
- resolve default / usable model ids
- read provider/model configuration snapshots needed for preparation

### `ChatSessionRepository`

Responsibilities:

- read existing session mode/team metadata required during preparation
- expose only the minimum session facts needed for pre-execution decisions

### `EmployeeRoutingCatalog`

Responsibilities:

- expose employee/team routing hints that affect preparation

Phase 4A should keep this minimal and allow a null adapter at first.

## First Migration Scope

Move these pure helpers first:

- `normalize_permission_mode_for_storage`
- `normalize_session_mode_for_storage`
- `normalize_team_id_for_storage`
- `parse_permission_mode_for_runtime`
- `permission_mode_label_for_display`
- `infer_capability_from_user_message`
- `classify_model_route_error`
- `should_retry_same_candidate`
- `retry_budget_for_error`
- `retry_backoff_ms`
- `parse_fallback_chain_targets`

Then move the orchestration that combines:

- routing settings
- routing policy lookup
- fallback candidate construction
- default model preparation
- execution guidance/context fragments

## Explicit Non-Goals

Phase 4A does not move:

- `AgentExecutor` main loop
- tauri event emit paths
- ask-user responder plumbing
- tool confirmation plumbing
- employee/team entry orchestration
- session/message persistence writes
- route log persistence

## Testing Strategy

### `runtime-chat-app` tests

Add lightweight tests for:

- mode normalization
- capability inference
- retry policy
- fallback parsing
- preparation orchestration with fake repositories

### `src-tauri` tests

Retain only narrow smoke coverage for:

- command/service wiring
- key preparation scenarios that prove no regression

## Acceptance Criteria

Phase 4A is successful when:

- `runtime-chat-app` exists with independent tests
- `chat.rs` is materially smaller
- pre-execution preparation no longer lives mainly in `chat.rs`
- dependency direction is `command -> app -> infra`
- runtime behavior remains unchanged
