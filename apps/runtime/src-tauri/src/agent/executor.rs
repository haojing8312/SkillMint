use super::registry::ToolRegistry;
use anyhow::Result;
use serde_json::Value;
use std::sync::Arc;

pub struct AgentExecutor {
    registry: Arc<ToolRegistry>,
    max_iterations: usize,
}

impl AgentExecutor {
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self {
            registry,
            max_iterations: 10,
        }
    }

    pub async fn execute_turn(
        &self,
        _api_format: &str,
        _base_url: &str,
        _api_key: &str,
        _model: &str,
        _system_prompt: &str,
        messages: Vec<Value>,
        _on_token: impl Fn(String) + Send + Clone,
    ) -> Result<Vec<Value>> {
        // Stub implementation for now
        Ok(messages)
    }
}
