# Runtime Test Governance And Trace/Eval Design

## Goal

Stabilize the current runtime/frontend regression tests and add a minimal internal trace/eval foundation for the agent runtime without introducing a new UI surface.

## Context

WorkClaw has just landed a large round of runtime-kernel alignment plus scene-layer refactors across chat, employee hub, and settings. The current system is functionally stronger, but two follow-up gaps remain:

1. Some Vitest suites pass reliably when run one file at a time but still show cross-file contamination when forced into the same worker.
2. The runtime already records useful run events, journals, diagnostics bundles, and route attempt logs, but there is no focused "trace builder" and no stable golden-fixture eval layer built on top of those events.

This design treats those as one coherent hardening phase:

- Priority 2: make the tests reliably batchable
- Priority 3: make runtime behavior easier to inspect, export, and regression-test

## Non-Goals

- Do not build a new frontend trace viewer in this phase.
- Do not create a full online eval platform or model-grading system.
- Do not replace the existing session journal, diagnostics, or event schema with a parallel system.
- Do not widen release scope with unrelated UX changes.

## Current Baseline

### Test Baseline

The runtime test environment already uses:

- [apps/runtime/vitest.config.ts](/d:/code/WorkClaw/apps/runtime/vitest.config.ts)
- [apps/runtime/src/test/setup.ts](/d:/code/WorkClaw/apps/runtime/src/test/setup.ts)

But cleanup behavior is still inconsistent across files. Some tests call `cleanup()`, some manage fake timers locally, and some rely on local mock reset behavior. This makes grouped runs more fragile than single-file runs.

The most visible fragile zones are:

- [EmployeeHubView.group-orchestrator.test.tsx](/d:/code/WorkClaw/apps/runtime/src/components/employees/__tests__/EmployeeHubView.group-orchestrator.test.tsx)
- [EmployeeHubView.team-template.test.tsx](/d:/code/WorkClaw/apps/runtime/src/components/employees/__tests__/EmployeeHubView.team-template.test.tsx)
- [useFeishuSettingsController.test.tsx](/d:/code/WorkClaw/apps/runtime/src/components/settings/feishu/__tests__/useFeishuSettingsController.test.tsx)

### Trace Baseline

The runtime already persists meaningful execution data through:

- [session_journal.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/session_journal.rs)
- [session_runs.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/session_runs.rs)
- [diagnostics_service.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/desktop_lifecycle/diagnostics_service.rs)

This means the right move is not "invent trace logging", but "shape existing events into a stable trace summary and export path".

## Chosen Approach

Use the smallest safe path:

1. Add a global runtime-test cleanup baseline in the Vitest setup layer.
2. Patch only the remaining stubborn suites whose local behavior still leaks across grouped execution.
3. Add a runtime trace query/export layer on top of `session_run_events` and `SessionRunEvent`.
4. Add golden trace fixtures as the first eval layer.

This is intentionally backend/internal first. The team can decide later whether to expose the same trace summary in the desktop UI.

## Design

### 1. Test Governance

Centralize cleanup responsibility in [setup.ts](/d:/code/WorkClaw/apps/runtime/src/test/setup.ts).

The global test baseline should:

- call `cleanup()` after every test
- clear and restore mocks
- restore real timers
- clear `localStorage` and `sessionStorage`
- reset `window.location.hash`

The goal is to make cleanup default rather than opt-in.

Local suites should only keep extra teardown when they own special state that truly cannot be handled globally.

The first success criterion is not "single file passes"; it is:

- the previously fragile `EmployeeHub` test group passes in one worker
- the previously fragile `Settings/Feishu` test group passes in one worker

### 2. Trace Model

`SessionRunEvent` remains the canonical event model.

No new event log should be introduced in this phase. Instead:

- query structured events from `session_run_events`
- derive stable trace summaries from `SessionRunEvent`
- export trace summaries through the existing diagnostics/export style

The new trace summary should include at least:

- run identity: `session_id`, `run_id`
- lifecycle: started, completed, failed, cancelled, stopped
- tool activity: tool start/completion count and ordered items
- approvals: requested approvals with key metadata
- guard behavior: loop/no-progress warnings and stop reasons
- child-session linkage when present
- timing: first event, last event, event count

### 3. Trace Query And Export

Add a runtime command layer that supports:

- `list_session_run_events(session_id, run_id?, limit?)`
- `export_session_run_trace(session_id, run_id)`

The list command should return structured, filtered, application-safe event summaries. It should not dump unlimited raw JSON by default.

The export command should produce a richer JSON trace artifact intended for diagnostics, debugging, and golden-fixture generation.

Malformed or partially unreadable events should degrade gracefully:

- skip or mark invalid event payloads
- include parse warnings in the exported summary
- never fail the entire export because of one bad row

### 4. Eval Foundation

The first eval layer should be fixture-driven, not model-scored.

Each fixture contains:

- input events
- expected normalized trace summary

Initial fixture scenarios:

- normal successful run
- loop interception before tool execution
- admission-gate conflict rejection
- approval requested then resumed
- child session completes
- child session fails

Normalization rules must remove unstable fields like:

- exact timestamps
- UUIDs
- provider-specific noisy substrings

This creates a repeatable regression harness for runtime behavior without needing a full evaluation platform.

## Data Flow

1. Runtime continues appending `SessionRunEvent` records.
2. Query/export services load those events from `session_run_events` and journal state where needed.
3. A trace builder converts raw events into a normalized run trace summary.
4. Diagnostics export can include this structured trace artifact.
5. Golden-fixture tests assert the trace builder output stays stable across refactors.

## Error Handling

- Global test cleanup must be idempotent.
- Trace query APIs should return summaries by default and reserve raw payload detail for export/debug paths.
- Unknown or malformed events must become warnings, not fatal trace-export errors.
- The phase should avoid schema changes when possible.
- If a new query path depends on data that may not exist in legacy databases, it must include a compatibility fallback.

## Verification

### Test Governance Verification

- Run grouped `EmployeeHub` tests in a single worker.
- Run grouped `Settings/Feishu` tests in a single worker.
- Confirm the grouped runs succeed without falling back to per-file execution.

### Runtime Trace Verification

- Add Rust unit tests for trace building across success/failure/approval/guard/child-session scenarios.
- Add tests for malformed payload and partial-read behavior.
- If a frontend service wrapper is added, cover it with a focused unit test.

### Final Verification

- `pnpm test:rust-fast`
- grouped target Vitest runs
- `pnpm --dir apps/runtime build`

## Risks

### Risk 1: Global cleanup breaks tests that rely on hidden shared state

Mitigation:

- roll out cleanup in setup first
- patch only the tests that were accidentally depending on leaked state

### Risk 2: Trace export grows into a second logging system

Mitigation:

- reuse `SessionRunEvent`
- reuse diagnostics/export patterns
- keep export read-only over existing persisted data

### Risk 3: Eval fixtures become too brittle

Mitigation:

- normalize timestamps/UUIDs/dynamic strings
- keep the first fixture set small and behavior-focused

## Rollout

Phase order:

1. test governance baseline
2. grouped regression stabilization
3. trace query/export
4. golden trace fixtures

Only after that should the team decide whether a frontend trace panel is worth building.

## Expected Outcome

After this phase:

- runtime regression suites are more batch-stable
- agent runs are easier to explain and debug
- core runtime behaviors have a minimal but durable eval harness
- future planner/queue/observability work can build on a stable base instead of guesswork
