use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use tempfile::TempDir;
use std::path::PathBuf;

/// 创建临时 SQLite 数据库，复制完整 schema（与 db.rs 保持一致）
pub async fn setup_test_db() -> (SqlitePool, TempDir) {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("test.db");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await
        .unwrap();

    // 创建所有表（与 db.rs init_db 保持一致）
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS installed_skills (
            id TEXT PRIMARY KEY,
            manifest TEXT NOT NULL,
            installed_at TEXT NOT NULL,
            last_used_at TEXT,
            username TEXT NOT NULL,
            pack_path TEXT NOT NULL DEFAULT '',
            source_type TEXT NOT NULL DEFAULT 'encrypted'
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            skill_id TEXT NOT NULL,
            title TEXT,
            created_at TEXT NOT NULL,
            model_id TEXT NOT NULL,
            permission_mode TEXT NOT NULL DEFAULT 'accept_edits',
            work_dir TEXT NOT NULL DEFAULT ''
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS model_configs (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            api_format TEXT NOT NULL,
            base_url TEXT NOT NULL,
            model_name TEXT NOT NULL,
            is_default INTEGER DEFAULT 0,
            api_key TEXT NOT NULL DEFAULT ''
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS mcp_servers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            command TEXT NOT NULL,
            args TEXT NOT NULL DEFAULT '[]',
            env TEXT NOT NULL DEFAULT '{}',
            enabled INTEGER DEFAULT 1,
            created_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    (pool, tmp)
}

/// 创建测试用 Skill 目录（含 SKILL.md + templates）
pub fn create_test_skill_dir() -> (TempDir, PathBuf) {
    let tmp = TempDir::new().unwrap();
    let skill_dir = tmp.path().join("test-skill");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: test-skill\ndescription: A test skill for E2E testing\nallowed_tools: \"ReadFile, Glob\"\nuser-invocable: true\n---\n\nYou are a helpful test assistant.\n",
    )
    .unwrap();
    let templates = skill_dir.join("templates");
    std::fs::create_dir_all(&templates).unwrap();
    std::fs::write(templates.join("greeting.md"), "Hello, {{name}}!").unwrap();
    (tmp, skill_dir)
}
