# OpenClaw-Style Hidden Observability And Contract Eval Design

## Goal

Refactor WorkClaw's current runtime trace and regression foundation so it follows the same architectural direction as `references/openclaw`: hidden observability, internal diagnostics exports, observability snapshots, and fixture/contract-driven runtime regression protection.

## Decision Summary

WorkClaw should not ship a normal-user-visible trace screen as the primary outcome of this phase.

This phase should instead align to OpenClaw's implementation shape:

- hidden assistant/runtime trace support rather than default UI exposure
- internal event buffering and diagnostics-oriented inspection
- aggregate observability snapshots for the runtime control plane
- contract-style runtime tests for stable behavioral guarantees
- fixture-driven golden regressions for normalized runtime outputs

## OpenClaw Reference Shape

The relevant OpenClaw patterns are visible in these reference points:

- hidden trace UI toggle in [references/openclaw/apps/shared/OpenClawKit/Sources/OpenClawChatUI/ChatView.swift](/d:/code/WorkClaw/references/openclaw/apps/shared/OpenClawKit/Sources/OpenClawChatUI/ChatView.swift)
- bounded internal event store in [references/openclaw/apps/macos/Sources/OpenClaw/AgentEventStore.swift](/d:/code/WorkClaw/references/openclaw/apps/macos/Sources/OpenClaw/AgentEventStore.swift)
- observability snapshot in [references/openclaw/src/acp/control-plane/manager.core.ts](/d:/code/WorkClaw/references/openclaw/src/acp/control-plane/manager.core.ts)
- runtime contract harness in [references/openclaw/src/acp/runtime/adapter-contract.testkit.ts](/d:/code/WorkClaw/references/openclaw/src/acp/runtime/adapter-contract.testkit.ts)
- fixture/contract style regression tests in [references/openclaw/apps/shared/OpenClawKit/Tests/OpenClawKitTests/TalkConfigContractTests.swift](/d:/code/WorkClaw/references/openclaw/apps/shared/OpenClawKit/Tests/OpenClawKitTests/TalkConfigContractTests.swift)

These references show that OpenClaw does not treat trace as an end-user feature first. It treats trace and observability as runtime engineering infrastructure.

## Current WorkClaw Baseline

WorkClaw already has good building blocks:

- structured run trace builder in [apps/runtime/src-tauri/src/agent/runtime/trace_builder.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/agent/runtime/trace_builder.rs)
- session-run event query and export in [apps/runtime/src-tauri/src/commands/session_runs.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/session_runs.rs)
- diagnostics bundle integration in [apps/runtime/src-tauri/src/commands/desktop_lifecycle/diagnostics_service.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/desktop_lifecycle/diagnostics_service.rs)
- golden trace fixtures under [apps/runtime/src-tauri/tests/fixtures/run_traces](/d:/code/WorkClaw/apps/runtime/src-tauri/tests/fixtures/run_traces)

That means this phase is not about inventing observability from zero. It is about refactoring the current pieces into an OpenClaw-like shape with cleaner responsibilities and stronger regression guarantees.

## Non-Goals

- Do not add a default visible trace page to the normal user chat flow.
- Do not replace `session_run_events` with a second persistent event database.
- Do not add a model-graded online eval platform.
- Do not widen scope into planner, reflection, or queue semantics during this phase.
- Do not introduce release-facing UX changes unless required for hidden diagnostics access.

## Target Architecture

### 1. Hidden Runtime Event Store

WorkClaw should gain a bounded, diagnostics-oriented in-memory event store analogous to OpenClaw's `AgentEventStore`.

Responsibilities:

- keep only the latest N high-signal runtime events
- support append and clear semantics
- stay internal to runtime diagnostics and developer support surfaces
- avoid becoming a second source of truth for persisted session history

This store is not a replacement for `session_run_events`. It is a short-window observability buffer.

### 2. Runtime Observability Snapshot

WorkClaw should add a single aggregate snapshot API similar to OpenClaw's `getObservabilitySnapshot()`.

The snapshot should summarize runtime health instead of replaying raw history. The first version should include:

- active run count
- recent hidden event buffer size
- admission conflict totals
- loop guard warning/interception totals
- approval waiting totals
- child-session totals
- compaction totals
- failover/error-kind totals
- average and maximum completed run latency when available

This gives diagnostics a control-plane summary rather than a pile of raw JSON.

### 3. Diagnostics Export As The Primary Delivery Mechanism

WorkClaw should treat diagnostics export as the primary surface for hidden observability.

The diagnostics bundle should include:

- recent runtime events
- observability snapshot
- session run projections
- session run event summaries
- session run trace exports

This follows the OpenClaw spirit: trace stays available and useful, but is not promoted into a default product UI.

### 4. Runtime Contract Testkit

WorkClaw should add a reusable runtime contract harness analogous to OpenClaw's ACP runtime adapter contract testkit.

The harness should validate invariant behavior across runtime scenarios:

- a run can start and emit meaningful events
- a run can finish with a stable success outcome
- failure paths expose stable failure signals
- admission gate conflict semantics remain stable
- approval pause/resume semantics remain stable
- loop-guard interception semantics remain stable
- child-session runtime semantics remain stable

This is the key shift from "we have tests" to "we have explicit runtime contracts".

### 5. Fixture-Driven Contract Outputs

The existing trace fixtures should remain, but their role should be upgraded.

They should become part of a first-class contract layer:

- normalized inputs
- normalized exported outputs
- stable, reviewable JSON fixtures
- minimal dynamic-field normalization

Like OpenClaw's contract fixtures, the goal is to lock down behavior, not to test every implementation detail.

## Data Flow

1. Runtime execution continues writing persisted `session_run_events`.
2. High-signal runtime events are also appended to the hidden bounded event store.
3. A runtime observability module aggregates counters and latency statistics.
4. Diagnostics export reads:
   - persisted run projections
   - summarized run events
   - structured run traces
   - hidden buffered runtime events
   - aggregate observability snapshot
5. Runtime contract tests and trace fixtures verify that exported behavior remains stable.

## Module Restructuring

### Current Module Roles

- `trace_builder.rs`: currently acts as both exporter logic and fixture-normalization host
- `session_runs.rs`: currently mixes projection queries, event reads, and trace export
- `diagnostics_service.rs`: already aggregates diagnostic artifacts but does not yet own a formal observability snapshot

### Target Module Roles

- `runtime/observability.rs`
  Owns counters, latency aggregation, hidden recent-event storage, and snapshot generation.

- `runtime/trace_builder.rs`
  Stays as the trace read model / trace-export normalization layer.

- `commands/session_runs.rs`
  Keeps session-run projection and event/trace query responsibilities, but should consume observability/trace primitives rather than own them.

- `commands/desktop_lifecycle/diagnostics_service.rs`
  Becomes the authoritative export aggregator for observability snapshot plus run diagnostics artifacts.

- `src-tauri/tests/...`
  Gains a reusable runtime contract harness and contract-oriented fixtures.

## Compatibility And Rollout

This phase should preserve current runtime behavior.

Safe rollout rules:

- preserve existing Tauri command names unless there is a compelling cleanup reason
- add snapshot/event exports without changing normal chat UX
- reuse current persisted event tables and avoid schema churn when possible
- if new metrics need persistence, prefer derived counters first; only add schema when runtime value is clear

## Verification Strategy

The minimum honest verification for this phase should include:

- targeted Rust unit tests for observability snapshot behavior
- runtime contract harness tests for success/failure/guard/approval/child-session invariants
- existing trace fixture tests
- `pnpm test:rust-fast`
- focused grouped Vitest runs only if the changed surface affects frontend diagnostics plumbing
- `pnpm build:runtime` once diagnostics export and runtime integration settle

## Expected Outcome

After this phase, WorkClaw should look much more like OpenClaw in this area:

- hidden observability, not default trace UI
- formal runtime snapshot reporting, not only raw trace exports
- reusable runtime contract tests, not only ad hoc case tests
- fixture-driven regression protection around exported runtime behavior

In high-school-level terms:

- trace becomes the black box
- observability snapshot becomes the dashboard
- contract tests become the driving test
- fixtures become the answer sheet we reuse every time we rebuild the engine
