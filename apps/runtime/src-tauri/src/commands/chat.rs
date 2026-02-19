use tauri::{AppHandle, Emitter, State};
use serde_json::{json, Value};
use uuid::Uuid;
use chrono::Utc;
use super::skills::DbState;
use crate::adapters;

#[derive(serde::Serialize, Clone)]
struct StreamToken {
    session_id: String,
    token: String,
    done: bool,
}

#[tauri::command]
pub async fn create_session(
    skill_id: String,
    model_id: String,
    db: State<'_, DbState>,
) -> Result<String, String> {
    let session_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO sessions (id, skill_id, title, created_at, model_id) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&session_id)
    .bind(&skill_id)
    .bind("New Chat")
    .bind(&now)
    .bind(&model_id)
    .execute(&db.0)
    .await
    .map_err(|e| e.to_string())?;
    Ok(session_id)
}

#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    session_id: String,
    user_message: String,
    db: State<'_, DbState>,
) -> Result<(), String> {
    // Save user message
    let msg_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO messages (id, session_id, role, content, created_at) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&msg_id)
    .bind(&session_id)
    .bind("user")
    .bind(&user_message)
    .bind(&now)
    .execute(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    // Load session info
    let (skill_id, model_id) = sqlx::query_as::<_, (String, String)>(
        "SELECT skill_id, model_id FROM sessions WHERE id = ?"
    )
    .bind(&session_id)
    .fetch_one(&db.0)
    .await
    .map_err(|e| format!("会话不存在 (session_id={session_id}): {e}"))?;

    // Load skill with pack_path for re-unpack
    let (manifest_json, username, pack_path) = sqlx::query_as::<_, (String, String, String)>(
        "SELECT manifest, username, pack_path FROM installed_skills WHERE id = ?"
    )
    .bind(&skill_id)
    .fetch_one(&db.0)
    .await
    .map_err(|e| format!("Skill 不存在 (skill_id={skill_id}): {e}"))?;

    // Re-unpack to get actual SKILL.md content as system prompt
    let system_prompt = match skillpack_rs::verify_and_unpack(&pack_path, &username) {
        Ok(unpacked) => {
            String::from_utf8_lossy(
                unpacked.files.get("SKILL.md").map(|v| v.as_slice()).unwrap_or_default()
            ).to_string()
        }
        Err(_) => {
            // Fallback to manifest description if re-unpack fails
            let manifest: skillpack_rs::SkillManifest = serde_json::from_str(&manifest_json)
                .map_err(|e| e.to_string())?;
            manifest.description
        }
    };

    // Load message history
    let history = sqlx::query_as::<_, (String, String)>(
        "SELECT role, content FROM messages WHERE session_id = ? ORDER BY created_at ASC"
    )
    .bind(&session_id)
    .fetch_all(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    let messages: Vec<Value> = history.iter()
        .map(|(role, content)| json!({"role": role, "content": content}))
        .collect();

    // Load model config (including api_key from DB)
    let (api_format, base_url, model_name, api_key) = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT api_format, base_url, model_name, api_key FROM model_configs WHERE id = ?"
    )
    .bind(&model_id)
    .fetch_one(&db.0)
    .await
    .map_err(|e| format!("模型配置不存在 (model_id={model_id}): {e}"))?;

    if api_key.is_empty() {
        return Err(format!("模型 API Key 为空，请在设置中重新配置 (model_id={model_id})"));
    }

    // Stream tokens to frontend via Tauri event
    let app_clone = app.clone();
    let session_id_clone = session_id.clone();
    let mut full_response = String::new();

    let result = if api_format == "anthropic" {
        adapters::anthropic::chat_stream(
            &base_url,
            &api_key,
            &model_name,
            &system_prompt,
            messages,
            |token: String| {
                full_response.push_str(&token);
                let _ = app_clone.emit("stream-token", StreamToken {
                    session_id: session_id_clone.clone(),
                    token,
                    done: false,
                });
            },
        ).await
    } else {
        adapters::openai::chat_stream(
            &base_url,
            &api_key,
            &model_name,
            &system_prompt,
            messages,
            |token: String| {
                full_response.push_str(&token);
                let _ = app_clone.emit("stream-token", StreamToken {
                    session_id: session_id_clone.clone(),
                    token,
                    done: false,
                });
            },
        ).await
    };

    // Emit done event
    let _ = app.emit("stream-token", StreamToken {
        session_id: session_id.clone(),
        token: String::new(),
        done: true,
    });

    if let Err(e) = result {
        return Err(e.to_string());
    }

    // Save assistant message
    let asst_id = Uuid::new_v4().to_string();
    let now2 = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO messages (id, session_id, role, content, created_at) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&asst_id)
    .bind(&session_id)
    .bind("assistant")
    .bind(&full_response)
    .bind(&now2)
    .execute(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_messages(
    session_id: String,
    db: State<'_, DbState>,
) -> Result<Vec<serde_json::Value>, String> {
    let rows = sqlx::query_as::<_, (String, String, String)>(
        "SELECT role, content, created_at FROM messages WHERE session_id = ? ORDER BY created_at ASC"
    )
    .bind(&session_id)
    .fetch_all(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.iter().map(|(role, content, created_at)| {
        json!({"role": role, "content": content, "created_at": created_at})
    }).collect())
}
