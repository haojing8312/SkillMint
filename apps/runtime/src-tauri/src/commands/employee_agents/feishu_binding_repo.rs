use sqlx::{Row, Sqlite, Transaction};

pub(crate) struct InsertFeishuBindingInput<'a> {
    pub id: &'a str,
    pub agent_id: &'a str,
    pub peer_kind: &'a str,
    pub peer_id: &'a str,
    pub connector_meta_json: &'a str,
    pub priority: i64,
    pub now: &'a str,
}

pub(crate) async fn delete_feishu_bindings_for_agent(
    tx: &mut Transaction<'_, Sqlite>,
    agent_id: &str,
) -> Result<(), String> {
    sqlx::query(
        "DELETE FROM im_routing_bindings WHERE channel = 'feishu' AND lower(agent_id) = lower(?)",
    )
    .bind(agent_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn find_displaced_default_feishu_agent_ids(
    tx: &mut Transaction<'_, Sqlite>,
    agent_id: &str,
) -> Result<Vec<String>, String> {
    let rows = sqlx::query_scalar::<_, String>(
        r#"
        SELECT DISTINCT agent_id
        FROM im_routing_bindings
        WHERE channel = 'feishu'
          AND trim(peer_id) = ''
          AND lower(agent_id) != lower(?)
        "#,
    )
    .bind(agent_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(rows)
}

pub(crate) async fn delete_displaced_default_feishu_bindings(
    tx: &mut Transaction<'_, Sqlite>,
    agent_id: &str,
) -> Result<(), String> {
    sqlx::query(
        r#"
        DELETE FROM im_routing_bindings
        WHERE channel = 'feishu'
          AND trim(peer_id) = ''
          AND lower(agent_id) != lower(?)
        "#,
    )
    .bind(agent_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn find_displaced_scoped_feishu_agent_ids(
    tx: &mut Transaction<'_, Sqlite>,
    agent_id: &str,
    peer_kind: &str,
    peer_id: &str,
) -> Result<Vec<String>, String> {
    let rows = sqlx::query_scalar::<_, String>(
        r#"
        SELECT DISTINCT agent_id
        FROM im_routing_bindings
        WHERE channel = 'feishu'
          AND lower(agent_id) != lower(?)
          AND lower(peer_kind) = ?
          AND trim(peer_id) = ?
        "#,
    )
    .bind(agent_id)
    .bind(peer_kind)
    .bind(peer_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(rows)
}

pub(crate) async fn delete_displaced_scoped_feishu_bindings(
    tx: &mut Transaction<'_, Sqlite>,
    agent_id: &str,
    peer_kind: &str,
    peer_id: &str,
) -> Result<(), String> {
    sqlx::query(
        r#"
        DELETE FROM im_routing_bindings
        WHERE channel = 'feishu'
          AND lower(agent_id) != lower(?)
          AND lower(peer_kind) = ?
          AND trim(peer_id) = ?
        "#,
    )
    .bind(agent_id)
    .bind(peer_kind)
    .bind(peer_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn insert_feishu_binding(
    tx: &mut Transaction<'_, Sqlite>,
    input: &InsertFeishuBindingInput<'_>,
) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO im_routing_bindings (
            id, agent_id, channel, account_id, peer_kind, peer_id, guild_id, team_id,
            role_ids_json, connector_meta_json, priority, enabled, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(input.id)
    .bind(input.agent_id)
    .bind("feishu")
    .bind("*")
    .bind(input.peer_kind)
    .bind(input.peer_id)
    .bind("")
    .bind("")
    .bind("[]")
    .bind(input.connector_meta_json)
    .bind(input.priority)
    .bind(1_i64)
    .bind(input.now)
    .bind(input.now)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn count_feishu_bindings_for_agent(
    tx: &mut Transaction<'_, Sqlite>,
    agent_id: &str,
) -> Result<i64, String> {
    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(1)
        FROM im_routing_bindings
        WHERE channel = 'feishu' AND lower(agent_id) = lower(?)
        "#,
    )
    .bind(agent_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(count)
}

pub(crate) async fn list_agent_scope_rows(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<Vec<(String, String, String, String, String)>, String> {
    let rows = sqlx::query(
        r#"
        SELECT id, employee_id, role_id, openclaw_agent_id, enabled_scopes_json
        FROM agent_employees
        "#,
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|row| {
            (
                row.try_get("id").expect("scope row id"),
                row.try_get("employee_id").expect("scope row employee_id"),
                row.try_get("role_id").expect("scope row role_id"),
                row.try_get("openclaw_agent_id")
                    .expect("scope row openclaw_agent_id"),
                row.try_get("enabled_scopes_json")
                    .expect("scope row enabled_scopes_json"),
            )
        })
        .collect())
}
