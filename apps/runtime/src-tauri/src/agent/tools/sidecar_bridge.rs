use crate::agent::types::Tool;
use anyhow::{anyhow, Result};
use serde_json::{json, Value};

pub struct SidecarBridgeTool {
    sidecar_url: String,
    endpoint: String,
    tool_name: String,
    tool_description: String,
    schema: Value,
}

impl SidecarBridgeTool {
    pub fn new(
        sidecar_url: String,
        endpoint: String,
        tool_name: String,
        tool_description: String,
        schema: Value,
    ) -> Self {
        Self {
            sidecar_url,
            endpoint,
            tool_name,
            tool_description,
            schema,
        }
    }
}

impl Tool for SidecarBridgeTool {
    fn name(&self) -> &str {
        &self.tool_name
    }

    fn description(&self) -> &str {
        &self.tool_description
    }

    fn input_schema(&self) -> Value {
        self.schema.clone()
    }

    fn execute(&self, input: Value) -> Result<String> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}{}", self.sidecar_url, self.endpoint);

        let resp = client.post(&url).json(&input).send()?;

        if !resp.status().is_success() {
            let error_body: Value = resp.json().unwrap_or(json!({}));
            return Err(anyhow!(
                "Sidecar 调用失败: {}",
                error_body["error"].as_str().unwrap_or("Unknown error")
            ));
        }

        let result: Value = resp.json()?;
        Ok(result["output"].as_str().unwrap_or("").to_string())
    }
}
