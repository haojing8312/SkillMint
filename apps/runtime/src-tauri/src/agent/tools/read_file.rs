use crate::agent::types::Tool;
use anyhow::{anyhow, Result};
use serde_json::{json, Value};

pub struct ReadFileTool;

impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "读取文件内容。返回文件的完整文本内容。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "要读取的文件路径（相对或绝对）"
                }
            },
            "required": ["path"]
        })
    }

    fn execute(&self, input: Value) -> Result<String> {
        let path = input["path"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 path 参数"))?;

        let content =
            std::fs::read_to_string(path).map_err(|e| anyhow!("读取文件失败: {}", e))?;

        Ok(content)
    }
}
