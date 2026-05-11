mod helpers;

use runtime_lib::commands::im_routing::{
    upsert_im_routing_binding_with_pool, UpsertImRoutingBindingInput,
};
use runtime_lib::commands::openclaw_gateway::resolve_openclaw_route_with_pool;
use runtime_lib::im::types::{ImEvent, ImEventType};

#[tokio::test]
async fn resolve_route_uses_event_channel_instead_of_feishu_default() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    upsert_im_routing_binding_with_pool(
        &pool,
        UpsertImRoutingBindingInput {
            id: None,
            agent_id: "discord-agent".to_string(),
            channel: "discord".to_string(),
            account_id: "*".to_string(),
            peer_kind: "".to_string(),
            peer_id: "".to_string(),
            guild_id: "".to_string(),
            team_id: "".to_string(),
            role_ids: vec![],
            connector_meta: serde_json::json!({}),
            priority: 100,
            enabled: true,
        },
    )
    .await
    .expect("seed discord binding");

    let out = resolve_openclaw_route_with_pool(
        &pool,
        &ImEvent {
            channel: "discord".to_string(),
            event_type: ImEventType::MessageCreated,
            thread_id: "discord-room-1".to_string(),
            event_id: Some("evt-discord".to_string()),
            message_id: Some("msg-discord".to_string()),
            text: Some("hello".to_string()),
            role_id: None,
            account_id: Some("tenant-discord".to_string()),
            tenant_id: Some("tenant-discord".to_string()),
            sender_id: None,
            chat_type: None,
            conversation_id: None,
            base_conversation_id: None,
            parent_conversation_candidates: Vec::new(),
            conversation_scope: None,
        },
    )
    .await
    .expect("resolve route");

    assert_eq!(out["agentId"], "discord-agent");
    assert_eq!(out["matchedBy"], "binding.channel");
}

#[tokio::test]
async fn resolve_route_supports_wecom_channel_via_native_resolver() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    upsert_im_routing_binding_with_pool(
        &pool,
        UpsertImRoutingBindingInput {
            id: None,
            agent_id: "wecom-agent".to_string(),
            channel: "wecom".to_string(),
            account_id: "agent-1000002".to_string(),
            peer_kind: "".to_string(),
            peer_id: "".to_string(),
            guild_id: "".to_string(),
            team_id: "".to_string(),
            role_ids: vec![],
            connector_meta: serde_json::json!({
                "connector_id": "wecom-main",
                "workspace_id": "corp-123"
            }),
            priority: 100,
            enabled: true,
        },
    )
    .await
    .expect("seed wecom binding");

    let out = resolve_openclaw_route_with_pool(
        &pool,
        &ImEvent {
            channel: "wecom".to_string(),
            event_type: ImEventType::MessageCreated,
            thread_id: "wecom-room-1".to_string(),
            event_id: Some("evt-wecom".to_string()),
            message_id: Some("msg-wecom".to_string()),
            text: Some("请处理企业微信消息".to_string()),
            role_id: None,
            account_id: Some("agent-1000002".to_string()),
            tenant_id: Some("corp-123".to_string()),
            sender_id: None,
            chat_type: None,
            conversation_id: None,
            base_conversation_id: None,
            parent_conversation_candidates: Vec::new(),
            conversation_scope: None,
        },
    )
    .await
    .expect("resolve wecom route");

    assert_eq!(out["agentId"], "wecom-agent");
    assert_eq!(out["matchedBy"], "binding.account");
}

#[tokio::test]
async fn resolve_route_preserves_group_channel_peer_kind_alias() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    upsert_im_routing_binding_with_pool(
        &pool,
        UpsertImRoutingBindingInput {
            id: None,
            agent_id: "channel-peer-agent".to_string(),
            channel: "discord".to_string(),
            account_id: "tenant-a".to_string(),
            peer_kind: "channel".to_string(),
            peer_id: "room-1".to_string(),
            guild_id: "".to_string(),
            team_id: "".to_string(),
            role_ids: vec![],
            connector_meta: serde_json::json!({}),
            priority: 100,
            enabled: true,
        },
    )
    .await
    .expect("seed channel peer binding");

    let out = resolve_openclaw_route_with_pool(
        &pool,
        &ImEvent {
            channel: "discord".to_string(),
            event_type: ImEventType::MessageCreated,
            thread_id: "room-1".to_string(),
            event_id: Some("evt-peer-alias".to_string()),
            message_id: Some("msg-peer-alias".to_string()),
            text: Some("hello".to_string()),
            role_id: None,
            account_id: Some("tenant-a".to_string()),
            tenant_id: Some("tenant-a".to_string()),
            sender_id: None,
            chat_type: None,
            conversation_id: None,
            base_conversation_id: None,
            parent_conversation_candidates: Vec::new(),
            conversation_scope: None,
        },
    )
    .await
    .expect("resolve peer alias route");

    assert_eq!(out["agentId"], "channel-peer-agent");
    assert_eq!(out["matchedBy"], "binding.peer");
}

#[tokio::test]
async fn resolve_route_normalizes_account_ids_like_openclaw() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    upsert_im_routing_binding_with_pool(
        &pool,
        UpsertImRoutingBindingInput {
            id: None,
            agent_id: "normalized-account-agent".to_string(),
            channel: "wecom".to_string(),
            account_id: "AGENT 1000002".to_string(),
            peer_kind: "".to_string(),
            peer_id: "".to_string(),
            guild_id: "".to_string(),
            team_id: "".to_string(),
            role_ids: vec![],
            connector_meta: serde_json::json!({}),
            priority: 100,
            enabled: true,
        },
    )
    .await
    .expect("seed normalized account binding");

    let out = resolve_openclaw_route_with_pool(
        &pool,
        &ImEvent {
            channel: "wecom".to_string(),
            event_type: ImEventType::MessageCreated,
            thread_id: "wecom-room-1".to_string(),
            event_id: Some("evt-account-normalized".to_string()),
            message_id: Some("msg-account-normalized".to_string()),
            text: Some("hello".to_string()),
            role_id: None,
            account_id: Some("agent-1000002".to_string()),
            tenant_id: Some("corp-123".to_string()),
            sender_id: None,
            chat_type: None,
            conversation_id: None,
            base_conversation_id: None,
            parent_conversation_candidates: Vec::new(),
            conversation_scope: None,
        },
    )
    .await
    .expect("resolve normalized account route");

    assert_eq!(out["agentId"], "normalized-account-agent");
    assert_eq!(out["matchedBy"], "binding.account");
}

#[tokio::test]
async fn resolve_route_does_not_treat_guild_binding_as_generic_channel_binding() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    upsert_im_routing_binding_with_pool(
        &pool,
        UpsertImRoutingBindingInput {
            id: None,
            agent_id: "guild-only-agent".to_string(),
            channel: "discord".to_string(),
            account_id: "*".to_string(),
            peer_kind: "".to_string(),
            peer_id: "".to_string(),
            guild_id: "guild-1".to_string(),
            team_id: "".to_string(),
            role_ids: vec![],
            connector_meta: serde_json::json!({}),
            priority: 100,
            enabled: true,
        },
    )
    .await
    .expect("seed guild binding");
    upsert_im_routing_binding_with_pool(
        &pool,
        UpsertImRoutingBindingInput {
            id: None,
            agent_id: "generic-channel-agent".to_string(),
            channel: "discord".to_string(),
            account_id: "*".to_string(),
            peer_kind: "".to_string(),
            peer_id: "".to_string(),
            guild_id: "".to_string(),
            team_id: "".to_string(),
            role_ids: vec![],
            connector_meta: serde_json::json!({}),
            priority: 200,
            enabled: true,
        },
    )
    .await
    .expect("seed generic channel binding");

    let out = resolve_openclaw_route_with_pool(
        &pool,
        &ImEvent {
            channel: "discord".to_string(),
            event_type: ImEventType::MessageCreated,
            thread_id: "discord-room-without-guild".to_string(),
            event_id: Some("evt-guild-no-fallthrough".to_string()),
            message_id: Some("msg-guild-no-fallthrough".to_string()),
            text: Some("hello".to_string()),
            role_id: None,
            account_id: Some("tenant-a".to_string()),
            tenant_id: Some("tenant-a".to_string()),
            sender_id: None,
            chat_type: None,
            conversation_id: None,
            base_conversation_id: None,
            parent_conversation_candidates: Vec::new(),
            conversation_scope: None,
        },
    )
    .await
    .expect("resolve route without guild scope");

    assert_eq!(out["agentId"], "generic-channel-agent");
    assert_eq!(out["matchedBy"], "binding.channel");
}

#[tokio::test]
async fn route_regression_vectors_match_expected_priority() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    let vectors = vec![
        (
            "peer",
            "chat-peer",
            "tenant-a",
            "peer-agent",
            "binding.peer",
        ),
        (
            "account",
            "chat-account",
            "tenant-a",
            "account-agent",
            "binding.account",
        ),
        (
            "channel",
            "chat-channel",
            "tenant-b",
            "channel-agent",
            "binding.channel",
        ),
        ("default", "chat-default", "tenant-c", "main", "default"),
    ];
    for (name, thread_id, tenant_id, expected_agent, expected_matched_by) in vectors {
        sqlx::query("DELETE FROM im_routing_bindings")
            .execute(&pool)
            .await
            .expect("clear bindings");

        if name != "default" {
            upsert_im_routing_binding_with_pool(
                &pool,
                UpsertImRoutingBindingInput {
                    id: None,
                    agent_id: "channel-agent".to_string(),
                    channel: "feishu".to_string(),
                    account_id: "*".to_string(),
                    peer_kind: "".to_string(),
                    peer_id: "".to_string(),
                    guild_id: "".to_string(),
                    team_id: "".to_string(),
                    role_ids: vec![],
                    connector_meta: serde_json::json!({}),
                    priority: 300,
                    enabled: true,
                },
            )
            .await
            .expect("seed channel binding");
        }

        if name == "peer" || name == "account" {
            upsert_im_routing_binding_with_pool(
                &pool,
                UpsertImRoutingBindingInput {
                    id: None,
                    agent_id: "account-agent".to_string(),
                    channel: "feishu".to_string(),
                    account_id: "tenant-a".to_string(),
                    peer_kind: "".to_string(),
                    peer_id: "".to_string(),
                    guild_id: "".to_string(),
                    team_id: "".to_string(),
                    role_ids: vec![],
                    connector_meta: serde_json::json!({}),
                    priority: 200,
                    enabled: true,
                },
            )
            .await
            .expect("seed account binding");
        }

        if name == "peer" {
            upsert_im_routing_binding_with_pool(
                &pool,
                UpsertImRoutingBindingInput {
                    id: None,
                    agent_id: "peer-agent".to_string(),
                    channel: "feishu".to_string(),
                    account_id: "tenant-a".to_string(),
                    peer_kind: "group".to_string(),
                    peer_id: "chat-peer".to_string(),
                    guild_id: "".to_string(),
                    team_id: "".to_string(),
                    role_ids: vec![],
                    connector_meta: serde_json::json!({}),
                    priority: 100,
                    enabled: true,
                },
            )
            .await
            .expect("seed peer binding");
        }

        let out = resolve_openclaw_route_with_pool(
            &pool,
            &ImEvent {
                channel: "feishu".to_string(),
                event_type: ImEventType::MessageCreated,
                thread_id: thread_id.to_string(),
                event_id: Some(format!("evt-{}", name)),
                message_id: Some(format!("msg-{}", name)),
                text: Some("hello".to_string()),
                role_id: None,
                account_id: Some(tenant_id.to_string()),
                tenant_id: Some(tenant_id.to_string()),
                sender_id: None,
                chat_type: None,
                conversation_id: None,
                base_conversation_id: None,
                parent_conversation_candidates: Vec::new(),
                conversation_scope: None,
            },
        )
        .await
        .expect("resolve route");

        assert_eq!(out["agentId"], expected_agent, "vector={}", name);
        assert_eq!(out["matchedBy"], expected_matched_by, "vector={}", name);
    }
}
