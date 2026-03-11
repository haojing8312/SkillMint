# src-tauri Phase 4A.5 Hardening Design

## Goal

Stabilize the Phase 4A `runtime-chat-app` extraction by adding focused integration evidence around `src-tauri` command wiring and the new `chat_repo` adapter, without expanding the architecture boundary further.

## Why This Phase Exists

Phase 4A already established the `command -> app -> infra` path for chat preparation:

- `packages/runtime-chat-app` owns session normalization and route preparation logic
- `apps/runtime/src-tauri/src/commands/chat_repo.rs` adapts SQLx-backed state into `ChatSettingsRepository`
- `apps/runtime/src-tauri/src/commands/chat.rs` delegates preparation to `ChatPreparationService`

What is still missing is enough integration evidence to treat that boundary as stable. We currently have:

- strong crate-level tests for `runtime-chat-app`
- `cargo check --lib` proof for `src-tauri`

We do not yet have enough targeted `src-tauri` tests proving that:

- the SQL adapter returns the expected snapshots
- `chat.rs` command-level preparation persists normalized session values correctly
- the new wiring is resistant to regression from nearby command-surface changes

## Non-Goals

This phase does not:

- continue extracting more `chat.rs` logic into new crates
- change `AgentExecutor` orchestration
- rework employee/team entry flows
- redesign database boundaries
- address general repository health issues beyond what is needed for verification

## Recommended Approach

Use a narrow hardening pass with three outputs:

1. Add adapter-level tests for `chat_repo`
2. Add one or two narrow `src-tauri` chat smoke tests
3. Write down the remaining `chat.rs` responsibilities that belong to Phase 4B

This is the highest-value next step because it increases confidence without reopening architecture scope.

## Test Boundaries

### 1. `chat_repo` adapter tests

Target file:

- `apps/runtime/src-tauri/src/commands/chat_repo.rs`

Test surface should cover:

- `load_routing_settings`
- `load_chat_routing`
- `load_route_policy`
- `resolve_default_model_id`
- `resolve_default_usable_model_id`
- `load_session_model`

These tests should run against the existing test database helpers under `apps/runtime/src-tauri/tests/helpers`.

### 2. `chat.rs` narrow smoke tests

Target file:

- `apps/runtime/src-tauri/src/commands/chat.rs`

Test surface should stay small and avoid full agent execution. Focus on:

- `create_session` writes normalized session metadata through `prepare_session_creation`
- helper/system-prompt assembly paths added by Phase 4A keep expected behavior
- optionally, a minimal preparation-oriented path that proves route preparation can read required DB state

Avoid tests that require:

- browser tools
- external providers
- full `execute_turn` completion
- IM / employee orchestration

## Remaining Phase 4B Candidates

This phase should end with a short inventory of what still remains inside `chat.rs` but looks like future application-layer work:

- execution-preparation aggregation beyond route selection
- session metadata read-model shaping
- employee/team collaboration pre-execution hints
- imported MCP guidance aggregation, if we later decide it belongs outside command layer

The inventory is for clarity only. No additional extraction happens in Phase 4A.5.

## Acceptance Criteria

Phase 4A.5 is complete when:

- `runtime-chat-app` remains green
- at least one new `chat_repo`-focused `src-tauri` test passes
- at least one narrow `chat.rs` smoke/integration test passes
- we can describe Phase 4A as "compiled and integration-backed", not just "compiled"

