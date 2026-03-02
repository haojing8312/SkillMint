# Feishu Multi-Role Collaboration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Deliver an MVP where Feishu group chat can run real-time multi-role agent collaboration with human interruption, using OpenClaw-style layered memory.

**Architecture:** Add a new IM gateway and orchestrator pipeline in Rust Tauri backend, bridge it to existing runtime streaming events, and expose observability in existing React chat panel. Reuse current `task_tool`/sub-agent execution chain and extend state/events instead of rewriting core agent loop.

**Tech Stack:** Rust (Tauri, tokio, serde, sqlx), TypeScript/React, SQLite, existing agent executor/event bus.

---

### Task 1: Define Feishu/IM Domain Types and Event Contracts

**Files:**
- Create: `apps/runtime/src-tauri/src/im/mod.rs`
- Create: `apps/runtime/src-tauri/src/im/types.rs`
- Modify: `apps/runtime/src-tauri/src/lib.rs`
- Test: `apps/runtime/src-tauri/tests/test_im_event_contract.rs`

**Step 1: Write the failing test**

```rust
use runtime_lib::im::types::{ImEvent, ImEventType};

#[test]
fn im_event_parses_minimal_message_created() {
    let raw = r#"{"event_type":"message.created","thread_id":"t1","message_id":"m1","text":"hello"}"#;
    let evt: ImEvent = serde_json::from_str(raw).unwrap();
    assert_eq!(evt.event_type, ImEventType::MessageCreated);
    assert_eq!(evt.thread_id, "t1");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test test_im_event_contract -- --nocapture`  
Expected: FAIL with module/type not found.

**Step 3: Write minimal implementation**

Implement `ImEvent`, `ImEventType`, and serde mapping in `im/types.rs`; export module in `im/mod.rs`; register module in `lib.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo test --test test_im_event_contract -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/im/mod.rs apps/runtime/src-tauri/src/im/types.rs apps/runtime/src-tauri/src/lib.rs apps/runtime/src-tauri/tests/test_im_event_contract.rs
git commit -m "feat(im): add normalized IM event contracts"
```

### Task 2: Add Feishu Callback Ingress Command and Idempotency Guard

**Files:**
- Create: `apps/runtime/src-tauri/src/commands/im_gateway.rs`
- Modify: `apps/runtime/src-tauri/src/commands/mod.rs`
- Modify: `apps/runtime/src-tauri/src/lib.rs`
- Modify: `apps/runtime/src-tauri/src/db.rs`
- Test: `apps/runtime/src-tauri/tests/test_feishu_callback_idempotency.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn callback_same_event_id_is_processed_once() {
    // call handle_feishu_callback twice with same event_id
    // assert second call returns deduped=true and does not enqueue duplicate work
    assert!(false, "placeholder");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test test_feishu_callback_idempotency -- --nocapture`  
Expected: FAIL.

**Step 3: Write minimal implementation**

- Add table `im_event_dedup(event_id TEXT PRIMARY KEY, created_at TEXT)`.
- Add tauri command `handle_feishu_callback(payload: String)`.
- Parse event id, perform insert-or-ignore, return `{accepted, deduped}`.

**Step 4: Run test to verify it passes**

Run: `cargo test --test test_feishu_callback_idempotency -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/im_gateway.rs apps/runtime/src-tauri/src/commands/mod.rs apps/runtime/src-tauri/src/lib.rs apps/runtime/src-tauri/src/db.rs apps/runtime/src-tauri/tests/test_feishu_callback_idempotency.rs
git commit -m "feat(im): add feishu callback ingress with idempotency guard"
```

### Task 3: Implement Conversation Orchestrator with Interruption Priority

**Files:**
- Create: `apps/runtime/src-tauri/src/im/orchestrator.rs`
- Modify: `apps/runtime/src-tauri/src/im/mod.rs`
- Test: `apps/runtime/src-tauri/tests/test_orchestrator_interrupt_priority.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn human_override_preempts_auto_turn() {
    // setup staged flow
    // send auto turn event, then human override
    // assert next action is override-applied, not auto-turn
    assert!(false, "placeholder");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test test_orchestrator_interrupt_priority -- --nocapture`  
Expected: FAIL.

**Step 3: Write minimal implementation**

Implement orchestrator with priority order:
1) override
2) pause/resume
3) mention-role
4) auto stage policy

**Step 4: Run test to verify it passes**

Run: `cargo test --test test_orchestrator_interrupt_priority -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/im/orchestrator.rs apps/runtime/src-tauri/src/im/mod.rs apps/runtime/src-tauri/tests/test_orchestrator_interrupt_priority.rs
git commit -m "feat(orchestrator): add interruption priority scheduler"
```

### Task 4: Bridge Orchestrator Actions to Existing Runtime Task Tool

**Files:**
- Create: `apps/runtime/src-tauri/src/im/runtime_bridge.rs`
- Modify: `apps/runtime/src-tauri/src/commands/chat.rs`
- Modify: `apps/runtime/src-tauri/src/im/mod.rs`
- Test: `apps/runtime/src-tauri/tests/test_im_runtime_bridge.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn bridge_dispatches_role_task_and_receives_stream_events() {
    // assert role task dispatch emits expected runtime invocation payload
    assert!(false, "placeholder");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test test_im_runtime_bridge -- --nocapture`  
Expected: FAIL.

**Step 3: Write minimal implementation**

- Convert orchestrator action into runtime request (`task_tool` compatible).
- Subscribe to `stream-token` / `agent-state-event`.
- Produce normalized role progress events.

**Step 4: Run test to verify it passes**

Run: `cargo test --test test_im_runtime_bridge -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/im/runtime_bridge.rs apps/runtime/src-tauri/src/commands/chat.rs apps/runtime/src-tauri/src/im/mod.rs apps/runtime/src-tauri/tests/test_im_runtime_bridge.rs
git commit -m "feat(im): bridge orchestrator actions to runtime sub-agent flow"
```

### Task 5: Add Role/Thread Binding and Scenario Template Persistence

**Files:**
- Modify: `apps/runtime/src-tauri/src/db.rs`
- Create: `apps/runtime/src-tauri/src/commands/im_config.rs`
- Modify: `apps/runtime/src-tauri/src/commands/mod.rs`
- Modify: `apps/runtime/src-tauri/src/lib.rs`
- Test: `apps/runtime/src-tauri/tests/test_im_role_binding.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn group_thread_can_bind_multiple_roles() {
    // insert binding and query
    // assert role count >= 2 and scenario template persisted
    assert!(false, "placeholder");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test test_im_role_binding -- --nocapture`  
Expected: FAIL.

**Step 3: Write minimal implementation**

Add tables:
- `im_thread_bindings(thread_id, tenant_id, scenario_template, status)`
- `im_thread_roles(thread_id, role_id, role_order, enabled)`

Add commands:
- bind roles to thread
- get thread config

**Step 4: Run test to verify it passes**

Run: `cargo test --test test_im_role_binding -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/db.rs apps/runtime/src-tauri/src/commands/im_config.rs apps/runtime/src-tauri/src/commands/mod.rs apps/runtime/src-tauri/src/lib.rs apps/runtime/src-tauri/tests/test_im_role_binding.rs
git commit -m "feat(im): persist thread-role bindings and scenario templates"
```

### Task 6: Implement OpenClaw-style Layered Memory (Role/Session/Daily/Org)

**Files:**
- Create: `apps/runtime/src-tauri/src/im/memory.rs`
- Modify: `apps/runtime/src-tauri/src/agent/tools/memory_tool.rs`
- Modify: `apps/runtime/src-tauri/src/im/mod.rs`
- Test: `apps/runtime/src-tauri/tests/test_im_memory_layers.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn capture_writes_session_and_long_term_with_gate() {
    // confirmed fact -> role memory write
    // unconfirmed statement -> session only
    assert!(false, "placeholder");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test test_im_memory_layers -- --nocapture`  
Expected: FAIL.

**Step 3: Write minimal implementation**

- Memory paths:
  - `memory/daily/YYYY-MM-DD.md`
  - `memory/sessions/<thread_id>.md`
  - `memory/roles/<role_id>/MEMORY.md`
  - `memory/org/CASEBOOK.md`
- Implement recall + capture API with metadata.
- Implement write gate for long-term memory.

**Step 4: Run test to verify it passes**

Run: `cargo test --test test_im_memory_layers -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/im/memory.rs apps/runtime/src-tauri/src/agent/tools/memory_tool.rs apps/runtime/src-tauri/src/im/mod.rs apps/runtime/src-tauri/tests/test_im_memory_layers.rs
git commit -m "feat(memory): add OpenClaw-style layered memory with write gate"
```

### Task 7: Emit IM-Oriented Real-Time Events and Add Frontend Trace Panel Models

**Files:**
- Modify: `apps/runtime/src-tauri/src/im/runtime_bridge.rs`
- Modify: `apps/runtime/src/types.ts`
- Modify: `apps/runtime/src/components/ChatView.tsx`
- Test: `apps/runtime/src/components/__tests__/ChatView.im-routing-panel.test.tsx`

**Step 1: Write the failing test**

```tsx
it("shows routing timeline events for role collaboration", async () => {
  // render ChatView, inject im-route events, assert timeline cards render
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --filter runtime test ChatView.im-routing-panel.test.tsx`  
Expected: FAIL.

**Step 3: Write minimal implementation**

- Add event model for role collaboration timeline.
- Render per-role status cards (running/completed/failed).
- Keep existing panel tabs and append IM timeline section.

**Step 4: Run test to verify it passes**

Run: `pnpm --filter runtime test ChatView.im-routing-panel.test.tsx`  
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src/types.ts apps/runtime/src/components/ChatView.tsx apps/runtime/src/components/__tests__/ChatView.im-routing-panel.test.tsx apps/runtime/src-tauri/src/im/runtime_bridge.rs
git commit -m "feat(ui): show real-time multi-role collaboration timeline"
```

### Task 8: Feishu Outbound Formatter (Conclusion/Evidence/Uncertainty/Next Step)

**Files:**
- Create: `apps/runtime/src-tauri/src/im/feishu_formatter.rs`
- Modify: `apps/runtime/src-tauri/src/im/mod.rs`
- Test: `apps/runtime/src-tauri/tests/test_feishu_formatter.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn formatter_outputs_required_sections() {
    // assert formatted message contains four required sections
    assert!(false, "placeholder");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test test_feishu_formatter -- --nocapture`  
Expected: FAIL.

**Step 3: Write minimal implementation**

Implement formatter ensuring each role output includes:
- 结论
- 依据
- 不确定项
- 下一步

**Step 4: Run test to verify it passes**

Run: `cargo test --test test_feishu_formatter -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/im/feishu_formatter.rs apps/runtime/src-tauri/src/im/mod.rs apps/runtime/src-tauri/tests/test_feishu_formatter.rs
git commit -m "feat(feishu): standardize outbound role message format"
```

### Task 9: Add 商机评审 Scenario Template and Stage Engine

**Files:**
- Create: `apps/runtime/src-tauri/src/im/scenarios/mod.rs`
- Create: `apps/runtime/src-tauri/src/im/scenarios/opportunity_review.rs`
- Modify: `apps/runtime/src-tauri/src/im/orchestrator.rs`
- Test: `apps/runtime/src-tauri/tests/test_opportunity_review_scenario.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn opportunity_review_reaches_final_recommendation_stage() {
    // feed stage inputs and assert output includes 承接建议 + 成本区间 + 风险
    assert!(false, "placeholder");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test test_opportunity_review_scenario -- --nocapture`  
Expected: FAIL.

**Step 3: Write minimal implementation**

Stage sequence:
1) 信息澄清
2) 可承接评估
3) 成本与风险估算
4) 最终建议

**Step 4: Run test to verify it passes**

Run: `cargo test --test test_opportunity_review_scenario -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/src/im/scenarios/mod.rs apps/runtime/src-tauri/src/im/scenarios/opportunity_review.rs apps/runtime/src-tauri/src/im/orchestrator.rs apps/runtime/src-tauri/tests/test_opportunity_review_scenario.rs
git commit -m "feat(scenario): add opportunity review stage engine"
```

### Task 10: End-to-End Validation for Feishu Multi-Role Collaboration

**Files:**
- Create: `apps/runtime/src-tauri/tests/test_im_multi_role_e2e.rs`
- Modify: `apps/runtime/src-tauri/tests/helpers/mod.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn feishu_thread_multi_role_collaboration_e2e() {
    // simulate callback events, role dispatch, interruption, and final summary
    assert!(false, "placeholder");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test test_im_multi_role_e2e -- --nocapture`  
Expected: FAIL.

**Step 3: Write minimal implementation**

Complete glue logic and fix integration gaps until test passes.

**Step 4: Run test to verify it passes**

Run: `cargo test --test test_im_multi_role_e2e -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add apps/runtime/src-tauri/tests/test_im_multi_role_e2e.rs apps/runtime/src-tauri/tests/helpers/mod.rs
git commit -m "test(im): add e2e coverage for feishu multi-role collaboration flow"
```

### Task 11: Full Verification Before Completion

**Files:**
- Modify: `README.zh-CN.md`
- Modify: `docs/plans/2026-03-02-feishu-multi-role-collab-design.md` (if implementation diverges)

**Step 1: Run backend test suite subset**

Run: `cargo test --tests -- --nocapture`  
Expected: all relevant IM tests PASS.

**Step 2: Run frontend test suite**

Run: `pnpm --filter runtime test`  
Expected: PASS.

**Step 3: Run build checks**

Run: `pnpm --filter runtime build`  
Expected: PASS.

**Step 4: Update docs**

Document:
- Feishu setup
- thread-role binding setup
- interruption commands
- memory layout

**Step 5: Commit**

```bash
git add README.zh-CN.md docs/plans/2026-03-02-feishu-multi-role-collab-design.md
git commit -m "docs(im): add feishu multi-role collaboration setup and operations"
```

Plan complete and saved to `docs/plans/2026-03-02-feishu-multi-role-collab-implementation.md`. Two execution options:

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

Which approach?

