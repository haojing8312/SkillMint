# IM Conversation Identity Cutover Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Finish the WorkClaw cutover from thread-first IM session reuse to conversation-first binding, while keeping backward-compatible reads and stable reply delivery.

**Architecture:** Keep sidecar adapters transport-only, derive canonical conversation metadata in the IM bridge, persist new binding/projection tables as the primary authority, and demote `im_thread_sessions` to a migration/fallback surface. The implementation should prefer finishing the in-flight modules already present in `apps/runtime/src-tauri/src/im/*`, `im_host/*`, and `employee_agents/*` instead of inventing a parallel path.

**Tech Stack:** Rust, Tauri runtime, SQLite via `sqlx`, WorkClaw IM bridge, Rust integration tests

---

### Task 1: Finish the shared IM conversation identity core

**Files:**
- Modify: `apps/runtime/src-tauri/src/im/conversation_surface.rs`
- Modify: `apps/runtime/src-tauri/src/im/conversation_id.rs`
- Modify: `apps/runtime/src-tauri/src/im/mod.rs`
- Modify: `apps/runtime/src-tauri/src/im/types.rs`
- Test: `apps/runtime/src-tauri/tests/test_normalized_im_conversation_identity.rs`
- Test: `apps/runtime/src-tauri/tests/test_feishu_conversation_identity.rs`

- [ ] **Step 1: Write or extend the failing identity tests**

```rust
#[test]
fn builds_peer_conversation_id_for_feishu_group_chat() {
    let surface = ImConversationSurface {
        channel: "feishu".to_string(),
        account_id: "default".to_string(),
        tenant_id: Some("tenant-a".to_string()),
        peer_kind: ImPeerKind::Group,
        peer_id: "chat-1".to_string(),
        topic_id: None,
        sender_id: None,
        scope: ImConversationScope::Peer,
        message_id: Some("msg-1".to_string()),
        raw_thread_id: Some("chat-1".to_string()),
        raw_root_id: None,
    };

    assert_eq!(
        build_conversation_id(&surface),
        "feishu:default:group:chat-1"
    );
    assert!(build_parent_conversation_candidates(&surface).is_empty());
}

#[test]
fn builds_topic_conversation_and_parent_candidate() {
    let surface = ImConversationSurface {
        channel: "wecom".to_string(),
        account_id: "agent-1".to_string(),
        tenant_id: Some("corp-1".to_string()),
        peer_kind: ImPeerKind::Group,
        peer_id: "room-1".to_string(),
        topic_id: Some("topic-42".to_string()),
        sender_id: Some("user-1".to_string()),
        scope: ImConversationScope::Topic,
        message_id: Some("msg-2".to_string()),
        raw_thread_id: Some("room-1".to_string()),
        raw_root_id: Some("topic-42".to_string()),
    };

    assert_eq!(
        build_conversation_id(&surface),
        "wecom:agent-1:group:room-1:topic:topic-42"
    );
    assert_eq!(
        build_parent_conversation_candidates(&surface),
        vec!["wecom:agent-1:group:room-1".to_string()]
    );
}
```

- [ ] **Step 2: Run the identity-focused tests and verify failures**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_normalized_im_conversation_identity test_feishu_conversation_identity -- --nocapture`  
Expected: FAIL if the builders, enum labels, or parent candidate logic are incomplete or inconsistent.

- [ ] **Step 3: Normalize the shared surface and builders**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImPeerKind {
    Direct,
    Group,
}

impl ImPeerKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Group => "group",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImConversationScope {
    Peer,
    PeerSender,
    Topic,
    TopicSender,
}

impl ImConversationScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Peer => "peer",
            Self::PeerSender => "peer_sender",
            Self::Topic => "topic",
            Self::TopicSender => "topic_sender",
        }
    }
}
```

- [ ] **Step 4: Export the stabilized API from `im/mod.rs`**

```rust
pub mod conversation_binding_store;
pub mod conversation_id;
pub mod conversation_surface;

pub use conversation_binding_store::{
    find_agent_conversation_binding,
    find_agent_conversation_binding_for_candidates,
    find_channel_delivery_route,
    upsert_agent_conversation_binding,
    upsert_channel_delivery_route,
    AgentConversationBindingUpsert,
    ChannelDeliveryRouteUpsert,
};
pub use conversation_id::{build_conversation_id, build_parent_conversation_candidates};
pub use conversation_surface::{ImConversationScope, ImConversationSurface, ImPeerKind};
```

- [ ] **Step 5: Re-run the identity tests and verify they pass**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_normalized_im_conversation_identity test_feishu_conversation_identity -- --nocapture`  
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add apps/runtime/src-tauri/src/im/conversation_surface.rs apps/runtime/src-tauri/src/im/conversation_id.rs apps/runtime/src-tauri/src/im/mod.rs apps/runtime/src-tauri/src/im/types.rs apps/runtime/src-tauri/tests/test_normalized_im_conversation_identity.rs apps/runtime/src-tauri/tests/test_feishu_conversation_identity.rs
git commit -m "feat: stabilize IM conversation identity core"
```

### Task 2: Complete the database cutover and compatibility reads

**Files:**
- Modify: `apps/runtime/src-tauri/src/db/schema.rs`
- Modify: `apps/runtime/src-tauri/src/db/migrations.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/session_repo.rs`
- Modify: `apps/runtime/src-tauri/src/im/conversation_binding_store.rs`
- Test: `apps/runtime/src-tauri/tests/helpers/mod.rs`
- Test: `apps/runtime/src-tauri/tests/test_im_route_session_mapping.rs`

- [ ] **Step 1: Add a failing legacy-schema migration regression**

```rust
#[tokio::test]
async fn legacy_thread_only_db_gains_conversation_binding_tables() {
    let (pool, _tmp) = setup_legacy_thread_only_db().await;

    runtime_lib::db::migrations::apply_legacy_migrations_for_test(&pool)
        .await
        .expect("apply migrations");

    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master
         WHERE type = 'table'
         AND name IN ('agent_conversation_bindings', 'channel_delivery_routes', 'im_conversation_sessions')",
    )
    .fetch_all(&pool)
    .await
    .expect("list migrated tables");

    assert_eq!(tables.len(), 3);
}
```

- [ ] **Step 2: Run the migration and route-mapping tests to verify failure**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_im_route_session_mapping legacy_thread_only_db_gains_conversation_binding_tables -- --nocapture`  
Expected: FAIL if migrations, backfill, or new-table reads are incomplete.

- [ ] **Step 3: Make the new binding tables authoritative in migration code**

```rust
let _ = sqlx::query(
    "CREATE TABLE IF NOT EXISTS agent_conversation_bindings (
        conversation_id TEXT NOT NULL,
        channel TEXT NOT NULL,
        account_id TEXT NOT NULL DEFAULT '',
        agent_id TEXT NOT NULL,
        session_key TEXT NOT NULL,
        session_id TEXT NOT NULL DEFAULT '',
        base_conversation_id TEXT NOT NULL DEFAULT '',
        parent_conversation_candidates_json TEXT NOT NULL DEFAULT '[]',
        scope TEXT NOT NULL DEFAULT '',
        peer_kind TEXT NOT NULL DEFAULT '',
        peer_id TEXT NOT NULL DEFAULT '',
        topic_id TEXT NOT NULL DEFAULT '',
        sender_id TEXT NOT NULL DEFAULT '',
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        PRIMARY KEY (conversation_id, agent_id)
    )"
)
.execute(pool)
.await?;
```

- [ ] **Step 4: Enforce lookup precedence in repository code**

```rust
pub(crate) async fn find_existing_session_for_conversation(
    pool: &SqlitePool,
    conversation_id: &str,
    employee_db_id: &str,
) -> Result<Option<String>, String> {
    if let Some(record) = find_conversation_session_record(pool, conversation_id, employee_db_id).await? {
        if record.session_exists {
            return Ok(Some(record.session_id));
        }
    }

    Ok(None)
}
```

- [ ] **Step 5: Re-run the migration and session mapping tests**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_im_route_session_mapping -- --nocapture`  
Expected: PASS, including different-thread isolation and legacy fallback behavior.

- [ ] **Step 6: Commit**

```bash
git add apps/runtime/src-tauri/src/db/schema.rs apps/runtime/src-tauri/src/db/migrations.rs apps/runtime/src-tauri/src/commands/employee_agents/session_repo.rs apps/runtime/src-tauri/src/im/conversation_binding_store.rs apps/runtime/src-tauri/tests/helpers/mod.rs apps/runtime/src-tauri/tests/test_im_route_session_mapping.rs
git commit -m "feat: cut over IM binding storage to conversation-first reads"
```

### Task 3: Finish IM host and Feishu mapper adoption

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/im_host/inbound_bridge.rs`
- Modify: `apps/runtime/src-tauri/src/commands/im_host/lifecycle.rs`
- Modify: `apps/runtime/src-tauri/src/commands/im_host/interactive_dispatch.rs`
- Modify: `apps/runtime/src-tauri/src/commands/feishu_gateway/conversation_mapper.rs`
- Modify: `apps/runtime/src-tauri/src/commands/feishu_gateway/payload_parser.rs`
- Modify: `apps/runtime/src-tauri/src/commands/feishu_gateway/tests.rs`
- Test: `apps/runtime/src-tauri/tests/test_im_host_windows_regressions.rs`

- [ ] **Step 1: Add a failing bridge-level regression for derived conversation metadata**

```rust
#[test]
fn parse_normalized_im_event_derives_topic_conversation_metadata() {
    let event = parse_normalized_im_event_value(&serde_json::json!({
        "channel": "feishu",
        "thread_id": "chat-1",
        "message_id": "msg-1",
        "workspace_id": "tenant-a",
        "topic_id": "topic-42",
        "chat_type": "group"
    }))
    .expect("parse normalized event");

    assert_eq!(
        event.conversation_id.as_deref(),
        Some("feishu:tenant-a:group:chat-1:topic:topic-42")
    );
    assert_eq!(
        event.base_conversation_id.as_deref(),
        Some("feishu:tenant-a:group:chat-1")
    );
}
```

- [ ] **Step 2: Run the bridge and Windows regression tests to verify failure**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_im_host_windows_regressions feishu_gateway -- --nocapture`  
Expected: FAIL if Feishu payload parsing or IM bridge derivation still drops scope metadata.

- [ ] **Step 3: Make Feishu and normalized IM inputs emit the same metadata contract**

```rust
let (conversation_id, base_conversation_id, parent_conversation_candidates, conversation_scope) =
    build_normalized_event_conversation_metadata(
        value,
        &channel,
        &thread_id,
        account_id.as_deref(),
        tenant_id.as_deref(),
        sender_id.as_deref(),
        chat_type.as_deref(),
        message_id.as_deref(),
    );
```

- [ ] **Step 4: Persist the bridge projection on dispatch**

```rust
record_openclaw_binding_projection_with_pool(pool, event, &dispatches).await?;
emit_inbound_dispatch_sessions(app, &event.channel, &dispatches);
```

- [ ] **Step 5: Re-run bridge and Feishu tests**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_im_host_windows_regressions test_feishu_conversation_identity feishu_gateway -- --nocapture`  
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/im_host/inbound_bridge.rs apps/runtime/src-tauri/src/commands/im_host/lifecycle.rs apps/runtime/src-tauri/src/commands/im_host/interactive_dispatch.rs apps/runtime/src-tauri/src/commands/feishu_gateway/conversation_mapper.rs apps/runtime/src-tauri/src/commands/feishu_gateway/payload_parser.rs apps/runtime/src-tauri/src/commands/feishu_gateway/tests.rs apps/runtime/src-tauri/tests/test_im_host_windows_regressions.rs
git commit -m "feat: derive and persist conversation-aware IM bridge metadata"
```

### Task 4: Remove coarse route-key reuse from the employee agent authority path

**Files:**
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/service.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/session_service.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/types.rs`
- Modify: `apps/runtime/src-tauri/src/commands/employee_agents/profile_service.rs`
- Test: `apps/runtime/src-tauri/tests/test_im_employee_agents/im_routing.rs`
- Test: `apps/runtime/src-tauri/tests/test_im_route_session_mapping.rs`

- [ ] **Step 1: Add a failing regression that forbids route-key-only reuse**

```rust
#[tokio::test]
async fn conversation_lookup_does_not_fall_back_to_route_key_only_reuse() {
    let (pool, _tmp) = helpers::setup_test_db().await;

    // Seed a session for chat-1, then send chat-2 with the same coarse route key.
    // The second dispatch must create a different session.

    assert_ne!(second[0].session_id, first[0].session_id);
}
```

- [ ] **Step 2: Run employee-agent and route-mapping tests to verify failure**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_im_employee_agents test_im_route_session_mapping -- --nocapture`  
Expected: FAIL if any code path still reuses sessions by coarse route key.

- [ ] **Step 3: Replace route-key authority with conversation-aware dispatch resolution**

```rust
let dispatches = resolve_agent_session_dispatches_with_pool(pool, event, route_decision.as_ref()).await?;

if dispatches.is_empty() {
    return Ok(Vec::new());
}
```

- [ ] **Step 4: Keep compatibility entrypoints but demote them to wrappers**

```rust
pub async fn ensure_employee_sessions_for_event_with_pool(
    pool: &SqlitePool,
    event: &ImEvent,
) -> Result<Vec<EmployeeInboundDispatchSession>, String> {
    let dispatches = resolve_agent_session_dispatches_for_event_with_pool(pool, event).await?;
    Ok(dispatches.into_iter().map(EmployeeInboundDispatchSession::from).collect())
}
```

- [ ] **Step 5: Re-run the employee-agent regression set**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_im_employee_agents test_im_route_session_mapping -- --nocapture`  
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add apps/runtime/src-tauri/src/commands/employee_agents.rs apps/runtime/src-tauri/src/commands/employee_agents/service.rs apps/runtime/src-tauri/src/commands/employee_agents/session_service.rs apps/runtime/src-tauri/src/commands/employee_agents/types.rs apps/runtime/src-tauri/src/commands/employee_agents/profile_service.rs apps/runtime/src-tauri/tests/test_im_employee_agents/im_routing.rs apps/runtime/src-tauri/tests/test_im_route_session_mapping.rs
git commit -m "refactor: demote route-key reuse in employee agent session authority"
```

### Task 5: Run cutover verification and document the deferred compaction boundary

**Files:**
- Modify: `docs/architecture/openclaw-im-reuse.md`
- Modify: `docs/superpowers/specs/2026-04-23-im-conversation-identity-cutover-design.md`
- Test: `apps/runtime/src-tauri/src/bin/employee_im_heavy_regression.rs`

- [ ] **Step 1: Add the final verification checklist to the architecture doc**

```md
## Verification Gate

- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_normalized_im_conversation_identity test_feishu_conversation_identity -- --nocapture`
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_im_route_session_mapping test_im_employee_agents -- --nocapture`
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_im_host_windows_regressions -- --nocapture`
```

- [ ] **Step 2: Add an explicit deferred note for compaction work**

```md
## Deferred

This cutover does not modify `agent/compactor.rs` or the long-context retention model.
Compaction redesign must be handled in a separate spec after conversation/session binding is stable.
```

- [ ] **Step 3: Run the full targeted verification set**

Run: `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml test_normalized_im_conversation_identity test_feishu_conversation_identity test_im_route_session_mapping test_im_employee_agents test_im_host_windows_regressions -- --nocapture`  
Expected: PASS

- [ ] **Step 4: Run the heavy regression binary in dry local mode if available**

Run: `cargo run --manifest-path apps/runtime/src-tauri/Cargo.toml --bin employee_im_heavy_regression`  
Expected: completes without reintroducing coarse session reuse; if the binary requires local setup that is unavailable, capture that explicitly in the final notes.

- [ ] **Step 5: Record the exact verification evidence in the handoff**

```md
- Verified new-table-first lookup
- Verified different-thread isolation
- Verified different-agent isolation inside the same conversation
- Verified legacy-schema migration fallback
- Not verified: compaction redesign, by design
```

- [ ] **Step 6: Commit**

```bash
git add docs/architecture/openclaw-im-reuse.md docs/superpowers/specs/2026-04-23-im-conversation-identity-cutover-design.md
git commit -m "docs: finalize IM conversation identity cutover verification notes"
```
