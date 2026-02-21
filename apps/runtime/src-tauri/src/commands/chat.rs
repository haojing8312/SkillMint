use tauri::{AppHandle, Emitter, State};
use serde_json::{json, Value};
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;
use super::skills::DbState;
use crate::agent::AgentExecutor;

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
    agent_executor: State<'_, Arc<AgentExecutor>>,
) -> Result<(), String> {
    // 保存用户消息
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

    // 如果是第一条消息，用消息前 20 个字符更新会话标题
    let msg_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM messages WHERE session_id = ?"
    )
    .bind(&session_id)
    .fetch_one(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    if msg_count.0 <= 1 {
        let title: String = user_message.chars().take(20).collect();
        sqlx::query("UPDATE sessions SET title = ? WHERE id = ?")
            .bind(&title)
            .bind(&session_id)
            .execute(&db.0)
            .await
            .map_err(|e| e.to_string())?;
    }

    // 加载会话信息
    let (skill_id, model_id) = sqlx::query_as::<_, (String, String)>(
        "SELECT skill_id, model_id FROM sessions WHERE id = ?"
    )
    .bind(&session_id)
    .fetch_one(&db.0)
    .await
    .map_err(|e| format!("会话不存在 (session_id={session_id}): {e}"))?;

    // 加载 Skill 信息（含 pack_path 用于重新解包）
    let (manifest_json, username, pack_path) = sqlx::query_as::<_, (String, String, String)>(
        "SELECT manifest, username, pack_path FROM installed_skills WHERE id = ?"
    )
    .bind(&skill_id)
    .fetch_one(&db.0)
    .await
    .map_err(|e| format!("Skill 不存在 (skill_id={skill_id}): {e}"))?;

    // 重新解包获取 SKILL.md 内容作为 system prompt
    let system_prompt = match skillpack_rs::verify_and_unpack(&pack_path, &username) {
        Ok(unpacked) => {
            String::from_utf8_lossy(
                unpacked.files.get("SKILL.md").map(|v| v.as_slice()).unwrap_or_default()
            ).to_string()
        }
        Err(_) => {
            // 解包失败时回退到 manifest 描述
            let manifest: skillpack_rs::SkillManifest = serde_json::from_str(&manifest_json)
                .map_err(|e| e.to_string())?;
            manifest.description
        }
    };

    // 加载消息历史
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

    // 加载模型配置（含 api_key）
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

    // 始终走 Agent 模式
    let app_clone = app.clone();
    let session_id_clone = session_id.clone();
    let final_messages = agent_executor
        .execute_turn(
            &api_format,
            &base_url,
            &api_key,
            &model_name,
            &system_prompt,
            messages,
            move |token: String| {
                let _ = app_clone.emit("stream-token", StreamToken {
                    session_id: session_id_clone.clone(),
                    token,
                    done: false,
                });
            },
            Some(&app),
            Some(&session_id),
        )
        .await
        .map_err(|e| e.to_string())?;

    // 发送结束事件
    let _ = app.emit("stream-token", StreamToken {
        session_id: session_id.clone(),
        token: String::new(),
        done: true,
    });

    // 从新消息中提取最终文本和 tool_calls，只保存一条 assistant 消息
    let new_messages: Vec<&Value> = final_messages.iter().skip(history.len()).collect();

    // 收集所有 tool_calls（来自 tool_use、tool_result、tool 角色的消息）
    let mut tool_calls: Vec<Value> = Vec::new();
    let mut final_text = String::new();

    for msg in &new_messages {
        let role = msg["role"].as_str().unwrap_or("");
        match role {
            "tool_use" | "tool_result" | "tool" => {
                // 中间工具消息：收集工具调用信息
                tool_calls.push((*msg).clone());
            }
            "assistant" => {
                // 取最后一条 assistant 纯文本消息
                if let Some(text) = msg["content"].as_str() {
                    final_text = text.to_string();
                }
            }
            _ => {}
        }
    }

    // 组装最终 content：有 tool_calls 时包裹为 JSON 对象，否则纯文本
    let content = if !tool_calls.is_empty() {
        serde_json::to_string(&json!({
            "text": final_text,
            "tool_calls": tool_calls,
        }))
        .unwrap_or(final_text.clone())
    } else {
        final_text.clone()
    };

    // 只有存在 assistant 回复时才保存
    if !final_text.is_empty() || !tool_calls.is_empty() {
        let msg_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO messages (id, session_id, role, content, created_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&msg_id)
        .bind(&session_id)
        .bind("assistant")
        .bind(&content)
        .bind(&now)
        .execute(&db.0)
        .await
        .map_err(|e| e.to_string())?;
    }

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
        // 对 assistant 消息尝试解析结构化 content
        if role == "assistant" {
            if let Ok(parsed) = serde_json::from_str::<Value>(content) {
                if let Some(text) = parsed.get("text") {
                    // 包含 text 字段，说明是带 tool_calls 的结构化消息
                    let tool_calls = parsed.get("tool_calls").cloned().unwrap_or(Value::Null);
                    return json!({
                        "role": role,
                        "content": text,
                        "created_at": created_at,
                        "tool_calls": tool_calls,
                    });
                }
            }
        }
        // 其他情况直接返回原始 content
        json!({"role": role, "content": content, "created_at": created_at})
    }).collect())
}
