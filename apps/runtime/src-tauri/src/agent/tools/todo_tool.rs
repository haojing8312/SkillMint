use crate::agent::types::Tool;
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::sync::RwLock;
use uuid::Uuid;

/// 单个任务条目
#[derive(Clone, Debug)]
struct TodoItem {
    id: String,
    subject: String,
    description: String,
    /// 任务状态：pending | in_progress | completed
    status: String,
}

/// TodoWrite 工具：管理 Agent 执行过程中的任务列表
pub struct TodoWriteTool {
    items: RwLock<Vec<TodoItem>>,
}

impl TodoWriteTool {
    pub fn new() -> Self {
        Self {
            items: RwLock::new(Vec::new()),
        }
    }
}

impl Tool for TodoWriteTool {
    fn name(&self) -> &str {
        "todo_write"
    }

    fn description(&self) -> &str {
        "管理任务列表。支持 create/update/list/delete 操作。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["create", "update", "list", "delete"],
                    "description": "操作类型"
                },
                "id": {
                    "type": "string",
                    "description": "任务 ID（update/delete 时必填）"
                },
                "subject": {
                    "type": "string",
                    "description": "任务标题（create 时必填）"
                },
                "description": {
                    "type": "string",
                    "description": "任务描述（可选）"
                },
                "status": {
                    "type": "string",
                    "enum": ["pending", "in_progress", "completed"],
                    "description": "任务状态（update 时使用）"
                }
            },
            "required": ["action"]
        })
    }

    fn execute(&self, input: Value) -> Result<String> {
        let action = input["action"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 action 参数"))?;

        match action {
            "create" => {
                let subject = input["subject"]
                    .as_str()
                    .ok_or_else(|| anyhow!("create 操作需要 subject 参数"))?;
                let description = input["description"].as_str().unwrap_or("").to_string();
                let id = Uuid::new_v4().to_string();
                let item = TodoItem {
                    id: id.clone(),
                    subject: subject.to_string(),
                    description,
                    status: "pending".to_string(),
                };
                self.items.write().unwrap().push(item);
                Ok(format!("已创建任务 ID: {}", id))
            }
            "update" => {
                let id = input["id"]
                    .as_str()
                    .ok_or_else(|| anyhow!("update 操作需要 id 参数"))?;
                let mut items = self.items.write().unwrap();
                let item = items
                    .iter_mut()
                    .find(|i| i.id == id)
                    .ok_or_else(|| anyhow!("任务不存在: {}", id))?;
                if let Some(status) = input["status"].as_str() {
                    item.status = status.to_string();
                }
                if let Some(subject) = input["subject"].as_str() {
                    item.subject = subject.to_string();
                }
                if let Some(desc) = input["description"].as_str() {
                    item.description = desc.to_string();
                }
                Ok(format!("已更新任务: {}", id))
            }
            "list" => {
                let items = self.items.read().unwrap();
                if items.is_empty() {
                    return Ok("暂无任务".to_string());
                }
                let list: Vec<String> = items
                    .iter()
                    .map(|item| {
                        format!(
                            "- [{}] {} (ID: {}){}",
                            item.status,
                            item.subject,
                            item.id,
                            if item.description.is_empty() {
                                String::new()
                            } else {
                                format!("\n  {}", item.description)
                            }
                        )
                    })
                    .collect();
                Ok(list.join("\n"))
            }
            "delete" => {
                let id = input["id"]
                    .as_str()
                    .ok_or_else(|| anyhow!("delete 操作需要 id 参数"))?;
                let mut items = self.items.write().unwrap();
                let len_before = items.len();
                items.retain(|i| i.id != id);
                if items.len() == len_before {
                    return Err(anyhow!("任务不存在: {}", id));
                }
                Ok(format!("已删除任务: {}", id))
            }
            _ => Err(anyhow!("未知操作: {}", action)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_list() {
        let tool = TodoWriteTool::new();

        // 列表为空时返回"暂无任务"
        let result = tool.execute(json!({"action": "list"})).unwrap();
        assert_eq!(result, "暂无任务");

        // 创建任务
        let result = tool
            .execute(json!({
                "action": "create",
                "subject": "实现 Edit 工具",
                "description": "精确替换文本"
            }))
            .unwrap();
        assert!(result.contains("已创建任务 ID:"));

        // 列表应包含新任务
        let list = tool.execute(json!({"action": "list"})).unwrap();
        assert!(list.contains("实现 Edit 工具"));
        assert!(list.contains("pending"));
        assert!(list.contains("精确替换文本"));
    }

    #[test]
    fn test_update_status() {
        let tool = TodoWriteTool::new();

        let create_result = tool
            .execute(json!({"action": "create", "subject": "Test task"}))
            .unwrap();
        let id = create_result.split("ID: ").nth(1).unwrap().trim();

        // 更新状态
        let update_result = tool
            .execute(json!({"action": "update", "id": id, "status": "in_progress"}))
            .unwrap();
        assert!(update_result.contains("已更新任务"));

        let list = tool.execute(json!({"action": "list"})).unwrap();
        assert!(list.contains("in_progress"));
    }

    #[test]
    fn test_delete() {
        let tool = TodoWriteTool::new();

        let create_result = tool
            .execute(json!({"action": "create", "subject": "Will delete"}))
            .unwrap();
        let id = create_result.split("ID: ").nth(1).unwrap().trim();

        let delete_result = tool
            .execute(json!({"action": "delete", "id": id}))
            .unwrap();
        assert!(delete_result.contains("已删除任务"));

        let list = tool.execute(json!({"action": "list"})).unwrap();
        assert!(!list.contains("Will delete"));
    }

    #[test]
    fn test_delete_nonexistent() {
        let tool = TodoWriteTool::new();
        let result = tool.execute(json!({"action": "delete", "id": "fake-id"}));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("任务不存在"));
    }

    #[test]
    fn test_update_nonexistent() {
        let tool = TodoWriteTool::new();
        let result = tool.execute(json!({"action": "update", "id": "fake-id", "status": "completed"}));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("任务不存在"));
    }

    #[test]
    fn test_missing_action() {
        let tool = TodoWriteTool::new();
        let result = tool.execute(json!({}));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("缺少 action 参数"));
    }

    #[test]
    fn test_unknown_action() {
        let tool = TodoWriteTool::new();
        let result = tool.execute(json!({"action": "unknown"}));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("未知操作"));
    }
}
