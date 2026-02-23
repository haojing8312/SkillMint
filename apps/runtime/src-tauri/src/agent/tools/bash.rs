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

    /// 检查命令是否包含危险操作模式
    fn is_dangerous(command: &str) -> bool {
        let lower = command.to_lowercase();
        let patterns = [
            "rm -rf /", "rm -rf /*", "rm -rf ~",
            "format c:", "format d:",
            "shutdown", "reboot",
            "> /dev/sda", "dd if=/dev/zero",
            ":(){ :|:& };:",
            "mkfs.", "wipefs",
        ];
        patterns.iter().any(|p| lower.contains(p))
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

        // 危险命令检查
        if Self::is_dangerous(command) {
            return Ok("错误: 危险命令已被拦截。此命令可能造成不可逆损害。".to_string());
        }

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
