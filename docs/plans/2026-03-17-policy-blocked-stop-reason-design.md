# Policy Blocked Stop Reason Design

**Date:** 2026-03-17
**Status:** Approved

## Goal

Add a unified stop reason for deterministic safety and policy rejections so WorkClaw stops immediately, explains the root cause clearly, and gives the user a recovery path instead of falling through to `loop_detected`, `no_progress`, or `max_turns`.

## Problem

The current runtime has a gap between two kinds of failure:

1. **Run guard failures**
   - already modeled as structured `RunStopReason`
   - examples: `loop_detected`, `no_progress`, `max_turns`
2. **Deterministic policy failures**
   - currently surface as ordinary tool errors
   - examples:
     - path outside current work directory
     - requested tool outside the Skill allowlist

This creates a bad user experience for requests like "整理桌面". The runtime already knows the task cannot succeed under the current workspace boundary, but it still lets the model keep trying alternate paths until another guardrail stops the run.

## Design Principles

- Deterministic policy rejection should stop the run immediately.
- The user-facing reason should describe the boundary, not the runtime symptom.
- Recoverable runtime stalls and deterministic policy blocks should remain separate categories.
- The first version should only cover failures that are clearly impossible to recover from without user intervention.

## Decision

Introduce a new structured stop reason:

```rust
RunStopReasonKind::PolicyBlocked
```

This reason is emitted when the runtime can determine that continuing the run is pointless because a stable safety or workspace rule already denies the requested action.

## First-Version Scope

### Included

The first version should stop immediately for these cases:

1. **Workspace boundary violation**
   - tool path fails `ToolContext::check_path`
   - error contains `不在工作目录`
   - example: user asks to read or reorganize desktop while the session work dir is `C:\Users\<user>\WorkClaw\workspace`

2. **Skill allowlist violation**
   - executor rejects the tool because it is not in `allowed_tools`
   - current message: `此 Skill 不允许使用工具: ...`

### Excluded

The first version should not convert these into `policy_blocked`:

- user approval deny
- approval timeout
- network failures
- missing files
- selector not found in browser tasks
- malformed tool input

These are either recoverable, user-driven, or not stable enough to guarantee that the run cannot succeed.

## Runtime Model

### New Stop Reason

Add a constructor in `run_guard.rs`:

```rust
RunStopReason::policy_blocked(message, detail)
```

Suggested user-facing copy:

- title: `当前任务无法继续执行`
- message: `本次请求触发了安全或工作区限制，系统已停止继续尝试。`

Use `detail` for actionable specifics, for example:

- `目标路径不在当前工作目录范围内`
- `当前 Skill 不允许使用工具 file_move`

### Classification Layer

Add a small runtime classifier in `executor.rs` that inspects tool execution failures before they are appended back into the model conversation.

Suggested helper shape:

```rust
fn classify_policy_blocked_tool_error(tool_name: &str, error_text: &str) -> Option<RunStopReason>
```

Responsibilities:

- detect workspace boundary violations from `check_path`
- detect Skill tool allowlist violations from executor-generated error text
- return `None` for ordinary tool failures

### Early Stop Flow

Current flow:

1. tool fails
2. tool error is appended into `tool_results`
3. model receives the error and may try again
4. a later run guard stops the run

New flow:

1. tool fails
2. executor checks `classify_policy_blocked_tool_error(...)`
3. if matched:
   - emit `agent-state-event` with `state = "stopped"`
   - persist `run_stopped`
   - return encoded `RunStopReason`
   - do not feed the error back into the model for another retry
4. if not matched:
   - keep the current behavior

## Frontend UX

Add a dedicated display mapping in `ChatView.tsx`:

- `policy_blocked` -> `当前任务无法继续执行`

Primary card copy:

- title: `当前任务无法继续执行`
- message: `本次请求触发了安全或工作区限制，系统已停止继续尝试。`

Workspace-specific recovery hint:

- `目标位置不在当前工作目录范围内。你可以先切换当前会话的工作目录后重试。`

Possible secondary hint:

- `例如：将工作目录切换到桌面，或切换到包含桌面的上级目录。`

The frontend should continue to treat:

- `policy_blocked` as a stopped product state
- `error` as a true runtime or provider failure

## Persistence And Events

No schema redesign is required.

Reuse existing stop-reason propagation:

- `agent-state-event`
- `session_runs.error_kind`
- `session_run_events.run_stopped`

Store `error_kind = "policy_blocked"` for these runs.

## Testing Strategy

### Rust

- `run_guard.rs`
  - stop reason kind serializes as `policy_blocked`
  - constructor returns expected title/message

- `executor.rs`
  - workspace-boundary tool failure is upgraded into structured stop reason
  - allowlist rejection is upgraded into structured stop reason
  - ordinary tool failure is not upgraded

### Frontend

- `ChatView.run-guardrails.test.tsx`
  - `policy_blocked` renders the new title and message
  - workspace hint is shown when `error_message` carries boundary detail

## Out Of Scope

- automatic work-dir switching
- recovery buttons that mutate session state
- approval denial unification
- browser policy-block detection beyond the first deterministic cases

## Expected Outcome

After this change, requests that are blocked by stable workspace or Skill boundaries stop immediately with a clear explanation and next step, instead of degrading into a confusing "执行步数上限" message.
