# Policy Blocked Stop Reason Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a structured `policy_blocked` stop reason so deterministic workspace and Skill-policy rejections end the run immediately with actionable recovery messaging.

**Architecture:** Extend the shared run-stop model with a `policy_blocked` kind, classify deterministic policy failures inside the Rust executor before tool errors are fed back into the model, persist the new stop reason through the existing run event pipeline, and update the chat UI to show a dedicated recovery message for workspace-boundary failures.

**Tech Stack:** Rust (Tauri, `serde`, `serde_json`, `anyhow`), React + TypeScript, Vitest, Cargo unit tests

---

### Task 1: Add The New Stop Reason Type

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/run_guard.rs`
- Test: `apps/runtime/src-tauri/src/agent/run_guard.rs`

**Step 1: Write the failing test**

Add tests in `apps/runtime/src-tauri/src/agent/run_guard.rs` for:

```rust
#[test]
fn run_stop_reason_kind_serializes_policy_blocked() {
    let value = serde_json::to_string(&RunStopReasonKind::PolicyBlocked).unwrap();
    assert_eq!(value, "\"policy_blocked\"");
}

#[test]
fn policy_blocked_constructor_returns_expected_copy() {
    let reason = RunStopReason::policy_blocked("目标路径不在当前工作目录范围内");
    assert_eq!(reason.title, "当前任务无法继续执行");
    assert_eq!(reason.message, "本次请求触发了安全或工作区限制，系统已停止继续尝试。");
    assert_eq!(reason.detail.as_deref(), Some("目标路径不在当前工作目录范围内"));
}
```

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml policy_blocked -- --nocapture
```

Expected: FAIL because `PolicyBlocked` and its constructor do not exist yet.

**Step 3: Write minimal implementation**

Update `apps/runtime/src-tauri/src/agent/run_guard.rs` to:

- add `RunStopReasonKind::PolicyBlocked`
- map it to `policy_blocked` in `as_key()`
- add `RunStopReason::policy_blocked(detail: impl Into<String>)`

Keep the title and message stable and put the actionable root cause into `detail`.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml policy_blocked -- --nocapture
```

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/run_guard.rs
git commit -m "feat(runtime): add policy blocked stop reason"
```

### Task 2: Classify Deterministic Policy Failures In The Executor

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/executor.rs`
- Test: `apps/runtime/src-tauri/src/agent/executor.rs`

**Step 1: Write the failing test**

Add focused tests in `apps/runtime/src-tauri/src/agent/executor.rs` for a classifier helper such as:

```rust
#[test]
fn workspace_boundary_error_maps_to_policy_blocked() {
    let reason = classify_policy_blocked_tool_error(
        "list_dir",
        "路径 C:\\Users\\Administrator\\Desktop 不在工作目录 C:\\Users\\Administrator\\WorkClaw\\workspace 范围内",
    )
    .expect("should classify");

    assert_eq!(reason.kind, RunStopReasonKind::PolicyBlocked);
}

#[test]
fn skill_allowlist_error_maps_to_policy_blocked() {
    let reason = classify_policy_blocked_tool_error(
        "bash",
        "此 Skill 不允许使用工具: bash",
    )
    .expect("should classify");

    assert_eq!(reason.kind, RunStopReasonKind::PolicyBlocked);
}

#[test]
fn ordinary_tool_failure_is_not_policy_blocked() {
    let reason = classify_policy_blocked_tool_error(
        "read_file",
        "文件不存在: missing.txt",
    );

    assert!(reason.is_none());
}
```

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml workspace_boundary_error_maps_to_policy_blocked -- --nocapture
```

Expected: FAIL because the classifier does not exist yet.

**Step 3: Write minimal implementation**

In `apps/runtime/src-tauri/src/agent/executor.rs`:

- add `classify_policy_blocked_tool_error(...)`
- detect `不在工作目录`
- detect `此 Skill 不允许使用工具`
- return `RunStopReason::policy_blocked(...)` with a normalized detail string

Do not match transient failures such as missing files, parse errors, or network issues.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml workspace_boundary_error_maps_to_policy_blocked -- --nocapture
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml skill_allowlist_error_maps_to_policy_blocked -- --nocapture
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml ordinary_tool_failure_is_not_policy_blocked -- --nocapture
```

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/executor.rs
git commit -m "feat(runtime): classify deterministic policy failures"
```

### Task 3: Stop Runs Immediately When Policy Blocking Is Detected

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/executor.rs`
- Modify: `apps/runtime/src-tauri/src/commands/chat_policy.rs`
- Test: `apps/runtime/src-tauri/src/agent/executor.rs`

**Step 1: Write the failing test**

Add an executor-level test that verifies a policy-blocked tool failure becomes a structured stopped run instead of a plain tool error retry.

Sketch:

```rust
#[tokio::test]
async fn executor_returns_policy_blocked_stop_reason_for_workspace_boundary() {
    let result = /* run executor with a mocked tool call that returns work-dir boundary error */;
    let err = result.expect_err("run should stop");
    let stop = parse_run_stop_reason(&err.to_string()).expect("structured stop");
    assert_eq!(stop.kind, RunStopReasonKind::PolicyBlocked);
}
```

Also add a small `chat_policy.rs` assertion that `policy_blocked` maps to its own error key instead of falling into `unknown`.

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml executor_returns_policy_blocked_stop_reason_for_workspace_boundary -- --nocapture
```

Expected: FAIL because the executor still feeds the tool error back into the loop.

**Step 3: Write minimal implementation**

Update the tool failure branch in `apps/runtime/src-tauri/src/agent/executor.rs` to:

- call `classify_policy_blocked_tool_error(...)`
- if matched:
  - emit `AgentStateEvent::stopped`
  - return `Err(anyhow!(encode_run_stop_reason(&reason)))`
  - skip appending the tool failure to `tool_results`

Update `apps/runtime/src-tauri/src/commands/chat_policy.rs` so the new reason survives downstream error-kind mapping.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml executor_returns_policy_blocked_stop_reason_for_workspace_boundary -- --nocapture
```

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/executor.rs apps/runtime/src-tauri/src/commands/chat_policy.rs
git commit -m "feat(runtime): stop immediately on policy blocked failures"
```

### Task 4: Render Friendly Recovery Messaging In ChatView

**Files:**
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Test: `apps/runtime/src/components/__tests__/ChatView.run-guardrails.test.tsx`

**Step 1: Write the failing test**

Add a test in `apps/runtime/src/components/__tests__/ChatView.run-guardrails.test.tsx` similar to:

```tsx
it("renders policy blocked recovery messaging", async () => {
  emitAgentStopped({
    stop_reason_kind: "policy_blocked",
    stop_reason_title: "当前任务无法继续执行",
    stop_reason_message: "本次请求触发了安全或工作区限制，系统已停止继续尝试。",
  });

  expect(await screen.findByText("当前任务无法继续执行")).toBeInTheDocument();
  expect(screen.getByText("本次请求触发了安全或工作区限制，系统已停止继续尝试。")).toBeInTheDocument();
});
```

Add a second test where the persisted run has boundary detail and assert the work-dir recovery hint appears.

**Step 2: Run test to verify it fails**

Run:

```bash
pnpm --dir apps/runtime test -- --run src/components/__tests__/ChatView.run-guardrails.test.tsx
```

Expected: FAIL because `policy_blocked` has no display mapping yet.

**Step 3: Write minimal implementation**

Update `apps/runtime/src/components/ChatView.tsx` to:

- treat `policy_blocked` as a dedicated stopped state
- render title `当前任务无法继续执行`
- render the generic safety/workspace message
- render a recovery hint when the detail indicates work-dir boundary failure

Keep `error` reserved for genuine runtime failures.

**Step 4: Run test to verify it passes**

Run:

```bash
pnpm --dir apps/runtime test -- --run src/components/__tests__/ChatView.run-guardrails.test.tsx
```

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/ChatView.tsx apps/runtime/src/components/__tests__/ChatView.run-guardrails.test.tsx
git commit -m "feat(chat): show policy blocked recovery guidance"
```

### Task 5: Verify End-To-End Regression Safety

**Files:**
- Modify: none unless failures require fixes
- Test: existing runtime and frontend tests above

**Step 1: Run targeted Rust verification**

Run:

```bash
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml policy_blocked -- --nocapture
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml workspace_boundary_error_maps_to_policy_blocked -- --nocapture
cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml executor_returns_policy_blocked_stop_reason_for_workspace_boundary -- --nocapture
```

Expected: PASS

**Step 2: Run targeted frontend verification**

Run:

```bash
pnpm --dir apps/runtime test -- --run src/components/__tests__/ChatView.run-guardrails.test.tsx
```

Expected: PASS

**Step 3: Smoke-check the user-facing scenario**

Run the desktop app and verify:

- create a session with default work dir under `WorkClaw\\workspace`
- ask to list or整理 `C:\\Users\\<user>\\Desktop`
- confirm the run stops immediately with `policy_blocked`
- confirm the UI suggests switching the work dir and retrying

**Step 4: Commit**

```bash
git add -A
git commit -m "feat(runtime): unify deterministic policy-blocked exits"
```

Plan complete and saved to `docs/plans/2026-03-17-policy-blocked-stop-reason-plan.md`. Two execution options:

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

**Which approach?**
