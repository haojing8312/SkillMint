use runtime_lib::agent::tools::TodoWriteTool;
use runtime_lib::agent::types::{Tool, ToolContext};
use serde_json::json;

#[test]
fn test_todo_create_and_list() {
    let tool = TodoWriteTool::new();
    let ctx = ToolContext::default();

    let result = tool
        .execute(json!({
            "action": "create",
            "subject": "实现 Edit 工具",
            "description": "精确替换文本"
        }), &ctx)
        .unwrap();
    assert!(result.contains("已创建"));

    let result = tool.execute(json!({"action": "list"}), &ctx).unwrap();
    assert!(result.contains("实现 Edit 工具"));
    assert!(result.contains("pending"));
}

#[test]
fn test_todo_update_status() {
    let tool = TodoWriteTool::new();
    let ctx = ToolContext::default();

    let result = tool
        .execute(json!({
            "action": "create",
            "subject": "Test task"
        }), &ctx)
        .unwrap();
    let id = result.split("ID: ").nth(1).unwrap().trim();

    let result = tool
        .execute(json!({
            "action": "update",
            "id": id,
            "status": "in_progress"
        }), &ctx)
        .unwrap();
    assert!(result.contains("已更新"));

    let result = tool.execute(json!({"action": "list"}), &ctx).unwrap();
    assert!(result.contains("in_progress"));
}

#[test]
fn test_todo_delete() {
    let tool = TodoWriteTool::new();
    let ctx = ToolContext::default();

    let result = tool
        .execute(json!({
            "action": "create",
            "subject": "Will delete"
        }), &ctx)
        .unwrap();
    let id = result.split("ID: ").nth(1).unwrap().trim();

    let result = tool
        .execute(json!({
            "action": "delete",
            "id": id
        }), &ctx)
        .unwrap();
    assert!(result.contains("已删除"));

    let result = tool.execute(json!({"action": "list"}), &ctx).unwrap();
    assert!(!result.contains("Will delete"));
}

#[test]
fn test_todo_missing_action() {
    let tool = TodoWriteTool::new();
    let ctx = ToolContext::default();
    let result = tool.execute(json!({}), &ctx);
    assert!(result.is_err());
}

#[test]
fn test_todo_empty_list() {
    let tool = TodoWriteTool::new();
    let ctx = ToolContext::default();
    let result = tool.execute(json!({"action": "list"}), &ctx).unwrap();
    assert_eq!(result, "暂无任务");
}

#[test]
fn test_todo_delete_nonexistent() {
    let tool = TodoWriteTool::new();
    let ctx = ToolContext::default();
    let result = tool.execute(json!({"action": "delete", "id": "fake-id"}), &ctx);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("任务不存在"));
}
