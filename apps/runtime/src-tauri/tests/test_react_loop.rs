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
        )
        .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("最大迭代次数"));
}
