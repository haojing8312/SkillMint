use super::contract::{ImReplyDeliveryPlan, ImReplyLifecyclePhase};
use crate::im::find_channel_delivery_route_by_session_id;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SessionChannelDeliveryRoute {
    pub channel: String,
    pub account_id: String,
    pub thread_id: String,
    pub conversation_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SessionLifecycleDispatch {
    pub request_id: String,
    pub account_id: String,
    pub logical_reply_id: Option<String>,
    pub phase: ImReplyLifecyclePhase,
    pub thread_id: String,
    pub message_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SessionProcessingStopDispatch {
    pub request_id: String,
    pub account_id: String,
    pub logical_reply_id: Option<String>,
    pub final_state: Option<String>,
    pub thread_id: String,
    pub message_id: String,
}

pub(crate) fn build_session_lifecycle_dispatch(
    account_id: Option<&str>,
    logical_reply_id: Option<&str>,
    phase: ImReplyLifecyclePhase,
    thread_id: &str,
    message_id: Option<&str>,
) -> Result<SessionLifecycleDispatch, String> {
    let normalized_thread_id = thread_id.trim();
    if normalized_thread_id.is_empty() {
        return Err("thread_id is required".to_string());
    }

    Ok(SessionLifecycleDispatch {
        request_id: format!("lifecycle-{}", Uuid::new_v4()),
        account_id: account_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("default")
            .to_string(),
        logical_reply_id: logical_reply_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        phase,
        thread_id: normalized_thread_id.to_string(),
        message_id: message_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
    })
}

pub(crate) fn build_session_processing_stop_dispatch(
    account_id: Option<&str>,
    logical_reply_id: Option<&str>,
    final_state: Option<&str>,
    thread_id: &str,
    message_id: &str,
) -> Result<SessionProcessingStopDispatch, String> {
    let normalized_thread_id = thread_id.trim();
    let normalized_message_id = message_id.trim();
    if normalized_thread_id.is_empty() {
        return Err("thread_id is required".to_string());
    }
    if normalized_message_id.is_empty() {
        return Err("message_id is required".to_string());
    }

    Ok(SessionProcessingStopDispatch {
        request_id: format!("processing-stop-{}", Uuid::new_v4()),
        account_id: account_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("default")
            .to_string(),
        logical_reply_id: logical_reply_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        final_state: final_state
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        thread_id: normalized_thread_id.to_string(),
        message_id: normalized_message_id.to_string(),
    })
}

pub(crate) async fn lookup_session_delivery_route_with_pool(
    pool: &SqlitePool,
    session_id: &str,
    channel: Option<&str>,
) -> Result<Option<SessionChannelDeliveryRoute>, String> {
    let authority_route =
        match find_channel_delivery_route_by_session_id(pool, session_id, channel).await {
            Ok(route) => route,
            Err(error) if error.contains("no such table") => None,
            Err(error) => return Err(error),
        };

    if let Some(route) = authority_route {
        let channel = route.channel.trim();
        let thread_id = route.reply_target.trim();
        if !channel.is_empty() && !thread_id.is_empty() {
            return Ok(Some(SessionChannelDeliveryRoute {
                channel: channel.to_string(),
                account_id: route.account_id.trim().to_string(),
                thread_id: thread_id.to_string(),
                conversation_id: route.conversation_id.trim().to_string(),
            }));
        }
    }

    Ok(None)
}

pub(crate) async fn resolve_session_delivery_route_with_pool(
    pool: &SqlitePool,
    session_id: &str,
    channel: Option<&str>,
) -> Result<Option<SessionChannelDeliveryRoute>, String> {
    if let Some(route) = lookup_session_delivery_route_with_pool(pool, session_id, channel).await? {
        return Ok(Some(route));
    }

    lookup_legacy_session_delivery_route_with_pool(pool, session_id, channel).await
}

async fn legacy_session_bindings_include_conversation_table(
    pool: &SqlitePool,
) -> Result<bool, String> {
    Ok(!sqlx::query_scalar::<_, String>(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'im_conversation_sessions'",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("查询 legacy 会话映射表失败: {e}"))?
    .is_empty())
}

async fn lookup_legacy_session_delivery_route_with_pool(
    pool: &SqlitePool,
    session_id: &str,
    channel: Option<&str>,
) -> Result<Option<SessionChannelDeliveryRoute>, String> {
    let normalized_session_id = session_id.trim();
    let normalized_channel = channel
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or_default();
    if normalized_session_id.is_empty() {
        return Ok(None);
    }

    let has_conversation_table = legacy_session_bindings_include_conversation_table(pool).await?;
    let row = if has_conversation_table {
        sqlx::query_as::<_, (String, String)>(
            "SELECT bindings.thread_id, e.source
             FROM (
               SELECT cs.thread_id AS thread_id, cs.updated_at AS updated_at, cs.created_at AS created_at
               FROM im_conversation_sessions cs
               WHERE cs.session_id = ?
               UNION ALL
               SELECT ts.thread_id AS thread_id, ts.updated_at AS updated_at, ts.created_at AS created_at
               FROM im_thread_sessions ts
               WHERE ts.session_id = ?
             ) bindings
             JOIN im_inbox_events e ON e.thread_id = bindings.thread_id
             WHERE e.source <> ''
               AND (? = '' OR e.source = ?)
             ORDER BY bindings.updated_at DESC, bindings.created_at DESC, e.created_at DESC, e.id DESC
             LIMIT 1",
        )
        .bind(normalized_session_id)
        .bind(normalized_session_id)
        .bind(normalized_channel)
        .bind(normalized_channel)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("查询 legacy 会话路由失败: {e}"))?
    } else {
        sqlx::query_as::<_, (String, String)>(
            "SELECT ts.thread_id, e.source
             FROM im_thread_sessions ts
             JOIN im_inbox_events e ON e.thread_id = ts.thread_id
             WHERE ts.session_id = ?
               AND e.source <> ''
               AND (? = '' OR e.source = ?)
             ORDER BY ts.updated_at DESC, ts.created_at DESC, e.created_at DESC, e.id DESC
             LIMIT 1",
        )
        .bind(normalized_session_id)
        .bind(normalized_channel)
        .bind(normalized_channel)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("查询 legacy 会话路由失败: {e}"))?
    };

    Ok(row.map(|(thread_id, source)| SessionChannelDeliveryRoute {
        channel: source,
        account_id: String::new(),
        thread_id,
        conversation_id: String::new(),
    }))
}

pub(crate) async fn lookup_channel_thread_for_session_with_pool(
    pool: &SqlitePool,
    source: &str,
    session_id: &str,
) -> Result<Option<String>, String> {
    Ok(
        resolve_session_delivery_route_with_pool(pool, session_id, Some(source))
            .await?
            .map(|route| route.thread_id),
    )
}

pub(crate) async fn lookup_channel_source_for_session_with_pool(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Option<String>, String> {
    Ok(
        resolve_session_delivery_route_with_pool(pool, session_id, None)
            .await?
            .map(|route| route.channel),
    )
}

pub(crate) async fn maybe_emit_registered_host_lifecycle_phase_for_session_with_pool(
    pool: &SqlitePool,
    session_id: &str,
    logical_reply_id: Option<&str>,
    phase: ImReplyLifecyclePhase,
    account_id: Option<&str>,
) -> Result<bool, String> {
    let Some(source) = lookup_channel_source_for_session_with_pool(pool, session_id).await? else {
        return Ok(false);
    };

    match source.trim() {
        "feishu" => {
            crate::commands::feishu_gateway::maybe_emit_registered_feishu_lifecycle_phase_for_session_with_pool(
                pool,
                session_id,
                logical_reply_id,
                phase,
                account_id,
            )
            .await
        }
        "wecom" => {
            crate::commands::wecom_gateway::maybe_emit_registered_wecom_lifecycle_phase_for_session_with_pool(
                pool,
                session_id,
                logical_reply_id,
                phase,
                account_id,
            )
            .await
        }
        _ => Ok(false),
    }
}

pub(crate) async fn maybe_stop_registered_host_processing_for_session_with_pool(
    pool: &SqlitePool,
    session_id: &str,
    logical_reply_id: Option<&str>,
    final_state: Option<&str>,
    account_id: Option<&str>,
) -> Result<bool, String> {
    let Some(source) = lookup_channel_source_for_session_with_pool(pool, session_id).await? else {
        return Ok(false);
    };

    match source.trim() {
        "feishu" => {
            crate::commands::feishu_gateway::maybe_stop_registered_feishu_processing_for_session_with_pool(
                pool,
                session_id,
                logical_reply_id,
                final_state,
                account_id,
            )
            .await
        }
        "wecom" => {
            crate::commands::wecom_gateway::maybe_stop_registered_wecom_processing_for_session_with_pool(
                pool,
                session_id,
                logical_reply_id,
                final_state,
                account_id,
            )
            .await
        }
        _ => Ok(false),
    }
}

pub(crate) async fn maybe_dispatch_registered_im_session_reply_with_pool(
    pool: &SqlitePool,
    session_id: &str,
    text: &str,
) -> Result<bool, String> {
    let normalized_session_id = session_id.trim();
    let normalized_text = text.trim();
    if normalized_session_id.is_empty() || normalized_text.is_empty() {
        return Ok(false);
    }

    let Some(source) =
        lookup_channel_source_for_session_with_pool(pool, normalized_session_id).await?
    else {
        return Ok(false);
    };

    match source.trim() {
        "feishu" => Ok(
            crate::commands::feishu_gateway::maybe_dispatch_feishu_session_reply_with_pool(
                pool,
                normalized_session_id,
                normalized_text,
            )
            .await?
            .is_some(),
        ),
        "wecom" => {
            let Some(thread_id) =
                lookup_channel_thread_for_session_with_pool(pool, "wecom", normalized_session_id)
                    .await?
            else {
                return Ok(false);
            };

            let plan = ImReplyDeliveryPlan {
                logical_reply_id: Uuid::new_v4().to_string(),
                session_id: normalized_session_id.to_string(),
                channel: "wecom".to_string(),
                thread_id,
                chunks: super::chunk_planner::plan_text_chunks(normalized_text, 1800),
            };

            crate::commands::wecom_gateway::execute_registered_wecom_reply_plan_with_pool(
                pool, &plan, None,
            )
            .await?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub(crate) async fn lookup_latest_inbox_message_id_for_thread_with_pool(
    pool: &SqlitePool,
    source: &str,
    thread_id: &str,
) -> Result<Option<String>, String> {
    let normalized_thread_id = thread_id.trim();
    if normalized_thread_id.is_empty() {
        return Ok(None);
    }

    let row = sqlx::query_as::<_, (String,)>(
        "SELECT message_id
         FROM im_inbox_events
         WHERE thread_id = ?
           AND source = ?
           AND message_id <> ''
         ORDER BY created_at DESC, id DESC
         LIMIT 1",
    )
    .bind(normalized_thread_id)
    .bind(source.trim())
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("查询 {source} 线程最新消息失败: {e}"))?;

    Ok(row.map(|(message_id,)| message_id))
}

pub(crate) async fn emit_registered_lifecycle_phase_for_session_with_pool<F>(
    pool: &SqlitePool,
    source: &str,
    session_id: &str,
    logical_reply_id: Option<&str>,
    phase: ImReplyLifecyclePhase,
    account_id: Option<&str>,
    send: F,
) -> Result<bool, String>
where
    F: FnOnce(SessionLifecycleDispatch) -> Result<(), String>,
{
    let Some(thread_id) =
        lookup_channel_thread_for_session_with_pool(pool, source, session_id).await?
    else {
        return Ok(false);
    };
    let message_id =
        lookup_latest_inbox_message_id_for_thread_with_pool(pool, source, &thread_id).await?;
    let dispatch = build_session_lifecycle_dispatch(
        account_id,
        logical_reply_id,
        phase,
        &thread_id,
        message_id.as_deref(),
    )?;
    send(dispatch)?;
    Ok(true)
}

pub(crate) async fn stop_registered_processing_for_session_with_pool<F>(
    pool: &SqlitePool,
    source: &str,
    session_id: &str,
    logical_reply_id: Option<&str>,
    final_state: Option<&str>,
    account_id: Option<&str>,
    send: F,
) -> Result<bool, String>
where
    F: FnOnce(SessionProcessingStopDispatch) -> Result<(), String>,
{
    let Some(thread_id) =
        lookup_channel_thread_for_session_with_pool(pool, source, session_id).await?
    else {
        return Ok(false);
    };
    let Some(message_id) =
        lookup_latest_inbox_message_id_for_thread_with_pool(pool, source, &thread_id).await?
    else {
        return Ok(false);
    };
    let dispatch = build_session_processing_stop_dispatch(
        account_id,
        logical_reply_id,
        final_state,
        &thread_id,
        &message_id,
    )?;
    send(dispatch)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::{
        build_session_lifecycle_dispatch, build_session_processing_stop_dispatch,
        lookup_channel_source_for_session_with_pool, lookup_channel_thread_for_session_with_pool,
        maybe_emit_registered_host_lifecycle_phase_for_session_with_pool,
        resolve_session_delivery_route_with_pool, ImReplyLifecyclePhase,
    };
    use crate::commands::feishu_gateway::{
        clear_feishu_runtime_state_for_outbound, remember_feishu_runtime_state_for_outbound,
    };
    use crate::commands::openclaw_plugins::{
        set_feishu_runtime_lifecycle_event_hook_for_tests,
        OpenClawPluginFeishuLifecycleEventRequest, OpenClawPluginFeishuRuntimeState,
    };
    use crate::commands::wecom_gateway::{
        set_wecom_lifecycle_event_hook_for_tests, set_wecom_outbound_send_hook_for_tests,
    };
    use sqlx::SqlitePool;
    use std::sync::{Arc, Mutex};

    #[test]
    fn build_session_lifecycle_dispatch_normalizes_optional_fields() {
        let dispatch = build_session_lifecycle_dispatch(
            Some(" default "),
            Some(" reply-1 "),
            ImReplyLifecyclePhase::AskUserRequested,
            " oc_chat_1 ",
            Some(" om_1 "),
        )
        .expect("build dispatch");

        assert_eq!(dispatch.account_id, "default");
        assert_eq!(dispatch.logical_reply_id.as_deref(), Some("reply-1"));
        assert_eq!(dispatch.thread_id, "oc_chat_1");
        assert_eq!(dispatch.message_id.as_deref(), Some("om_1"));
    }

    #[test]
    fn build_session_lifecycle_dispatch_rejects_empty_thread() {
        let error = build_session_lifecycle_dispatch(
            None,
            None,
            ImReplyLifecyclePhase::Failed,
            "   ",
            None,
        )
        .expect_err("empty thread should fail");

        assert!(error.contains("thread_id"));
    }

    #[test]
    fn build_session_processing_stop_dispatch_normalizes_optional_fields() {
        let dispatch = build_session_processing_stop_dispatch(
            Some(" default "),
            Some(" reply-1 "),
            Some(" completed "),
            " oc_chat_1 ",
            " om_1 ",
        )
        .expect("build processing stop dispatch");

        assert_eq!(dispatch.account_id, "default");
        assert_eq!(dispatch.logical_reply_id.as_deref(), Some("reply-1"));
        assert_eq!(dispatch.final_state.as_deref(), Some("completed"));
        assert_eq!(dispatch.thread_id, "oc_chat_1");
        assert_eq!(dispatch.message_id, "om_1");
    }

    async fn setup_lifecycle_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("create sqlite memory pool");

        sqlx::query(
            "CREATE TABLE agent_conversation_bindings (
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
            )",
        )
        .execute(&pool)
        .await
        .expect("create agent_conversation_bindings");

        sqlx::query(
            "CREATE TABLE channel_delivery_routes (
                session_key TEXT NOT NULL PRIMARY KEY,
                channel TEXT NOT NULL,
                account_id TEXT NOT NULL DEFAULT '',
                conversation_id TEXT NOT NULL,
                reply_target TEXT NOT NULL DEFAULT '',
                updated_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("create channel_delivery_routes");

        sqlx::query(
            "CREATE TABLE im_thread_sessions (
                thread_id TEXT NOT NULL,
                employee_id TEXT NOT NULL DEFAULT '',
                session_id TEXT NOT NULL,
                route_session_key TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT '',
                updated_at TEXT NOT NULL DEFAULT ''
            )",
        )
        .execute(&pool)
        .await
        .expect("create im_thread_sessions");

        sqlx::query(
            "CREATE TABLE im_inbox_events (
                id TEXT PRIMARY KEY,
                event_id TEXT NOT NULL DEFAULT '',
                thread_id TEXT NOT NULL,
                message_id TEXT NOT NULL DEFAULT '',
                text_preview TEXT NOT NULL DEFAULT '',
                source TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT ''
            )",
        )
        .execute(&pool)
        .await
        .expect("create im_inbox_events");

        pool
    }

    async fn setup_authority_only_lifecycle_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("create sqlite memory pool");

        sqlx::query(
            "CREATE TABLE agent_conversation_bindings (
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
            )",
        )
        .execute(&pool)
        .await
        .expect("create agent_conversation_bindings");

        sqlx::query(
            "CREATE TABLE channel_delivery_routes (
                session_key TEXT NOT NULL PRIMARY KEY,
                channel TEXT NOT NULL,
                account_id TEXT NOT NULL DEFAULT '',
                conversation_id TEXT NOT NULL,
                reply_target TEXT NOT NULL DEFAULT '',
                updated_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("create channel_delivery_routes");

        pool
    }

    async fn seed_session_channel(
        pool: &SqlitePool,
        session_id: &str,
        thread_id: &str,
        source: &str,
        message_id: &str,
    ) {
        let session_key = format!("sk-{session_id}");
        sqlx::query(
            "INSERT INTO im_thread_sessions (thread_id, employee_id, session_id, route_session_key, created_at, updated_at)
             VALUES (?, '', ?, '', '2026-04-19T00:00:00Z', '2026-04-19T00:00:01Z')",
        )
        .bind(thread_id)
        .bind(session_id)
        .execute(pool)
        .await
        .expect("seed im_thread_sessions");

        sqlx::query(
            "INSERT INTO agent_conversation_bindings (
                conversation_id, channel, account_id, agent_id, session_key, session_id,
                base_conversation_id, parent_conversation_candidates_json, scope, peer_kind,
                peer_id, topic_id, sender_id, created_at, updated_at
             )
             VALUES (?, ?, 'default', 'main-agent', ?, ?, ?, '[]', 'peer', 'group', ?, '', '', '2026-04-19T00:00:00Z', '2026-04-19T00:00:01Z')",
        )
        .bind(thread_id)
        .bind(source)
        .bind(&session_key)
        .bind(session_id)
        .bind(thread_id)
        .bind(thread_id)
        .execute(pool)
        .await
        .expect("seed agent_conversation_bindings");

        sqlx::query(
            "INSERT INTO channel_delivery_routes (session_key, channel, account_id, conversation_id, reply_target, updated_at)
             VALUES (?, ?, 'default', ?, ?, '2026-04-19T00:00:01Z')",
        )
        .bind(&session_key)
        .bind(source)
        .bind(thread_id)
        .bind(thread_id)
        .execute(pool)
        .await
        .expect("seed channel_delivery_routes");

        sqlx::query(
            "INSERT INTO im_inbox_events (id, event_id, thread_id, message_id, text_preview, source, created_at)
             VALUES (?, ?, ?, ?, 'hello', ?, '2026-04-19T00:00:02Z')",
        )
        .bind(format!("evt-{session_id}"))
        .bind(format!("evt-{session_id}"))
        .bind(thread_id)
        .bind(message_id)
        .bind(source)
        .execute(pool)
        .await
        .expect("seed im_inbox_events");
    }

    async fn seed_authority_only_session_channel(
        pool: &SqlitePool,
        session_id: &str,
        thread_id: &str,
        source: &str,
        conversation_id: &str,
    ) {
        let session_key = format!("sk-{session_id}");
        sqlx::query(
            "INSERT INTO agent_conversation_bindings (
                conversation_id, channel, account_id, agent_id, session_key, session_id,
                base_conversation_id, parent_conversation_candidates_json, scope, peer_kind,
                peer_id, topic_id, sender_id, created_at, updated_at
             )
             VALUES (?, ?, 'default', 'main-agent', ?, ?, ?, '[]', 'peer', 'group', ?, '', '', '2026-04-19T00:00:00Z', '2026-04-19T00:00:01Z')",
        )
        .bind(conversation_id)
        .bind(source)
        .bind(&session_key)
        .bind(session_id)
        .bind(conversation_id)
        .bind(thread_id)
        .execute(pool)
        .await
        .expect("seed authority-only agent_conversation_bindings");

        sqlx::query(
            "INSERT INTO channel_delivery_routes (session_key, channel, account_id, conversation_id, reply_target, updated_at)
             VALUES (?, ?, 'default', ?, ?, '2026-04-19T00:00:01Z')",
        )
        .bind(&session_key)
        .bind(source)
        .bind(conversation_id)
        .bind(thread_id)
        .execute(pool)
        .await
        .expect("seed authority-only channel_delivery_routes");
    }

    fn install_feishu_lifecycle_hook() -> Arc<Mutex<Vec<String>>> {
        let runtime_state = OpenClawPluginFeishuRuntimeState::default();
        remember_feishu_runtime_state_for_outbound(&runtime_state);

        let recorded = Arc::new(Mutex::new(Vec::<String>::new()));
        let recorded_for_hook = recorded.clone();
        set_feishu_runtime_lifecycle_event_hook_for_tests(Some(Arc::new(
            move |request: &OpenClawPluginFeishuLifecycleEventRequest| {
                recorded_for_hook
                    .lock()
                    .expect("lock lifecycle records")
                    .push(format!(
                        "{}:{}",
                        request.message_id.as_deref().unwrap_or(""),
                        serde_json::to_string(&request.phase)
                            .unwrap_or_else(|_| "\"unknown\"".to_string())
                    ));
                Ok(())
            },
        )));

        recorded
    }

    fn cleanup_feishu_lifecycle_hook() {
        set_feishu_runtime_lifecycle_event_hook_for_tests(None);
        clear_feishu_runtime_state_for_outbound();
    }

    fn install_wecom_lifecycle_hook() -> Arc<Mutex<Vec<String>>> {
        let recorded = Arc::new(Mutex::new(Vec::<String>::new()));
        let recorded_for_hook = recorded.clone();
        set_wecom_lifecycle_event_hook_for_tests(Some(Arc::new(move |request| {
            recorded_for_hook
                .lock()
                .expect("lock wecom lifecycle records")
                .push(format!(
                    "{}:{}",
                    request.message_id.as_deref().unwrap_or(""),
                    serde_json::to_string(&request.phase)
                        .unwrap_or_else(|_| "\"unknown\"".to_string())
                ));
            Ok(())
        })));

        recorded
    }

    fn cleanup_wecom_lifecycle_hook() {
        set_wecom_lifecycle_event_hook_for_tests(None);
    }

    fn install_wecom_send_hook() -> Arc<Mutex<Vec<String>>> {
        let sent_texts = Arc::new(Mutex::new(Vec::<String>::new()));
        let sent_texts_for_hook = sent_texts.clone();
        set_wecom_outbound_send_hook_for_tests(Some(Arc::new(move |_thread_id, text| {
            sent_texts_for_hook
                .lock()
                .expect("lock wecom sent texts")
                .push(text.to_string());
            Ok(serde_json::json!({
                "message_id": "wm_reply_dispatch_1",
            }))
        })));
        sent_texts
    }

    fn cleanup_wecom_send_hook() {
        set_wecom_outbound_send_hook_for_tests(None);
    }

    #[tokio::test]
    async fn host_lifecycle_emit_routes_answer_and_resume_phases_to_feishu_runtime() {
        let pool = setup_lifecycle_pool().await;
        seed_session_channel(
            &pool,
            "session-feishu-lifecycle",
            "oc_chat_lifecycle",
            "feishu",
            "om_parent_lifecycle",
        )
        .await;
        let recorded = install_feishu_lifecycle_hook();

        let answered = maybe_emit_registered_host_lifecycle_phase_for_session_with_pool(
            &pool,
            "session-feishu-lifecycle",
            None,
            ImReplyLifecyclePhase::AskUserAnswered,
            None,
        )
        .await
        .expect("emit ask_user_answered");
        let resolved = maybe_emit_registered_host_lifecycle_phase_for_session_with_pool(
            &pool,
            "session-feishu-lifecycle",
            Some("reply-approval"),
            ImReplyLifecyclePhase::ApprovalResolved,
            None,
        )
        .await
        .expect("emit approval_resolved");
        let resumed = maybe_emit_registered_host_lifecycle_phase_for_session_with_pool(
            &pool,
            "session-feishu-lifecycle",
            Some("reply-resumed"),
            ImReplyLifecyclePhase::Resumed,
            None,
        )
        .await
        .expect("emit resumed");

        cleanup_feishu_lifecycle_hook();

        assert!(answered);
        assert!(resolved);
        assert!(resumed);
        let entries = recorded.lock().expect("lock lifecycle records");
        assert_eq!(
            entries.as_slice(),
            [
                "om_parent_lifecycle:\"ask_user_answered\"",
                "om_parent_lifecycle:\"approval_resolved\"",
                "om_parent_lifecycle:\"resumed\"",
            ]
        );
    }

    #[tokio::test]
    async fn lifecycle_lookup_prefers_authority_store_without_legacy_tables() {
        let pool = setup_authority_only_lifecycle_pool().await;
        seed_authority_only_session_channel(
            &pool,
            "session-authority-only",
            "oc_chat_authority_only",
            "feishu",
            "feishu:default:group:oc_chat_authority_only",
        )
        .await;

        let source = lookup_channel_source_for_session_with_pool(&pool, "session-authority-only")
            .await
            .expect("lookup source");
        let thread =
            lookup_channel_thread_for_session_with_pool(&pool, "feishu", "session-authority-only")
                .await
                .expect("lookup thread");

        assert_eq!(source.as_deref(), Some("feishu"));
        assert_eq!(thread.as_deref(), Some("oc_chat_authority_only"));
    }

    #[tokio::test]
    async fn authority_lookup_preserves_topic_conversation_projection() {
        let pool = setup_authority_only_lifecycle_pool().await;
        seed_authority_only_session_channel(
            &pool,
            "session-authority-topic",
            "oc_chat_topic",
            "feishu",
            "feishu:default:group:oc_chat_topic:topic:om_root_topic",
        )
        .await;

        let route = resolve_session_delivery_route_with_pool(
            &pool,
            "session-authority-topic",
            Some("feishu"),
        )
        .await
        .expect("resolve authority route")
        .expect("authority route exists");

        assert_eq!(
            route.conversation_id,
            "feishu:default:group:oc_chat_topic:topic:om_root_topic"
        );
        assert_eq!(route.thread_id, "oc_chat_topic");
    }

    #[cfg(not(target_os = "windows"))]
    #[tokio::test]
    async fn host_lifecycle_emit_routes_answer_and_resume_phases_to_wecom_host() {
        let pool = setup_lifecycle_pool().await;
        seed_session_channel(
            &pool,
            "session-wecom-lifecycle",
            "wecom_chat_lifecycle",
            "wecom",
            "wm_parent_lifecycle",
        )
        .await;
        let recorded = install_wecom_lifecycle_hook();

        let answered = maybe_emit_registered_host_lifecycle_phase_for_session_with_pool(
            &pool,
            "session-wecom-lifecycle",
            None,
            ImReplyLifecyclePhase::AskUserAnswered,
            None,
        )
        .await
        .expect("emit wecom ask_user_answered");
        let resolved = maybe_emit_registered_host_lifecycle_phase_for_session_with_pool(
            &pool,
            "session-wecom-lifecycle",
            Some("reply-wecom-approval"),
            ImReplyLifecyclePhase::ApprovalResolved,
            None,
        )
        .await
        .expect("emit wecom approval_resolved");
        let resumed = maybe_emit_registered_host_lifecycle_phase_for_session_with_pool(
            &pool,
            "session-wecom-lifecycle",
            Some("reply-wecom-resumed"),
            ImReplyLifecyclePhase::Resumed,
            None,
        )
        .await
        .expect("emit wecom resumed");

        cleanup_wecom_lifecycle_hook();

        assert!(answered);
        assert!(resolved);
        assert!(resumed);
        let entries = recorded.lock().expect("lock wecom lifecycle records");
        assert_eq!(
            entries.as_slice(),
            [
                "wm_parent_lifecycle:\"ask_user_answered\"",
                "wm_parent_lifecycle:\"approval_resolved\"",
                "wm_parent_lifecycle:\"resumed\"",
            ]
        );
    }

    #[cfg(not(target_os = "windows"))]
    #[tokio::test]
    async fn host_reply_dispatch_routes_wecom_session_via_unified_host() {
        let pool = setup_lifecycle_pool().await;
        seed_session_channel(
            &pool,
            "session-wecom-dispatch",
            "wecom_chat_dispatch",
            "wecom",
            "wm_parent_dispatch",
        )
        .await;
        let sent_texts = install_wecom_send_hook();

        let handled = maybe_dispatch_registered_im_session_reply_with_pool(
            &pool,
            "session-wecom-dispatch",
            "企微 unified host 最终回复",
        )
        .await
        .expect("dispatch wecom reply");

        cleanup_wecom_send_hook();

        assert!(handled);
        let sent = sent_texts.lock().expect("lock wecom sent texts");
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0], "企微 unified host 最终回复");
    }
}
