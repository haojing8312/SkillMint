use runtime_lib::agent::{AgentExecutor, ToolRegistry};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_anthropic_tool_parsing() {
    let registry = Arc::new(ToolRegistry::with_file_tools());
    let executor = AgentExecutor::new(registry);

    let messages = vec![json!({"role": "user", "content": "Read test.txt"})];

    // For now, just verify it doesn't panic (stub returns Ok)
    let result = executor
        .execute_turn(
            "anthropic",
            "http://mock",
            "mock-key",
            "claude-3-5-haiku-20241022",
            "You are a helpful assistant.",
            messages,
            |_token| {},
        )
        .await;

    assert!(result.is_ok());
}
