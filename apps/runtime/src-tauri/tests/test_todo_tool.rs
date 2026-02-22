use runtime_lib::agent::tools::TodoWriteTool;
use runtime_lib::agent::types::Tool;
use serde_json::json;

#[test]
fn test_todo_create_and_list() {
    let tool = TodoWriteTool::new();

    let result = tool
        .execute(json!({
            "action": "create",
            "subject": "实现 Edit 工具",
            "description": "精确替换文本"
        }))
        .unwrap();
    assert!(result.contains("已创建"));

    let result = tool.execute(json!({"action": "list"})).unwrap();
    assert!(result.contains("实现 Edit 工具"));
    assert!(result.contains("pending"));
}

#[test]
fn test_todo_update_status() {
    let tool = TodoWriteTool::new();

    let result = tool
        .execute(json!({
            "action": "create",
            "subject": "Test task"
        }))
        .unwrap();
    let id = result.split("ID: ").nth(1).unwrap().trim();

    let result = tool
        .execute(json!({
            "action": "update",
            "id": id,
            "status": "in_progress"
        }))
        .unwrap();
    assert!(result.contains("已更新"));

    let result = tool.execute(json!({"action": "list"})).unwrap();
    assert!(result.contains("in_progress"));
}

#[test]
fn test_todo_delete() {
    let tool = TodoWriteTool::new();

    let result = tool
        .execute(json!({
            "action": "create",
            "subject": "Will delete"
        }))
        .unwrap();
    let id = result.split("ID: ").nth(1).unwrap().trim();

    let result = tool
        .execute(json!({
            "action": "delete",
            "id": id
        }))
        .unwrap();
    assert!(result.contains("已删除"));

    let result = tool.execute(json!({"action": "list"})).unwrap();
    assert!(!result.contains("Will delete"));
}

#[test]
fn test_todo_missing_action() {
    let tool = TodoWriteTool::new();
    let result = tool.execute(json!({}));
    assert!(result.is_err());
}

#[test]
fn test_todo_empty_list() {
    let tool = TodoWriteTool::new();
    let result = tool.execute(json!({"action": "list"})).unwrap();
    assert_eq!(result, "暂无任务");
}

#[test]
fn test_todo_delete_nonexistent() {
    let tool = TodoWriteTool::new();
    let result = tool.execute(json!({"action": "delete", "id": "fake-id"}));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("任务不存在"));
}
