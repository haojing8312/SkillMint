use super::chat_attachments;
use super::chat_compaction;
use super::chat_runtime_io as chat_io;
use super::chat_session_io;
use super::skills::DbState;
use crate::agent::runtime::{RuntimeTranscript, SessionAdmissionGateState, SessionRuntime};
use crate::agent::AgentExecutor;
use crate::approval_bus::ApprovalManager;
use crate::diagnostics::{self, ManagedDiagnosticsState};
use crate::runtime_environment::runtime_paths_from_app;
use crate::session_journal::SessionJournalStateHandle;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

pub use crate::agent::runtime::AskUserPendingSessionState;
/// 全局 AskUser 响应通道（用于 answer_user_question command）
pub use crate::agent::runtime::AskUserState;

/// 工具确认通道（用于 confirm_tool_execution command）
pub use crate::agent::runtime::ToolConfirmResponder;
pub struct ToolConfirmState(pub ToolConfirmResponder);

/// 通用审批管理器（高风险审批总线）
pub struct ApprovalManagerState(pub Arc<ApprovalManager>);

/// 旧版桌面确认对话框与审批总线之间的过渡桥接（仅保留最近一条待确认 approval）
pub struct PendingApprovalBridgeState(pub Arc<std::sync::Mutex<Option<String>>>);

/// 全局搜索缓存（跨会话共享，在 lib.rs 中创建）
pub use crate::agent::runtime::SearchCacheState;

/// Agent 取消标志（用于 cancel_agent command 停止正在执行的 Agent）
pub use crate::agent::runtime::CancelFlagState;

pub use crate::agent::runtime::{SkillRouteEvent, StreamToken};

#[cfg(test)]
pub(crate) fn build_group_orchestrator_report_preview(
    request: crate::agent::group_orchestrator::GroupRunRequest,
) -> String {
    let outcome = crate::agent::group_orchestrator::simulate_group_run(request);
    outcome.final_report
}

pub fn emit_skill_route_event(app: &AppHandle, event: SkillRouteEvent) {
    let _ = app.emit("skill-route-node-updated", event);
}

#[derive(Debug, Clone, Deserialize)]
pub struct SendMessageRequest {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub parts: Vec<SendMessagePart>,
    #[serde(rename = "maxIterations", default)]
    pub max_iterations: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct AttachmentInput {
    pub id: String,
    pub kind: String,
    #[serde(rename = "sourceType")]
    pub source_type: String,
    pub name: String,
    #[serde(rename = "declaredMimeType", default)]
    pub declared_mime_type: Option<String>,
    #[serde(rename = "sizeBytes", default)]
    pub size_bytes: Option<usize>,
    #[serde(rename = "sourcePayload", default)]
    pub source_payload: Option<String>,
    #[serde(rename = "sourceUri", default)]
    pub source_uri: Option<String>,
    #[serde(rename = "extractedText", default)]
    pub extracted_text: Option<String>,
    #[serde(default)]
    pub truncated: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(tag = "type")]
pub enum SendMessagePart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "attachment")]
    Attachment { attachment: AttachmentInput },
    #[serde(rename = "image")]
    Image {
        name: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
        size: usize,
        data: String,
    },
    #[serde(rename = "file_text")]
    FileText {
        name: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
        size: usize,
        text: String,
        truncated: Option<bool>,
    },
    #[serde(rename = "pdf_file")]
    PdfFile {
        name: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
        size: usize,
        data: String,
    },
}

impl SendMessageRequest {
    fn summary_text(&self) -> String {
        let mut summary_parts = Vec::new();
        let text = self
            .parts
            .iter()
            .filter_map(|part| match part {
                SendMessagePart::Text { text } => Some(text.trim()),
                _ => None,
            })
            .find(|text| !text.is_empty())
            .unwrap_or("");
        if !text.is_empty() {
            summary_parts.push(text.to_string());
        }
        let image_count = self
            .parts
            .iter()
            .filter(|part| match part {
                SendMessagePart::Image { .. } => true,
                SendMessagePart::Attachment { attachment } => attachment.kind == "image",
                _ => false,
            })
            .count();
        let text_file_count = self
            .parts
            .iter()
            .filter(|part| match part {
                SendMessagePart::FileText { .. } => true,
                SendMessagePart::Attachment { attachment } => {
                    attachment.kind == "document" && attachment_is_text_document(attachment)
                }
                _ => false,
            })
            .count();
        let pdf_file_count = self
            .parts
            .iter()
            .filter(|part| match part {
                SendMessagePart::PdfFile { .. } => true,
                SendMessagePart::Attachment { attachment } => attachment_is_pdf(attachment),
                _ => false,
            })
            .count();
        let document_count = self
            .parts
            .iter()
            .filter(|part| match part {
                SendMessagePart::Attachment { attachment } => {
                    attachment.kind == "document"
                        && !attachment_is_pdf(attachment)
                        && !attachment_is_text_document(attachment)
                }
                _ => false,
            })
            .count();
        let audio_count = self
            .parts
            .iter()
            .filter(|part| match part {
                SendMessagePart::Attachment { attachment } => attachment.kind == "audio",
                _ => false,
            })
            .count();
        let video_count = self
            .parts
            .iter()
            .filter(|part| match part {
                SendMessagePart::Attachment { attachment } => attachment.kind == "video",
                _ => false,
            })
            .count();
        if image_count > 0 {
            summary_parts.push(format!("[图片 {} 张]", image_count));
        }
        if text_file_count > 0 {
            summary_parts.push(format!("[文本文件 {} 个]", text_file_count));
        }
        if pdf_file_count > 0 {
            summary_parts.push(format!("[PDF {} 个]", pdf_file_count));
        }
        if document_count > 0 {
            summary_parts.push(format!("[文档 {} 个]", document_count));
        }
        if audio_count > 0 {
            summary_parts.push(format!("[音频 {} 个]", audio_count));
        }
        if video_count > 0 {
            summary_parts.push(format!("[视频 {} 个]", video_count));
        }
        summary_parts.join(" ")
    }
}

fn attachment_is_pdf(attachment: &AttachmentInput) -> bool {
    attachment.declared_mime_type.as_deref() == Some("application/pdf")
        || attachment.name.to_ascii_lowercase().ends_with(".pdf")
}

fn attachment_is_text_document(attachment: &AttachmentInput) -> bool {
    if attachment.kind != "document" || attachment_is_pdf(attachment) {
        return false;
    }

    let mime = attachment
        .declared_mime_type
        .as_deref()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if mime.starts_with("text/") {
        return true;
    }
    if matches!(mime.as_str(), "application/json" | "text/csv") {
        return true;
    }

    matches!(
        attachment
            .name
            .rsplit('.')
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str(),
        "txt"
            | "md"
            | "json"
            | "yaml"
            | "yml"
            | "xml"
            | "csv"
            | "tsv"
            | "log"
            | "ini"
            | "conf"
            | "env"
            | "js"
            | "jsx"
            | "ts"
            | "tsx"
            | "py"
            | "rs"
            | "go"
            | "java"
            | "c"
            | "cpp"
            | "h"
            | "cs"
            | "sh"
            | "ps1"
            | "sql"
    )
}

pub fn normalize_send_message_parts(parts: &[SendMessagePart]) -> Result<Vec<Value>, String> {
    chat_attachments::normalize_message_parts(parts)
}

pub fn normalize_send_message_parts_with_runtime_root(
    parts: &[SendMessagePart],
    runtime_root: PathBuf,
) -> Result<Vec<Value>, String> {
    let runtime_paths = crate::runtime_paths::RuntimePaths::new(runtime_root);
    chat_attachments::normalize_message_parts_with_runtime_paths(parts, &runtime_paths)
}

pub fn build_current_turn_message_with_runtime_root(
    api_format: &str,
    parts: &[Value],
    runtime_root: PathBuf,
) -> Result<Option<Value>, String> {
    let runtime_paths = crate::runtime_paths::RuntimePaths::new(runtime_root);
    RuntimeTranscript::build_current_turn_message_with_runtime_paths(
        api_format,
        parts,
        &runtime_paths,
    )
}

pub async fn normalize_send_message_parts_with_pool(
    parts: &[SendMessagePart],
    pool: &sqlx::SqlitePool,
) -> Result<Vec<Value>, String> {
    chat_attachments::normalize_message_parts_with_pool(parts, pool).await
}

pub async fn normalize_send_message_parts_with_pool_and_runtime_root(
    parts: &[SendMessagePart],
    pool: &sqlx::SqlitePool,
    runtime_root: PathBuf,
) -> Result<Vec<Value>, String> {
    let runtime_paths = crate::runtime_paths::RuntimePaths::new(runtime_root);
    chat_attachments::normalize_message_parts_with_pool_and_runtime_paths(
        parts,
        pool,
        &runtime_paths,
    )
    .await
}

#[tauri::command]
pub async fn create_session(
    app: AppHandle,
    skill_id: String,
    model_id: String,
    work_dir: Option<String>,
    employee_id: Option<String>,
    title: Option<String>,
    permission_mode: Option<String>,
    session_mode: Option<String>,
    team_id: Option<String>,
    db: State<'_, DbState>,
) -> Result<String, String> {
    let session_id = create_session_with_pool(
        &db.0,
        skill_id.clone(),
        model_id.clone(),
        work_dir.clone(),
        employee_id.clone(),
        title.clone(),
        permission_mode.clone(),
        session_mode.clone(),
        team_id.clone(),
    )
    .await?;

    if let Some(diagnostics_state) = app.try_state::<ManagedDiagnosticsState>() {
        let storage = super::desktop_lifecycle::collect_database_storage_snapshot(&app);
        let counts = super::desktop_lifecycle::collect_database_counts(&db.0).await;
        let _ = diagnostics::write_audit_record(
            &diagnostics_state.0.paths,
            "session",
            "create_session",
            "session created",
            Some(serde_json::json!({
                "session_id": session_id,
                "skill_id": skill_id,
                "model_id": model_id,
                "work_dir": work_dir,
                "employee_id": employee_id,
                "title": title,
                "permission_mode": permission_mode,
                "session_mode": session_mode,
                "team_id": team_id,
                "counts": counts,
                "storage": storage,
            })),
        );
    }

    Ok(session_id)
}

pub async fn create_session_with_pool(
    pool: &sqlx::SqlitePool,
    skill_id: String,
    model_id: String,
    work_dir: Option<String>,
    employee_id: Option<String>,
    title: Option<String>,
    permission_mode: Option<String>,
    session_mode: Option<String>,
    team_id: Option<String>,
) -> Result<String, String> {
    chat_session_io::create_session_with_pool(
        pool,
        skill_id,
        model_id,
        work_dir,
        employee_id,
        title,
        permission_mode,
        session_mode,
        team_id,
    )
    .await
}

#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    request: SendMessageRequest,
    db: State<'_, DbState>,
    agent_executor: State<'_, Arc<AgentExecutor>>,
    journal: State<'_, SessionJournalStateHandle>,
    cancel_flag: State<'_, CancelFlagState>,
) -> Result<(), String> {
    let session_id = request.session_id.clone();
    let user_message = request.summary_text();
    let runtime_paths = runtime_paths_from_app(&app)?;
    let user_message_parts = chat_attachments::normalize_message_parts_with_pool_and_runtime_paths(
        &request.parts,
        &db.0,
        &runtime_paths,
    )
    .await?;

    let admission_gate = app
        .try_state::<SessionAdmissionGateState>()
        .ok_or_else(|| "SessionAdmissionGateState unavailable".to_string())?;
    let _admission_lease = admission_gate
        .0
        .try_acquire(&session_id)
        .map_err(|conflict| conflict.to_string())?;

    if let Some(diagnostics_state) = app.try_state::<ManagedDiagnosticsState>() {
        let _ = diagnostics::write_log_record(
            &diagnostics_state.0.paths,
            diagnostics::LogLevel::Info,
            "chat",
            "send_message",
            "chat send_message invoked",
            Some(serde_json::json!({
                "session_id": session_id,
                "user_message_preview": user_message.chars().take(80).collect::<String>(),
            })),
        );
    }

    // 重置取消标志
    cancel_flag.0.store(false, Ordering::SeqCst);
    let cancel_flag_clone = cancel_flag.0.clone();

    // 保存用户消息
    let user_message_parts_json = serde_json::to_string(&user_message_parts)
        .map_err(|err| format!("序列化附件消息失败: {err}"))?;
    let msg_id = chat_io::insert_session_message_with_pool(
        &db.0,
        &session_id,
        "user",
        &user_message,
        Some(&user_message_parts_json),
    )
    .await?;
    if let Some(diagnostics_state) = app.try_state::<ManagedDiagnosticsState>() {
        let counts = super::desktop_lifecycle::collect_database_counts(&db.0).await;
        let storage = super::desktop_lifecycle::collect_database_storage_snapshot(&app);
        let _ = diagnostics::write_audit_record(
            &diagnostics_state.0.paths,
            "message",
            "message_inserted",
            "user message inserted",
            Some(serde_json::json!({
                "session_id": session_id,
                "message_id": msg_id,
                "role": "user",
                "content_preview": user_message.chars().take(120).collect::<String>(),
                "content_parts_count": user_message_parts.len(),
                "counts": counts,
                "storage": storage,
            })),
        );
    }
    chat_io::maybe_update_session_title_from_first_user_message_with_pool(
        &db.0,
        &session_id,
        &user_message,
    )
    .await?;
    if let Some(diagnostics_state) = app.try_state::<ManagedDiagnosticsState>() {
        let title_row = sqlx::query_scalar::<_, String>(
            "SELECT COALESCE(title, '') FROM sessions WHERE id = ?",
        )
        .bind(&session_id)
        .fetch_optional(&db.0)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();
        if !title_row.trim().is_empty() {
            let counts = super::desktop_lifecycle::collect_database_counts(&db.0).await;
            let storage = super::desktop_lifecycle::collect_database_storage_snapshot(&app);
            let _ = diagnostics::write_audit_record(
                &diagnostics_state.0.paths,
                "session",
                "session_title_updated",
                "session title evaluated after first user message",
                Some(serde_json::json!({
                    "session_id": session_id,
                    "title": title_row,
                    "counts": counts,
                    "storage": storage,
                })),
            );
        }
    }

    if chat_io::maybe_handle_team_entry_pre_execution_with_pool(
        &app,
        &db.0,
        journal.0.as_ref(),
        &session_id,
        &msg_id,
        &user_message,
    )
    .await?
    {
        return Ok(());
    }

    // 使用全局工具确认通道（在 lib.rs 中创建）
    let tool_confirm_responder = app.state::<ToolConfirmState>().0.clone();
    SessionRuntime::run_send_message(
        &app,
        agent_executor.inner(),
        &db.0,
        journal.0.as_ref(),
        &session_id,
        &msg_id,
        &user_message,
        &user_message_parts,
        request.max_iterations,
        cancel_flag_clone.clone(),
        tool_confirm_responder,
    )
    .await
    .map_err(|error| {
        if let Some(diagnostics_state) = app.try_state::<ManagedDiagnosticsState>() {
            let _ = diagnostics::write_log_record(
                &diagnostics_state.0.paths,
                diagnostics::LogLevel::Error,
                "chat",
                "send_message_finalize_failed",
                &error,
                Some(serde_json::json!({
                    "session_id": session_id,
                    "message_id": msg_id,
                })),
            );
        }
        error
    })
}

#[cfg(test)]
mod tests {
    use super::build_group_orchestrator_report_preview;
    use crate::agent::runtime::SessionAdmissionConflict;
    use crate::commands::chat::{AttachmentInput, SendMessagePart, SendMessageRequest};
    use crate::commands::chat_runtime_io;
    use std::collections::HashMap;
    use std::path::Path;

    #[test]
    fn profile_memory_locator_points_to_profile_home() {
        let runtime_root = Path::new("C:/workclaw/runtime-root");
        let memory_root = runtime_root.join("memory");

        let locator = chat_runtime_io::build_profile_memory_locator(
            runtime_root,
            &memory_root,
            Some(Path::new("E:/workspace/acme")),
            "builtin-general",
            "Sales Lead/华东",
            Some("profile-1"),
            Some("planner-role"),
        );

        assert_eq!(
            locator.profile_memory_dir.as_deref(),
            Some(
                runtime_root
                    .join("profiles")
                    .join("profile-1")
                    .join("memories")
                    .as_path()
            )
        );
        assert_eq!(
            locator
                .project_memory_file
                .as_ref()
                .and_then(|path| path.parent())
                .and_then(|path| path.file_name())
                .map(|name| name.to_string_lossy().to_string()),
            Some("PROJECTS".to_string())
        );
    }

    #[test]
    fn profile_memory_bundle_reads_only_profile_memory_file() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let runtime_root = tmp.path().join("runtime-root");
        let memory_root = runtime_root.join("memory");
        let profile_memory_dir = runtime_root
            .join("profiles")
            .join("profile-1")
            .join("memories");
        std::fs::create_dir_all(&profile_memory_dir).expect("profile memory dir");
        std::fs::write(profile_memory_dir.join("MEMORY.md"), "profile memory")
            .expect("write profile memory");

        let locator = chat_runtime_io::build_profile_memory_locator(
            &runtime_root,
            &memory_root,
            None,
            "builtin-general",
            "planner",
            Some("profile-1"),
            None,
        );
        let bundle = chat_runtime_io::load_profile_memory_bundle(&locator);

        assert_eq!(bundle.content, "profile memory");
        assert_eq!(bundle.source, "profile");
        assert_eq!(
            bundle.source_path,
            Some(profile_memory_dir.join("MEMORY.md"))
        );
    }

    #[test]
    fn profile_memory_bundle_includes_workspace_project_memory() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let runtime_root = tmp.path().join("runtime-root");
        let memory_root = runtime_root.join("memory");
        let profile_memory_dir = runtime_root
            .join("profiles")
            .join("profile-1")
            .join("memories");
        std::fs::create_dir_all(&profile_memory_dir).expect("profile memory dir");
        std::fs::write(profile_memory_dir.join("MEMORY.md"), "profile fact")
            .expect("write profile memory");

        let locator = chat_runtime_io::build_profile_memory_locator(
            &runtime_root,
            &memory_root,
            Some(Path::new("E:/workspace/acme")),
            "builtin-general",
            "planner",
            Some("profile-1"),
            None,
        );
        let project_memory_file = locator
            .project_memory_file
            .as_ref()
            .expect("project memory file");
        std::fs::create_dir_all(project_memory_file.parent().expect("project dir"))
            .expect("create project dir");
        std::fs::write(project_memory_file, "project fact").expect("write project memory");

        let bundle = chat_runtime_io::load_profile_memory_bundle(&locator);

        assert_eq!(bundle.source, "profile");
        assert!(bundle.content.contains("profile fact"));
        assert!(bundle.content.contains("Project Memory"));
        assert!(bundle.content.contains("project fact"));
    }

    #[test]
    fn profile_memory_bundle_trims_to_budget() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let runtime_root = tmp.path().join("runtime-root");
        let memory_root = runtime_root.join("memory");
        let profile_memory_dir = runtime_root
            .join("profiles")
            .join("profile-1")
            .join("memories");
        std::fs::create_dir_all(&profile_memory_dir).expect("profile memory dir");
        std::fs::write(
            profile_memory_dir.join("MEMORY.md"),
            format!(
                "ancient-start {}\n{}",
                "old ".repeat(200),
                "fresh memory tail"
            ),
        )
        .expect("write profile memory");

        let locator = chat_runtime_io::build_profile_memory_locator(
            &runtime_root,
            &memory_root,
            None,
            "builtin-general",
            "planner",
            Some("profile-1"),
            None,
        );
        let bundle = chat_runtime_io::load_profile_memory_bundle_with_budget(&locator, 80);

        assert!(bundle.content.contains("fresh memory tail"));
        assert!(!bundle.content.contains("ancient-start"));
        assert!(bundle.content.chars().count() <= 140);
    }

    #[test]
    fn profile_memory_bundle_falls_back_to_legacy_memory_file() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let runtime_root = tmp.path().join("runtime-root");
        let memory_root = runtime_root.join("memory");
        let legacy_memory_dir = memory_root
            .join("employees")
            .join("planner")
            .join("skills")
            .join("builtin-general");
        std::fs::create_dir_all(&legacy_memory_dir).expect("legacy memory dir");
        std::fs::write(legacy_memory_dir.join("MEMORY.md"), "legacy memory")
            .expect("write legacy memory");

        let locator = chat_runtime_io::build_profile_memory_locator(
            &runtime_root,
            &memory_root,
            None,
            "builtin-general",
            "planner",
            Some("profile-1"),
            None,
        );
        let bundle = chat_runtime_io::load_profile_memory_bundle(&locator);

        assert_eq!(bundle.content, "legacy memory");
        assert_eq!(bundle.source, "legacy");
        assert_eq!(
            bundle.source_path,
            Some(legacy_memory_dir.join("MEMORY.md"))
        );
    }

    #[test]
    fn profile_memory_status_reports_profile_memory_as_active_source() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let runtime_root = tmp.path().join("runtime-root");
        let memory_root = runtime_root.join("memory");
        let profile_memory_dir = runtime_root
            .join("profiles")
            .join("profile-1")
            .join("memories");
        std::fs::create_dir_all(&profile_memory_dir).expect("profile memory dir");
        std::fs::write(profile_memory_dir.join("MEMORY.md"), "profile memory")
            .expect("write profile memory");

        let locator = chat_runtime_io::build_profile_memory_locator(
            &runtime_root,
            &memory_root,
            None,
            "builtin-general",
            "planner",
            Some("profile-1"),
            None,
        );
        let status = chat_runtime_io::collect_profile_memory_status(&locator);

        assert_eq!(status.active_source, "profile");
        assert_eq!(
            status.active_source_path,
            Some(profile_memory_dir.join("MEMORY.md"))
        );
        assert!(status.profile_memory_file_exists);
    }

    #[test]
    fn profile_memory_status_ignores_legacy_fallback() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let runtime_root = tmp.path().join("runtime-root");
        let memory_root = runtime_root.join("memory");
        let legacy_memory_dir = memory_root
            .join("employees")
            .join("planner")
            .join("skills")
            .join("builtin-general");
        std::fs::create_dir_all(&legacy_memory_dir).expect("legacy memory dir");
        std::fs::write(legacy_memory_dir.join("MEMORY.md"), "legacy memory")
            .expect("write legacy memory");

        let locator = chat_runtime_io::build_profile_memory_locator(
            &runtime_root,
            &memory_root,
            None,
            "builtin-general",
            "planner",
            Some("profile-1"),
            None,
        );
        let status = chat_runtime_io::collect_profile_memory_status(&locator);

        assert_eq!(status.active_source, "none");
        assert_eq!(status.active_source_path, None);
        assert!(!status.profile_memory_file_exists);
    }

    #[test]
    fn profile_memory_status_reports_none_when_no_memory_file_exists() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let runtime_root = tmp.path().join("runtime-root");
        let memory_root = runtime_root.join("memory");

        let locator = chat_runtime_io::build_profile_memory_locator(
            &runtime_root,
            &memory_root,
            None,
            "builtin-general",
            "planner",
            Some("profile-1"),
            None,
        );
        let status = chat_runtime_io::collect_profile_memory_status(&locator);

        assert_eq!(status.active_source, "none");
        assert_eq!(status.active_source_path, None);
        assert!(!status.profile_memory_file_exists);
    }

    #[test]
    fn extract_skill_prompt_supports_lowercase_skill_md() {
        let mut files = HashMap::new();
        files.insert("skill.md".to_string(), b"# lowercase skill".to_vec());
        let content = chat_runtime_io::extract_skill_prompt_from_decrypted_files(&files);
        assert_eq!(content.as_deref(), Some("# lowercase skill"));
    }

    #[test]
    fn group_orchestrator_preview_contains_plan_execute_summary_sections() {
        let report = build_group_orchestrator_report_preview(
            crate::agent::group_orchestrator::GroupRunRequest {
                group_id: "group-1".to_string(),
                coordinator_employee_id: "project_manager".to_string(),
                planner_employee_id: None,
                reviewer_employee_id: None,
                member_employee_ids: vec![
                    "project_manager".to_string(),
                    "dev_team".to_string(),
                    "qa_team".to_string(),
                ],
                execute_targets: Vec::new(),
                user_goal: "实现群组协作编排".to_string(),
                execution_window: 3,
                timeout_employee_ids: Vec::new(),
                max_retry_per_step: 1,
            },
        );

        assert!(report.contains("计划"));
        assert!(report.contains("执行"));
        assert!(report.contains("汇报"));
    }

    #[test]
    fn session_run_conflict_error_is_stable() {
        let error = SessionAdmissionConflict::new("session-1").to_string();

        assert!(error.starts_with("SESSION_RUN_CONFLICT:"));
        assert!(error.contains("当前会话仍在执行中"));
    }

    #[test]
    fn send_message_summary_text_counts_pdf_attachments() {
        let request = super::SendMessageRequest {
            session_id: "session-1".to_string(),
            parts: vec![
                SendMessagePart::Text {
                    text: "请阅读附件".to_string(),
                },
                SendMessagePart::PdfFile {
                    name: "brief.pdf".to_string(),
                    mime_type: "application/pdf".to_string(),
                    size: 10,
                    data: "abc".to_string(),
                },
            ],
            max_iterations: None,
        };

        assert!(request.summary_text().contains("[PDF 1 个]"));
    }

    #[test]
    fn send_message_summary_text_counts_audio_and_video_attachments() {
        let request = SendMessageRequest {
            session_id: "session-1".to_string(),
            parts: vec![
                SendMessagePart::Text {
                    text: "请处理这些媒体附件".to_string(),
                },
                SendMessagePart::Attachment {
                    attachment: AttachmentInput {
                        id: "att-audio-1".to_string(),
                        kind: "audio".to_string(),
                        source_type: "browser_file".to_string(),
                        name: "memo.mp3".to_string(),
                        declared_mime_type: Some("audio/mpeg".to_string()),
                        size_bytes: Some(128),
                        source_payload: Some("ZmFrZQ==".to_string()),
                        source_uri: None,
                        extracted_text: None,
                        truncated: None,
                    },
                },
                SendMessagePart::Attachment {
                    attachment: AttachmentInput {
                        id: "att-video-1".to_string(),
                        kind: "video".to_string(),
                        source_type: "browser_file".to_string(),
                        name: "demo.mp4".to_string(),
                        declared_mime_type: Some("video/mp4".to_string()),
                        size_bytes: Some(256),
                        source_payload: Some("ZmFrZV92aWRlbw==".to_string()),
                        source_uri: None,
                        extracted_text: None,
                        truncated: None,
                    },
                },
            ],
            max_iterations: None,
        };

        let summary = request.summary_text();
        assert!(summary.contains("[音频 1 个]"));
        assert!(summary.contains("[视频 1 个]"));
    }

    #[test]
    fn send_message_summary_text_counts_binary_document_attachments() {
        let request = SendMessageRequest {
            session_id: "session-1".to_string(),
            parts: vec![SendMessagePart::Attachment {
                attachment: AttachmentInput {
                    id: "att-doc-1".to_string(),
                    kind: "document".to_string(),
                    source_type: "browser_file".to_string(),
                    name: "budget.xlsx".to_string(),
                    declared_mime_type: Some(
                        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                            .to_string(),
                    ),
                    size_bytes: Some(256),
                    source_payload: None,
                    source_uri: None,
                    extracted_text: None,
                    truncated: None,
                },
            }],
            max_iterations: None,
        };

        let summary = request.summary_text();
        assert!(summary.contains("[文档 1 个]"));
    }
}

pub use super::chat_session_commands::export_session_markdown_with_pool;

pub use super::chat_compaction::CompactionResult;

/// 手动触发上下文压缩
#[tauri::command]
pub async fn compact_context(
    session_id: String,
    db: State<'_, DbState>,
    app: AppHandle,
) -> Result<CompactionResult, String> {
    let runtime_paths = runtime_paths_from_app(&app)?;
    chat_compaction::compact_context_with_pool(&db.0, &session_id, &runtime_paths.transcripts_dir)
        .await
}
