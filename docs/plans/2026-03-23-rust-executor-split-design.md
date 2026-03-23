# Rust Executor Split Design

**Goal:** Turn [executor.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/agent/executor.rs) into a maintainable execution core by splitting it along the actual runtime boundaries that already exist in the file: approval handling, destructive-action safety checks, session/run event bridging, tool-context preparation, progress tracking, and the main agent turn loop. The split must preserve the current `AgentExecutor` API, event names, and approval / cancel / timeout behavior.

## Why This File Is Hard

[executor.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/agent/executor.rs) is not just a loop that calls tools. It is currently the place where several independent concerns meet:

- the main agent turn loop and model call orchestration
- tool execution and tool-result truncation
- approval flow registration, waiting, and resolution
- manual confirmation fallback for older approval paths
- destructive-action safety classification for file deletion, file writing, shell commands, and browser actions
- session run event persistence and run-guard warning persistence
- progress fingerprinting and browser-progress tracking
- tool-context construction and task-temp-dir setup
- skill-route event emission for nested skill execution

That mixture makes the file an easy target for new AI-generated work, because it already owns the helper functions and runtime state wiring that other pieces need. If we keep adding behavior here, the file will stay as a control-plane dumping ground.

## Current Boundary Map

### What The File Owns Today

- `ToolCallEvent` and `AgentStateEvent`
- tool confirmation timeout handling
- approval-runtime lookup and approval waiting
- destructive-action summaries for `file_delete`, `write_file`, `edit`, `bash`, and browser actions
- policy-blocked error classification
- session-run and run-guard event persistence helpers
- progress signatures for text and JSON payloads
- `ToolContext` construction and task-temp-dir creation
- skill-route event payload creation
- the `AgentExecutor` struct and its `execute_turn` loop
- file-local tests for confirmation, safety, tool context, and approval flow behavior

### Who Depends On It

- [apps/runtime/src-tauri/src/agent/mod.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/agent/mod.rs) re-exports `AgentExecutor` from this module.
- [apps/runtime/src-tauri/src/commands/feishu_gateway.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/feishu_gateway.rs) and other runtime modules rely on the approval, event, and tool-execution behavior that this file coordinates.
- [apps/runtime/src-tauri/src/approval_bus.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/approval_bus.rs), [apps/runtime/src-tauri/src/agent/run_guard.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/agent/run_guard.rs), and [apps/runtime/src-tauri/src/commands/chat_runtime_io.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/chat_runtime_io.rs) provide supporting primitives that executor code already consumes.

That means the split must preserve current event names, approval semantics, and the public `AgentExecutor` entrypoint while moving the implementation details into narrower modules.

## Recommended Design

### 1. Split By Execution Boundary, Not By Helper Count

The strongest boundary here is not "small helpers versus big helpers". It is the actual execution pipeline:

- prepare tool context
- classify risky actions
- request or wait for approval
- execute tools
- record events and progress
- decide whether to stop or continue the turn

That gives us modules that match how the runtime behaves today, instead of generic utility buckets.

### 2. Keep The Root File As The Orchestration Shell

The root [executor.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/agent/executor.rs) should eventually keep only:

- the `AgentExecutor` type
- a thin `execute_turn` orchestration path
- compatibility re-exports if any adjacent callers need them during the migration

It should stop owning all approval, safety, and event-persistence details.

### 3. Preserve Existing Runtime Contracts

This split must not change:

- `AgentExecutor::new` and `AgentExecutor::with_max_iterations`
- `AgentExecutor::execute_turn` input/output shape
- `agent-state-event`, `tool-call-event`, and `skill-route-node-updated` payload semantics
- approval request and approval-resolution behavior
- cancel behavior
- policy-blocked stop reasons
- tool output truncation and repeated-failure handling
- browser-progress and run-guard stop conditions

The split should be internal first, not behavioral.

## Suggested Module Layout

The final structure under `apps/runtime/src-tauri/src/agent/` should be organized by responsibility:

- `types.rs`
  - `ToolCallEvent`, `AgentStateEvent`, and any executor-specific event payloads
- `context.rs`
  - `build_tool_context`, `build_task_temp_dir`, and execution-context assembly
- `safety.rs`
  - `normalize_policy_blocked_detail`, `classify_policy_blocked_tool_error`, delete-target detection, and critical-action summaries
- `approval_flow.rs`
  - `ToolConfirmationDecision`, `wait_for_tool_confirmation`, `ApprovalWaitRuntime`, `resolve_approval_wait_runtime`, and `request_tool_approval_and_wait`
- `event_bridge.rs`
  - `resolve_current_session_run_id`, `append_tool_run_event`, `append_run_guard_warning_event`, and `build_skill_route_event`
- `progress.rs`
  - `text_progress_signature`, `json_progress_signature`, and other progress-fingerprint helpers
- `turn_executor.rs`
  - the core `execute_turn` loop and the cancel check wrapper if we want to keep the root file very thin
- `tests.rs`
  - the file-local tests currently embedded in the giant root file

If the split needs one compatibility layer for Tauri or for adjacent imports, keep that in the root file rather than creating a second orchestration layer.

## Responsibility Split

### Tool-Context And Capability Setup

This slice owns the code that prepares the runtime context passed into tools:

- build the task temp directory
- attach the current session ID
- attach allowed-tool restrictions
- attach execution-capability metadata

This is a clean first extraction because it is deterministic and does not depend on the main turn loop.

### Safety And Policy Classification

This slice owns the code that decides whether a tool call should be framed as dangerous:

- normalize policy-blocked error text
- classify workspace boundary failures
- classify file delete / write / edit / shell / browser operations
- describe destructive actions to the user in a confirmation prompt

This is also a strong first-cut candidate because it is mostly pure logic with simple tests.

### Approval Flow

This slice owns the code that connects the agent loop to the approval bus:

- lookup the current approval-runtime state
- register a waiter
- create the pending approval record
- emit approval-created and tool-confirm events
- notify Feishu about the approval request
- wait for the final approval resolution

This is the riskiest shared slice because it crosses app state, database state, and UI/event emission, but it is still logically separate from the execution loop.

### Event And Run Bridging

This slice owns the code that persists the agent execution record:

- load the current run ID from the session journal
- append tool-started and tool-completed events
- append run-guard warnings
- build skill-route progress events

This is a good boundary because it is not the same thing as approval, even though both are emitted from the same loop today.

### Main Turn Loop

This slice owns the orchestration behavior:

- model request and response handling
- micro-compaction and token trimming
- tool-call iteration
- repeated-failure circuit breaking
- progress evaluation
- cancel handling
- final stop / finish decisions

This should stay in one place until the supporting slices are stable, because breaking the loop apart too early makes the flow harder to reason about.

## Recommended Split Options

### Option 1: Execution-Boundary Split

Recommended path.

- Extract pure helpers first.
- Extract approval and event-bridge helpers next.
- Keep the main turn loop in a dedicated `turn_executor.rs`.
- Leave the root file as a thin `AgentExecutor` shell.

Trade-offs:

- best match for the current code shape
- easiest to verify incrementally
- keeps approval, safety, and progress logic separate
- still lets the turn loop remain readable

### Option 2: Phase-Oriented Split

Split by the life of a turn:

- `prepare`
- `request`
- `execute`
- `record`
- `finish`

Trade-offs:

- easier to explain conceptually
- but phase modules tend to reach across each other
- helpers become harder to reuse cleanly
- approval and event persistence can get mixed back into "execute"

### Option 3: Minimal Helper Extraction Only

Only move small pure helpers out and leave `execute_turn` intact.

Trade-offs:

- lowest risk in the short term
- but the root file remains the control-plane dumping ground
- file size stays large enough that future AI work will still pile into it

## Recommended Smallest Safe Split Order

1. Extract `types.rs` for executor-specific event payloads.
2. Extract `context.rs` and keep all tool-context construction together.
3. Extract `safety.rs` for delete-target detection and policy-block classification.
4. Extract `approval_flow.rs` so the approval bus logic is isolated.
5. Extract `event_bridge.rs` for session-run and run-guard persistence.
6. Extract `progress.rs` if the signature helpers are still living in the root file after the earlier cuts.
7. Move the core loop into `turn_executor.rs`.
8. Move file-local tests into `tests.rs` last.

The main sequencing rule is to keep the approval flow and event bridge coherent while the turn loop still references them, and to avoid splitting the loop before the pure helpers are already extracted.

## Verification Suggestions

The verification should follow the slices:

- after `safety.rs`, run the delete-confirmation and policy-classification tests
- after `context.rs`, run the tool-context construction tests
- after `approval_flow.rs`, run the approval-bus integration test
- after `event_bridge.rs`, run the run-guard persistence and event-emission tests
- after `turn_executor.rs`, run `pnpm test:rust-fast`

The important thing is to keep each test batch narrow enough to point to one boundary at a time.

## Risks

- breaking approval resolution timing or the fallback confirmation path
- changing the text or shape of emitted runtime events
- moving the run-journal helpers in a way that breaks session restoration or event persistence
- splitting the turn loop too early and making cancellation / timeout behavior harder to reason about
- creating a replacement child module that is still too large and becomes the new dumping ground

## Success Criteria

- [executor.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/agent/executor.rs) becomes a thin orchestration shell instead of the home for every execution concern
- approval flow, destructive-action safety, event bridging, tool-context setup, and progress helpers each live in a focused child module
- `AgentExecutor` stays compatible for existing callers
- existing event names and stop reasons remain unchanged
- the split pattern is clear enough to reuse for other runtime control-plane files without inventing a new layout
