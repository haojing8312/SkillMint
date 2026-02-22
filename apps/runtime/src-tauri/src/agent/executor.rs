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
    ) -> Result<Vec<Value>> {
        let mut iteration = 0;

        loop {
            if iteration >= self.max_iterations {
                return Err(anyhow!("达到最大迭代次数 {}", self.max_iterations));
            }
            iteration += 1;

            eprintln!("[agent] Iteration {}/{}", iteration, self.max_iterations);

            // 根据白名单过滤工具定义
            let tools = match allowed_tools {
                Some(whitelist) => self.registry.get_filtered_tool_definitions(whitelist),
                None => self.registry.get_tool_definitions(),
            };

            // 上下文裁剪：将传给 LLM 的消息裁剪到 token 预算内
            let trimmed = trim_messages(&messages, DEFAULT_TOKEN_BUDGET);

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
                    return Ok(messages);
                }
                LLMResponse::ToolCalls(tool_calls) => {
                    eprintln!("[agent] Executing {} tool calls", tool_calls.len());

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
