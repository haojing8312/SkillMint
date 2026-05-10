use super::session_view::{
    derive_session_display_title_with_pool, im_thread_sessions_has_channel_column,
    normalize_stream_items, resolve_im_session_source,
};
use crate::commands::runtime_preferences::resolve_default_work_dir_with_pool;
use chrono::Utc;
use runtime_chat_app::{ChatPreparationService, SessionCreationRequest};
use serde_json::{json, Value};
use uuid::Uuid;

pub(crate) async fn create_session_with_pool(
    pool: &sqlx::SqlitePool,
    skill_id: String,
    model_id: String,
    work_dir: Option<String>,
    employee_id: Option<String>,
    title: Option<String>,
    permission_mode: Option<String>,
    session_mode: Option<String>,
    team_id: Option<String>,
) -> Result<String, String> {
    let session_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let prepared = ChatPreparationService::new().prepare_session_creation(SessionCreationRequest {
        permission_mode,
        session_mode,
        team_id,
        title,
        work_dir,
        employee_id,
    });
    let resolved_work_dir = if prepared.normalized_work_dir.is_empty() {
        resolve_default_work_dir_with_pool(pool).await?
    } else {
        prepared.normalized_work_dir
    };
    let has_profile_id_column = sessions_has_profile_id_column(pool).await?;
    let resolved_profile_id =
        if has_profile_id_column && !prepared.normalized_employee_id.trim().is_empty() {
            crate::profile_runtime::resolve_profile_for_alias_with_pool(
                pool,
                &prepared.normalized_employee_id,
            )
            .await?
            .map(|resolved| resolved.profile_id)
        } else {
            None
        };

    if has_profile_id_column {
        sqlx::query(
            "INSERT INTO sessions (id, skill_id, title, created_at, model_id, permission_mode, work_dir, employee_id, profile_id, session_mode, team_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&session_id)
        .bind(&skill_id)
        .bind(&prepared.normalized_title)
        .bind(&now)
        .bind(&model_id)
        .bind(&prepared.permission_mode_storage)
        .bind(&resolved_work_dir)
        .bind(&prepared.normalized_employee_id)
        .bind(&resolved_profile_id)
        .bind(&prepared.session_mode_storage)
        .bind(&prepared.normalized_team_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    } else {
        sqlx::query(
            "INSERT INTO sessions (id, skill_id, title, created_at, model_id, permission_mode, work_dir, employee_id, session_mode, team_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&session_id)
        .bind(&skill_id)
        .bind(&prepared.normalized_title)
        .bind(&now)
        .bind(&model_id)
        .bind(&prepared.permission_mode_storage)
        .bind(&resolved_work_dir)
        .bind(&prepared.normalized_employee_id)
        .bind(&prepared.session_mode_storage)
        .bind(&prepared.normalized_team_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }
    Ok(session_id)
}

async fn sessions_has_profile_id_column(pool: &sqlx::SqlitePool) -> Result<bool, String> {
    let columns: Vec<String> = sqlx::query_scalar("SELECT name FROM pragma_table_info('sessions')")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(columns.iter().any(|name| name == "profile_id"))
}

pub(crate) async fn get_messages_with_pool(
    pool: &sqlx::SqlitePool,
    session_id: &str,
) -> Result<Vec<Value>, String> {
    let rows = sqlx::query_as::<
        _,
        (
            String,
            String,
            String,
            Option<String>,
            String,
            Option<String>,
        ),
    >(
        "SELECT
            m.id,
            m.role,
            m.content,
            m.content_json,
            m.created_at,
            NULLIF(sr.id, '') AS run_id
         FROM messages m
         LEFT JOIN session_runs sr ON sr.assistant_message_id = m.id
         WHERE m.session_id = ?
         ORDER BY m.created_at ASC",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .iter()
        .map(|(id, role, content, content_json, created_at, run_id)| {
            if role == "assistant" {
                if let Ok(parsed) = serde_json::from_str::<Value>(content) {
                    if let Some(text) = parsed.get("text") {
                        let reasoning = parsed.get("reasoning").cloned().unwrap_or(Value::Null);
                        if let Some(items) = parsed.get("items") {
                            let normalized = normalize_stream_items(items);
                            return json!({
                                "id": id,
                                "role": role,
                                "content": text,
                                "created_at": created_at,
                                "runId": run_id,
                                "reasoning": reasoning,
                                "streamItems": normalized,
                            });
                        }
                        let tool_calls = parsed.get("tool_calls").cloned().unwrap_or(Value::Null);
                        return json!({
                            "id": id,
                            "role": role,
                            "content": text,
                            "created_at": created_at,
                            "runId": run_id,
                            "reasoning": reasoning,
                            "tool_calls": tool_calls,
                        });
                    }
                }
            }
            let mut message = json!({
                "id": id,
                "role": role,
                "content": content,
                "created_at": created_at,
                "runId": run_id,
            });
            if let Some(parts) = content_json
                .as_deref()
                .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
                .filter(|value| value.is_array())
            {
                message["contentParts"] = parts;
            }
            message
        })
        .collect())
}

pub(crate) async fn list_sessions_with_pool(
    pool: &sqlx::SqlitePool,
    permission_mode_label_for_display: fn(&str) -> &'static str,
) -> Result<Vec<Value>, String> {
    let im_source_channel_select = if im_thread_sessions_has_channel_column(pool).await {
        "COALESCE((
                SELECT ts.channel
                FROM im_thread_sessions ts
                WHERE ts.session_id = s.id
                ORDER BY ts.updated_at DESC, ts.created_at DESC
                LIMIT 1
            ), '') AS im_source_channel"
    } else {
        "'' AS im_source_channel"
    };
    let runtime_status_rows = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT
            s.id,
            (
                SELECT CASE
                    WHEN sr.status = 'waiting_approval' THEN 'waiting_approval'
                    WHEN sr.status IN ('thinking', 'tool_calling', 'waiting_user') THEN 'running'
                    WHEN sr.status = 'completed' THEN 'completed'
                    WHEN sr.status IN ('failed', 'cancelled') THEN 'failed'
                    ELSE NULL
                END
                FROM session_runs sr
                WHERE sr.session_id = s.id
                ORDER BY
                    CASE
                        WHEN sr.status = 'waiting_approval' THEN 0
                        WHEN sr.status IN ('thinking', 'tool_calling', 'waiting_user') THEN 1
                        ELSE 2
                    END,
                    sr.updated_at DESC,
                    sr.created_at DESC,
                    sr.id DESC
                LIMIT 1
            ) AS runtime_status
         FROM sessions s",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    let runtime_status_by_session_id = runtime_status_rows
        .into_iter()
        .map(|(session_id, runtime_status)| (session_id, runtime_status))
        .collect::<std::collections::HashMap<_, _>>();

    let rows_query = format!(
        "SELECT
            s.id,
            COALESCE(s.skill_id, ''),
            s.title,
            s.created_at,
            s.model_id,
            COALESCE(s.work_dir, ''),
            COALESCE(s.employee_id, ''),
            COALESCE(s.permission_mode, 'standard'),
            COALESCE(s.session_mode, 'general'),
            COALESCE(s.team_id, ''),
            {im_source_channel_select}
         FROM sessions s
         ORDER BY s.created_at DESC"
    );
    let rows = sqlx::query_as::<
        _,
        (
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        ),
    >(&rows_query)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut employee_name_by_code = std::collections::HashMap::<String, String>::new();
    let employee_rows = sqlx::query_as::<_, (String, String, String)>(
        "SELECT COALESCE(employee_id, ''), COALESCE(role_id, ''), COALESCE(name, '') FROM agent_employees",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    for (employee_id, role_id, name) in employee_rows {
        let trimmed_name = name.trim();
        if trimmed_name.is_empty() {
            continue;
        }
        let display_name = trimmed_name.to_string();
        if !employee_id.trim().is_empty() {
            employee_name_by_code.insert(employee_id.trim().to_string(), display_name.clone());
        }
        if !role_id.trim().is_empty() {
            employee_name_by_code.insert(role_id.trim().to_string(), display_name);
        }
    }

    let team_rows = sqlx::query_as::<_, (String, String)>(
        "SELECT COALESCE(id, ''), COALESCE(name, '') FROM employee_groups",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    let team_name_by_id = team_rows
        .into_iter()
        .filter_map(|(id, name)| {
            let id = id.trim().to_string();
            let name = name.trim().to_string();
            if id.is_empty() || name.is_empty() {
                None
            } else {
                Some((id, name))
            }
        })
        .collect::<std::collections::HashMap<_, _>>();

    let mut sessions = Vec::with_capacity(rows.len());
    for (
        id,
        skill_id,
        title,
        created_at,
        model_id,
        work_dir,
        employee_id,
        permission_mode,
        session_mode,
        team_id,
        im_source_channel,
    ) in rows
    {
        let title = title
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("New Chat")
            .to_string();
        let created_at = created_at.unwrap_or_default();
        let model_id = model_id.unwrap_or_default();
        let work_dir = work_dir.unwrap_or_default();
        let employee_id = employee_id.unwrap_or_default();
        let permission_mode = permission_mode.unwrap_or_else(|| "standard".to_string());
        let session_mode = session_mode.unwrap_or_else(|| "general".to_string());
        let team_id = team_id.unwrap_or_default();
        let im_source_channel = im_source_channel.unwrap_or_default();
        let employee_name = employee_name_by_code
            .get(employee_id.trim())
            .cloned()
            .unwrap_or_default();
        let (source_channel, source_label) = resolve_im_session_source(Some(&im_source_channel));
        let runtime_status = runtime_status_by_session_id.get(&id).cloned().flatten();
        let display_title = derive_session_display_title_with_pool(
            pool,
            &id,
            &title,
            &session_mode,
            &employee_id,
            &team_id,
            &employee_name_by_code,
            &team_name_by_id,
        )
        .await;
        sessions.push(json!({
            "id": id,
            "skill_id": skill_id,
            "title": title,
            "display_title": display_title,
            "created_at": created_at,
            "model_id": model_id,
            "work_dir": work_dir,
            "employee_id": employee_id,
            "employee_name": employee_name,
            "permission_mode": permission_mode,
            "session_mode": session_mode,
            "team_id": team_id,
            "permission_mode_label": permission_mode_label_for_display(&permission_mode),
            "source_channel": source_channel,
            "source_label": source_label,
            "runtime_status": runtime_status,
        }));
    }

    Ok(sessions)
}

pub(crate) async fn update_session_workspace_with_pool(
    pool: &sqlx::SqlitePool,
    session_id: &str,
    workspace: &str,
) -> Result<(), String> {
    sqlx::query("UPDATE sessions SET work_dir = ? WHERE id = ?")
        .bind(workspace)
        .bind(session_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn delete_session_with_pool(
    pool: &sqlx::SqlitePool,
    session_id: &str,
) -> Result<(), String> {
    sqlx::query("DELETE FROM messages WHERE session_id = ?")
        .bind(session_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(session_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub(crate) async fn search_sessions_global_with_pool(
    pool: &sqlx::SqlitePool,
    query: &str,
) -> Result<Vec<Value>, String> {
    let im_source_channel_select = if im_thread_sessions_has_channel_column(pool).await {
        "COALESCE((
                SELECT ts.channel
                FROM im_thread_sessions ts
                WHERE ts.session_id = s.id
                ORDER BY ts.updated_at DESC, ts.created_at DESC
                LIMIT 1
            ), '') AS im_source_channel"
    } else {
        "'' AS im_source_channel"
    };
    let pattern = format!("%{}%", query);
    let rows_query = format!(
        "SELECT DISTINCT
            s.id,
            COALESCE(s.skill_id, ''),
            s.title,
            s.created_at,
            s.model_id,
            COALESCE(s.work_dir, ''),
            COALESCE(s.employee_id, ''),
            {im_source_channel_select}
         FROM sessions s
         LEFT JOIN messages m ON m.session_id = s.id
         WHERE (s.title LIKE ? OR m.content LIKE ?)
         ORDER BY s.created_at DESC"
    );
    let rows = sqlx::query_as::<
        _,
        (
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
        ),
    >(&rows_query)
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .iter()
        .map(
            |(
                id,
                skill_id,
                title,
                created_at,
                model_id,
                work_dir,
                employee_id,
                im_source_channel,
            )| {
                let (source_channel, source_label) =
                    resolve_im_session_source(Some(im_source_channel));
                json!({
                    "id": id,
                    "skill_id": skill_id,
                    "title": title,
                    "display_title": title,
                    "created_at": created_at,
                    "model_id": model_id,
                    "work_dir": work_dir,
                    "employee_id": employee_id,
                    "source_channel": source_channel,
                    "source_label": source_label
                })
            },
        )
        .collect())
}

#[cfg(test)]
mod profile_tests {
    use super::create_session_with_pool;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_pool() -> sqlx::SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("create sqlite memory pool");

        sqlx::query(
            "CREATE TABLE sessions (
                id TEXT PRIMARY KEY,
                skill_id TEXT NOT NULL,
                title TEXT,
                created_at TEXT NOT NULL,
                model_id TEXT NOT NULL,
                permission_mode TEXT NOT NULL DEFAULT 'accept_edits',
                work_dir TEXT NOT NULL DEFAULT '',
                employee_id TEXT NOT NULL DEFAULT '',
                profile_id TEXT,
                session_mode TEXT NOT NULL DEFAULT 'general',
                team_id TEXT NOT NULL DEFAULT ''
            )",
        )
        .execute(&pool)
        .await
        .expect("create sessions");

        sqlx::query("CREATE TABLE app_settings (key TEXT PRIMARY KEY, value TEXT NOT NULL)")
            .execute(&pool)
            .await
            .expect("create app_settings");

        sqlx::query(
            "CREATE TABLE agent_employees (
                id TEXT PRIMARY KEY,
                employee_id TEXT NOT NULL DEFAULT '',
                role_id TEXT NOT NULL DEFAULT '',
                openclaw_agent_id TEXT NOT NULL DEFAULT '',
                name TEXT NOT NULL DEFAULT ''
            )",
        )
        .execute(&pool)
        .await
        .expect("create agent_employees");

        sqlx::query(
            "CREATE TABLE agent_profiles (
                id TEXT PRIMARY KEY,
                legacy_employee_row_id TEXT NOT NULL DEFAULT '',
                display_name TEXT NOT NULL DEFAULT '',
                route_aliases_json TEXT NOT NULL DEFAULT '[]',
                profile_home TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("create agent_profiles");

        sqlx::query(
            "INSERT INTO agent_employees (id, employee_id, role_id, openclaw_agent_id, name)
             VALUES ('employee-row-1', 'planner', 'planner-role', 'oc-planner', 'Planner')",
        )
        .execute(&pool)
        .await
        .expect("seed employee");

        sqlx::query(
            "INSERT INTO agent_profiles (id, legacy_employee_row_id, display_name, created_at, updated_at)
             VALUES ('profile-1', 'employee-row-1', 'Planner', '2026-05-06T00:00:00Z', '2026-05-06T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("seed profile");

        pool
    }

    #[tokio::test]
    async fn create_session_stores_profile_id_when_employee_alias_resolves() {
        let pool = setup_pool().await;

        let session_id = create_session_with_pool(
            &pool,
            "builtin-general".to_string(),
            "model-1".to_string(),
            Some("D:/work".to_string()),
            Some("planner".to_string()),
            Some("Task".to_string()),
            None,
            Some("employee_direct".to_string()),
            None,
        )
        .await
        .expect("create session");

        let row: (String, String) = sqlx::query_as(
            "SELECT employee_id, COALESCE(profile_id, '') FROM sessions WHERE id = ?",
        )
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .expect("query session");

        assert_eq!(row.0, "planner");
        assert_eq!(row.1, "profile-1");
    }
}
