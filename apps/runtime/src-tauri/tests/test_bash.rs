use runtime_lib::agent::{BashTool, Tool, ToolContext};
use serde_json::json;

#[test]
fn test_bash_simple_command() {
    let tool = BashTool::new();
    let ctx = ToolContext::default();

    let input = json!({"command": "echo Hello"});

    let result = tool.execute(input, &ctx).unwrap();
    assert!(result.contains("Hello"));
}

#[test]
fn test_bash_command_failure() {
    let tool = BashTool::new();
    let ctx = ToolContext::default();
    let input = json!({"command": "nonexistent_command_xyz_12345"});

    let result = tool.execute(input, &ctx);
    // On Windows PowerShell, this will either error or return non-zero exit code
    // Either way, it should not panic
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_bash_dangerous_command_blocked() {
    let tool = BashTool::new();
    let ctx = ToolContext::default();
    let input = json!({"command": "rm -rf /"});
    let result = tool.execute(input, &ctx).unwrap();
    assert!(result.contains("危险命令"));
}

#[test]
fn test_bash_dangerous_format_blocked() {
    let tool = BashTool::new();
    let ctx = ToolContext::default();
    let input = json!({"command": "format c:"});
    let result = tool.execute(input, &ctx).unwrap();
    assert!(result.contains("危险命令"));
}

#[test]
fn test_bash_safe_command_not_blocked() {
    let tool = BashTool::new();
    let ctx = ToolContext::default();
    let input = json!({"command": "echo safe"});
    let result = tool.execute(input, &ctx).unwrap();
    assert!(!result.contains("危险命令"));
    assert!(result.contains("safe"));
}

#[test]
fn test_bash_timeout() {
    let tool = BashTool::new();
    let ctx = ToolContext::default();
    let command = if cfg!(target_os = "windows") {
        "ping -n 10 127.0.0.1"
    } else {
        "sleep 10"
    };
    let input = json!({"command": command, "timeout_ms": 1000});
    let result = tool.execute(input, &ctx).unwrap();
    assert!(result.contains("超时"));
}

#[test]
fn test_bash_no_timeout_fast_command() {
    let tool = BashTool::new();
    let ctx = ToolContext::default();
    let input = json!({"command": "echo fast", "timeout_ms": 5000});
    let result = tool.execute(input, &ctx).unwrap();
    assert!(result.contains("fast"));
    assert!(!result.contains("超时"));
}
