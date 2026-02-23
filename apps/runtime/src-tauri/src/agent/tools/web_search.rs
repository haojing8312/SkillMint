use crate::agent::types::{Tool, ToolContext};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};

/// Web 搜索工具 - 通过 Sidecar 调用 DuckDuckGo 搜索
pub struct WebSearchTool {
    sidecar_url: String,
}

impl WebSearchTool {
    pub fn new(sidecar_url: String) -> Self {
        Self { sidecar_url }
    }
}

impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "搜索互联网，返回相关网页结果（使用 DuckDuckGo）"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "搜索关键词"
                },
                "count": {
                    "type": "integer",
                    "description": "返回结果数量（默认 5）",
                    "default": 5
                }
            },
            "required": ["query"]
        })
    }

    fn execute(&self, input: Value, _ctx: &ToolContext) -> Result<String> {
        let query = input["query"]
            .as_str()
            .ok_or(anyhow!("缺少 query 参数"))?;

        if query.trim().is_empty() {
            return Err(anyhow!("query 不能为空"));
        }

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let url = format!("{}/api/web/search", self.sidecar_url);
        let body = json!({
            "query": query,
            "count": input["count"].as_i64().unwrap_or(5)
        });

        let resp = client.post(&url).json(&body).send()?;

        if !resp.status().is_success() {
            let error_body: Value = resp.json().unwrap_or(json!({}));
            return Err(anyhow!(
                "搜索失败: {}",
                error_body["error"].as_str().unwrap_or("未知错误")
            ));
        }

        let result: Value = resp.json()?;
        if let Some(output) = result["output"].as_str() {
            Ok(output.to_string())
        } else {
            Ok(serde_json::to_string(&result).unwrap_or_default())
        }
    }
}
