use crate::agent::types::Tool;
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
                    "description": "搜索的基础目录（可选，默认为当前目录）"
                }
            },
            "required": ["pattern"]
        })
    }

    fn execute(&self, input: Value) -> Result<String> {
        let pattern = input["pattern"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 pattern 参数"))?;
        let base_dir = input["base_dir"].as_str().unwrap_or(".");

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
