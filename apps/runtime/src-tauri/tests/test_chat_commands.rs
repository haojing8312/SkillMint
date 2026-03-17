mod helpers;

use runtime_lib::commands::chat::create_session_with_pool;
use sqlx::sqlite::SqlitePoolOptions;

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

#[tokio::test]
async fn session_row_persists_after_reopening_sqlite_pool() {
    let (pool, tmp) = helpers::setup_test_db().await;
    let db_path = tmp.path().join("test.db");

    let session_id = create_session_with_pool(
        &pool,
        "skill-1".to_string(),
        "model-1".to_string(),
        None,
        None,
        Some("Persistent Session".to_string()),
        Some("standard".to_string()),
        Some("general".to_string()),
        None,
    )
    .await
    .expect("session should be created");

    pool.close().await;

    let reopened_pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite://{}?mode=rwc", db_path.to_string_lossy()))
        .await
        .expect("reopen sqlite pool");

    let row = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT id, title, skill_id, model_id FROM sessions WHERE id = ?",
    )
    .bind(&session_id)
    .fetch_one(&reopened_pool)
    .await
    .expect("persisted session row");

    assert_eq!(row.0, session_id);
    assert_eq!(row.1, "Persistent Session");
    assert_eq!(row.2, "skill-1");
    assert_eq!(row.3, "model-1");

    reopened_pool.close().await;
}
