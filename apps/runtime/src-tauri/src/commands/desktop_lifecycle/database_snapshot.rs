use crate::diagnostics;
use serde_json::Value;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager};

pub(crate) async fn collect_database_counts(pool: &SqlitePool) -> Value {
    let session_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sessions")
        .fetch_one(pool)
        .await
        .ok();
    let message_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM messages")
        .fetch_one(pool)
        .await
        .ok();
    serde_json::json!({
        "session_count": session_count,
        "message_count": message_count,
    })
}

pub(crate) fn collect_database_storage_snapshot(app: &AppHandle) -> Value {
    match app.path().app_data_dir() {
        Ok(app_data_dir) => serde_json::json!({
            "app_data_dir": app_data_dir.to_string_lossy().to_string(),
            "sqlite_files": diagnostics::collect_sqlite_storage_snapshot(&app_data_dir),
        }),
        Err(error) => serde_json::json!({
            "app_data_dir_error": error.to_string(),
        }),
    }
}
