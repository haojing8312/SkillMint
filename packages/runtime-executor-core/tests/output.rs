use runtime_executor_core::{
    split_error_code_and_message, truncate_tool_output, update_tool_failure_streak,
    ToolFailureStreak,
};
use serde_json::json;

#[test]
fn truncate_long_output() {
    let long_output = "x".repeat(40_000);
    let truncated = truncate_tool_output(&long_output, 30_000);
    assert!(truncated.len() < 31_100);
    assert!(truncated.contains("[输出已截断"));
    assert!(truncated.contains("40000"));
}

#[test]
fn split_error_code_parses_prefixed_errors() {
    let (code, msg) = split_error_code_and_message("SKILL_NOT_FOUND: missing child");
    assert_eq!(code, "SKILL_NOT_FOUND");
    assert_eq!(msg, "missing child");
}

#[test]
fn repeated_failure_streak_trips_after_threshold() {
    let mut streak: Option<ToolFailureStreak> = None;
    let input = json!({"path": "a.txt"});

    assert!(update_tool_failure_streak(&mut streak, "write_file", &input, "boom").is_none());
    assert!(update_tool_failure_streak(&mut streak, "write_file", &input, "boom").is_none());
    let summary = update_tool_failure_streak(&mut streak, "write_file", &input, "boom");
    assert!(summary.is_some());
}
