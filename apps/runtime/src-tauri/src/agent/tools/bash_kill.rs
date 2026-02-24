use crate::agent::tools::process_manager::ProcessManager;
use crate::agent::types::{Tool, ToolContext};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::sync::Arc;

/// 终止后台进程的工具
pub struct BashKillTool {
    process_manager: Arc<ProcessManager>,
}

impl BashKillTool {
    pub fn new(process_manager: Arc<ProcessManager>) -> Self {
        Self { process_manager }
    }
}

impl Tool for BashKillTool {
    fn name(&self) -> &str {
        "bash_kill"
    }

    fn description(&self) -> &str {
        "终止指定的后台进程。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "process_id": {
                    "type": "string",
                    "description": "要终止的后台进程 ID"
                }
            },
            "required": ["process_id"]
        })
    }

    fn execute(&self, input: Value, _ctx: &ToolContext) -> Result<String> {
        let process_id = input["process_id"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 process_id 参数"))?;

        self.process_manager.kill(process_id)?;

        Ok(format!("已终止进程 {}", process_id))
    }
}
