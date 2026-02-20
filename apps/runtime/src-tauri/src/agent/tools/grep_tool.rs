use crate::agent::types::Tool;
use anyhow::{anyhow, Result};
use regex::RegexBuilder;
use serde_json::{json, Value};

pub struct GrepTool;

impl Tool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "在文件或目录中搜索文本模式（正则表达式）。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "正则表达式搜索模式"
                },
                "path": {
                    "type": "string",
                    "description": "要搜索的文件或目录路径"
                },
                "case_insensitive": {
                    "type": "boolean",
                    "description": "是否忽略大小写（可选，默认 false）"
                }
            },
            "required": ["pattern", "path"]
        })
    }

    fn execute(&self, input: Value) -> Result<String> {
        let pattern = input["pattern"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 pattern 参数"))?;
        let path = input["path"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 path 参数"))?;
        let case_insensitive = input["case_insensitive"].as_bool().unwrap_or(false);

        let re = RegexBuilder::new(pattern)
            .case_insensitive(case_insensitive)
            .build()
            .map_err(|e| anyhow!("正则表达式错误: {}", e))?;

        let content = std::fs::read_to_string(path).map_err(|e| anyhow!("读取文件失败: {}", e))?;

        let matches: Vec<String> = content
            .lines()
            .enumerate()
            .filter(|(_, line)| re.is_match(line))
            .map(|(i, line)| format!("{}:{}", i + 1, line))
            .collect();

        Ok(format!(
            "找到 {} 处匹配:\n{}",
            matches.len(),
            matches.join("\n")
        ))
    }
}
