use tauri::{AppHandle, Emitter, Manager, State};
use serde_json::{json, Value};
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;
use super::skills::DbState;
use crate::agent::AgentExecutor;
use crate::agent::permissions::PermissionMode;
use crate::agent::tools::{
    CompactTool, TaskTool, MemoryTool, WebSearchTool, AskUserTool, AskUserResponder, new_responder,
};

/// 全局 AskUser 响应通道（用于 answer_user_question command）
pub struct AskUserState(pub AskUserResponder);

/// 工具确认通道（用于 confirm_tool_execution command）
pub type ToolConfirmResponder = std::sync::Arc<std::sync::Mutex<Option<std::sync::mpsc::Sender<bool>>>>;
pub struct ToolConfirmState(pub ToolConfirmResponder);

#[derive(serde::Serialize, Clone)]
struct StreamToken {
    session_id: String,
    token: String,
    done: bool,
    #[serde(default)]
    sub_agent: bool,
}

#[tauri::command]
pub async fn create_session(
    skill_id: String,
    model_id: String,
    work_dir: String,
    db: State<'_, DbState>,
) -> Result<String, String> {
    let session_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO sessions (id, skill_id, title, created_at, model_id, work_dir) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&session_id)
    .bind(&skill_id)
    .bind("New Chat")
    .bind(&now)
    .bind(&model_id)
    .bind(&work_dir)
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

    // 加载会话信息（含权限模式和工作目录）
    let (skill_id, model_id, perm_str, work_dir) = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT skill_id, model_id, permission_mode, COALESCE(work_dir, '') FROM sessions WHERE id = ?"
    )
    .bind(&session_id)
    .fetch_one(&db.0)
    .await
    .map_err(|e| format!("会话不存在 (session_id={session_id}): {e}"))?;

    let permission_mode = match perm_str.as_str() {
        "accept_edits" => PermissionMode::AcceptEdits,
        "unrestricted" => PermissionMode::Unrestricted,
        _ => PermissionMode::Default,
    };

    // 加载 Skill 信息（含 pack_path 和 source_type，用 COALESCE 兼容旧数据）
    let (manifest_json, username, pack_path, source_type) = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT manifest, username, pack_path, COALESCE(source_type, 'encrypted') FROM installed_skills WHERE id = ?"
    )
    .bind(&skill_id)
    .fetch_one(&db.0)
    .await
    .map_err(|e| format!("Skill 不存在 (skill_id={skill_id}): {e}"))?;

    // 根据 source_type 决定如何读取 SKILL.md 内容
    let raw_prompt = if source_type == "builtin" {
        // 内置 Skill：使用硬编码的 system prompt
        "你是一个通用 AI 助手。你可以：\n\
        - 读取和编写文件\n\
        - 在终端中执行命令\n\
        - 搜索文件和代码\n\
        - 搜索网页获取信息\n\
        - 管理记忆和上下文\n\n\
        请根据用户的需求，自主分析、规划和执行任务。\n\
        工作目录为用户指定的目录，所有文件操作限制在该目录范围内。".to_string()
    } else if source_type == "local" {
        // 本地 Skill：直接从目录读取 SKILL.md
        let skill_md_path = std::path::Path::new(&pack_path).join("SKILL.md");
        std::fs::read_to_string(&skill_md_path)
            .unwrap_or_else(|_| {
                // 读取失败时回退到 manifest 描述
                serde_json::from_str::<skillpack_rs::SkillManifest>(&manifest_json)
                    .map(|m| m.description)
                    .unwrap_or_default()
            })
    } else {
        // 加密 Skill：重新解包获取 SKILL.md
        match skillpack_rs::verify_and_unpack(&pack_path, &username) {
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

    // 解析 Skill 元数据（frontmatter + system prompt）
    let skill_config = crate::agent::skill_config::SkillConfig::parse(&raw_prompt);

    // 确定工具白名单
    let allowed_tools = skill_config.allowed_tools.clone();

    // 获取工具名称列表用于模板渲染
    let tool_names = match &allowed_tools {
        Some(whitelist) => whitelist.join(", "),
        None => agent_executor
            .registry()
            .get_tool_definitions()
            .iter()
            .filter_map(|t| t["name"].as_str().map(String::from))
            .collect::<Vec<_>>()
            .join(", "),
    };

    let max_iter = skill_config.max_iterations.unwrap_or(10);

    // 构建完整 system prompt（含运行环境信息）
    let system_prompt = if work_dir.is_empty() {
        format!(
            "{}\n\n---\n运行环境:\n- 可用工具: {}\n- 模型: {}\n- 最大迭代次数: {}",
            skill_config.system_prompt, tool_names, model_name, max_iter,
        )
    } else {
        format!(
            "{}\n\n---\n运行环境:\n- 工作目录: {}\n- 可用工具: {}\n- 模型: {}\n- 最大迭代次数: {}\n\n注意: 所有文件操作必须限制在工作目录范围内。",
            skill_config.system_prompt, work_dir, tool_names, model_name, max_iter,
        )
    };

    // 动态注册运行时工具
    let task_tool = TaskTool::new(
        agent_executor.registry_arc(),
        api_format.clone(),
        base_url.clone(),
        api_key.clone(),
        model_name.clone(),
    )
    .with_app_handle(app.clone(), session_id.clone());
    agent_executor.registry().register(Arc::new(task_tool));

    // 注册 WebSearch 工具（通过 Sidecar 代理）
    let web_search = WebSearchTool::new("http://localhost:8765".to_string());
    agent_executor.registry().register(Arc::new(web_search));

    // 注册 Memory 工具（基于 Skill ID 的持久存储）
    let app_data_dir = app.path().app_data_dir().unwrap_or_default();
    let memory_dir = app_data_dir.join("memory").join(&skill_id);
    let memory_tool = MemoryTool::new(memory_dir.clone());
    agent_executor.registry().register(Arc::new(memory_tool));

    // 注册 Compact 工具（手动触发上下文压缩）
    let compact_tool = CompactTool::new();
    agent_executor.registry().register(Arc::new(compact_tool));

    // 注册 AskUser 工具（交互式问答）
    let ask_user_responder = new_responder();
    let ask_user_tool = AskUserTool::new(
        app.clone(),
        session_id.clone(),
        ask_user_responder.clone(),
    );
    agent_executor.registry().register(Arc::new(ask_user_tool));

    // 将 responder 存入全局状态，供 answer_user_question command 使用
    app.manage(AskUserState(ask_user_responder));

    // 如果存在 MEMORY.md，注入到 system prompt
    let memory_content = {
        let memory_file = memory_dir.join("MEMORY.md");
        if memory_file.exists() {
            std::fs::read_to_string(&memory_file).unwrap_or_default()
        } else {
            String::new()
        }
    };
    let system_prompt = if memory_content.is_empty() {
        system_prompt
    } else {
        format!("{}\n\n---\n持久内存:\n{}", system_prompt, memory_content)
    };

    // 创建工具确认通道，存入全局状态供 confirm_tool_execution command 使用
    let tool_confirm_responder: ToolConfirmResponder =
        std::sync::Arc::new(std::sync::Mutex::new(None));
    app.manage(ToolConfirmState(tool_confirm_responder.clone()));

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
                    sub_agent: false,
                });
            },
            Some(&app),
            Some(&session_id),
            allowed_tools.as_deref(),
            permission_mode,
            Some(tool_confirm_responder.clone()),
            if work_dir.is_empty() { None } else { Some(work_dir.clone()) },
        )
        .await
        .map_err(|e| e.to_string())?;

    // 发送结束事件
    let _ = app.emit("stream-token", StreamToken {
        session_id: session_id.clone(),
        token: String::new(),
        done: true,
        sub_agent: false,
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

#[tauri::command]
pub async fn get_sessions(
    skill_id: String,
    db: State<'_, DbState>,
) -> Result<Vec<serde_json::Value>, String> {
    let rows = sqlx::query_as::<_, (String, String, String, String, String)>(
        "SELECT id, title, created_at, model_id, COALESCE(work_dir, '') FROM sessions WHERE skill_id = ? ORDER BY created_at DESC"
    )
    .bind(&skill_id)
    .fetch_all(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.iter().map(|(id, title, created_at, model_id, work_dir)| {
        json!({
            "id": id,
            "title": title,
            "created_at": created_at,
            "model_id": model_id,
            "work_dir": work_dir,
        })
    }).collect())
}

#[tauri::command]
pub async fn delete_session(
    session_id: String,
    db: State<'_, DbState>,
) -> Result<(), String> {
    // 先删除该会话下的所有消息
    sqlx::query("DELETE FROM messages WHERE session_id = ?")
        .bind(&session_id)
        .execute(&db.0)
        .await
        .map_err(|e| e.to_string())?;

    // 再删除会话本身
    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(&session_id)
        .execute(&db.0)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// 搜索会话标题和消息内容
#[tauri::command]
pub async fn search_sessions(
    skill_id: String,
    query: String,
    db: State<'_, DbState>,
) -> Result<Vec<serde_json::Value>, String> {
    let pattern = format!("%{}%", query);
    let rows = sqlx::query_as::<_, (String, String, String, String, String)>(
        "SELECT DISTINCT s.id, s.title, s.created_at, s.model_id, COALESCE(s.work_dir, '')
         FROM sessions s
         LEFT JOIN messages m ON m.session_id = s.id
         WHERE s.skill_id = ? AND (s.title LIKE ? OR m.content LIKE ?)
         ORDER BY s.created_at DESC"
    )
    .bind(&skill_id)
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.iter().map(|(id, title, created_at, model_id, work_dir)| {
        json!({"id": id, "title": title, "created_at": created_at, "model_id": model_id, "work_dir": work_dir})
    }).collect())
}

/// 将会话消息导出为 Markdown 字符串
#[tauri::command]
pub async fn export_session(
    session_id: String,
    db: State<'_, DbState>,
) -> Result<String, String> {
    let (title,): (String,) = sqlx::query_as("SELECT title FROM sessions WHERE id = ?")
        .bind(&session_id)
        .fetch_one(&db.0)
        .await
        .map_err(|e| e.to_string())?;

    let messages = sqlx::query_as::<_, (String, String, String)>(
        "SELECT role, content, created_at FROM messages WHERE session_id = ? ORDER BY created_at ASC"
    )
    .bind(&session_id)
    .fetch_all(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    let mut md = format!("# {}\n\n", title);
    for (role, content, created_at) in &messages {
        let label = if role == "user" { "用户" } else { "助手" };
        md.push_str(&format!("## {} ({})\n\n{}\n\n---\n\n", label, created_at, content));
    }
    Ok(md)
}

/// 写入导出文件
#[tauri::command]
pub async fn write_export_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, &content).map_err(|e| format!("写入失败: {}", e))
}

/// 用户回答 AskUser 工具的问题
#[tauri::command]
pub async fn answer_user_question(
    answer: String,
    ask_user_state: State<'_, AskUserState>,
) -> Result<(), String> {
    let guard = ask_user_state
        .0
        .lock()
        .map_err(|e| format!("锁获取失败: {}", e))?;

    if let Some(sender) = guard.as_ref() {
        sender
            .send(answer)
            .map_err(|e| format!("发送响应失败: {}", e))?;
        Ok(())
    } else {
        Err("没有等待中的用户问题".to_string())
    }
}

/// 用户确认或拒绝工具执行
#[tauri::command]
pub async fn confirm_tool_execution(
    confirmed: bool,
    tool_confirm_state: State<'_, ToolConfirmState>,
) -> Result<(), String> {
    let guard = tool_confirm_state
        .0
        .lock()
        .map_err(|e| format!("锁获取失败: {}", e))?;
    if let Some(sender) = guard.as_ref() {
        sender
            .send(confirmed)
            .map_err(|e| format!("发送确认失败: {}", e))?;
        Ok(())
    } else {
        Err("没有等待中的工具确认请求".to_string())
    }
}
