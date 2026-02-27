use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

use crate::agent::types::{Tool, ToolContext};

/// 持久内存工具 - 跨会话的知识存储
///
/// 每个 Skill 拥有独立的内存目录，以 Markdown 文件形式存储知识。
/// 支持 read/write/list/delete 四种操作。
///
/// # 示例
///
/// ```rust
/// use std::path::PathBuf;
/// use runtime_lib::agent::tools::MemoryTool;
/// use runtime_lib::agent::types::{Tool, ToolContext};
/// use serde_json::json;
///
/// let tool = MemoryTool::new(PathBuf::from("/tmp/memory"));
/// let ctx = ToolContext::default();
/// let result = tool.execute(json!({
///     "action": "write",
///     "key": "greeting",
///     "content": "你好，世界！"
/// }), &ctx).unwrap();
/// assert!(result.contains("已写入"));
/// ```
pub struct MemoryTool {
    memory_dir: PathBuf,
}

impl MemoryTool {
    /// 创建新的 MemoryTool 实例
    ///
    /// # 参数
    /// - `memory_dir`: 内存文件存储目录，通常为 `{app_data_dir}/memory/{skill_id}`
    pub fn new(memory_dir: PathBuf) -> Self {
        Self { memory_dir }
    }
}

impl Tool for MemoryTool {
    fn name(&self) -> &str {
        "memory"
    }

    fn description(&self) -> &str {
        "跨会话的持久内存，用于存储和读取知识。支持 read/write/list/delete 操作。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["read", "write", "list", "delete"],
                    "description": "操作类型"
                },
                "key": {
                    "type": "string",
                    "description": "内存键名（文件名，不含扩展名）"
                },
                "content": {
                    "type": "string",
                    "description": "写入内容（仅 write 操作需要）"
                }
            },
            "required": ["action"]
        })
    }

    fn execute(&self, input: Value, _ctx: &ToolContext) -> Result<String> {
        let action = input["action"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 action 参数"))?;

        match action {
            "read" => {
                let key = input["key"]
                    .as_str()
                    .ok_or_else(|| anyhow!("read 操作缺少 key 参数"))?;
                let path = self.memory_dir.join(format!("{}.md", key));
                if !path.exists() {
                    return Ok(format!("内存键 '{}' 不存在", key));
                }
                let content = fs::read_to_string(&path)?;
                Ok(content)
            }
            "write" => {
                let key = input["key"]
                    .as_str()
                    .ok_or_else(|| anyhow!("write 操作缺少 key 参数"))?;
                let content = input["content"]
                    .as_str()
                    .ok_or_else(|| anyhow!("write 操作缺少 content 参数"))?;
                // 确保目录存在
                fs::create_dir_all(&self.memory_dir)?;
                let path = self.memory_dir.join(format!("{}.md", key));
                fs::write(&path, content)?;
                Ok(format!("已写入内存键 '{}'", key))
            }
            "list" => {
                if !self.memory_dir.exists() {
                    return Ok("内存为空".to_string());
                }
                let mut entries: Vec<String> = fs::read_dir(&self.memory_dir)?
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path()
                            .extension()
                            .map_or(false, |ext| ext == "md")
                    })
                    .filter_map(|e| {
                        e.path()
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                    })
                    .collect();
                if entries.is_empty() {
                    Ok("内存为空".to_string())
                } else {
                    // 排序保证输出稳定
                    entries.sort();
                    Ok(format!("内存键列表:\n{}", entries.join("\n")))
                }
            }
            "delete" => {
                let key = input["key"]
                    .as_str()
                    .ok_or_else(|| anyhow!("delete 操作缺少 key 参数"))?;
                let path = self.memory_dir.join(format!("{}.md", key));
                if !path.exists() {
                    return Ok(format!("内存键 '{}' 不存在", key));
                }
                fs::remove_file(&path)?;
                Ok(format!("已删除内存键 '{}'", key))
            }
            _ => Err(anyhow!("未知操作: {}", action)),
        }
    }
}
