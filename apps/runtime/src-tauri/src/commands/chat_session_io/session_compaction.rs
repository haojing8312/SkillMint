use super::session_view::render_user_content_parts;
use crate::commands::chat_runtime_io::extract_assistant_text_content;
use chrono::Utc;
use serde_json::{json, Value};
use uuid::Uuid;

pub(crate) async fn load_compaction_inputs_with_pool(
    pool: &sqlx::SqlitePool,
    session_id: &str,
) -> Result<(Vec<Value>, String, String, String, String), String> {
    let rows = sqlx::query_as::<_, (String, String, Option<String>)>(
        "SELECT role, content, content_json FROM messages WHERE session_id = ? ORDER BY created_at ASC",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let messages: Vec<Value> = rows
        .iter()
        .map(|(role, content, content_json)| {
            let normalized_content = if role == "assistant" {
                extract_assistant_text_content(content)
            } else if let Some(parts_json) = content_json {
                render_user_content_parts(parts_json).unwrap_or_else(|| content.clone())
            } else {
                content.clone()
            };
            json!({ "role": role, "content": normalized_content })
        })
        .collect();

    let (model_id,): (String,) = sqlx::query_as("SELECT model_id FROM sessions WHERE id = ?")
        .bind(session_id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    let (api_format, base_url, api_key, model_name) =
        sqlx::query_as::<_, (String, String, String, String)>(
            "SELECT api_format, base_url, api_key, model_name FROM model_configs WHERE id = ?",
        )
        .bind(&model_id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok((messages, api_format, base_url, api_key, model_name))
}

pub(crate) async fn replace_messages_with_compacted_with_pool(
    pool: &sqlx::SqlitePool,
    session_id: &str,
    compacted: &[Value],
) -> Result<(), String> {
    sqlx::query("DELETE FROM messages WHERE session_id = ?")
        .bind(session_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    let now = Utc::now().to_rfc3339();
    for msg in compacted {
        sqlx::query(
            "INSERT INTO messages (id, session_id, role, content, content_json, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(session_id)
        .bind(msg["role"].as_str().unwrap_or("user"))
        .bind(msg["content"].as_str().unwrap_or(""))
        .bind(Option::<String>::None)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::load_compaction_inputs_with_pool;
    use crate::commands::chat_session_io::session_store::create_session_with_pool;
    use serde_json::json;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn load_compaction_inputs_with_pool_renders_user_content_parts() {
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
                permission_mode TEXT NOT NULL DEFAULT 'standard',
                work_dir TEXT NOT NULL DEFAULT '',
                employee_id TEXT NOT NULL DEFAULT '',
                session_mode TEXT NOT NULL DEFAULT 'general',
                team_id TEXT NOT NULL DEFAULT ''
            )",
        )
        .execute(&pool)
        .await
        .expect("create sessions table");

        sqlx::query(
            "CREATE TABLE messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                content_json TEXT,
                created_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("create messages table");

        sqlx::query(
            "CREATE TABLE model_configs (
                id TEXT PRIMARY KEY,
                api_format TEXT NOT NULL,
                base_url TEXT NOT NULL,
                api_key TEXT NOT NULL,
                model_name TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("create model_configs table");

        let session_id = create_session_with_pool(
            &pool,
            "skill-1".to_string(),
            "model-1".to_string(),
            Some("E:/workspace/chat-session".to_string()),
            None,
            Some("Compaction Source".to_string()),
            Some("standard".to_string()),
            Some("general".to_string()),
            None,
        )
        .await
        .expect("create session");

        sqlx::query(
            "INSERT INTO model_configs (id, api_format, base_url, api_key, model_name)
             VALUES ('model-1', 'openai', 'https://api.openai.com/v1', 'sk-test', 'gpt-4.1')",
        )
        .execute(&pool)
        .await
        .expect("insert model config");

        sqlx::query(
            "INSERT INTO messages (id, session_id, role, content, content_json, created_at)
             VALUES (?, ?, 'user', 'fallback text', ?, '2026-03-23T00:00:01Z')",
        )
        .bind("msg-attachment")
        .bind(&session_id)
        .bind(
            serde_json::to_string(&json!([
                { "type": "text", "text": "请结合附件分析" },
                { "type": "file_text", "name": "brief.md", "mimeType": "text/markdown", "text": "# brief" }
            ]))
            .expect("serialize content parts"),
        )
        .execute(&pool)
        .await
        .expect("insert message");

        let (messages, api_format, base_url, api_key, model_name) =
            load_compaction_inputs_with_pool(&pool, &session_id)
                .await
                .expect("load compaction inputs");

        assert_eq!(api_format, "openai");
        assert_eq!(base_url, "https://api.openai.com/v1");
        assert_eq!(api_key, "sk-test");
        assert_eq!(model_name, "gpt-4.1");
        assert_eq!(messages.len(), 1);
        assert!(messages[0]["content"]
            .as_str()
            .expect("content string")
            .contains("brief.md"));
        assert!(messages[0]["content"]
            .as_str()
            .expect("content string")
            .contains("请结合附件分析"));
    }
}
