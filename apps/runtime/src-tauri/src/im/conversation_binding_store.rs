use super::agent_session_binding::{
    AgentConversationBinding, AgentConversationBindingUpsert, ChannelDeliveryRoute,
    ChannelDeliveryRouteUpsert,
};
use sqlx::{Row, SqlitePool};

fn deserialize_candidates(raw: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(raw).unwrap_or_default()
}

async fn table_exists(pool: &SqlitePool, table_name: &str) -> Result<bool, String> {
    let row = sqlx::query_scalar::<_, String>(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?",
    )
    .bind(table_name)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(row.is_some())
}

pub async fn find_agent_conversation_binding(
    pool: &SqlitePool,
    conversation_id: &str,
    agent_id: &str,
) -> Result<Option<AgentConversationBinding>, String> {
    if !table_exists(pool, "agent_conversation_bindings").await? {
        return Ok(None);
    }

    let row = sqlx::query(
        "SELECT conversation_id,
                channel,
                account_id,
                agent_id,
                session_key,
                session_id,
                base_conversation_id,
                parent_conversation_candidates_json,
                scope,
                peer_kind,
                peer_id,
                topic_id,
                sender_id,
                created_at,
                updated_at
         FROM agent_conversation_bindings
         WHERE conversation_id = ? AND agent_id = ?
         LIMIT 1",
    )
    .bind(conversation_id.trim())
    .bind(agent_id.trim())
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|record| AgentConversationBinding {
        conversation_id: record.try_get(0).expect("binding conversation_id"),
        channel: record.try_get(1).expect("binding channel"),
        account_id: record.try_get(2).expect("binding account_id"),
        agent_id: record.try_get(3).expect("binding agent_id"),
        session_key: record.try_get(4).expect("binding session_key"),
        session_id: record.try_get(5).expect("binding session_id"),
        base_conversation_id: record.try_get(6).expect("binding base conversation"),
        parent_conversation_candidates: deserialize_candidates(
            record
                .try_get::<String, _>(7)
                .expect("binding parent_conversation_candidates_json")
                .as_str(),
        ),
        scope: record.try_get(8).expect("binding scope"),
        peer_kind: record.try_get(9).expect("binding peer_kind"),
        peer_id: record.try_get(10).expect("binding peer_id"),
        topic_id: record.try_get(11).expect("binding topic_id"),
        sender_id: record.try_get(12).expect("binding sender_id"),
        created_at: record.try_get(13).expect("binding created_at"),
        updated_at: record.try_get(14).expect("binding updated_at"),
    }))
}

pub async fn find_agent_conversation_binding_for_candidates(
    pool: &SqlitePool,
    conversation_id: &str,
    parent_candidates: &[String],
    agent_id: &str,
) -> Result<Option<AgentConversationBinding>, String> {
    if let Some(binding) = find_agent_conversation_binding(pool, conversation_id, agent_id).await? {
        return Ok(Some(binding));
    }

    for candidate in parent_candidates {
        if let Some(binding) = find_agent_conversation_binding(pool, candidate, agent_id).await? {
            return Ok(Some(binding));
        }
    }

    Ok(None)
}

pub async fn upsert_agent_conversation_binding(
    pool: &SqlitePool,
    input: &AgentConversationBindingUpsert<'_>,
) -> Result<(), String> {
    let parent_conversation_candidates_json =
        serde_json::to_string(input.parent_conversation_candidates).map_err(|e| e.to_string())?;

    sqlx::query(
        "INSERT INTO agent_conversation_bindings (
            conversation_id,
            channel,
            account_id,
            agent_id,
            session_key,
            session_id,
            base_conversation_id,
            parent_conversation_candidates_json,
            scope,
            peer_kind,
            peer_id,
            topic_id,
            sender_id,
            created_at,
            updated_at
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(conversation_id, agent_id) DO UPDATE SET
            channel = excluded.channel,
            account_id = excluded.account_id,
            session_key = excluded.session_key,
            session_id = excluded.session_id,
            base_conversation_id = excluded.base_conversation_id,
            parent_conversation_candidates_json = excluded.parent_conversation_candidates_json,
            scope = excluded.scope,
            peer_kind = excluded.peer_kind,
            peer_id = excluded.peer_id,
            topic_id = excluded.topic_id,
            sender_id = excluded.sender_id,
            updated_at = excluded.updated_at",
    )
    .bind(input.conversation_id.trim())
    .bind(input.channel.trim())
    .bind(input.account_id.trim())
    .bind(input.agent_id.trim())
    .bind(input.session_key.trim())
    .bind(input.session_id.trim())
    .bind(input.base_conversation_id.trim())
    .bind(parent_conversation_candidates_json)
    .bind(input.scope.trim())
    .bind(input.peer_kind.trim())
    .bind(input.peer_id.trim())
    .bind(input.topic_id.trim())
    .bind(input.sender_id.trim())
    .bind(input.created_at.trim())
    .bind(input.updated_at.trim())
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn find_channel_delivery_route(
    pool: &SqlitePool,
    session_key: &str,
) -> Result<Option<ChannelDeliveryRoute>, String> {
    if !table_exists(pool, "channel_delivery_routes").await? {
        return Ok(None);
    }

    let row = sqlx::query(
        "SELECT session_key,
                channel,
                account_id,
                conversation_id,
                reply_target,
                updated_at
         FROM channel_delivery_routes
         WHERE session_key = ?
         LIMIT 1",
    )
    .bind(session_key.trim())
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|record| ChannelDeliveryRoute {
        session_key: record.try_get(0).expect("route session_key"),
        channel: record.try_get(1).expect("route channel"),
        account_id: record.try_get(2).expect("route account_id"),
        conversation_id: record.try_get(3).expect("route conversation_id"),
        reply_target: record.try_get(4).expect("route reply_target"),
        updated_at: record.try_get(5).expect("route updated_at"),
    }))
}

pub async fn upsert_channel_delivery_route(
    pool: &SqlitePool,
    input: &ChannelDeliveryRouteUpsert<'_>,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO channel_delivery_routes (
            session_key,
            channel,
            account_id,
            conversation_id,
            reply_target,
            updated_at
         )
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(session_key) DO UPDATE SET
            channel = excluded.channel,
            account_id = excluded.account_id,
            conversation_id = excluded.conversation_id,
            reply_target = excluded.reply_target,
            updated_at = excluded.updated_at",
    )
    .bind(input.session_key.trim())
    .bind(input.channel.trim())
    .bind(input.account_id.trim())
    .bind(input.conversation_id.trim())
    .bind(input.reply_target.trim())
    .bind(input.updated_at.trim())
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn find_channel_delivery_route_by_session_id(
    pool: &SqlitePool,
    session_id: &str,
    channel: Option<&str>,
) -> Result<Option<ChannelDeliveryRoute>, String> {
    if !table_exists(pool, "agent_conversation_bindings").await?
        || !table_exists(pool, "channel_delivery_routes").await?
    {
        return Ok(None);
    }

    let channel_filter = channel
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or_default();
    let row = sqlx::query(
        "SELECT r.session_key,
                r.channel,
                r.account_id,
                r.conversation_id,
                r.reply_target,
                r.updated_at
         FROM agent_conversation_bindings b
         JOIN channel_delivery_routes r ON r.session_key = b.session_key
         WHERE b.session_id = ?
           AND (? = '' OR r.channel = ?)
         ORDER BY b.updated_at DESC, r.updated_at DESC
         LIMIT 1",
    )
    .bind(session_id.trim())
    .bind(channel_filter)
    .bind(channel_filter)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|record| ChannelDeliveryRoute {
        session_key: record.try_get(0).expect("route session_key"),
        channel: record.try_get(1).expect("route channel"),
        account_id: record.try_get(2).expect("route account_id"),
        conversation_id: record.try_get(3).expect("route conversation_id"),
        reply_target: record.try_get(4).expect("route reply_target"),
        updated_at: record.try_get(5).expect("route updated_at"),
    }))
}
