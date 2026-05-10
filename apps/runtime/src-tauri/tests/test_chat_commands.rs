mod helpers;

use chrono::Utc;
use runtime_chat_app::ChatSessionContextRepository;
use runtime_lib::agent::runtime::PoolChatSettingsRepository;
use runtime_lib::commands::chat::create_session_with_pool;
use sqlx::sqlite::SqlitePoolOptions;

#[tokio::test]
async fn setup_test_db_exposes_profile_runtime_schema() {
    let (pool, _tmp) = helpers::setup_test_db().await;

    let session_columns: Vec<String> =
        sqlx::query_scalar("SELECT name FROM pragma_table_info('sessions')")
            .fetch_all(&pool)
            .await
            .expect("query session columns");
    assert!(session_columns.iter().any(|name| name == "profile_id"));

    let profile_tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master
         WHERE type = 'table' AND name = 'agent_profiles'",
    )
    .fetch_all(&pool)
    .await
    .expect("query profile table");
    assert_eq!(profile_tables, vec!["agent_profiles".to_string()]);
}

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
async fn create_session_with_pool_stores_profile_id_for_employee_alias() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO agent_employees (
            id, employee_id, name, role_id, persona, primary_skill_id, default_work_dir,
            openclaw_agent_id, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("employee-row-1")
    .bind("planner")
    .bind("Planner")
    .bind("planner-role")
    .bind("")
    .bind("skill-1")
    .bind("")
    .bind("oc-planner")
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .expect("insert employee");

    sqlx::query(
        "INSERT INTO agent_profiles (
            id, legacy_employee_row_id, display_name, route_aliases_json, profile_home, created_at, updated_at
        ) VALUES (?, ?, ?, '[]', ?, ?, ?)",
    )
    .bind("profile-1")
    .bind("employee-row-1")
    .bind("Planner")
    .bind("profiles/profile-1")
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .expect("insert profile");

    let session_id = create_session_with_pool(
        &pool,
        "skill-1".to_string(),
        "model-1".to_string(),
        None,
        Some(" planner ".to_string()),
        Some("Profile Session".to_string()),
        Some("standard".to_string()),
        Some("general".to_string()),
        None,
    )
    .await
    .expect("session should be created");

    let row = sqlx::query_as::<_, (String, String)>(
        "SELECT employee_id, profile_id FROM sessions WHERE id = ?",
    )
    .bind(&session_id)
    .fetch_one(&pool)
    .await
    .expect("session row");

    assert_eq!(row.0, "planner");
    assert_eq!(row.1, "profile-1");
}

#[tokio::test]
async fn session_execution_context_loads_profile_id() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO sessions (
            id, skill_id, title, created_at, model_id, permission_mode, work_dir,
            employee_id, profile_id, session_mode, team_id
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("session-profile")
    .bind("skill-1")
    .bind("Profile Session")
    .bind(&now)
    .bind("model-1")
    .bind("standard")
    .bind("E:/work")
    .bind("planner")
    .bind("profile-1")
    .bind("employee_direct")
    .bind("")
    .execute(&pool)
    .await
    .expect("insert session");

    let repo = PoolChatSettingsRepository::new(&pool);
    let context = repo
        .load_session_execution_context(Some("session-profile"))
        .await
        .expect("load execution context");

    assert_eq!(context.employee_id, "planner");
    assert_eq!(context.profile_id, "profile-1");
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
