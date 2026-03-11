mod helpers;

use runtime_lib::commands::chat::create_session_with_pool;

#[tokio::test]
async fn create_session_with_pool_normalizes_session_fields() {
    let (pool, _tmp) = helpers::setup_test_db().await;

    sqlx::query(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES ('runtime_default_work_dir', 'E:/workspace/default')",
    )
    .execute(&pool)
    .await
    .expect("set default workdir");

    let session_id = create_session_with_pool(
        &pool,
        "skill-1".to_string(),
        "model-1".to_string(),
        None,
        Some(" emp-1 ".to_string()),
        Some("   ".to_string()),
        Some("accept_edits".to_string()),
        Some("team_entry".to_string()),
        Some(" team-a ".to_string()),
    )
    .await
    .expect("session should be created");

    let row = sqlx::query_as::<_, (String, String, String, String, String, String, String)>(
        "SELECT id, title, permission_mode, work_dir, employee_id, session_mode, team_id FROM sessions WHERE id = ?",
    )
    .bind(&session_id)
    .fetch_one(&pool)
    .await
    .expect("session row");

    assert_eq!(row.0, session_id);
    assert_eq!(row.1, "New Chat");
    assert_eq!(row.2, "standard");
    assert_eq!(row.3, "E:/workspace/default");
    assert_eq!(row.4, "emp-1");
    assert_eq!(row.5, "team_entry");
    assert_eq!(row.6, "team-a");
}
