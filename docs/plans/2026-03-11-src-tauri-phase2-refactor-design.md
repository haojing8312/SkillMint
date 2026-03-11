# src-tauri Phase 2 Refactor Design

**Goal:** Continue shrinking `apps/runtime/src-tauri` by extracting pure routing/model rules and executor helper logic into lightweight crates without touching database, provider I/O, or Tauri command surfaces.

## Why Phase 2

Phase 1 removed two high-value pure logic clusters:
- skill registry/config parsing
- permission policy logic

The next remaining compile-heavy areas with meaningful pure logic are:
- `commands/models.rs`
- `agent/executor.rs`

Both files still combine pure rules with heavyweight runtime concerns. Phase 2 isolates the pure portions first.

## Phase 2 Principles

1. Extract pure logic only.
2. Do not move SQLx calls, provider network calls, or Tauri state handling in this phase.
3. Keep command and executor public behavior unchanged.
4. Prefer re-export compatibility over widespread call site rewrites.

## Target Extractions

### 1. `packages/runtime-routing-core`

Purpose:
- capability route template definitions
- route template listing
- provider recommendation lists
- protocol default model selection
- capability-based model filtering
- route cache freshness helpers

Good candidates from `commands/models.rs`:
- `builtin_capability_route_templates`
- `list_capability_route_templates_for`
- `default_model_for_protocol`
- `recommended_models_for_provider`
- `filter_models_by_capability`
- `cache_row_is_fresh`

Remain in `src-tauri`:
- `*_from_pool` DB functions
- provider health checks
- tauri commands
- adapter/provider registry integration

### 2. `packages/runtime-executor-core`

Purpose:
- output truncation
- token estimation
- micro compaction
- message trimming
- stable error parsing helpers
- repeated tool failure streak logic

Good candidates from `agent/executor.rs`:
- `truncate_tool_output`
- `estimate_tokens`
- `micro_compact`
- `trim_messages`
- `split_error_code_and_message`
- `stable_tool_input_signature`
- `extract_tool_call_parse_error`
- `update_tool_failure_streak`

Remain in `src-tauri`:
- tool execution loop
- LLM adapters
- Tauri event emission
- permission prompt / confirmation wiring

## Why This Is the Right Boundary

### `commands/models.rs`

This file currently mixes:
- static routing templates
- recommendation heuristics
- capability filtering rules
- DB read/write
- provider health checks
- Tauri command wrappers

The static templates and recommendation rules are ideal lightweight-crate material. The DB and provider behavior should stay put for now.

### `agent/executor.rs`

This file currently mixes:
- conversation compaction heuristics
- generic string/message utilities
- route event/error shaping
- Tauri event emission
- runtime tool orchestration

The helper functions are reusable, testable, and independent from Tauri. The orchestration loop is not.

## Expected Outcomes

- More Rust logic changes can be validated without touching the heavy Tauri crate.
- `commands/models.rs` becomes more obviously an adapter/integration module.
- `agent/executor.rs` becomes smaller and more orchestration-focused.
- Phase 3 can later target database/application-service boundaries with less risk.

## Risks

### Risk: Hidden coupling in route template application

Mitigation:
- only extract template definitions and pure recommendation/filter helpers
- keep provider resolution against DB in `src-tauri`

### Risk: Executor helper extraction creates awkward type sharing

Mitigation:
- keep extracted helper APIs string/JSON-based
- do not extract executor state structs that are tightly bound to the loop unless clearly pure

## Definition of Done

- `runtime-routing-core` exists and owns routing/model pure helpers
- `runtime-executor-core` exists and owns executor pure helpers
- corresponding tests move into lightweight crates
- `commands/models.rs` and `agent/executor.rs` are materially smaller and more focused
