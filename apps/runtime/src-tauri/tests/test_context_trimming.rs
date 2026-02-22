use runtime_lib::agent::executor::trim_messages;
use serde_json::json;

#[test]
fn test_trim_under_budget() {
    let messages = vec![
        json!({"role": "user", "content": "hello"}),
        json!({"role": "assistant", "content": "hi"}),
    ];
    let trimmed = trim_messages(&messages, 10_000);
    assert_eq!(trimmed.len(), 2);
}

#[test]
fn test_trim_over_budget() {
    let long_text = "x".repeat(5000);
    let messages = vec![
        json!({"role": "user", "content": &long_text}),
        json!({"role": "assistant", "content": &long_text}),
        json!({"role": "user", "content": &long_text}),
        json!({"role": "assistant", "content": &long_text}),
        json!({"role": "user", "content": "latest question"}),
    ];
    // 预算 3000 tokens ≈ 12000 字符，5 条消息总约 20000+ 字符
    let trimmed = trim_messages(&messages, 3_000);
    assert!(trimmed.len() < 5);
    // 最后一条消息必须保留
    let last = trimmed.last().unwrap();
    assert_eq!(last["content"].as_str().unwrap(), "latest question");
    // 存在裁剪标记
    let has_marker = trimmed.iter().any(|m| {
        m["content"]
            .as_str()
            .map_or(false, |c| c.contains("已省略"))
    });
    assert!(has_marker);
}

#[test]
fn test_trim_preserves_first_and_last() {
    let text = "x".repeat(5000);
    let messages = vec![
        json!({"role": "user", "content": &text}),
        json!({"role": "assistant", "content": &text}),
        json!({"role": "user", "content": &text}),
        json!({"role": "assistant", "content": &text}),
        json!({"role": "user", "content": "final"}),
    ];
    let trimmed = trim_messages(&messages, 2_000);
    assert_eq!(trimmed.first().unwrap()["content"].as_str().unwrap(), &text);
    assert_eq!(trimmed.last().unwrap()["content"].as_str().unwrap(), "final");
}

#[test]
fn test_trim_two_messages_never_trimmed() {
    let long = "x".repeat(100_000);
    let messages = vec![
        json!({"role": "user", "content": &long}),
        json!({"role": "assistant", "content": &long}),
    ];
    // 即使超预算，只有 2 条消息也不裁剪
    let trimmed = trim_messages(&messages, 100);
    assert_eq!(trimmed.len(), 2);
}
