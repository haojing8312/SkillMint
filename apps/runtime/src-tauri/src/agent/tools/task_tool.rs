use crate::agent::types::Tool;
use crate::agent::{AgentExecutor, ToolRegistry};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::sync::Arc;

/// 子 Agent 分发工具
///
/// 允许主 Agent 将独立子任务分发给子 Agent 执行，子 Agent 拥有独立上下文。
/// 支持三种类型：
/// - `explore`：只读操作（read_file、glob、grep）
/// - `plan`：只读 + bash
/// - `general-purpose`：全部工具
pub struct TaskTool {
    registry: Arc<ToolRegistry>,
    api_format: String,
    base_url: String,
    api_key: String,
    model: String,
}

impl TaskTool {
    pub fn new(
        registry: Arc<ToolRegistry>,
        api_format: String,
        base_url: String,
        api_key: String,
        model: String,
    ) -> Self {
        Self {
            registry,
            api_format,
            base_url,
            api_key,
            model,
        }
    }

    /// explore 类型：只读工具列表
    pub fn get_explore_tools() -> Vec<String> {
        vec![
            "read_file".to_string(),
            "glob".to_string(),
            "grep".to_string(),
        ]
    }

    /// plan 类型：只读 + bash 工具列表
    pub fn get_plan_tools() -> Vec<String> {
        vec![
            "read_file".to_string(),
            "glob".to_string(),
            "grep".to_string(),
            "bash".to_string(),
        ]
    }
}

impl Tool for TaskTool {
    fn name(&self) -> &str {
        "task"
    }

    fn description(&self) -> &str {
        "分发子 Agent 执行独立任务。子 Agent 拥有独立上下文，完成后返回结果。支持 explore（只读）、plan（只读+bash）、general-purpose（全部工具）三种类型。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "子 Agent 的任务描述"
                },
                "agent_type": {
                    "type": "string",
                    "enum": ["general-purpose", "explore", "plan"],
                    "description": "子 Agent 类型（默认 general-purpose）"
                }
            },
            "required": ["prompt"]
        })
    }

    fn execute(&self, input: Value) -> Result<String> {
        let prompt = input["prompt"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 prompt 参数"))?
            .to_string();
        let agent_type = input["agent_type"]
            .as_str()
            .unwrap_or("general-purpose")
            .to_string();

        // 根据类型确定工具白名单和迭代限制
        let (allowed_tools, max_iter): (Option<Vec<String>>, usize) = match agent_type.as_str() {
            "explore" => (Some(Self::get_explore_tools()), 5),
            "plan" => (Some(Self::get_plan_tools()), 10),
            _ => (None, 10), // general-purpose: 全部工具
        };

        // 在闭包外保留副本，用于之后的格式化输出
        let agent_type_display = agent_type.clone();

        let registry = Arc::clone(&self.registry);
        let api_format = self.api_format.clone();
        let base_url = self.base_url.clone();
        let api_key = self.api_key.clone();
        let model = self.model.clone();

        // 必须在新线程中创建新的 tokio runtime，否则会死锁
        // （Tool::execute 是同步的，但被 async 上下文调用，不能用 block_on）
        let handle = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| anyhow!("创建运行时失败: {}", e))?;

            rt.block_on(async {
                let sub_executor = AgentExecutor::with_max_iterations(registry, max_iter);

                let system_prompt = format!(
                    "你是一个专注的子 Agent (类型: {})。完成以下任务后返回结果。简洁地报告你的发现。",
                    agent_type,
                );

                let messages = vec![json!({"role": "user", "content": prompt})];

                sub_executor
                    .execute_turn(
                        &api_format,
                        &base_url,
                        &api_key,
                        &model,
                        &system_prompt,
                        messages,
                        |_| {}, // 子 agent 不需要流式输出到前端
                        None,   // 无 app_handle
                        None,   // 无 session_id
                        allowed_tools.as_deref(),
                    )
                    .await
            })
        })
        .join()
        .map_err(|_| anyhow!("子 Agent 线程异常"))?;

        match handle {
            Ok(final_messages) => {
                // 提取最后一条 assistant 消息
                let last_text = final_messages
                    .iter()
                    .rev()
                    .find_map(|m| {
                        if m["role"].as_str() == Some("assistant") {
                            m["content"].as_str().map(String::from)
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "子 Agent 未返回文本结果".to_string());

                Ok(format!(
                    "子 Agent ({}) 执行完成:\n\n{}",
                    agent_type_display, last_text
                ))
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("最大迭代次数") {
                    Ok(format!(
                        "子 Agent ({}) 达到最大迭代次数 ({}):\n\n最后状态: 未完成",
                        agent_type_display, max_iter
                    ))
                } else {
                    Err(anyhow!("子 Agent 执行失败: {}", err_str))
                }
            }
        }
    }
}
