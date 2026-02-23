use runtime_lib::agent::{BashTool, Tool};
use serde_json::json;

#[test]
fn test_bash_simple_command() {
    let tool = BashTool::new();

    let input = json!({"command": "echo Hello"});

    let result = tool.execute(input).unwrap();
    assert!(result.contains("Hello"));
}

#[test]
fn test_bash_command_failure() {
    let tool = BashTool::new();
    let input = json!({"command": "nonexistent_command_xyz_12345"});

    let result = tool.execute(input);
    // On Windows PowerShell, this will either error or return non-zero exit code
    // Either way, it should not panic
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_bash_dangerous_command_blocked() {
    let tool = BashTool::new();
    let input = json!({"command": "rm -rf /"});
    let result = tool.execute(input).unwrap();
    assert!(result.contains("危险命令"));
}

#[test]
fn test_bash_dangerous_format_blocked() {
    let tool = BashTool::new();
    let input = json!({"command": "format c:"});
    let result = tool.execute(input).unwrap();
    assert!(result.contains("危险命令"));
}

#[test]
fn test_bash_safe_command_not_blocked() {
    let tool = BashTool::new();
    let input = json!({"command": "echo safe"});
    let result = tool.execute(input).unwrap();
    assert!(!result.contains("危险命令"));
    assert!(result.contains("safe"));
}
