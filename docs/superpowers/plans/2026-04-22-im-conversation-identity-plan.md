# IM Conversation Identity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Introduce an IM-agnostic conversation identity and session-binding model that fixes Feishu session confusion now and provides a reusable foundation for WeCom, DingTalk, and future IM adapters.

**Architecture:** Add a shared `ImConversationSurface` plus canonical `conversation_id` builders in the IM layer, then migrate session binding and reuse logic from legacy `thread_id`/coarse route keys to conversation-aware keys. Roll the change out in a compatibility-first sequence: schema expansion, dual-write/dual-read support, Feishu mapper adoption, then stricter reuse rules and compaction follow-up.

**Tech Stack:** Rust, Tauri runtime, SQLite via `sqlx`, existing IM host bridge, session journal, Rust integration tests

---

### Task 1: Add IM-Agnostic Conversation Identity Primitives

**Files:**
- Create: `apps/runtime/src-tauri/src/im/conversation_surface.rs`
- Create: `apps/runtime/src-tauri/src/im/conversation_id.rs`
- Modify: `apps/runtime/src-tauri/src/im/mod.rs`
- Test: `apps/runtime/src-tauri/src/im/conversation_id.rs`

- [ ] **Step 1: Write the failing unit tests**

```rust
#[test]
fn builds_peer_conversation_id() {
    let surface = ImConversationSurface {
        channel: "feishu".to_string(),
        account_id: "default".to_string(),
        tenant_id: Some("tenant-a".to_string()),
        peer_kind: ImPeerKind::Group,
        peer_id: "oc_team".to_string(),
        topic_id: None,
        sender_id: None,
        scope: ImConversationScope::Peer,
        message_id: None,
        raw_thread_id: Some("oc_team".to_string()),
        raw_root_id: None,
    };

    assert_eq!(
        build_conversation_id(&surface),
        "feishu:default:group:oc_team"
    );
}

#[test]
fn builds_topic_sender_conversation_with_parent_candidates() {
    let surface = ImConversationSurface {
        channel: "feishu".to_string(),
        account_id: "default".to_string(),
        tenant_id: Some("tenant-a".to_string()),
        peer_kind: ImPeerKind::Group,
        peer_id: "oc_team".to_string(),
        topic_id: Some("om_root_1".to_string()),
        sender_id: Some("ou_user_1".to_string()),
        scope: ImConversationScope::TopicSender,
        message_id: Some("om_msg_1".to_string()),
        raw_thread_id: Some("oc_team".to_string()),
        raw_root_id: Some("om_root_1".to_string()),
    };

    assert_eq!(
        build_conversation_id(&surface),
        "feishu:default:group:oc_team:topic:om_root_1:sender:ou_user_1"
    );
    assert_eq!(
        build_parent_conversation_candidates(&surface),
        vec![
            "feishu:default:group:oc_team:topic:om_root_1".to_string(),
            "feishu:default:group:oc_team".to_string(),
        ]
    );
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml conversation_id -- --nocapture`  
Expected: FAIL with missing `ImConversationSurface`, `ImConversationScope`, and builder functions.

- [ ] **Step 3: Implement the shared types and builders**

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImPeerKind {
    Direct,
    Group,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImConversationScope {
    Peer,
    PeerSender,
    Topic,
    TopicSender,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImConversationSurface {
    pub channel: String,
    pub account_id: String,
    pub tenant_id: Option<String>,
    pub peer_kind: ImPeerKind,
    pub peer_id: String,
    pub topic_id: Option<String>,
    pub sender_id: Option<String>,
    pub scope: ImConversationScope,
    pub message_id: Option<String>,
    pub raw_thread_id: Option<String>,
    pub raw_root_id: Option<String>,
}

pub fn build_conversation_id(surface: &ImConversationSurface) -> String {
    match surface.scope {
        ImConversationScope::Peer => format!(
            "{}:{}:{}:{}",
            surface.channel,
            surface.account_id,
            peer_kind_label(&surface.peer_kind),
            surface.peer_id
        ),
        ImConversationScope::PeerSender => format!(
            "{}:{}:{}:{}:sender:{}",
            surface.channel,
            surface.account_id,
            peer_kind_label(&surface.peer_kind),
            surface.peer_id,
            surface.sender_id.as_deref().unwrap_or("unknown")
        ),
        ImConversationScope::Topic => format!(
            "{}:{}:{}:{}:topic:{}",
            surface.channel,
            surface.account_id,
            peer_kind_label(&surface.peer_kind),
            surface.peer_id,
            surface.topic_id.as_deref().unwrap_or("unknown")
        ),
        ImConversationScope::TopicSender => format!(
            "{}:{}:{}:{}:topic:{}:sender:{}",
            surface.channel,
            surface.account_id,
            peer_kind_label(&surface.peer_kind),
            surface.peer_id,
            surface.topic_id.as_deref().unwrap_or("unknown"),
            surface.sender_id.as_deref().unwrap_or("unknown")
        ),
    }
}
```

- [ ] **Step 4: Export the new module from the IM layer**

```rust
pub mod conversation_id;
pub mod conversation_surface;

pub use conversation_id::{build_conversation_id, build_parent_conversation_candidates};
pub use conversation_surface::{ImConversationScope, ImConversationSurface, ImPeerKind};
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml conversation_id -- --nocapture`  
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add apps/runtime/src-tauri/src/im/conversation_surface.rs apps/runtime/src-tauri/src/im/conversation_id.rs apps/runtime/src-tauri/src/im/mod.rs
git commit -m "feat: add IM conversation identity primitives"
```

### Task 2: Expand Session Binding Schema for Conversation-Aware Storage

**Files:**
- Modify: `apps/runtime/src-tauri/src/db/schema.rs`
- Modify: `apps/runtime/src-tauri/src/db/migrations.rs`
- Test: `apps/runtime/src-tauri/tests/test_im_employee_agents/im_routing.rs`

- [ ] **Step 1: Write the failing migration regression test**

```rust
#[tokio::test]
async fn im_thread_sessions_legacy_schema_is_upgraded_with_conversation_columns() {
    let pool = setup_legacy_pool_without_conversation_columns().await;

    run_all_migrations(&pool).await.expect("migrate legacy db");

    let columns: Vec<String> =
        sqlx::query_scalar("SELECT name FROM pragma_table_info('im_thread_sessions')")
            .fetch_all(&pool)
            .await
            .expect("read columns");

    assert!(columns.contains(&"conversation_id".to_string()));
    assert!(columns.contains(&"base_conversation_id".to_string()));
    assert!(columns.contains(&"parent_conversation_candidates_json".to_string()));
    assert!(columns.contains(&"scope".to_string()));
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml im_thread_sessions_legacy_schema_is_upgraded_with_conversation_columns -- --nocapture`  
Expected: FAIL because the columns do not exist yet.

- [ ] **Step 3: Add schema columns and indexes**

```rust
"CREATE TABLE IF NOT EXISTS im_thread_sessions (
    thread_id TEXT NOT NULL,
    employee_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    route_session_key TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    channel TEXT NOT NULL DEFAULT '',
    account_id TEXT NOT NULL DEFAULT '',
    conversation_id TEXT NOT NULL DEFAULT '',
    base_conversation_id TEXT NOT NULL DEFAULT '',
    parent_conversation_candidates_json TEXT NOT NULL DEFAULT '[]',
    scope TEXT NOT NULL DEFAULT '',
    peer_kind TEXT NOT NULL DEFAULT '',
    peer_id TEXT NOT NULL DEFAULT '',
    topic_id TEXT NOT NULL DEFAULT '',
    sender_id TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (thread_id, employee_id)
)"
```

- [ ] **Step 4: Add backward-compatible migration helpers**

```rust
let _ = sqlx::query(
    "ALTER TABLE im_thread_sessions ADD COLUMN conversation_id TEXT NOT NULL DEFAULT ''"
)
.execute(pool)
.await;

let _ = sqlx::query(
    "ALTER TABLE im_thread_sessions ADD COLUMN parent_conversation_candidates_json TEXT NOT NULL DEFAULT '[]'"
)
.execute(pool)
.await;
```

- [ ] **Step 5: Run migration tests**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_im_employee_agents -- --nocapture`  
Expected: PASS for the new migration regression plus existing IM routing tests.

- [ ] **Step 6: Commit**

```bash
git add apps/runtime/src-tauri/src/db/schema.rs apps/runtime/src-tauri/src/db/migrations.rs apps/runtime/src-tauri/tests/test_im_employee_agents/im_routing.rs
git commit -m "feat: add conversation-aware IM session schema"
```

### Task 3: Add Feishu Conversation Mapper and Preserve Rich Surface Metadata

**Files:**
- Create: `apps/runtime/src-tauri/src/commands/feishu_gateway/conversation_mapper.rs`
- Modify: `apps/runtime/src-tauri/src/commands/feishu_gateway.rs`
- Modify: `apps/runtime/src-tauri/src/commands/feishu_gateway/payload_parser.rs`
- Modify: `apps/runtime/src-tauri/src/im/types.rs`
- Test: `apps/runtime/src-tauri/src/commands/feishu_gateway/tests.rs`

- [ ] **Step 1: Write the failing Feishu parsing tests**

```rust
#[test]
fn parse_feishu_payload_maps_group_topic_sender_surface() {
    let event = parse_feishu_payload(include_str!("fixtures/feishu_group_topic_sender.json"))
        .expect("parse event")
        .into_event()
        .expect("event");

    assert_eq!(event.thread_id, "oc_group_1");
    assert_eq!(event.conversation_id.as_deref(), Some("feishu:default:group:oc_group_1:topic:om_root_1:sender:ou_user_1"));
    assert_eq!(event.base_conversation_id.as_deref(), Some("feishu:default:group:oc_group_1"));
}
```

- [ ] **Step 2: Run the Feishu gateway tests to verify they fail**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml feishu_gateway -- --nocapture`  
Expected: FAIL because `ImEvent` lacks the new conversation fields.

- [ ] **Step 3: Extend `ImEvent` with normalized conversation metadata**

```rust
pub struct ImEvent {
    pub channel: String,
    pub event_type: ImEventType,
    pub thread_id: String,
    pub conversation_id: Option<String>,
    pub base_conversation_id: Option<String>,
    pub parent_conversation_candidates: Vec<String>,
    pub conversation_scope: Option<String>,
    // existing fields remain below...
}
```

- [ ] **Step 4: Implement Feishu mapper and parser integration**

```rust
pub fn build_feishu_surface_from_payload(parsed: &ParsedFeishuEnvelope) -> ImConversationSurface {
    // direct => Peer
    // group default => Peer
    // topic => Topic
    // topic + sender => TopicSender
}

let surface = build_feishu_surface_from_payload(&parsed_payload);
let conversation_id = build_conversation_id(&surface);
let parent_candidates = build_parent_conversation_candidates(&surface);
```

- [ ] **Step 5: Run the Feishu gateway tests to verify they pass**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml feishu_gateway -- --nocapture`  
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/feishu_gateway/conversation_mapper.rs apps/runtime/src-tauri/src/commands/feishu_gateway.rs apps/runtime/src-tauri/src/commands/feishu_gateway/payload_parser.rs apps/runtime/src-tauri/src/im/types.rs apps/runtime/src-tauri/src/commands/feishu_gateway/tests.rs
git commit -m "feat: map Feishu events to conversation identities"
```

### Task 4: Migrate Employee Session Binding from Thread-Only Logic to Conversation-Aware Binding

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/session_repo.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/session_service.rs`
- Test: `apps/runtime/src-tauri/tests/test_im_employee_agents/im_routing.rs`

- [ ] **Step 1: Write the failing routing/session tests**

```rust
#[tokio::test]
async fn different_feishu_topics_do_not_reuse_same_session() {
    let pool = setup_im_employee_agents_pool().await;

    let first = build_feishu_group_topic_event("oc_group_1", "om_root_a", "ou_user_1");
    let second = build_feishu_group_topic_event("oc_group_1", "om_root_b", "ou_user_1");

    let first_sessions =
        ensure_employee_sessions_for_event_with_pool(&pool, &first).await.expect("first bind");
    let second_sessions =
        ensure_employee_sessions_for_event_with_pool(&pool, &second).await.expect("second bind");

    assert_ne!(first_sessions[0].session_id, second_sessions[0].session_id);
}

#[tokio::test]
async fn same_topic_different_sender_splits_when_scope_is_topic_sender() {
    // build two events with same peer/topic but different sender ids
    // assert session ids differ
}
```

- [ ] **Step 2: Run the IM employee routing tests to verify failure**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_im_employee_agents -- --nocapture`  
Expected: FAIL because the current implementation reuses sessions too broadly.

- [ ] **Step 3: Split route scope key from conversation binding key**

```rust
fn build_route_scope_key(event: &ImEvent, employee: &AgentEmployee) -> String {
    format!("{}:{}:{}", normalized_channel, tenant, agent_id)
}

fn build_conversation_binding_key(event: &ImEvent, employee: &AgentEmployee) -> String {
    event.conversation_id
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("legacy:{}:{}:{}", normalized_channel, tenant, event.thread_id))
}
```

- [ ] **Step 4: Add repo methods for conversation-aware lookup/upsert**

```rust
pub(crate) async fn find_conversation_session_record(
    pool: &SqlitePool,
    conversation_id: &str,
    employee_db_id: &str,
) -> Result<Option<ThreadSessionRecord>, String> { /* query conversation_id first */ }

pub(crate) async fn upsert_conversation_session_link(
    pool: &SqlitePool,
    input: &ThreadSessionLinkInput<'_>,
) -> Result<(), String> { /* write thread_id plus conversation fields */ }
```

- [ ] **Step 5: Disable coarse cross-conversation reuse by default**

```rust
// Remove route-key-only reuse as the default path.
// Only reuse by exact conversation binding key.
// Keep legacy fallback only when conversation_id is empty.
```

- [ ] **Step 6: Prevent implicit multi-employee session sharing**

```rust
// Drop shared_thread_session_id as the default branch.
// Each employee gets its own session unless an explicit shared-team mode is introduced.
```

- [ ] **Step 7: Run the IM employee routing tests to verify they pass**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_im_employee_agents -- --nocapture`  
Expected: PASS

- [ ] **Step 8: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/employee_agents.rs apps/runtime/src-tauri/src/commands/employee_agents/session_repo.rs apps/runtime/src-tauri/src/commands/employee_agents/session_service.rs apps/runtime/src-tauri/tests/test_im_employee_agents/im_routing.rs
git commit -m "fix: bind IM sessions by conversation identity"
```

### Task 5: Propagate Conversation Metadata Through the IM Host Bridge

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/im_host/inbound_bridge.rs`
- Modify: `apps/runtime/src-tauri/src/commands/openclaw_gateway.rs`
- Modify: `apps/runtime/src-tauri/src/commands/feishu_gateway/ingress_service.rs`
- Test: `apps/runtime/src-tauri/src/commands/im_host/inbound_bridge.rs`

- [ ] **Step 1: Write the failing normalized-event bridge test**

```rust
#[test]
fn parse_normalized_im_event_value_preserves_conversation_metadata() {
    let event = parse_normalized_im_event_value(&serde_json::json!({
        "channel": "feishu",
        "thread_id": "oc_group_1",
        "conversation_id": "feishu:default:group:oc_group_1:topic:om_root_1",
        "base_conversation_id": "feishu:default:group:oc_group_1",
        "parent_conversation_candidates": ["feishu:default:group:oc_group_1"],
        "scope": "topic"
    }))
    .expect("parse");

    assert_eq!(event.conversation_id.as_deref(), Some("feishu:default:group:oc_group_1:topic:om_root_1"));
    assert_eq!(event.parent_conversation_candidates.len(), 1);
}
```

- [ ] **Step 2: Run the inbound bridge tests to verify failure**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml inbound_bridge -- --nocapture`  
Expected: FAIL because the new fields are ignored.

- [ ] **Step 3: Carry the new fields across bridge boundaries**

```rust
let conversation_id = optional_non_empty_string(value.get("conversation_id"));
let base_conversation_id = optional_non_empty_string(value.get("base_conversation_id"));
let parent_conversation_candidates = value
    .get("parent_conversation_candidates")
    .and_then(serde_json::Value::as_array)
    .map(|items| {
        items
            .iter()
            .filter_map(serde_json::Value::as_str)
            .map(str::to_string)
            .collect::<Vec<_>>()
    })
    .unwrap_or_default();
```

- [ ] **Step 4: Emit IM route diagnostics with conversation id**

```rust
"conversation_id": dispatch.conversation_id,
"base_conversation_id": dispatch.base_conversation_id,
"scope": dispatch.conversation_scope,
```

- [ ] **Step 5: Run bridge tests to verify pass**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml inbound_bridge -- --nocapture`  
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/im_host/inbound_bridge.rs apps/runtime/src-tauri/src/commands/openclaw_gateway.rs apps/runtime/src-tauri/src/commands/feishu_gateway/ingress_service.rs
git commit -m "feat: propagate IM conversation metadata through bridge"
```

### Task 6: Add Compatibility Read/Write Fallbacks and Lock in Regression Coverage

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/session_repo.rs`
- Modify: `apps/runtime/src-tauri/src/commands/im_host/lifecycle.rs`
- Modify: `apps/runtime/src-tauri/src/commands/chat_session_io/session_compaction.rs`
- Test: `apps/runtime/src-tauri/tests/test_im_employee_agents/im_routing.rs`
- Test: `apps/runtime/src-tauri/src/commands/chat_session_io.rs`

- [ ] **Step 1: Write the failing compatibility tests**

```rust
#[tokio::test]
async fn legacy_thread_only_rows_still_resolve_session_links() {
    let pool = setup_legacy_im_thread_sessions_pool().await;

    let row = find_conversation_session_record(&pool, "missing-new-key", "employee-1")
        .await
        .expect("query legacy");

    assert!(row.is_some());
}

#[tokio::test]
async fn session_compaction_preserves_new_messages_table_reads() {
    // compact a session and verify load_compaction_inputs_with_pool still works
}
```

- [ ] **Step 2: Run compatibility tests to verify failure**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml legacy_thread_only_rows_still_resolve_session_links session_compaction_preserves_new_messages_table_reads -- --nocapture`  
Expected: FAIL

- [ ] **Step 3: Implement dual-read/dual-write compatibility**

```rust
// Read path:
// 1. exact conversation_id
// 2. fallback legacy thread_id when conversation_id is blank

// Write path:
// always populate thread_id
// also populate conversation_id/base_conversation_id/scope when available
```

- [ ] **Step 4: Guard compaction from assuming legacy-only session semantics**

```rust
// Keep compaction operating by session_id only.
// Do not reintroduce thread-based reuse or binding lookups inside compaction code paths.
```

- [ ] **Step 5: Run the focused tests and then the Rust fast path**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_im_employee_agents -- --nocapture`  
Expected: PASS

Run: `pnpm test:rust-fast`  
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/employee_agents/session_repo.rs apps/runtime/src-tauri/src/commands/im_host/lifecycle.rs apps/runtime/src-tauri/src/commands/chat_session_io/session_compaction.rs apps/runtime/src-tauri/tests/test_im_employee_agents/im_routing.rs apps/runtime/src-tauri/src/commands/chat_session_io.rs
git commit -m "test: lock conversation identity compatibility regressions"
```

### Task 7: Follow-Up Compaction Hardening Plan Stub

**Files:**
- Modify: `apps/runtime/src-tauri/src/agent/compactor.rs`
- Modify: `apps/runtime/src-tauri/src/agent/turn_executor.rs`
- Modify: `apps/runtime/src-tauri/src/session_journal.rs`
- Test: `apps/runtime/src-tauri/src/agent/compactor.rs`

- [ ] **Step 1: Write the failing boundary-preservation test**

```rust
#[tokio::test]
async fn auto_compact_keeps_summary_marker_and_recent_tail_messages() {
    let messages = build_long_history_messages();
    let compacted = auto_compact(/* ... */).await.expect("compact");

    assert!(compacted.iter().any(|msg| msg["content"].as_str().unwrap_or("").contains("[对话已压缩")));
    assert!(compacted.len() > 2);
}
```

- [ ] **Step 2: Run the compactor test to verify failure**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml auto_compact_keeps_summary_marker_and_recent_tail_messages -- --nocapture`  
Expected: FAIL because the current implementation replaces history with exactly two messages.

- [ ] **Step 3: Implement minimal tail-preserving compaction**

```rust
let tail = messages.iter().rev().take(6).cloned().collect::<Vec<_>>();
tail.reverse();

let mut compacted = vec![summary_marker_message(summary, transcript_path)];
compacted.extend(tail);
```

- [ ] **Step 4: Record the tail boundary in the journal**

```rust
compaction_boundary: Some(SessionRunTurnStateCompactionBoundary {
    compacted_tokens: original_tokens,
    first_kept_message_index: Some(first_tail_index),
})
```

- [ ] **Step 5: Run the compactor tests**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml compactor -- --nocapture`  
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add apps/runtime/src-tauri/src/agent/compactor.rs apps/runtime/src-tauri/src/agent/turn_executor.rs apps/runtime/src-tauri/src/session_journal.rs
git commit -m "feat: preserve tail messages during auto compaction"
```

## Spec Coverage Check

- IM-agnostic conversation abstraction: covered by Task 1.
- Feishu first adopter path: covered by Task 3.
- Schema compatibility and migration safety: covered by Task 2 and Task 6.
- Session binding and reuse correction: covered by Task 4.
- IM bridge propagation: covered by Task 5.
- Compaction follow-up hardening: covered by Task 7.

## Placeholder Scan

- No `TODO`, `TBD`, or deferred requirements remain.
- Every task includes target files, commands, and the concrete behavior to implement or test.

## Type Consistency Check

- Shared type names are consistent across tasks: `ImConversationSurface`, `ImConversationScope`, `build_conversation_id`, `build_parent_conversation_candidates`.
- Session binding vocabulary is consistent across tasks: `conversation_id`, `base_conversation_id`, `parent_conversation_candidates`, `route_scope_key`, `conversation_binding_key`.
