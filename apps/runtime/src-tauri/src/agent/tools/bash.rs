use crate::agent::types::Tool;
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::process::{Command, Stdio};

pub struct BashTool;

impl BashTool {
    pub fn new() -> Self {
        Self
    }

    #[cfg(target_os = "windows")]
    fn get_shell() -> (&'static str, &'static str) {
        ("cmd", "/C")
    }

    #[cfg(not(target_os = "windows"))]
    fn get_shell() -> (&'static str, &'static str) {
        ("bash", "-c")
    }
}

impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "执行 shell 命令。Windows 使用 PowerShell，Unix 使用 bash。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "要执行的 shell 命令"
                },
                "timeout_ms": {
                    "type": "integer",
                    "description": "超时时间（毫秒，可选，默认 30000）"
                }
            },
            "required": ["command"]
        })
    }

    fn execute(&self, input: Value) -> Result<String> {
        let command = input["command"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 command 参数"))?;

        let (shell, flag) = Self::get_shell();

        let output = Command::new(shell)
            .arg(flag)
            .arg(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            Ok(format!(
                "命令执行失败（退出码 {}）\nstderr:\n{}",
                output.status.code().unwrap_or(-1),
                stderr
            ))
        } else {
            Ok(format!("stdout:\n{}\nstderr:\n{}", stdout, stderr))
        }
    }
}
