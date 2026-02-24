use crate::agent::tools::process_manager::ProcessManager;
use crate::agent::types::{Tool, ToolContext};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::sync::Arc;

/// 获取后台进程输出的工具
pub struct BashOutputTool {
    process_manager: Arc<ProcessManager>,
}

impl BashOutputTool {
    pub fn new(process_manager: Arc<ProcessManager>) -> Self {
        Self { process_manager }
    }
}

impl Tool for BashOutputTool {
    fn name(&self) -> &str {
        "bash_output"
    }

    fn description(&self) -> &str {
        "获取后台进程的输出。可以选择阻塞等待进程完成或立即返回当前输出。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "process_id": {
                    "type": "string",
                    "description": "后台进程的 ID"
                },
                "block": {
                    "type": "boolean",
                    "description": "是否阻塞等待进程结束（默认 false）"
                }
            },
            "required": ["process_id"]
        })
    }

    fn execute(&self, input: Value, _ctx: &ToolContext) -> Result<String> {
        let process_id = input["process_id"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 process_id 参数"))?;

        let block = input["block"].as_bool().unwrap_or(false);

        let output = self.process_manager.get_output(process_id, block)?;

        let status = if output.exited {
            format!("已退出 (退出码: {})", output.exit_code.unwrap_or(-1))
        } else {
            "运行中".to_string()
        };

        Ok(format!(
            "状态: {}\nstdout:\n{}\nstderr:\n{}",
            status, output.stdout, output.stderr
        ))
    }
}
