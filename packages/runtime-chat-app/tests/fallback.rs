use runtime_chat_app::parse_fallback_chain_targets;

#[test]
fn parses_fallback_chain_targets_from_json() {
    let targets = parse_fallback_chain_targets(
        r#"[{"provider_id":"p1","model":"gpt-4o"},{"provider_id":"p2"}]"#,
    );

    assert_eq!(
        targets,
        vec![
            ("p1".to_string(), "gpt-4o".to_string()),
            ("p2".to_string(), "".to_string())
        ]
    );
}
