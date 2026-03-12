# Agent Session Resilience and Task Journey Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add hybrid session journaling, explicit failure terminal states, and run-scoped task journey rendering so WorkClaw no longer loses streamed records or places delivery summaries above the actual conversation.

**Architecture:** Introduce an append-only session journal under app data as the durable fact source, then project the latest state back into SQLite and the existing frontend message model. Split run lifecycle from session-level aggregates: the main chat area shows only the current live run banner plus inline per-run summary cards, while the side panel remains a session-level aggregate view.

**Tech Stack:** Tauri Rust, SQLite via `sqlx`, local filesystem JSONL/Markdown journaling, React + TypeScript, Vitest, cargo tests.

---

### Task 1: Create the session journal domain model and append-only writer

**Files:**
- Create: `apps/runtime/src-tauri/src/session_journal.rs`
- Modify: `apps/runtime/src-tauri/src/lib.rs`
- Modify: `apps/runtime/src-tauri/src/db.rs`
- Test: `apps/runtime/src-tauri/tests/test_session_journal.rs`

**Step 1: Write the failing Rust test**

```rust
#[tokio::test]
async fn append_event_persists_jsonl_and_updates_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    let store = SessionJournalStore::new(dir.path().to_path_buf());

    store
        .append_event(
            "sess-1",
            SessionRunEvent::RunStarted {
                run_id: "run-1".into(),
                user_message_id: "user-1".into(),
            },
        )
        .await
        .unwrap();

    store
        .append_event(
            "sess-1",
            SessionRunEvent::AssistantChunkAppended {
                run_id: "run-1".into(),
                chunk: "hello".into(),
            },
        )
        .await
        .unwrap();

    let snapshot = store.read_state("sess-1").await.unwrap();
    assert_eq!(snapshot.current_run_id.as_deref(), Some("run-1"));
    assert_eq!(snapshot.runs[0].buffered_text, "hello");
}
```

**Step 2: Run test to verify it fails**

Run: `cd apps/runtime/src-tauri && cargo test --test test_session_journal -- --nocapture`

Expected: FAIL because `session_journal.rs` and the store types do not exist.

**Step 3: Write the minimal journal implementation**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SessionRunEvent {
    RunStarted { run_id: String, user_message_id: String },
    AssistantChunkAppended { run_id: String, chunk: String },
    ToolStarted { run_id: String, tool_name: String, call_id: String },
    ToolCompleted { run_id: String, tool_name: String, call_id: String, output: String },
    RunCompleted { run_id: String },
    RunFailed { run_id: String, error_kind: String, error_message: String },
}

pub struct SessionJournalStore {
    root: PathBuf,
}

impl SessionJournalStore {
    pub fn new(root: PathBuf) -> Self { Self { root } }

    pub async fn append_event(
        &self,
        session_id: &str,
        event: SessionRunEvent,
    ) -> Result<(), String> {
        // append to events.jsonl, then refresh state.json
    }
}
```

Also add database tables for lightweight projections:

- `session_runs`
- `session_run_events`

**Step 4: Run test to verify it passes**

Run: `cd apps/runtime/src-tauri && cargo test --test test_session_journal -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/session_journal.rs apps/runtime/src-tauri/src/lib.rs apps/runtime/src-tauri/src/db.rs apps/runtime/src-tauri/tests/test_session_journal.rs
git commit -m "feat(runtime): add session journal store"
```

### Task 2: Persist run lifecycle incrementally from the chat command path

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`
- Create: `apps/runtime/src-tauri/src/commands/session_runs.rs`
- Modify: `apps/runtime/src-tauri/src/commands/mod.rs`
- Modify: `apps/runtime/src-tauri/src/lib.rs`
- Test: `apps/runtime/src-tauri/tests/test_session_run_commands.rs`

**Step 1: Write the failing Rust tests**

```rust
#[tokio::test]
async fn send_message_creates_run_and_journal_entry_before_model_returns() {
    let harness = TestChatHarness::new().await;

    harness
        .send_message("sess-1", "帮我看看当前目录")
        .await
        .unwrap();

    let runs = harness.list_session_runs("sess-1").await.unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].status, "thinking");
}

#[tokio::test]
async fn failed_run_is_visible_even_when_no_final_assistant_message_is_generated() {
    let harness = TestChatHarness::with_mock_error("insufficient_balance").await;

    let err = harness.send_message("sess-1", "继续执行").await.unwrap_err();
    assert!(err.contains("insufficient_balance"));

    let runs = harness.list_session_runs("sess-1").await.unwrap();
    assert_eq!(runs[0].status, "failed");
}
```

**Step 2: Run tests to verify they fail**

Run: `cd apps/runtime/src-tauri && cargo test --test test_session_run_commands -- --nocapture`

Expected: FAIL because no run projection commands exist yet.

**Step 3: Implement incremental run persistence**

```rust
#[tauri::command]
pub async fn list_session_runs(
    session_id: String,
    db: State<'_, DbState>,
) -> Result<Vec<serde_json::Value>, String> {
    // return latest run projections ordered by created_at asc
}

fn append_run_started(...) -> Result<(), String> {
    // write journal + insert projection row
}

fn append_run_failure(...) -> Result<(), String> {
    // write journal + mark projection failed + keep partial content
}
```

Update `send_message` so it:

- creates a `run_id` at the start of each assistant turn
- writes `RunStarted` before the provider call
- writes `AssistantChunkAppended` batches via a buffered callback
- records `ToolStarted` / `ToolCompleted`
- always records `RunCompleted` or `RunFailed`

**Step 4: Run tests to verify they pass**

Run: `cd apps/runtime/src-tauri && cargo test --test test_session_run_commands -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/chat.rs apps/runtime/src-tauri/src/commands/session_runs.rs apps/runtime/src-tauri/src/commands/mod.rs apps/runtime/src-tauri/src/lib.rs apps/runtime/src-tauri/tests/test_session_run_commands.rs
git commit -m "feat(runtime): persist run lifecycle incrementally"
```

### Task 3: Harden provider termination and classify billing failures explicitly

**Files:**
- Modify: `apps/runtime/src-tauri/src/adapters/anthropic.rs`
- Modify: `apps/runtime/src-tauri/src/adapters/openai.rs`
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`
- Modify: `apps/runtime/src-tauri/src/agent/executor.rs`
- Test: `apps/runtime/src-tauri/tests/test_anthropic_tools.rs`
- Test: `apps/runtime/src-tauri/tests/test_openai_tools.rs`
- Create: `apps/runtime/src-tauri/tests/test_session_run_failures.rs`

**Step 1: Write the failing tests**

```rust
#[tokio::test]
async fn anthropic_message_stop_ends_outer_stream_loop() {
    let response = read_mock_sse("anthropic_message_stop_then_idle.txt");
    let parsed = parse_anthropic_stream_for_test(response).await.unwrap();
    assert_eq!(parsed.finish_reason.as_deref(), Some("message_stop"));
}

#[tokio::test]
async fn insufficient_balance_is_classified_as_billing_and_emits_failed_run() {
    let err = classify_model_route_error("insufficient_balance: account balance too low");
    assert_eq!(err, ModelRouteErrorKind::Billing);
}
```

**Step 2: Run tests to verify they fail**

Run: `cd apps/runtime/src-tauri && cargo test --test test_session_run_failures test_openai_tools test_anthropic_tools -- --nocapture`

Expected: FAIL because billing is not a distinct error kind and the stream parser does not short-circuit the outer loop.

**Step 3: Implement minimal hardening**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModelRouteErrorKind {
    Billing,
    Auth,
    RateLimit,
    Timeout,
    Network,
    Unknown,
}

let mut stop_stream = false;
while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    for line in text.lines() {
        if data.trim() == "[DONE]" {
            stop_stream = true;
            break;
        }
    }
    if stop_stream {
        break;
    }
}
```

Also:

- add `reqwest::Client::builder().timeout(...)`
- map insufficient balance / quota text to billing errors
- emit `agent-state-event = failed` and `stream-token.done = true` on all terminal provider failures

**Step 4: Run tests to verify they pass**

Run: `cd apps/runtime/src-tauri && cargo test --test test_session_run_failures test_openai_tools test_anthropic_tools -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/adapters/anthropic.rs apps/runtime/src-tauri/src/adapters/openai.rs apps/runtime/src-tauri/src/commands/chat.rs apps/runtime/src-tauri/src/agent/executor.rs apps/runtime/src-tauri/tests/test_session_run_failures.rs apps/runtime/src-tauri/tests/test_anthropic_tools.rs apps/runtime/src-tauri/tests/test_openai_tools.rs
git commit -m "fix(runtime): close failed runs and classify billing errors"
```

### Task 4: Expose journal-backed run summaries to the frontend and preserve partial output

**Files:**
- Modify: `apps/runtime/src/types.ts`
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Create: `apps/runtime/src/components/__tests__/ChatView.session-resilience.test.tsx`

**Step 1: Write the failing frontend test**

```tsx
test("clears live banner and shows partial failure card when the run ends with billing failure", async () => {
  render(<ChatView {...props} />);

  emit("agent-state-event", {
    session_id: "sess-1",
    state: "thinking",
    detail: null,
    iteration: 1,
  });

  emit("stream-token", {
    session_id: "sess-1",
    token: "已经生成 2 个文件",
    done: false,
  });

  emit("session-run-event", {
    session_id: "sess-1",
    run_id: "run-1",
    status: "failed",
    error_kind: "insufficient_balance",
    error_message: "模型余额不足",
  });

  emit("stream-token", { session_id: "sess-1", token: "", done: true });

  expect(screen.queryByText("正在分析任务")).not.toBeInTheDocument();
  expect(screen.getByText("模型余额不足")).toBeInTheDocument();
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm exec vitest run apps/runtime/src/components/__tests__/ChatView.session-resilience.test.tsx`

Expected: FAIL because `ChatView` does not listen for run terminal events or retain partial output on failed runs.

**Step 3: Implement minimal UI state changes**

```ts
export interface SessionRunEvent {
  session_id: string;
  run_id: string;
  status: "thinking" | "tool_calling" | "completed" | "failed" | "cancelled";
  error_kind?: string;
  error_message?: string;
}

const [liveRun, setLiveRun] = useState<SessionRunEvent | null>(null);

if (payload.status === "failed") {
  setAgentState(null);
  finalizeBufferedAssistantMessage(payload.run_id, {
    terminalState: "failed",
    errorMessage: payload.error_message,
  });
}
```

**Step 4: Run test to verify it passes**

Run: `pnpm exec vitest run apps/runtime/src/components/__tests__/ChatView.session-resilience.test.tsx`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/types.ts apps/runtime/src/components/ChatView.tsx apps/runtime/src/components/__tests__/ChatView.session-resilience.test.tsx
git commit -m "feat(ui): preserve partial output for failed runs"
```

### Task 5: Move task journey and delivery summaries from global top cards to inline run cards

**Files:**
- Modify: `apps/runtime/src/components/chat-side-panel/view-model.ts`
- Modify: `apps/runtime/src/components/chat-journey/TaskJourneySummary.tsx`
- Create: `apps/runtime/src/components/chat-journey/RunJourneyCard.tsx`
- Modify: `apps/runtime/src/components/chat-journey/TaskJourneyTimeline.tsx`
- Modify: `apps/runtime/src/components/chat-journey/DeliverySummaryCard.tsx`
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Test: `apps/runtime/src/components/chat-side-panel/view-model.test.ts`
- Test: `apps/runtime/src/components/__tests__/ChatView.side-panel-redesign.test.tsx`

**Step 1: Write the failing tests**

```ts
it("groups deliverables and warnings by run_id instead of aggregating the whole session into one top card", () => {
  const model = buildTaskJourneyViewModel([
    assistantMessageForRun("run-1", ["a.md"], []),
    assistantMessageForRun("run-2", ["b.md"], ["list_dir failed"]),
  ]);

  expect(model.runs).toHaveLength(2);
  expect(model.runs[0].deliverables[0].path).toBe("a.md");
  expect(model.runs[1].warnings[0]).toContain("list_dir failed");
});
```

```tsx
test("renders run summary after the matching assistant message instead of above the whole transcript", async () => {
  render(<ChatView {...propsWithCompletedRun} />);

  const cards = screen.getAllByTestId("run-journey-card");
  const transcript = screen.getAllByTestId(/chat-message-/);

  expect(cards[0].compareDocumentPosition(transcript[0]) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
});
```

**Step 2: Run tests to verify they fail**

Run: `pnpm exec vitest run apps/runtime/src/components/chat-side-panel/view-model.test.ts apps/runtime/src/components/__tests__/ChatView.side-panel-redesign.test.tsx`

Expected: FAIL because the current model is session-aggregate only and `TaskJourneySummary` is rendered before all messages.

**Step 3: Implement run-scoped summaries**

```ts
export interface TaskJourneyRunView {
  runId: string;
  status: "running" | "completed" | "failed" | "partial";
  title: string;
  steps: TaskJourneyStepView[];
  deliverables: DeliverableView[];
  warnings: string[];
}

export interface TaskJourneyViewModel {
  liveRunId?: string;
  runs: TaskJourneyRunView[];
}
```

Render rules:

- top of transcript: live banner only
- after each assistant message: one `RunJourneyCard` for that message's `runId`
- side panel: keep aggregate counts, but clearly label them as session-level

**Step 4: Run tests to verify they pass**

Run: `pnpm exec vitest run apps/runtime/src/components/chat-side-panel/view-model.test.ts apps/runtime/src/components/__tests__/ChatView.side-panel-redesign.test.tsx`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src/components/chat-side-panel/view-model.ts apps/runtime/src/components/chat-journey/TaskJourneySummary.tsx apps/runtime/src/components/chat-journey/RunJourneyCard.tsx apps/runtime/src/components/chat-journey/TaskJourneyTimeline.tsx apps/runtime/src/components/chat-journey/DeliverySummaryCard.tsx apps/runtime/src/components/ChatView.tsx apps/runtime/src/components/chat-side-panel/view-model.test.ts apps/runtime/src/components/__tests__/ChatView.side-panel-redesign.test.tsx
git commit -m "refactor(ui): render task journey inline per run"
```

### Task 6: Make export and recovery journal-aware

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`
- Modify: `apps/runtime/src-tauri/src/commands/session_runs.rs`
- Create: `apps/runtime/src-tauri/tests/test_session_export_recovery.rs`

**Step 1: Write the failing Rust test**

```rust
#[tokio::test]
async fn export_session_uses_journal_when_sqlite_projection_is_incomplete() {
    let harness = TestChatHarness::new().await;
    harness.seed_partial_projection_but_complete_journal("sess-1").await;

    let md = harness.export_session("sess-1").await.unwrap();

    assert!(md.contains("已经生成 2 个文件"));
    assert!(md.contains("模型余额不足"));
}
```

**Step 2: Run test to verify it fails**

Run: `cd apps/runtime/src-tauri && cargo test --test test_session_export_recovery -- --nocapture`

Expected: FAIL because `export_session` currently reads only `messages`.

**Step 3: Implement journal-aware export**

```rust
pub async fn export_session(session_id: String, db: State<'_, DbState>) -> Result<String, String> {
    let projected = load_messages_projection(&session_id, &db).await?;
    let recovered = recover_missing_terminal_run_segments(&session_id, projected).await?;
    render_markdown_export(recovered)
}
```

Also add:

- a helper to rebuild missing draft content from `events.jsonl`
- a small command to open the session journal folder for diagnostics if needed later

**Step 4: Run test to verify it passes**

Run: `cd apps/runtime/src-tauri && cargo test --test test_session_export_recovery -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/chat.rs apps/runtime/src-tauri/src/commands/session_runs.rs apps/runtime/src-tauri/tests/test_session_export_recovery.rs
git commit -m "feat(runtime): recover exports from session journal"
```

### Task 7: Run full verification across Rust and frontend surfaces

**Files:**
- Modify: `docs/plans/2026-03-11-agent-session-resilience-and-task-journey-design.md`
- Modify: `docs/plans/2026-03-11-agent-session-resilience-and-task-journey-implementation-plan.md`

**Step 1: Run targeted Rust tests**

Run:

```bash
cd apps/runtime/src-tauri
cargo test --test test_session_journal --test test_session_run_commands --test test_session_run_failures --test test_session_export_recovery -- --nocapture
```

Expected: PASS

**Step 2: Run targeted frontend tests**

Run:

```bash
pnpm exec vitest run apps/runtime/src/components/__tests__/ChatView.session-resilience.test.tsx apps/runtime/src/components/__tests__/ChatView.side-panel-redesign.test.tsx apps/runtime/src/components/chat-side-panel/view-model.test.ts
```

Expected: PASS

**Step 3: Run a local manual scenario**

Run:

```bash
pnpm app
```

Manual checks:

- 模型余额不足时会话进入失败终态
- “正在分析任务”会消失
- 已展示的部分输出不会丢失
- 交付结果卡显示在对应 assistant 回复之后
- 导出结果包含失败原因与部分产物

**Step 4: Update plan/docs with verification notes**

Add exact results, screenshots if needed, and any deviations discovered during manual validation.

**Step 5: Commit**

```bash
git add docs/plans/2026-03-11-agent-session-resilience-and-task-journey-design.md docs/plans/2026-03-11-agent-session-resilience-and-task-journey-implementation-plan.md
git commit -m "docs: record session resilience verification notes"
```
