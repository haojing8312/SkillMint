use sqlx::{Row, Sqlite, SqlitePool, Transaction};

#[derive(Debug, Clone)]
pub(super) struct AgentEmployeeRow {
    pub id: String,
    pub employee_id: String,
    pub name: String,
    pub role_id: String,
    pub persona: String,
    pub feishu_open_id: String,
    pub feishu_app_id: String,
    pub feishu_app_secret: String,
    pub primary_skill_id: String,
    pub default_work_dir: String,
    pub openclaw_agent_id: String,
    pub routing_priority: i64,
    pub enabled_scopes_json: String,
    pub enabled: bool,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

pub(super) struct EmployeeAssociationRow {
    pub employee_id: String,
    pub role_id: String,
    pub openclaw_agent_id: String,
    pub enabled_scopes_json: String,
}

pub(super) async fn list_agent_employee_rows(
    pool: &SqlitePool,
) -> Result<Vec<AgentEmployeeRow>, String> {
    let rows = sqlx::query(
        r#"
        SELECT
            id,
            employee_id,
            name,
            role_id,
            persona,
            feishu_open_id,
            feishu_app_id,
            feishu_app_secret,
            primary_skill_id,
            default_work_dir,
            openclaw_agent_id,
            routing_priority,
            enabled_scopes_json,
            enabled,
            is_default,
            created_at,
            updated_at
        FROM agent_employees
        ORDER BY is_default DESC, updated_at DESC
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|row| AgentEmployeeRow {
            id: row.try_get("id").expect("employee row id"),
            employee_id: row.try_get("employee_id").expect("employee row employee_id"),
            name: row.try_get("name").expect("employee row name"),
            role_id: row.try_get("role_id").expect("employee row role_id"),
            persona: row.try_get("persona").expect("employee row persona"),
            feishu_open_id: row.try_get("feishu_open_id").expect("employee row feishu_open_id"),
            feishu_app_id: row.try_get("feishu_app_id").expect("employee row feishu_app_id"),
            feishu_app_secret: row
                .try_get("feishu_app_secret")
                .expect("employee row feishu_app_secret"),
            primary_skill_id: row
                .try_get("primary_skill_id")
                .expect("employee row primary_skill_id"),
            default_work_dir: row
                .try_get("default_work_dir")
                .expect("employee row default_work_dir"),
            openclaw_agent_id: row
                .try_get("openclaw_agent_id")
                .expect("employee row openclaw_agent_id"),
            routing_priority: row
                .try_get("routing_priority")
                .expect("employee row routing_priority"),
            enabled_scopes_json: row
                .try_get("enabled_scopes_json")
                .expect("employee row enabled_scopes_json"),
            enabled: row.try_get::<i64, _>("enabled").expect("employee row enabled") != 0,
            is_default: row.try_get::<i64, _>("is_default").expect("employee row is_default") != 0,
            created_at: row.try_get("created_at").expect("employee row created_at"),
            updated_at: row.try_get("updated_at").expect("employee row updated_at"),
        })
        .collect())
}

pub(super) async fn list_skill_ids_for_employee(
    pool: &SqlitePool,
    employee_db_id: &str,
) -> Result<Vec<String>, String> {
    let rows = sqlx::query(
        r#"
        SELECT skill_id
        FROM agent_employee_skills
        WHERE employee_id = ?
        ORDER BY sort_order ASC
        "#,
    )
    .bind(employee_db_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|row| row.try_get("skill_id").expect("skill row skill_id"))
        .collect())
}

pub(super) async fn find_employee_db_id_by_employee_id(
    pool: &SqlitePool,
    employee_id: &str,
) -> Result<Option<String>, String> {
    let row = sqlx::query("SELECT id FROM agent_employees WHERE employee_id = ? LIMIT 1")
        .bind(employee_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(row.map(|record| record.try_get("id").expect("employee id row id")))
}

pub(super) async fn get_employee_association_row(
    pool: &SqlitePool,
    employee_db_id: &str,
) -> Result<Option<EmployeeAssociationRow>, String> {
    let row = sqlx::query(
        r#"
        SELECT employee_id, role_id, openclaw_agent_id, enabled_scopes_json
        FROM agent_employees
        WHERE id = ?
        "#,
    )
    .bind(employee_db_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|record| EmployeeAssociationRow {
        employee_id: record.try_get("employee_id").expect("association employee_id"),
        role_id: record.try_get("role_id").expect("association role_id"),
        openclaw_agent_id: record
            .try_get("openclaw_agent_id")
            .expect("association openclaw_agent_id"),
        enabled_scopes_json: record
            .try_get("enabled_scopes_json")
            .expect("association enabled_scopes_json"),
    }))
}

pub(super) async fn clear_default_employee_flag(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<(), String> {
    sqlx::query("UPDATE agent_employees SET is_default = 0")
        .execute(&mut **tx)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub(super) async fn upsert_agent_employee_record(
    tx: &mut Transaction<'_, Sqlite>,
    input: &UpsertAgentEmployeeRecordInput<'_>,
) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO agent_employees (
            id, employee_id, name, role_id, persona, feishu_open_id, feishu_app_id, feishu_app_secret,
            primary_skill_id, default_work_dir, openclaw_agent_id, routing_priority, enabled_scopes_json,
            enabled, is_default, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            employee_id = excluded.employee_id,
            name = excluded.name,
            role_id = excluded.role_id,
            persona = excluded.persona,
            feishu_open_id = excluded.feishu_open_id,
            feishu_app_id = excluded.feishu_app_id,
            feishu_app_secret = excluded.feishu_app_secret,
            primary_skill_id = excluded.primary_skill_id,
            default_work_dir = excluded.default_work_dir,
            openclaw_agent_id = excluded.openclaw_agent_id,
            routing_priority = excluded.routing_priority,
            enabled_scopes_json = excluded.enabled_scopes_json,
            enabled = excluded.enabled,
            is_default = excluded.is_default,
            updated_at = excluded.updated_at
        "#
    )
    .bind(input.id)
    .bind(input.employee_id)
    .bind(input.name)
    .bind(input.role_id)
    .bind(input.persona)
    .bind(input.feishu_open_id)
    .bind(input.feishu_app_id)
    .bind(input.feishu_app_secret)
    .bind(input.primary_skill_id)
    .bind(input.default_work_dir)
    .bind(input.openclaw_agent_id)
    .bind(input.routing_priority)
    .bind(input.enabled_scopes_json)
    .bind(if input.enabled { 1_i64 } else { 0_i64 })
    .bind(if input.is_default { 1_i64 } else { 0_i64 })
    .bind(input.now)
    .bind(input.now)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(super) async fn replace_employee_skill_bindings(
    tx: &mut Transaction<'_, Sqlite>,
    employee_db_id: &str,
    skill_ids: &[String],
) -> Result<(), String> {
    sqlx::query("DELETE FROM agent_employee_skills WHERE employee_id = ?")
        .bind(employee_db_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| e.to_string())?;

    for (idx, skill_id) in skill_ids.iter().enumerate() {
        sqlx::query("INSERT INTO agent_employee_skills (employee_id, skill_id, sort_order) VALUES (?, ?, ?)")
        .bind(employee_db_id)
        .bind(skill_id)
        .bind(idx as i64)
        .execute(&mut **tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub(super) async fn delete_agent_employee_record(
    tx: &mut Transaction<'_, Sqlite>,
    employee_db_id: &str,
) -> Result<(), String> {
    sqlx::query("DELETE FROM agent_employee_skills WHERE employee_id = ?")
        .bind(employee_db_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM im_thread_sessions WHERE employee_id = ?")
        .bind(employee_db_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM agent_employees WHERE id = ?")
        .bind(employee_db_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub(super) async fn update_employee_enabled_scopes(
    tx: &mut Transaction<'_, Sqlite>,
    employee_db_id: &str,
    enabled_scopes_json: &str,
    now: &str,
) -> Result<(), String> {
    sqlx::query("UPDATE agent_employees SET enabled_scopes_json = ?, updated_at = ? WHERE id = ?")
    .bind(enabled_scopes_json)
    .bind(now)
    .bind(employee_db_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(super) async fn delete_feishu_bindings_for_agent(
    tx: &mut Transaction<'_, Sqlite>,
    agent_id: &str,
) -> Result<(), String> {
    sqlx::query("DELETE FROM im_routing_bindings WHERE channel = 'feishu' AND lower(agent_id) = lower(?)")
    .bind(agent_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(super) async fn find_displaced_default_feishu_agent_ids(
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
        "#
    )
    .bind(agent_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(rows)
}

pub(super) async fn delete_displaced_default_feishu_bindings(
    tx: &mut Transaction<'_, Sqlite>,
    agent_id: &str,
) -> Result<(), String> {
    sqlx::query(
        r#"
        DELETE FROM im_routing_bindings
        WHERE channel = 'feishu'
          AND trim(peer_id) = ''
          AND lower(agent_id) != lower(?)
        "#
    )
    .bind(agent_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(super) async fn find_displaced_scoped_feishu_agent_ids(
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
        "#
    )
    .bind(agent_id)
    .bind(peer_kind)
    .bind(peer_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(rows)
}

pub(super) async fn delete_displaced_scoped_feishu_bindings(
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
        "#
    )
    .bind(agent_id)
    .bind(peer_kind)
    .bind(peer_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(super) async fn insert_feishu_binding(
    tx: &mut Transaction<'_, Sqlite>,
    input: &InsertFeishuBindingInput<'_>,
) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO im_routing_bindings (
            id, agent_id, channel, account_id, peer_kind, peer_id, guild_id, team_id,
            role_ids_json, connector_meta_json, priority, enabled, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#
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

pub(super) async fn count_feishu_bindings_for_agent(
    tx: &mut Transaction<'_, Sqlite>,
    agent_id: &str,
) -> Result<i64, String> {
    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(1)
        FROM im_routing_bindings
        WHERE channel = 'feishu' AND lower(agent_id) = lower(?)
        "#
    )
    .bind(agent_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(count)
}

pub(super) async fn list_agent_scope_rows(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<Vec<(String, String, String, String, String)>, String> {
    let rows = sqlx::query(
        r#"
        SELECT id, employee_id, role_id, openclaw_agent_id, enabled_scopes_json
        FROM agent_employees
        "#
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
                row.try_get("openclaw_agent_id").expect("scope row openclaw_agent_id"),
                row.try_get("enabled_scopes_json").expect("scope row enabled_scopes_json"),
            )
        })
        .collect())
}

pub(super) struct InsertFeishuBindingInput<'a> {
    pub id: &'a str,
    pub agent_id: &'a str,
    pub peer_kind: &'a str,
    pub peer_id: &'a str,
    pub connector_meta_json: &'a str,
    pub priority: i64,
    pub now: &'a str,
}

pub(super) struct UpsertAgentEmployeeRecordInput<'a> {
    pub id: &'a str,
    pub employee_id: &'a str,
    pub name: &'a str,
    pub role_id: &'a str,
    pub persona: &'a str,
    pub feishu_open_id: &'a str,
    pub feishu_app_id: &'a str,
    pub feishu_app_secret: &'a str,
    pub primary_skill_id: &'a str,
    pub default_work_dir: &'a str,
    pub openclaw_agent_id: &'a str,
    pub routing_priority: i64,
    pub enabled_scopes_json: &'a str,
    pub enabled: bool,
    pub is_default: bool,
    pub now: &'a str,
}
