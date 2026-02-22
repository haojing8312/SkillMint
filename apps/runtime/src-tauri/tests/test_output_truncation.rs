use runtime_lib::agent::executor::truncate_tool_output;

#[test]
fn test_truncate_long_output() {
    let long_output = "x".repeat(40_000);
    let truncated = truncate_tool_output(&long_output, 30_000);
    assert!(truncated.len() < 31_100); // 截断文本 + 提示信息
    assert!(truncated.contains("[输出已截断"));
    assert!(truncated.contains("40000"));
}

#[test]
fn test_no_truncate_short_output() {
    let short_output = "hello world";
    let result = truncate_tool_output(short_output, 30_000);
    assert_eq!(result, "hello world");
}

#[test]
fn test_truncate_exact_boundary() {
    let exact = "x".repeat(30_000);
    let result = truncate_tool_output(&exact, 30_000);
    assert_eq!(result, exact); // 刚好不超过，不截断
}

#[test]
fn test_truncate_one_over() {
    let over = "x".repeat(30_001);
    let result = truncate_tool_output(&over, 30_000);
    assert!(result.contains("[输出已截断"));
}
