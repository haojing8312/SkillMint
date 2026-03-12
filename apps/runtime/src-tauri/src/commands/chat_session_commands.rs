use super::chat_policy;
use super::chat_session_io;
use super::skills::DbState;
use crate::session_journal::{SessionJournalStateHandle, SessionJournalStore};
use tauri::State;

#[tauri::command]
pub async fn get_messages(
    session_id: String,
    db: State<'_, DbState>,
) -> Result<Vec<serde_json::Value>, String> {
    chat_session_io::get_messages_with_pool(&db.0, &session_id).await
}

#[tauri::command]
pub async fn list_sessions(db: State<'_, DbState>) -> Result<Vec<serde_json::Value>, String> {
    chat_session_io::list_sessions_with_pool(&db.0, chat_policy::permission_mode_label_for_display)
        .await
}

#[tauri::command]
pub async fn get_sessions(
    skill_id: String,
    db: State<'_, DbState>,
) -> Result<Vec<serde_json::Value>, String> {
    let _ = &skill_id;
    list_sessions(db).await
}

#[tauri::command]
pub async fn update_session_workspace(
    session_id: String,
    workspace: String,
    db: State<'_, DbState>,
) -> Result<(), String> {
    chat_session_io::update_session_workspace_with_pool(&db.0, &session_id, &workspace).await
}

#[tauri::command]
pub async fn delete_session(session_id: String, db: State<'_, DbState>) -> Result<(), String> {
    chat_session_io::delete_session_with_pool(&db.0, &session_id).await
}

#[tauri::command]
pub async fn search_sessions_global(
    query: String,
    db: State<'_, DbState>,
) -> Result<Vec<serde_json::Value>, String> {
    chat_session_io::search_sessions_global_with_pool(&db.0, &query).await
}

#[tauri::command]
pub async fn search_sessions(
    skill_id: String,
    query: String,
    db: State<'_, DbState>,
) -> Result<Vec<serde_json::Value>, String> {
    let _ = &skill_id;
    search_sessions_global(query, db).await
}

#[tauri::command]
pub async fn export_session(
    session_id: String,
    db: State<'_, DbState>,
    journal: State<'_, SessionJournalStateHandle>,
) -> Result<String, String> {
    chat_session_io::export_session_markdown_with_pool(&db.0, &session_id, Some(journal.0.as_ref()))
        .await
}

pub async fn export_session_markdown_with_pool(
    pool: &sqlx::SqlitePool,
    session_id: &str,
    journal: Option<&SessionJournalStore>,
) -> Result<String, String> {
    chat_session_io::export_session_markdown_with_pool(pool, session_id, journal).await
}

#[tauri::command]
pub async fn write_export_file(path: String, content: String) -> Result<(), String> {
    chat_session_io::write_export_file_to_path(&path, &content)
}
