use runtime_lib::agent::tools::web_fetch::strip_html_tags;
use runtime_lib::agent::ToolRegistry;

/// 验证 script 标签及其内容被完整移除
#[test]
fn test_strip_html_removes_script() {
    let html = "<html><script>alert('x')</script><body>Hello</body></html>";
    let result = strip_html_tags(html);
    assert!(result.contains("Hello"), "纯文本内容应保留");
    assert!(!result.contains("alert"), "script 内容应被移除");
    assert!(!result.contains("<script>"), "script 标签应被移除");
}

/// 验证 style 标签及其内容被完整移除
#[test]
fn test_strip_html_removes_style() {
    let html = "<style>body{color:red}</style><p>Text</p>";
    let result = strip_html_tags(html);
    assert!(result.contains("Text"), "纯文本内容应保留");
    assert!(!result.contains("color:red"), "style 内容应被移除");
    assert!(!result.contains("<style>"), "style 标签应被移除");
}

/// 验证所有普通 HTML 标签被移除，纯文本正确提取
#[test]
fn test_strip_html_removes_tags() {
    let html = "<div><p>Hello <b>World</b></p></div>";
    let result = strip_html_tags(html);
    assert_eq!(result, "Hello World");
}

/// 验证多余空行被压缩为最多两个空行
#[test]
fn test_strip_html_compresses_blank_lines() {
    let html = "<p>Line 1</p>\n\n\n\n<p>Line 2</p>";
    let result = strip_html_tags(html);
    // 不应出现三个或以上连续换行
    assert!(!result.contains("\n\n\n"), "连续三个换行应被压缩");
    assert!(result.contains("Line 1"), "第一行应保留");
    assert!(result.contains("Line 2"), "第二行应保留");
}

/// 验证多行 script 标签被正确移除
#[test]
fn test_strip_html_multiline_script() {
    let html = r#"<body>
<script type="text/javascript">
    var x = 1;
    var y = 2;
</script>
<p>Content here</p>
</body>"#;
    let result = strip_html_tags(html);
    assert!(result.contains("Content here"), "正文内容应保留");
    assert!(!result.contains("var x"), "script 内部变量声明应被移除");
}

/// 验证大小写不同的标签也能正确处理
#[test]
fn test_strip_html_case_insensitive_tags() {
    let html = "<SCRIPT>evil()</SCRIPT><P>Good text</P>";
    let result = strip_html_tags(html);
    assert!(result.contains("Good text"), "大写标签包裹的文本应保留");
    assert!(!result.contains("evil"), "大写 SCRIPT 标签内容应被移除");
}

/// 验证 WebFetchTool 已注册到 with_file_tools 中
#[test]
fn test_registry_includes_web_fetch() {
    let registry = ToolRegistry::with_file_tools();
    assert!(
        registry.get("web_fetch").is_some(),
        "web_fetch 工具应已注册"
    );
}
