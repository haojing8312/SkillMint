use runtime_chat_app::{ChatPreparationService, PreparedSessionCreation, SessionCreationRequest};

#[test]
fn prepare_session_creation_normalizes_modes_and_team_id() {
    let prepared = ChatPreparationService::new().prepare_session_creation(SessionCreationRequest {
        permission_mode: Some("accept_edits".to_string()),
        session_mode: Some("team_entry".to_string()),
        team_id: Some(" team-1 ".to_string()),
        title: Some("  Demo  ".to_string()),
        work_dir: Some("  C:/work  ".to_string()),
        employee_id: Some(" emp-1 ".to_string()),
    });

    assert_eq!(
        prepared,
        PreparedSessionCreation {
            permission_mode_storage: "standard".to_string(),
            session_mode_storage: "team_entry".to_string(),
            normalized_team_id: "team-1".to_string(),
            normalized_title: "Demo".to_string(),
            normalized_work_dir: "C:/work".to_string(),
            normalized_employee_id: "emp-1".to_string(),
        }
    );
}

#[test]
fn prepare_session_creation_applies_defaults_for_empty_values() {
    let prepared = ChatPreparationService::new().prepare_session_creation(SessionCreationRequest {
        permission_mode: None,
        session_mode: Some("unknown".to_string()),
        team_id: Some("team-1".to_string()),
        title: Some("   ".to_string()),
        work_dir: None,
        employee_id: Some("   ".to_string()),
    });

    assert_eq!(prepared.permission_mode_storage, "standard");
    assert_eq!(prepared.session_mode_storage, "general");
    assert_eq!(prepared.normalized_team_id, "");
    assert_eq!(prepared.normalized_title, "New Chat");
    assert_eq!(prepared.normalized_work_dir, "");
    assert_eq!(prepared.normalized_employee_id, "");
}
