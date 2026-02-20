use anyhow::Result;
use serde_json::Value;

pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    fn execute(&self, input: Value) -> Result<String>;
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub input: Value,
}

#[derive(Debug)]
pub struct ToolResult {
    pub tool_use_id: String,
    pub content: String,
}

#[derive(Debug)]
pub enum LLMResponse {
    Text(String),
    ToolCalls(Vec<ToolCall>),
}

#[derive(Debug)]
pub enum AgentState {
    Thinking,
    ToolCalling,
    Finished,
    Error(String),
}
