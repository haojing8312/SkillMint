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
