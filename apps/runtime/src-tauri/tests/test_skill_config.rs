use runtime_lib::agent::skill_config::SkillConfig;

#[test]
fn test_parse_with_frontmatter() {
    let content = "---\nname: test-skill\ndescription: A test skill\nallowed_tools:\n  - read_file\n  - edit\n  - bash\nmodel: gpt-4o\nmax_iterations: 5\n---\nYou are a helpful assistant.\n\nDo your best work.\n";
    let config = SkillConfig::parse(content);
    assert_eq!(config.name.as_deref(), Some("test-skill"));
    assert_eq!(config.description.as_deref(), Some("A test skill"));
    assert_eq!(
        config.allowed_tools,
        Some(vec!["read_file".into(), "edit".into(), "bash".into()])
    );
    assert_eq!(config.model.as_deref(), Some("gpt-4o"));
    assert_eq!(config.max_iterations, Some(5));
    assert!(config.system_prompt.contains("You are a helpful assistant."));
    assert!(config.system_prompt.contains("Do your best work."));
}

#[test]
fn test_parse_without_frontmatter() {
    let content = "You are a helpful assistant.\n\nDo stuff.";
    let config = SkillConfig::parse(content);
    assert!(config.name.is_none());
    assert!(config.allowed_tools.is_none());
    assert_eq!(config.system_prompt, content);
}

#[test]
fn test_parse_empty_frontmatter() {
    let content = "---\n---\nJust a prompt.";
    let config = SkillConfig::parse(content);
    assert!(config.name.is_none());
    assert_eq!(config.system_prompt.trim(), "Just a prompt.");
}

#[test]
fn test_parse_empty_content() {
    let config = SkillConfig::parse("");
    assert!(config.name.is_none());
    assert_eq!(config.system_prompt, "");
}

#[test]
fn test_parse_partial_frontmatter() {
    let content = "---\nname: partial\n---\nPrompt here.";
    let config = SkillConfig::parse(content);
    assert_eq!(config.name.as_deref(), Some("partial"));
    assert!(config.allowed_tools.is_none());
    assert!(config.model.is_none());
    assert_eq!(config.system_prompt.trim(), "Prompt here.");
}

#[test]
fn test_parse_no_closing_frontmatter() {
    let content = "---\nname: broken\nno closing marker";
    let config = SkillConfig::parse(content);
    // 没有结束标记，整个内容作为 prompt
    assert!(config.name.is_none());
    assert_eq!(config.system_prompt, content);
}
