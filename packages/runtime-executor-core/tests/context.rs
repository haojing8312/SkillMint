use runtime_executor_core::{micro_compact, trim_messages};
use serde_json::json;

#[test]
fn trim_under_budget() {
    let messages = vec![
        json!({"role": "user", "content": "hello"}),
        json!({"role": "assistant", "content": "hi"}),
    ];
    let trimmed = trim_messages(&messages, 10_000);
    assert_eq!(trimmed.len(), 2);
}

#[test]
fn trim_over_budget() {
    let long_text = "x".repeat(5000);
    let messages = vec![
        json!({"role": "user", "content": &long_text}),
        json!({"role": "assistant", "content": &long_text}),
        json!({"role": "user", "content": &long_text}),
        json!({"role": "assistant", "content": &long_text}),
        json!({"role": "user", "content": "latest question"}),
    ];
    let trimmed = trim_messages(&messages, 3_000);
    assert!(trimmed.len() < 5);
    let last = trimmed.last().expect("last message");
    assert_eq!(last["content"].as_str().expect("string"), "latest question");
    let has_marker = trimmed
        .iter()
        .any(|m| m["content"].as_str().is_some_and(|c| c.contains("已省略")));
    assert!(has_marker);
}

#[test]
fn micro_compact_replaces_old_tool_results() {
    let messages = vec![
        json!({"role": "user", "content": "start"}),
        json!({"role": "user", "content": [{"type": "tool_result", "tool_use_id": "1", "content": "long output 1"}]}),
        json!({"role": "user", "content": [{"type": "tool_result", "tool_use_id": "2", "content": "long output 2"}]}),
        json!({"role": "user", "content": [{"type": "tool_result", "tool_use_id": "3", "content": "recent output"}]}),
        json!({"role": "assistant", "content": "done"}),
    ];

    let result = micro_compact(&messages, 1);
    let r1 = serde_json::to_string(&result[1]).expect("serialize r1");
    assert!(r1.contains("[已执行]"));
    let r3 = serde_json::to_string(&result[3]).expect("serialize r3");
    assert!(r3.contains("recent output"));
}
