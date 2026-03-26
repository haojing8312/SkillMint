# WorkClaw Session Serialization Admission Gate Design

**Date:** 2026-03-26

**Goal:** Start WorkClaw agent-kernel phase 2 by enforcing same-session run serialization at the runtime boundary, using a minimal reject-on-conflict admission gate that matches the current desktop UX.

## Context

Phase 1 moved WorkClaw's general chat runtime toward a single kernel-centered architecture:

- preparation now lives behind `runtime-chat-app`
- `SessionRuntime` owns the main send-message execution path
- transcript, failover, tool dispatch, and run projection now live under `apps/runtime/src-tauri/src/agent/runtime/`

That refactor improved ownership, but it did not yet make same-session execution admission explicit.

Today, the desktop UI already behaves as if a session should only have one active run at a time:

- active sessions render running states such as `thinking`, `tool_calling`, and `waiting_approval`
- the main chat composer disables send while streaming
- tab logic already treats local runtime state as blocking

The remaining gap is that this rule is still mostly a frontend convention, not a hard runtime contract.

## Problem Statement

Without a runtime admission gate, two `send_message` commands for the same session can still race at the Tauri boundary.

That creates several risks:

- duplicate or out-of-order user message insertion
- conflicting `run_started` projections
- tool and approval events landing on the wrong logical turn
- transcript corruption when two runs append assistant/tool output to the same session
- future IM and multi-entry surfaces bypassing the desktop-only send-button guard

OpenClaw solves this class of problem with per-session serialized runs. WorkClaw does not need the full queue model in this batch, but it does need the same invariant: one active real run per session.

## Options Considered

### Option 1: Keep the current frontend-only guard

Pros:

- zero backend work
- no command-contract change

Cons:

- does not protect against concurrent command calls
- cannot safely scale to IM or multi-surface entrypoints
- leaves transcript and run projection races possible

### Option 2: Add a runtime reject-on-conflict admission gate

Pros:

- smallest safe step toward OpenClaw-style serialization
- preserves current UX expectations
- keeps implementation narrow and testable
- blocks same-session races before persistence begins

Cons:

- does not yet support queued follow-up execution
- still relies on future work for richer session-lane scheduling

### Option 3: Implement a full per-session queue now

Pros:

- closest to OpenClaw's long-term architecture
- naturally supports follow-up requests

Cons:

- much larger state-machine surface
- requires queue projection, dequeue semantics, cancel semantics, and UI changes
- too large for the first phase 2 step

## Recommendation

Use **Option 2** now.

This batch should introduce a dedicated session admission gate that rejects a second same-session `send_message` while another run is active. The rejection should happen before user-message persistence.

This gets WorkClaw the most important safety property immediately, while keeping the queue model deferred to a later phase.

## Scope

### In scope

- runtime-level same-session send admission
- structured conflict detection for `send_message`
- app-managed admission state for same-process concurrency
- rejecting conflicting sends before user message insertion
- Rust-side regression coverage

### Out of scope

- queued follow-up execution
- per-session cancel routing
- replacing the global cancel flag
- IM or background entrypoint admission unification
- stale-run crash recovery policy
- session write lock persistence across app restarts

## Architecture

### 1. Add a dedicated `SessionAdmissionGate`

Create a new runtime module under `apps/runtime/src-tauri/src/agent/runtime/`:

- `admission_gate.rs`

This module owns same-process session admission, not run projection.

Responsibilities:

- atomically reserve a session for a new run
- reject duplicate admission for the same session
- allow different sessions to run in parallel
- release the reservation when the command scope ends

This module should not know about transcript details, tools, or message persistence.

### 2. Keep `RunRegistry` focused on run identity and projection

`RunRegistry` should continue to answer:

- which run id is active for a session
- how journal snapshots rehydrate active run ids

It should not become the sole admission controller for this batch, because admission needs a reservation before a real `run_id` exists.

That means phase 2 starts with a clean split:

- `SessionAdmissionGate`: whether the session may enter a run
- `RunRegistry`: which run currently represents that session in projections

### 3. Move admission ahead of user message persistence

`commands/chat.rs::send_message` currently persists the user message before runtime execution begins.

For this batch, admission must happen before:

- `insert_session_message_with_pool`
- title updates
- team-entry pre-execution handling
- `SessionRuntime::run_send_message`

If admission fails, the command should return immediately and leave the session transcript untouched.

### 4. Use a stable structured conflict error

The first conflict contract should be explicit and machine-readable. A simple string contract is enough for this batch, but it must include a stable code.

Recommended shape:

- `SESSION_RUN_CONFLICT: 当前会话仍在执行中，请等待当前任务完成后再发送新消息`

This keeps the current command signature unchanged (`Result<(), String>`) while making frontend and future multi-surface consumers able to detect and special-case the conflict.

## Runtime Flow

### Accepted send

1. `send_message` receives request
2. `SessionAdmissionGate` reserves the session
3. user message is persisted
4. `SessionRuntime::run_send_message` starts and registers the real run
5. run completes or fails
6. command returns and the admission lease is released

### Rejected send

1. `send_message` receives request
2. `SessionAdmissionGate` detects the session is already reserved
3. command returns `SESSION_RUN_CONFLICT`
4. no user message is inserted
5. no new run projection is created

## Data Model And State Notes

- Admission state is process-local and app-managed in `lib.rs`
- It intentionally does not persist across restarts in this batch
- `SessionJournalStore` and `RunRegistry` continue to model runtime projections and historical run state
- crash recovery and stale active-run cleanup remain follow-up work

## Testing Strategy

### Rust unit tests

- acquiring a session lease succeeds when idle
- re-acquiring the same session while leased returns conflict
- different sessions can be leased concurrently
- dropping or releasing a lease makes the session available again

### Rust command-layer regression

- when admission fails, `send_message` returns the structured conflict error
- when admission fails, no user message is inserted

### Verification

- targeted admission-gate tests
- targeted chat command tests
- `pnpm test:rust-fast`

## Follow-On Phase 2 Work

Once the reject-on-conflict gate is stable, the next serialization upgrades should be:

1. replace reject-on-conflict with optional queued follow-up lanes
2. scope cancellation and stop requests to session/run identity
3. add kernel-owned compaction pipeline and overflow retry
4. unify non-desktop entrypoints behind the same admission surface

## Success Criteria

This batch is successful when:

- same-session concurrent `send_message` requests cannot both enter execution
- conflicting sends are rejected before transcript persistence
- different sessions can still run in parallel
- current desktop UX remains unchanged for ordinary sends
- runtime ownership is clearer than before, not more coupled
