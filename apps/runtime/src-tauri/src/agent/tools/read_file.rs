use crate::agent::types::{Tool, ToolContext};
use crate::agent::tools::tool_result;
use anyhow::{anyhow, Result};
use serde_json::{json, Value};

pub struct ReadFileTool;

impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "读取文件内容。返回结构化结果，其中 details.content 包含完整文本。"
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

    fn execute(&self, input: Value, ctx: &ToolContext) -> Result<String> {
        let path = input["path"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 path 参数"))?;

        let checked = ctx.check_path(path)?;
        let content =
            std::fs::read_to_string(&checked).map_err(|e| anyhow!("读取文件失败: {}", e))?;
        let line_count = content.lines().count().max(1);

        tool_result::success(
            self.name(),
            format!("已读取文件 {}", path),
            json!({
                "path": path,
                "absolute_path": checked.to_string_lossy().to_string(),
                "content": content,
                "line_count": line_count,
                "truncated": false,
            }),
        )
    }
}
