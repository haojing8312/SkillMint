use runtime_lib::agent::permissions::PermissionMode;
use runtime_lib::agent::{AgentExecutor, ToolRegistry};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

fn setup_work_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("test_openai_tools_{}", name));
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap();
    }
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// 验证 execute_turn 对 OpenAI 格式的行为
///
/// OpenAI 分支已接入 chat_stream_with_tools，
/// 使用无效 URL 时应返回网络错误（而非 "not yet implemented"）。
#[tokio::test]
async fn test_openai_tool_calling_executor_branch() {
    let registry = Arc::new(ToolRegistry::with_file_tools());
    let executor = AgentExecutor::new(registry);

    let messages = vec![json!({"role": "user", "content": "hello"})];

    let result = executor
        .execute_turn(
            "openai",
            "http://invalid-openai-mock-url",
            "mock-key",
            "gpt-4",
            "You are a helpful assistant.",
            messages,
            |_| {},
            None,
            None,
            None,
            PermissionMode::Unrestricted,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await;

    // 使用无效 URL 应返回网络错误
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        !err_msg.contains("not yet implemented"),
        "OpenAI tool calling 应该已实现，但得到: {}",
        err_msg
    );
}

/// 集成测试：需要真实 OpenAI 兼容 API 端点才能通过
/// 运行方式：OPENAI_API_KEY=xxx cargo test test_openai_tool_calling_real -- --ignored
#[tokio::test]
#[ignore]
async fn test_openai_tool_calling_real() {
    let registry = Arc::new(ToolRegistry::with_file_tools());
    let executor = AgentExecutor::new(registry);

    let messages = vec![json!({"role": "user", "content": "Read the file test.txt"})];

    let result = executor
        .execute_turn(
            "openai",
            "https://api.openai.com/v1",
            &std::env::var("OPENAI_API_KEY").unwrap_or_default(),
            "gpt-4",
            "You are a helpful assistant with file tools.",
            messages,
            |token| {
                eprint!("{:?}", token);
            },
            None,
            None,
            None,
            PermissionMode::Unrestricted,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await;

    assert!(result.is_ok(), "OpenAI tool calling 失败: {:?}", result);
}

#[tokio::test]
async fn test_openai_responses_mock_runs_full_tool_loop() {
    let registry = Arc::new(ToolRegistry::with_file_tools());
    let executor = AgentExecutor::with_max_iterations(Arc::clone(&registry), 4);
    let work_dir = setup_work_dir("responses_full_tool_loop");
    let target = work_dir.join("notes.txt");
    fs::write(&target, "hello from mock responses").expect("write test file");

    let messages = vec![json!({
        "role": "user",
        "content": json!({
            "path": target.to_string_lossy().to_string()
        }).to_string()
    })];

    let result = executor
        .execute_turn(
            "openai",
            "http://mock-responses-read-file-from-user-path",
            "mock-key",
            "gpt-5.4",
            "You are a helpful assistant with file tools.",
            messages,
            |_| {},
            None,
            None,
            None,
            PermissionMode::AcceptEdits,
            None,
            Some(work_dir.to_string_lossy().to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

    assert!(result.is_ok(), "responses mock tool loop should succeed: {:?}", result);

    let messages = result.unwrap();
    assert!(
        messages.iter().any(|message| message["role"].as_str() == Some("tool")),
        "expected at least one tool message in final transcript: {:?}",
        messages
    );

    let last = messages.last().expect("assistant final message");
    let content = last["content"].as_str().unwrap_or_default();
    assert!(
        content.contains("已读取文件内容") && content.contains("hello from mock responses"),
        "unexpected final content: {}",
        content
    );

    fs::remove_dir_all(&work_dir).unwrap();
}

#[tokio::test]
async fn test_openai_responses_malformed_tool_call_does_not_fail_task_start() {
    let registry = Arc::new(ToolRegistry::with_file_tools());
    let executor = AgentExecutor::with_max_iterations(Arc::clone(&registry), 4);

    let messages = vec![json!({
        "role": "user",
        "content": "帮我开始处理这个任务"
    })];

    let result = executor
        .execute_turn(
            "openai",
            "http://mock-responses-malformed-tool-call-start-task",
            "mock-key",
            "gpt-5.4",
            "You are a helpful assistant.",
            messages,
            |_| {},
            None,
            None,
            None,
            PermissionMode::AcceptEdits,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await;

    assert!(
        result.is_ok(),
        "malformed responses tool call should not fail task start: {:?}",
        result
    );

    let messages = result.unwrap();
    assert!(
        messages.iter().all(|message| message["role"].as_str() != Some("tool")),
        "malformed tool call should not execute tools: {:?}",
        messages
    );

    let last = messages.last().expect("assistant final message");
    let content = last["content"].as_str().unwrap_or_default();
    assert!(
        content.contains("继续处理请求"),
        "unexpected final content: {}",
        content
    );
}
