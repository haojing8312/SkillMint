use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use tauri::{AppHandle, Manager};
use anyhow::Result;

pub async fn init_db(app: &AppHandle) -> Result<SqlitePool> {
    let app_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_dir)?;
    let db_path = app_dir.join("skillmint.db");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS installed_skills (
            id TEXT PRIMARY KEY,
            manifest TEXT NOT NULL,
            installed_at TEXT NOT NULL,
            last_used_at TEXT,
            username TEXT NOT NULL,
            pack_path TEXT NOT NULL DEFAULT ''
        )"
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            skill_id TEXT NOT NULL,
            title TEXT,
            created_at TEXT NOT NULL,
            model_id TEXT NOT NULL,
            permission_mode TEXT NOT NULL DEFAULT 'accept_edits',
            work_dir TEXT NOT NULL DEFAULT ''
        )"
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL
        )"
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS model_configs (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            api_format TEXT NOT NULL,
            base_url TEXT NOT NULL,
            model_name TEXT NOT NULL,
            is_default INTEGER DEFAULT 0,
            api_key TEXT NOT NULL DEFAULT ''
        )"
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS mcp_servers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            command TEXT NOT NULL,
            args TEXT NOT NULL DEFAULT '[]',
            env TEXT NOT NULL DEFAULT '{}',
            enabled INTEGER DEFAULT 1,
            created_at TEXT NOT NULL
        )"
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )"
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS provider_configs (
            id TEXT PRIMARY KEY,
            provider_key TEXT NOT NULL,
            display_name TEXT NOT NULL,
            protocol_type TEXT NOT NULL,
            base_url TEXT NOT NULL,
            auth_type TEXT NOT NULL DEFAULT 'api_key',
            api_key_encrypted TEXT NOT NULL DEFAULT '',
            org_id TEXT NOT NULL DEFAULT '',
            extra_json TEXT NOT NULL DEFAULT '{}',
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )"
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS provider_capabilities (
            provider_id TEXT NOT NULL,
            capability TEXT NOT NULL,
            supported INTEGER NOT NULL DEFAULT 1,
            priority INTEGER NOT NULL DEFAULT 100,
            default_model TEXT NOT NULL DEFAULT '',
            fallback_models_json TEXT NOT NULL DEFAULT '[]',
            PRIMARY KEY (provider_id, capability)
        )"
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS model_catalog_cache (
            provider_id TEXT NOT NULL,
            model_id TEXT NOT NULL,
            raw_json TEXT NOT NULL,
            fetched_at TEXT NOT NULL,
            ttl_seconds INTEGER NOT NULL DEFAULT 3600,
            PRIMARY KEY (provider_id, model_id)
        )"
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS routing_policies (
            capability TEXT PRIMARY KEY,
            primary_provider_id TEXT NOT NULL,
            primary_model TEXT NOT NULL DEFAULT '',
            fallback_chain_json TEXT NOT NULL DEFAULT '[]',
            timeout_ms INTEGER NOT NULL DEFAULT 60000,
            retry_count INTEGER NOT NULL DEFAULT 0,
            enabled INTEGER NOT NULL DEFAULT 1
        )"
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS route_attempt_logs (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            capability TEXT NOT NULL,
            api_format TEXT NOT NULL,
            model_name TEXT NOT NULL,
            attempt_index INTEGER NOT NULL DEFAULT 1,
            retry_index INTEGER NOT NULL DEFAULT 0,
            error_kind TEXT NOT NULL DEFAULT '',
            success INTEGER NOT NULL DEFAULT 0,
            error_message TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL
        )"
    )
    .execute(&pool)
    .await?;

    // Migration: add api_key column for databases created before this column existed
    let _ = sqlx::query("ALTER TABLE model_configs ADD COLUMN api_key TEXT NOT NULL DEFAULT ''")
        .execute(&pool)
        .await;

    // Migration: add permission_mode column to sessions
    let _ = sqlx::query("ALTER TABLE sessions ADD COLUMN permission_mode TEXT NOT NULL DEFAULT 'accept_edits'")
        .execute(&pool)
        .await;

    // Migration: add source_type column to installed_skills（区分加密 vs 本地 Skill）
    let _ = sqlx::query("ALTER TABLE installed_skills ADD COLUMN source_type TEXT NOT NULL DEFAULT 'encrypted'")
        .execute(&pool)
        .await;

    // Migration: add work_dir column to sessions（每会话独立工作目录）
    let _ = sqlx::query("ALTER TABLE sessions ADD COLUMN work_dir TEXT NOT NULL DEFAULT ''")
        .execute(&pool)
        .await;

    // 默认路由配置
    let _ = sqlx::query("INSERT OR IGNORE INTO app_settings (key, value) VALUES ('route_max_call_depth', '4')")
        .execute(&pool)
        .await;
    let _ = sqlx::query("INSERT OR IGNORE INTO app_settings (key, value) VALUES ('route_node_timeout_seconds', '60')")
        .execute(&pool)
        .await;
    let _ = sqlx::query("INSERT OR IGNORE INTO app_settings (key, value) VALUES ('route_retry_count', '0')")
        .execute(&pool)
        .await;

    // 内置通用 Skill：始终存在，无需用户安装
    let builtin_manifest = serde_json::json!({
        "id": "builtin-general",
        "name": "通用助手",
        "description": "通用 AI 助手，可以读写文件、执行命令、搜索代码、搜索网页",
        "version": "1.0.0",
        "author": "SkillMint",
        "recommended_model": "",
        "tags": [],
        "created_at": "2026-01-01T00:00:00Z",
        "username_hint": null,
        "encrypted_verify": ""
    });
    let builtin_json = builtin_manifest.to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let _ = sqlx::query(
        "INSERT OR IGNORE INTO installed_skills (id, manifest, installed_at, username, pack_path, source_type) VALUES ('builtin-general', ?, ?, '', '', 'builtin')"
    )
    .bind(&builtin_json)
    .bind(&now)
    .execute(&pool)
    .await;

    Ok(pool)
}
