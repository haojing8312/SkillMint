use super::chat_policy;
use super::chat_session_io;
use super::skills::DbState;
use crate::diagnostics::{self, ManagedDiagnosticsState};
use crate::session_journal::{SessionJournalStateHandle, SessionJournalStore};
use serde_json::{json, Value};
use tauri::{AppHandle, Manager, State};

fn session_ids_preview(list: &[serde_json::Value]) -> Vec<String> {
    list.iter()
        .filter_map(|item| item.get("id").and_then(Value::as_str))
        .map(ToString::to_string)
        .take(10)
        .collect()
}

async fn audit_session_list_result(
    app: &AppHandle,
    pool: &sqlx::SqlitePool,
    event: &str,
    message: &str,
    returned_sessions: &[serde_json::Value],
    extra: Option<Value>,
) {
    let Some(diagnostics_state) = app.try_state::<ManagedDiagnosticsState>() else {
        return;
    };
    let counts = super::desktop_lifecycle::collect_database_counts(pool).await;
    let storage = super::desktop_lifecycle::collect_database_storage_snapshot(app);
    let mut context = json!({
        "returned_session_count": returned_sessions.len(),
        "returned_session_ids_preview": session_ids_preview(returned_sessions),
        "counts": counts,
        "storage": storage,
    });
    if let Some(extra) = extra {
        context["extra"] = extra;
    }
    let _ = diagnostics::write_audit_record(
        &diagnostics_state.0.paths,
        "session",
        event,
        message,
        Some(context),
    );
}

#[tauri::command]
pub async fn get_messages(
    session_id: String,
    db: State<'_, DbState>,
) -> Result<Vec<serde_json::Value>, String> {
    chat_session_io::get_messages_with_pool(&db.0, &session_id).await
}

#[tauri::command]
pub async fn list_sessions(
    app: AppHandle,
    db: State<'_, DbState>,
) -> Result<Vec<serde_json::Value>, String> {
    let sessions = chat_session_io::list_sessions_with_pool(
        &db.0,
        chat_policy::permission_mode_label_for_display,
    )
    .await?;
    audit_session_list_result(
        &app,
        &db.0,
        "session_list_returned",
        "session list returned",
        &sessions,
        None,
    )
    .await;
    Ok(sessions)
}

#[tauri::command]
pub async fn get_sessions(
    app: AppHandle,
    skill_id: String,
    db: State<'_, DbState>,
) -> Result<Vec<serde_json::Value>, String> {
    let sessions = chat_session_io::list_sessions_with_pool(
        &db.0,
        chat_policy::permission_mode_label_for_display,
    )
    .await?;
    audit_session_list_result(
        &app,
        &db.0,
        "session_restore_snapshot",
        "session restore snapshot captured",
        &sessions,
        Some(json!({
            "requested_skill_id": skill_id,
        })),
    )
    .await;
    Ok(sessions)
}

#[tauri::command]
pub async fn update_session_workspace(
    app: AppHandle,
    session_id: String,
    workspace: String,
    db: State<'_, DbState>,
) -> Result<(), String> {
    chat_session_io::update_session_workspace_with_pool(&db.0, &session_id, &workspace).await?;
    if let Some(diagnostics_state) = app.try_state::<ManagedDiagnosticsState>() {
        let counts = super::desktop_lifecycle::collect_database_counts(&db.0).await;
        let storage = super::desktop_lifecycle::collect_database_storage_snapshot(&app);
        let _ = diagnostics::write_audit_record(
            &diagnostics_state.0.paths,
            "session",
            "session_workspace_updated",
            "session workspace updated",
            Some(json!({
                "session_id": session_id,
                "workspace": workspace,
                "counts": counts,
                "storage": storage,
            })),
        );
    }
    Ok(())
}

#[tauri::command]
pub async fn delete_session(
    app: AppHandle,
    session_id: String,
    db: State<'_, DbState>,
) -> Result<(), String> {
    chat_session_io::delete_session_with_pool(&db.0, &session_id).await?;
    if let Some(diagnostics_state) = app.try_state::<ManagedDiagnosticsState>() {
        let counts = super::desktop_lifecycle::collect_database_counts(&db.0).await;
        let storage = super::desktop_lifecycle::collect_database_storage_snapshot(&app);
        let _ = diagnostics::write_audit_record(
            &diagnostics_state.0.paths,
            "session",
            "delete_session",
            "session deleted",
            Some(serde_json::json!({
                "session_id": session_id,
                "counts": counts,
                "storage": storage,
            })),
        );
    }
    Ok(())
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
    app: AppHandle,
    session_id: String,
    db: State<'_, DbState>,
    journal: State<'_, SessionJournalStateHandle>,
) -> Result<String, String> {
    let markdown = chat_session_io::export_session_markdown_with_pool(
        &db.0,
        &session_id,
        Some(journal.0.as_ref()),
    )
    .await?;
    if let Some(diagnostics_state) = app.try_state::<ManagedDiagnosticsState>() {
        let counts = super::desktop_lifecycle::collect_database_counts(&db.0).await;
        let storage = super::desktop_lifecycle::collect_database_storage_snapshot(&app);
        let _ = diagnostics::write_audit_record(
            &diagnostics_state.0.paths,
            "session",
            "export_session",
            "session markdown exported",
            Some(serde_json::json!({
                "session_id": session_id,
                "markdown_length": markdown.len(),
                "counts": counts,
                "storage": storage,
            })),
        );
    }
    Ok(markdown)
}

pub async fn export_session_markdown_with_pool(
    pool: &sqlx::SqlitePool,
    session_id: &str,
    journal: Option<&SessionJournalStore>,
) -> Result<String, String> {
    chat_session_io::export_session_markdown_with_pool(pool, session_id, journal).await
}

#[tauri::command]
pub async fn write_export_file(
    app: AppHandle,
    path: String,
    content: String,
) -> Result<(), String> {
    chat_session_io::write_export_file_to_path(&path, &content)?;
    if let Some(diagnostics_state) = app.try_state::<ManagedDiagnosticsState>() {
        let storage = super::desktop_lifecycle::collect_database_storage_snapshot(&app);
        let _ = diagnostics::write_audit_record(
            &diagnostics_state.0.paths,
            "session",
            "write_export_file",
            "session export file written",
            Some(serde_json::json!({
                "path": path,
                "content_length": content.len(),
                "storage": storage,
            })),
        );
    }
    Ok(())
}
