# src-tauri Phase 4B Chat Candidates

This note records the `chat.rs` responsibilities intentionally left behind after Phase 4A and Phase 4A.5.

## What Is Already Out

These responsibilities are already outside the command file boundary:

- session creation normalization in `runtime-chat-app`
- permission/session mode normalization helpers
- capability inference
- retry/error classification
- route candidate preparation
- SQL-backed chat preparation reads through `chat_repo`

## What Still Lives In `chat.rs`

### 1. Execution-time system prompt assembly

`chat.rs` still owns the final prompt build-up for:

- tool list rendering
- workdir guidance
- employee collaboration guidance
- imported MCP guidance injection
- memory file injection

Why it stayed:

- this logic still depends directly on runtime-only state and local file access
- moving it now would widen Phase 4A.5 beyond verification hardening

### 2. Command/runtime orchestration

`chat.rs` still coordinates:

- Tauri command inputs
- DB message persistence
- session title updates
- cancellation wiring
- event emission
- tool registration
- `AgentExecutor::execute_turn(...)`

Why it stayed:

- this is the core command/runtime boundary
- changing it requires a broader execution-layer design, not just preparation extraction

### 3. Employee and team-entry branching

`chat.rs` still directly integrates:

- `maybe_handle_team_entry_session_message_with_pool`
- employee collaboration guidance generation
- employee-aware memory path decisions

Why it stayed:

- this path is entangled with IM/team behavior
- it is not just chat preparation; it is business orchestration

### 4. Imported MCP guidance loading

The `build_imported_external_mcp_guidance` and `load_imported_external_mcp_guidance` path is still local to `chat.rs`.

Why it stayed:

- it is adjacent to prompt composition and registry inspection
- it may later belong in a chat application/query service, but it is not required for the Phase 4A boundary to be correct

## Recommended Phase 4B Direction

When Phase 4B starts, prefer this order:

1. extract prompt/context assembly that does not require executor wiring
2. isolate employee/team pre-execution orchestration from pure command handling
3. only then consider splitting execution-loop orchestration

## What Not To Do First

Avoid starting Phase 4B by moving:

- the full `send_message` loop
- event emission
- `AgentExecutor` invocation
- tool registration

Those changes are high-risk and should come only after prompt/context orchestration has a clearer boundary.
