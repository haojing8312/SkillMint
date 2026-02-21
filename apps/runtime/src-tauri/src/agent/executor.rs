use super::registry::ToolRegistry;
use super::types::{LLMResponse, ToolResult};
use crate::adapters;
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::sync::Arc;

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

    pub async fn execute_turn(
        &self,
        api_format: &str,
        base_url: &str,
        api_key: &str,
        model: &str,
        system_prompt: &str,
        mut messages: Vec<Value>,
        on_token: impl Fn(String) + Send + Clone,
    ) -> Result<Vec<Value>> {
        let mut iteration = 0;

        loop {
            if iteration >= self.max_iterations {
                return Err(anyhow!("达到最大迭代次数 {}", self.max_iterations));
            }
            iteration += 1;

            eprintln!("[agent] Iteration {}/{}", iteration, self.max_iterations);

            // 获取工具定义
            let tools = self.registry.get_tool_definitions();

            // 调用 LLM
            let response = if api_format == "anthropic" {
                adapters::anthropic::chat_stream_with_tools(
                    base_url,
                    api_key,
                    model,
                    system_prompt,
                    messages.clone(),
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
                    messages.clone(),
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

                        let result = match self.registry.get(&call.name) {
                            Some(tool) => match tool.execute(call.input.clone()) {
                                Ok(output) => output,
                                Err(e) => format!("工具执行错误: {}", e),
                            },
                            None => format!("工具不存在: {}", call.name),
                        };

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
