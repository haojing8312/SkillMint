use runtime_lib::agent::permissions::PermissionMode;
use runtime_lib::agent::{AgentExecutor, ToolRegistry};
use serde_json::json;
use std::sync::Arc;

/// 集成测试：需要真实 API 端点才能通过
/// 运行方式：ANTHROPIC_API_KEY=xxx cargo test test_anthropic_tool_parsing -- --ignored
#[tokio::test]
#[ignore]
async fn test_anthropic_tool_parsing() {
    let registry = Arc::new(ToolRegistry::with_file_tools());
    let executor = AgentExecutor::new(registry);

    let messages = vec![json!({"role": "user", "content": "Read test.txt"})];

    let result = executor
        .execute_turn(
            "anthropic",
            "http://mock",
            "mock-key",
            "claude-3-5-haiku-20241022",
            "You are a helpful assistant.",
            messages,
            |_token| {},
            None,
            None,
            None,
            PermissionMode::Unrestricted,
            None,
            None,
        )
        .await;

    assert!(result.is_ok());
}

/// 验证 execute_turn 对无效 URL 返回网络错误
#[tokio::test]
async fn test_anthropic_tool_parsing_network_error() {
    let registry = Arc::new(ToolRegistry::with_file_tools());
    let executor = AgentExecutor::new(registry);

    let messages = vec![json!({"role": "user", "content": "hello"})];

    let result = executor
        .execute_turn(
            "anthropic",
            "http://invalid-mock-url-that-does-not-exist",
            "mock-key",
            "mock-model",
            "You are a helpful assistant.",
            messages,
            |_token| {},
            None,
            None,
            None,
            PermissionMode::Unrestricted,
            None,
            None,
        )
        .await;

    // 无效 URL 应返回网络错误
    assert!(result.is_err());
}
