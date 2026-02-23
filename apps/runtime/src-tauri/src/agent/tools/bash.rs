use crate::agent::types::Tool;
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::io::Read;
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

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
        "执行 shell 命令。Windows 使用 cmd，Unix 使用 bash。"
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
                    "description": "超时时间（毫秒，可选，默认 120000）"
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

        // 提取超时参数，默认 120 秒
        let timeout_ms = input["timeout_ms"].as_u64().unwrap_or(120_000);
        let timeout = Duration::from_millis(timeout_ms);

        let (shell, flag) = Self::get_shell();

        // 使用 spawn 启动子进程，以便后续进行超时控制
        let mut child = Command::new(shell)
            .arg(flag)
            .arg(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // 等待子进程完成或超时
        match child.wait_timeout(timeout)? {
            Some(status) => {
                // 子进程已正常退出，读取输出
                let mut stdout_str = String::new();
                let mut stderr_str = String::new();
                if let Some(mut out) = child.stdout.take() {
                    out.read_to_string(&mut stdout_str).ok();
                }
                if let Some(mut err) = child.stderr.take() {
                    err.read_to_string(&mut stderr_str).ok();
                }

                if !status.success() {
                    Ok(format!(
                        "命令执行失败（退出码 {}）\nstderr:\n{}",
                        status.code().unwrap_or(-1),
                        stderr_str
                    ))
                } else {
                    Ok(format!("stdout:\n{}\nstderr:\n{}", stdout_str, stderr_str))
                }
            }
            None => {
                // 超时：终止子进程
                let _ = child.kill();
                let _ = child.wait();
                Ok(format!("命令执行超时（{}ms），已终止", timeout_ms))
            }
        }
    }
}
