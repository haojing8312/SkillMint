use crate::agent::types::{Tool, ToolContext};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};

pub struct GlobTool;

impl Tool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }

    fn description(&self) -> &str {
        "使用 glob 模式搜索文件。支持 ** 递归、* 通配符、? 单字符。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Glob 模式，例如 '**/*.rs' 或 'src/**/*.ts'"
                },
                "base_dir": {
                    "type": "string",
                    "description": "搜索的基础目录（可选，默认为当前目录或工作目录）"
                }
            },
            "required": ["pattern"]
        })
    }

    fn execute(&self, input: Value, ctx: &ToolContext) -> Result<String> {
        let pattern = input["pattern"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 pattern 参数"))?;

        // 优先使用用户指定的 base_dir，其次使用 ToolContext 的 work_dir，最后回退到 "."
        let base_dir = match input["base_dir"].as_str() {
            Some(dir) => dir.to_string(),
            None => ctx
                .work_dir
                .as_ref()
                .map(|wd| wd.to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string()),
        };

        let full_pattern = format!("{}/{}", base_dir, pattern);
        let paths: Vec<String> = glob::glob(&full_pattern)
            .map_err(|e| anyhow!("Glob 模式错误: {}", e))?
            .filter_map(|r| r.ok())
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        Ok(format!(
            "找到 {} 个文件:\n{}",
            paths.len(),
            paths.join("\n")
        ))
    }
}
