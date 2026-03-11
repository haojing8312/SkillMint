use runtime_chat_app::{
    normalize_permission_mode_for_storage, normalize_session_mode_for_storage,
    normalize_team_id_for_storage, parse_permission_mode_for_runtime, permission_mode_label,
    ChatPermissionMode,
};

#[test]
fn normalizes_storage_modes_and_team_id() {
    assert_eq!(
        normalize_permission_mode_for_storage(Some("accept_edits")),
        "standard"
    );
    assert_eq!(
        normalize_permission_mode_for_storage(Some("unrestricted")),
        "full_access"
    );
    assert_eq!(
        normalize_session_mode_for_storage(Some("team_entry")),
        "team_entry"
    );
    assert_eq!(
        normalize_session_mode_for_storage(Some("unknown")),
        "general"
    );
    assert_eq!(
        normalize_team_id_for_storage("team_entry", Some(" team-1 ")),
        "team-1"
    );
    assert_eq!(normalize_team_id_for_storage("general", Some("team-1")), "");
}

#[test]
fn parses_runtime_permission_mode_and_display_label() {
    assert_eq!(
        parse_permission_mode_for_runtime("standard"),
        ChatPermissionMode::AcceptEdits
    );
    assert_eq!(
        parse_permission_mode_for_runtime("full_access"),
        ChatPermissionMode::Unrestricted
    );
    assert_eq!(permission_mode_label("standard"), "标准模式");
    assert_eq!(permission_mode_label("full_access"), "全自动模式");
}
