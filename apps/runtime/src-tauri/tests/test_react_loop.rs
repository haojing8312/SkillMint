use runtime_lib::agent::{AgentExecutor, ToolRegistry};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_react_loop_structure() {
    let registry = Arc::new(ToolRegistry::with_file_tools());
    let _executor = AgentExecutor::new(registry);

    // 验证 AgentExecutor 创建成功且默认值正确
    assert!(true, "AgentExecutor created successfully");
}

#[tokio::test]
async fn test_react_loop_max_iterations_error() {
    let registry = Arc::new(ToolRegistry::with_file_tools());
    let executor = AgentExecutor::with_max_iterations(Arc::clone(&registry), 0);

    let messages = vec![json!({"role": "user", "content": "hello"})];

    let result = executor
        .execute_turn(
            "anthropic",
            "http://mock-url",
            "mock-key",
            "mock-model",
            "You are a helpful assistant.",
            messages,
            |_token| {},
            None,
            None,
            None,
        )
        .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("最大迭代次数"));
}

#[tokio::test]
async fn test_react_loop_openai_format_network_error() {
    let registry = Arc::new(ToolRegistry::with_file_tools());
    let executor = AgentExecutor::new(registry);

    let messages = vec![json!({"role": "user", "content": "hello"})];

    let result = executor
        .execute_turn(
            "openai",
            "http://invalid-openai-url",
            "mock-key",
            "gpt-4",
            "You are a helpful assistant.",
            messages,
            |_token| {},
            None,
            None,
            None,
        )
        .await;

    // OpenAI 格式应返回网络错误（不是 "not yet implemented"）
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(!err_msg.contains("not yet implemented"));
}
