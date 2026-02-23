use super::permissions::PermissionMode;
use super::registry::ToolRegistry;
use super::types::{LLMResponse, ToolResult};
use crate::adapters;
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

/// 单次工具输出允许的最大字符数
const MAX_TOOL_OUTPUT_CHARS: usize = 30_000;

/// 截断过长的工具输出
///
/// 当输出超过 `max_chars` 字符时，保留前 `max_chars` 个字符并附加截断提示信息。
pub fn truncate_tool_output(output: &str, max_chars: usize) -> String {
    if output.len() <= max_chars {
        return output.to_string();
    }
    let truncated: String = output.chars().take(max_chars).collect();
    format!(
        "{}\n\n[输出已截断，共 {} 字符，已显示前 {} 字符]",
        truncated,
        output.len(),
        max_chars
    )
}

const CHARS_PER_TOKEN: usize = 4;
const DEFAULT_TOKEN_BUDGET: usize = 100_000; // 约 400k 字符

/// 估算消息列表的 token 数（简单估算：字符数 / 4）
fn estimate_tokens(messages: &[Value]) -> usize {
    let total_chars: usize = messages
        .iter()
        .map(|m| {
            // 纯文本 content
            let text_len = m["content"].as_str().map_or(0, |s| s.len());
            // 数组型 content（如 tool_use / tool_result blocks）
            let array_len = m["content"].as_array().map_or(0, |arr| {
                arr.iter()
                    .map(|v| serde_json::to_string(v).map_or(0, |s| s.len()))
                    .sum()
            });
            text_len + array_len
        })
        .sum();
    total_chars / CHARS_PER_TOKEN
}

/// Layer 1 微压缩：替换旧的 tool_result 内容为占位符
///
/// 保留最近 `keep_recent` 条 tool_result 的完整内容，
/// 将更早的替换为 "[已执行]" 占位符。
/// 仅修改发送给 LLM 的副本，不影响原始数据。
///
/// 同时支持两种格式：
/// - Anthropic：`content` 数组中 `type == "tool_result"` 的条目
/// - OpenAI：`role == "tool"` 的消息
pub fn micro_compact(messages: &[Value], keep_recent: usize) -> Vec<Value> {
    // 找出所有包含 tool_result 的消息索引
    let tool_result_indices: Vec<usize> = messages
        .iter()
        .enumerate()
        .filter(|(_, m)| {
            // Anthropic: content 是数组且包含 tool_result
            m["content"].as_array().map_or(false, |arr| {
                arr.iter().any(|v| v["type"].as_str() == Some("tool_result"))
            })
            // OpenAI: role == "tool"
            || m["role"].as_str() == Some("tool")
        })
        .map(|(i, _)| i)
        .collect();

    if tool_result_indices.len() <= keep_recent {
        return messages.to_vec();
    }

    let cutoff = tool_result_indices.len() - keep_recent;
    let old_indices: std::collections::HashSet<usize> =
        tool_result_indices[..cutoff].iter().copied().collect();

    messages
        .iter()
        .enumerate()
        .map(|(i, m)| {
            if old_indices.contains(&i) {
                if m["role"].as_str() == Some("tool") {
                    // OpenAI 格式
                    json!({
                        "role": "tool",
                        "tool_call_id": m["tool_call_id"],
                        "content": "[已执行]"
                    })
                } else {
                    // Anthropic 格式：替换 content 数组中的 tool_result 条目
                    let replaced = m["content"].as_array().map(|arr| {
                        arr.iter()
                            .map(|v| {
                                if v["type"].as_str() == Some("tool_result") {
                                    json!({
                                        "type": "tool_result",
                                        "tool_use_id": v["tool_use_id"],
                                        "content": "[已执行]"
                                    })
                                } else {
                                    v.clone()
                                }
                            })
                            .collect::<Vec<_>>()
                    });
                    match replaced {
                        Some(arr) => json!({"role": "user", "content": arr}),
                        None => m.clone(),
                    }
                }
            } else {
                m.clone()
            }
        })
        .collect()
}

/// 裁剪消息列表到 token 预算内
/// 保留第一条消息和最后的消息，从第二条开始裁剪中间的
pub fn trim_messages(messages: &[Value], token_budget: usize) -> Vec<Value> {
    if messages.len() <= 2 || estimate_tokens(messages) <= token_budget {
        return messages.to_vec();
    }

    let first = &messages[0];
    let last = &messages[messages.len() - 1];

    // 从后往前累加保留的消息
    let budget_chars = token_budget * CHARS_PER_TOKEN * 70 / 100;
    let first_chars = first["content"].as_str().map_or(0, |s| s.len());
    let last_chars = last["content"].as_str().map_or(0, |s| s.len());
    let mut char_count = first_chars + last_chars;

    let mut keep_from_end: Vec<&Value> = Vec::new();

    for msg in messages[1..messages.len() - 1].iter().rev() {
        let msg_chars = msg["content"].as_str().map_or(0, |s| s.len())
            + msg["content"].as_array().map_or(0, |arr| {
                arr.iter()
                    .map(|v| serde_json::to_string(v).map_or(0, |s| s.len()))
                    .sum()
            });
        if char_count + msg_chars > budget_chars {
            break;
        }
        char_count += msg_chars;
        keep_from_end.push(msg);
    }
    keep_from_end.reverse();

    let trimmed_count = messages.len() - 2 - keep_from_end.len();
    let mut result = vec![first.clone()];

    if trimmed_count > 0 {
        result.push(json!({
            "role": "user",
            "content": format!("[前 {} 条消息已省略]", trimmed_count)
        }));
    }

    for msg in keep_from_end {
        result.push(msg.clone());
    }
    result.push(last.clone());

    result
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct ToolCallEvent {
    pub session_id: String,
    pub tool_name: String,
    pub tool_input: Value,
    pub tool_output: Option<String>,
    pub status: String, // "started" | "completed" | "error"
}

/// Agent 状态事件，用于前端展示当前执行阶段
#[derive(serde::Serialize, Clone, Debug)]
pub struct AgentStateEvent {
    pub session_id: String,
    /// 状态类型: "thinking" | "tool_calling" | "finished" | "error"
    pub state: String,
    /// 工具名列表（tool_calling 时）或错误信息（error 时）
    pub detail: Option<String>,
    pub iteration: usize,
}

pub struct AgentExecutor {
    registry: Arc<ToolRegistry>,
    max_iterations: usize,
}

impl AgentExecutor {
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self {
            registry,
            max_iterations: 10,
        }
    }

    pub fn with_max_iterations(registry: Arc<ToolRegistry>, max_iterations: usize) -> Self {
        Self {
            registry,
            max_iterations,
        }
    }

    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }

    pub fn registry_arc(&self) -> Arc<ToolRegistry> {
        Arc::clone(&self.registry)
    }

    pub async fn execute_turn(
        &self,
        api_format: &str,
        base_url: &str,
        api_key: &str,
        model: &str,
        system_prompt: &str,
        mut messages: Vec<Value>,
        on_token: impl Fn(String) + Send + Clone,
        app_handle: Option<&AppHandle>,
        session_id: Option<&str>,
        allowed_tools: Option<&[String]>,
        permission_mode: PermissionMode,
        tool_confirm_tx: Option<std::sync::Arc<std::sync::Mutex<Option<std::sync::mpsc::Sender<bool>>>>>,
    ) -> Result<Vec<Value>> {
        let mut iteration = 0;

        loop {
            if iteration >= self.max_iterations {
                // 发射 error 状态事件
                if let (Some(app), Some(sid)) = (app_handle, session_id) {
                    let _ = app.emit("agent-state-event", AgentStateEvent {
                        session_id: sid.to_string(),
                        state: "error".to_string(),
                        detail: Some(format!("达到最大迭代次数 {}", self.max_iterations)),
                        iteration,
                    });
                }
                return Err(anyhow!("达到最大迭代次数 {}", self.max_iterations));
            }
            iteration += 1;

            eprintln!("[agent] Iteration {}/{}", iteration, self.max_iterations);

            // 发射 thinking 状态事件
            if let (Some(app), Some(sid)) = (app_handle, session_id) {
                let _ = app.emit("agent-state-event", AgentStateEvent {
                    session_id: sid.to_string(),
                    state: "thinking".to_string(),
                    detail: None,
                    iteration,
                });
            }

            // 根据白名单过滤工具定义
            let tools = match allowed_tools {
                Some(whitelist) => self.registry.get_filtered_tool_definitions(whitelist),
                None => self.registry.get_tool_definitions(),
            };

            // 上下文压缩：Layer 1 微压缩 + token 预算裁剪
            let compacted = micro_compact(&messages, 3);
            let trimmed = trim_messages(&compacted, DEFAULT_TOKEN_BUDGET);

            // 调用 LLM
            let response = if api_format == "anthropic" {
                adapters::anthropic::chat_stream_with_tools(
                    base_url,
                    api_key,
                    model,
                    system_prompt,
                    trimmed.clone(),
                    tools,
                    on_token.clone(),
                )
                .await?
            } else {
                // OpenAI 兼容格式
                adapters::openai::chat_stream_with_tools(
                    base_url,
                    api_key,
                    model,
                    system_prompt,
                    trimmed.clone(),
                    tools,
                    on_token.clone(),
                )
                .await?
            };

            // 处理响应
            match response {
                LLMResponse::Text(content) => {
                    // 纯文本响应 - 结束循环
                    messages.push(json!({
                        "role": "assistant",
                        "content": content
                    }));
                    eprintln!("[agent] Finished with text response");

                    // 发射 finished 状态事件
                    if let (Some(app), Some(sid)) = (app_handle, session_id) {
                        let _ = app.emit("agent-state-event", AgentStateEvent {
                            session_id: sid.to_string(),
                            state: "finished".to_string(),
                            detail: None,
                            iteration,
                        });
                    }

                    return Ok(messages);
                }
                LLMResponse::ToolCalls(tool_calls) => {
                    eprintln!("[agent] Executing {} tool calls", tool_calls.len());

                    // 发射 tool_calling 状态事件
                    if let (Some(app), Some(sid)) = (app_handle, session_id) {
                        let tool_names: Vec<&str> = tool_calls.iter().map(|tc| tc.name.as_str()).collect();
                        let _ = app.emit("agent-state-event", AgentStateEvent {
                            session_id: sid.to_string(),
                            state: "tool_calling".to_string(),
                            detail: Some(tool_names.join(", ")),
                            iteration,
                        });
                    }

                    // 执行所有工具调用
                    let mut tool_results = vec![];
                    for call in &tool_calls {
                        eprintln!("[agent] Calling tool: {}", call.name);

                        // 发送工具开始事件
                        if let (Some(app), Some(sid)) = (app_handle, session_id) {
                            let _ = app.emit("tool-call-event", ToolCallEvent {
                                session_id: sid.to_string(),
                                tool_name: call.name.clone(),
                                tool_input: call.input.clone(),
                                tool_output: None,
                                status: "started".to_string(),
                            });
                        }

                        // 权限确认检查：在执行工具前判断是否需要用户确认
                        if permission_mode.needs_confirmation(&call.name) {
                            if let (Some(app), Some(sid)) = (app_handle, session_id) {
                                // 发射确认请求事件，前端弹出确认对话框
                                let _ = app.emit("tool-confirm-event", serde_json::json!({
                                    "session_id": sid,
                                    "tool_name": call.name,
                                    "tool_input": call.input,
                                }));

                                // 创建一次性通道并将发送端存入全局状态
                                let (tx, rx) = std::sync::mpsc::channel::<bool>();
                                if let Some(ref confirm_state) = tool_confirm_tx {
                                    if let Ok(mut guard) = confirm_state.lock() {
                                        *guard = Some(tx);
                                    }
                                }

                                // 阻塞等待用户确认（最多 300 秒），超时视为拒绝
                                let confirmed = rx
                                    .recv_timeout(std::time::Duration::from_secs(300))
                                    .unwrap_or(false);

                                // 清理发送端，避免下次误用
                                if let Some(ref confirm_state) = tool_confirm_tx {
                                    if let Ok(mut guard) = confirm_state.lock() {
                                        *guard = None;
                                    }
                                }

                                if !confirmed {
                                    // 用户拒绝 — 记录拒绝事件并跳过此工具
                                    let _ = app.emit("tool-call-event", ToolCallEvent {
                                        session_id: sid.to_string(),
                                        tool_name: call.name.clone(),
                                        tool_input: call.input.clone(),
                                        tool_output: Some("用户拒绝了此操作".to_string()),
                                        status: "error".to_string(),
                                    });
                                    tool_results.push(ToolResult {
                                        tool_use_id: call.id.clone(),
                                        content: "用户拒绝了此操作".to_string(),
                                    });
                                    continue;
                                }
                            }
                        }

                        let result = match self.registry.get(&call.name) {
                            Some(tool) => {
                                // 检查白名单：若设置了白名单但工具不在其中，拒绝执行
                                if let Some(whitelist) = allowed_tools {
                                    if !whitelist.iter().any(|w| w == &call.name) {
                                        format!("此 Skill 不允许使用工具: {}", call.name)
                                    } else {
                                        match tool.execute(call.input.clone()) {
                                            Ok(output) => output,
                                            Err(e) => format!("工具执行错误: {}", e),
                                        }
                                    }
                                } else {
                                    match tool.execute(call.input.clone()) {
                                        Ok(output) => output,
                                        Err(e) => format!("工具执行错误: {}", e),
                                    }
                                }
                            }
                            None => format!("工具不存在: {}", call.name),
                        };
                        // 截断过长的工具输出，防止超出上下文窗口
                        let result = truncate_tool_output(&result, MAX_TOOL_OUTPUT_CHARS);

                        // 发送工具完成事件
                        if let (Some(app), Some(sid)) = (app_handle, session_id) {
                            let _ = app.emit("tool-call-event", ToolCallEvent {
                                session_id: sid.to_string(),
                                tool_name: call.name.clone(),
                                tool_input: call.input.clone(),
                                tool_output: Some(result.clone()),
                                status: "completed".to_string(),
                            });
                        }

                        tool_results.push(ToolResult {
                            tool_use_id: call.id.clone(),
                            content: result,
                        });
                    }

                    // 添加工具调用和结果到消息历史
                    if api_format == "anthropic" {
                        // Anthropic 格式: assistant 消息包含 tool_use blocks
                        messages.push(json!({
                            "role": "assistant",
                            "content": tool_calls.iter().map(|tc| json!({
                                "type": "tool_use",
                                "id": tc.id,
                                "name": tc.name,
                                "input": tc.input,
                            })).collect::<Vec<_>>()
                        }));

                        // user 消息包含 tool_result blocks
                        messages.push(json!({
                            "role": "user",
                            "content": tool_results.iter().map(|tr| json!({
                                "type": "tool_result",
                                "tool_use_id": tr.tool_use_id,
                                "content": tr.content,
                            })).collect::<Vec<_>>()
                        }));
                    } else {
                        // OpenAI 格式
                        messages.push(json!({
                            "role": "assistant",
                            "tool_calls": tool_calls.iter().map(|tc| json!({
                                "id": tc.id,
                                "type": "function",
                                "function": {
                                    "name": tc.name,
                                    "arguments": serde_json::to_string(&tc.input).unwrap_or_default(),
                                }
                            })).collect::<Vec<_>>()
                        }));
                        // OpenAI: 每个工具结果是独立的 "tool" 角色消息
                        for tr in &tool_results {
                            messages.push(json!({
                                "role": "tool",
                                "tool_call_id": tr.tool_use_id,
                                "content": tr.content,
                            }));
                        }
                    }

                    // 继续下一轮迭代
                    continue;
                }
            }
        }
    }
}
