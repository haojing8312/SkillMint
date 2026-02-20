use crate::agent::types::Tool;
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::path::Path;

pub struct WriteFileTool;

impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "写入内容到文件。如果文件不存在会创建，已存在会覆盖。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "要写入的文件路径"
                },
                "content": {
                    "type": "string",
                    "description": "要写入的文本内容"
                }
            },
            "required": ["path", "content"]
        })
    }

    fn execute(&self, input: Value) -> Result<String> {
        let path = input["path"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 path 参数"))?;
        let content = input["content"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 content 参数"))?;

        // 确保父目录存在
        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| anyhow!("创建目录失败: {}", e))?;
        }

        std::fs::write(path, content).map_err(|e| anyhow!("写入文件失败: {}", e))?;

        Ok(format!("成功写入 {} 字节到 {}", content.len(), path))
    }
}
