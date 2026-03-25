use crate::agent::run_guard::RunStopReason;
use crate::agent::runtime::RunRegistry;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone)]
pub struct SessionJournalStore {
    root: PathBuf,
    run_registry: Arc<RunRegistry>,
}

impl SessionJournalStore {
    pub fn new(root: PathBuf) -> Self {
        Self::with_registry(root, Arc::new(RunRegistry::new()))
    }

    pub fn with_registry(root: PathBuf, run_registry: Arc<RunRegistry>) -> Self {
        Self { root, run_registry }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn run_registry(&self) -> Arc<RunRegistry> {
        Arc::clone(&self.run_registry)
    }

    pub async fn append_event(
        &self,
        session_id: &str,
        event: SessionRunEvent,
    ) -> Result<(), String> {
        let session_dir = self.session_dir(session_id);
        fs::create_dir_all(&session_dir)
            .await
            .map_err(|e| format!("创建 session journal 目录失败: {e}"))?;

        let record = SessionJournalRecord {
            session_id: session_id.to_string(),
            recorded_at: Utc::now().to_rfc3339(),
            event: event.clone(),
        };

        let mut events_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(session_dir.join("events.jsonl"))
            .await
            .map_err(|e| format!("打开 session events 文件失败: {e}"))?;
        let line = serde_json::to_string(&record)
            .map_err(|e| format!("序列化 session event 失败: {e}"))?;
        events_file
            .write_all(format!("{line}\n").as_bytes())
            .await
            .map_err(|e| format!("写入 session event 失败: {e}"))?;
        events_file
            .flush()
            .await
            .map_err(|e| format!("刷新 session event 失败: {e}"))?;

        let mut state = self.read_state(session_id).await?; 
        apply_event(&mut state, &event);
        self.run_registry.sync_session_projection(
            session_id,
            state.current_run_id.as_deref(),
        );
        let state_json = serde_json::to_string_pretty(&state)
            .map_err(|e| format!("序列化 session state 失败: {e}"))?;
        fs::write(session_dir.join("state.json"), state_json)
            .await
            .map_err(|e| format!("写入 session state 失败: {e}"))?;

        let transcript = render_transcript_markdown(&state);
        fs::write(session_dir.join("transcript.md"), transcript)
            .await
            .map_err(|e| format!("写入 session transcript 失败: {e}"))?;

        Ok(())
    }

    pub async fn read_state(&self, session_id: &str) -> Result<SessionJournalState, String> {
        let path = self.session_dir(session_id).join("state.json");
        if !path.exists() {
            let state = SessionJournalState {
                session_id: session_id.to_string(),
                ..SessionJournalState::default()
            };
            self.run_registry.hydrate_from_session_state(&state);
            return Ok(state);
        }

        let raw = fs::read_to_string(&path)
            .await
            .map_err(|e| format!("读取 session state 失败: {e}"))?;
        let mut state = serde_json::from_str::<SessionJournalState>(&raw)
            .map_err(|e| format!("解析 session state 失败: {e}"))?;
        if state.session_id.trim().is_empty() {
            state.session_id = session_id.to_string();
        }
        state.current_run_id = derive_current_run_id(&state);
        self.run_registry.hydrate_from_session_state(&state);
        Ok(state)
    }

    fn session_dir(&self, session_id: &str) -> PathBuf {
        self.root.join(session_id)
    }
}

#[derive(Debug, Clone)]
pub struct SessionJournalStateHandle(pub Arc<SessionJournalStore>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionJournalState {
    pub session_id: String,
    #[serde(default)]
    pub current_run_id: Option<String>,
    #[serde(default)]
    pub runs: Vec<SessionRunSnapshot>,
}

impl Default for SessionJournalState {
    fn default() -> Self {
        Self {
            session_id: String::new(),
            current_run_id: None,
            runs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRunSnapshot {
    pub run_id: String,
    pub user_message_id: String,
    pub status: SessionRunStatus,
    pub buffered_text: String,
    pub last_error_kind: Option<String>,
    pub last_error_message: Option<String>,
}

impl SessionRunSnapshot {
    fn new(run_id: &str) -> Self {
        Self {
            run_id: run_id.to_string(),
            user_message_id: String::new(),
            status: SessionRunStatus::Queued,
            buffered_text: String::new(),
            last_error_kind: None,
            last_error_message: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionRunStatus {
    Queued,
    Thinking,
    ToolCalling,
    WaitingApproval,
    WaitingUser,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SessionRunEvent {
    RunStarted {
        run_id: String,
        user_message_id: String,
    },
    AssistantChunkAppended {
        run_id: String,
        chunk: String,
    },
    ToolStarted {
        run_id: String,
        tool_name: String,
        call_id: String,
        input: Value,
    },
    ToolCompleted {
        run_id: String,
        tool_name: String,
        call_id: String,
        input: Value,
        output: String,
        is_error: bool,
    },
    ApprovalRequested {
        run_id: String,
        approval_id: String,
        tool_name: String,
        call_id: String,
        input: Value,
        summary: String,
        impact: Option<String>,
        irreversible: bool,
    },
    RunCompleted {
        run_id: String,
    },
    RunGuardWarning {
        run_id: String,
        warning_kind: String,
        title: String,
        message: String,
        detail: Option<String>,
        last_completed_step: Option<String>,
    },
    RunStopped {
        run_id: String,
        stop_reason: RunStopReason,
    },
    RunFailed {
        run_id: String,
        error_kind: String,
        error_message: String,
    },
    RunCancelled {
        run_id: String,
        reason: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionJournalRecord {
    pub session_id: String,
    pub recorded_at: String,
    #[serde(flatten)]
    pub event: SessionRunEvent,
}

fn apply_event(state: &mut SessionJournalState, event: &SessionRunEvent) {
    let run_id = match event {
        SessionRunEvent::RunStarted { run_id, .. }
        | SessionRunEvent::AssistantChunkAppended { run_id, .. }
        | SessionRunEvent::ToolStarted { run_id, .. }
        | SessionRunEvent::ToolCompleted { run_id, .. }
        | SessionRunEvent::ApprovalRequested { run_id, .. }
        | SessionRunEvent::RunCompleted { run_id }
        | SessionRunEvent::RunGuardWarning { run_id, .. }
        | SessionRunEvent::RunStopped { run_id, .. }
        | SessionRunEvent::RunFailed { run_id, .. }
        | SessionRunEvent::RunCancelled { run_id, .. } => run_id.clone(),
    };
    let run_index = upsert_run_index(state, &run_id);

    match event {
        SessionRunEvent::RunStarted {
            user_message_id,
            ..
        } => {
            let run = &mut state.runs[run_index];
            run.user_message_id = user_message_id.clone();
            run.status = SessionRunStatus::Thinking;
            run.last_error_kind = None;
            run.last_error_message = None;
        }
        SessionRunEvent::AssistantChunkAppended { chunk, .. } => {
            let run = &mut state.runs[run_index];
            run.buffered_text.push_str(chunk);
            if matches!(run.status, SessionRunStatus::Queued) {
                run.status = SessionRunStatus::Thinking;
            }
        }
        SessionRunEvent::ToolStarted { .. } => {
            let run = &mut state.runs[run_index];
            run.status = SessionRunStatus::ToolCalling;
        }
        SessionRunEvent::ToolCompleted { .. } => {
            let run = &mut state.runs[run_index];
            run.status = SessionRunStatus::Thinking;
        }
        SessionRunEvent::ApprovalRequested { .. } => {
            let run = &mut state.runs[run_index];
            run.status = SessionRunStatus::WaitingApproval;
        }
        SessionRunEvent::RunCompleted { .. } => {
            state.runs[run_index].status = SessionRunStatus::Completed;
        }
        SessionRunEvent::RunGuardWarning { .. } => {}
        SessionRunEvent::RunStopped { stop_reason, .. } => {
            let run = &mut state.runs[run_index];
            run.status = SessionRunStatus::Failed;
            run.last_error_kind = Some(stop_reason.kind.as_key().to_string());
            run.last_error_message = Some(format_run_stop_message(stop_reason));
        }
        SessionRunEvent::RunFailed {
            error_kind,
            error_message,
            ..
        } => {
            let run = &mut state.runs[run_index];
            run.status = SessionRunStatus::Failed;
            run.last_error_kind = Some(error_kind.clone());
            run.last_error_message = Some(error_message.clone());
        }
        SessionRunEvent::RunCancelled { reason, .. } => {
            let run = &mut state.runs[run_index];
            run.status = SessionRunStatus::Cancelled;
            run.last_error_kind = Some("cancelled".to_string());
            run.last_error_message = reason.clone();
        }
    }

    state.current_run_id = derive_current_run_id(state);
}

fn upsert_run_index(state: &mut SessionJournalState, run_id: &str) -> usize {
    if let Some(index) = state.runs.iter().position(|run| run.run_id == run_id) {
        return index;
    }
    state.runs.push(SessionRunSnapshot::new(run_id));
    state.runs.len() - 1
}

fn derive_current_run_id(state: &SessionJournalState) -> Option<String> {
    state
        .runs
        .iter()
        .rev()
        .find(|run| {
            matches!(
                run.status,
                SessionRunStatus::Thinking
                    | SessionRunStatus::ToolCalling
                    | SessionRunStatus::WaitingApproval
            )
        })
        .map(|run| run.run_id.clone())
}

fn format_run_stop_message(stop_reason: &RunStopReason) -> String {
    let mut lines = vec![stop_reason.message.clone()];
    if let Some(detail) = stop_reason.detail.as_deref() {
        if !detail.trim().is_empty() && detail != stop_reason.message {
            lines.push(detail.to_string());
        }
    }
    if let Some(step) = stop_reason.last_completed_step.as_deref() {
        if !step.trim().is_empty() {
            lines.push(format!("最后完成步骤：{step}"));
        }
    }
    lines.join("\n")
}

fn render_transcript_markdown(state: &SessionJournalState) -> String {
    let mut lines = vec![format!("# Session {}", state.session_id), String::new()];

    for run in &state.runs {
        lines.push(format!("## Run {}", run.run_id));
        lines.push(format!("- status: {}", run.status.as_str()));
        if !run.user_message_id.trim().is_empty() {
            lines.push(format!("- user_message_id: {}", run.user_message_id));
        }
        if let Some(error_kind) = &run.last_error_kind {
            if !error_kind.trim().is_empty() {
                lines.push(format!("- error_kind: {}", error_kind));
            }
        }
        if let Some(error_message) = &run.last_error_message {
            if !error_message.trim().is_empty() {
                lines.push(format!("- error_message: {}", error_message));
            }
        }
        lines.push(String::new());
        if !run.buffered_text.trim().is_empty() {
            lines.push("```text".to_string());
            lines.push(run.buffered_text.clone());
            lines.push("```".to_string());
            lines.push(String::new());
        }
    }

    lines.join("\n")
}

impl SessionRunStatus {
    fn as_str(&self) -> &'static str {
        match self {
            SessionRunStatus::Queued => "queued",
            SessionRunStatus::Thinking => "thinking",
            SessionRunStatus::ToolCalling => "tool_calling",
            SessionRunStatus::WaitingApproval => "waiting_approval",
            SessionRunStatus::WaitingUser => "waiting_user",
            SessionRunStatus::Completed => "completed",
            SessionRunStatus::Failed => "failed",
            SessionRunStatus::Cancelled => "cancelled",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::format_run_stop_message;
    use crate::agent::run_guard::RunStopReason;
    use tempfile::tempdir;
    use tokio::fs;

    #[tokio::test]
    async fn read_state_recovers_active_run_id_from_legacy_snapshot() {
        let journal_root = tempdir().expect("journal tempdir");
        let session_id = "session-legacy-active";
        let session_dir = journal_root.path().join(session_id);
        fs::create_dir_all(&session_dir)
            .await
            .expect("create session dir");

        let state_json = serde_json::json!({
            "session_id": session_id,
            "runs": [
                {
                    "run_id": "run-finished",
                    "user_message_id": "user-1",
                    "status": "completed",
                    "buffered_text": "已完成",
                    "last_error_kind": null,
                    "last_error_message": null
                },
                {
                    "run_id": "run-active",
                    "user_message_id": "user-2",
                    "status": "waiting_approval",
                    "buffered_text": "等待确认",
                    "last_error_kind": null,
                    "last_error_message": null
                }
            ]
        })
        .to_string();
        fs::write(session_dir.join("state.json"), state_json)
            .await
            .expect("write legacy state");

        let store = super::SessionJournalStore::new(journal_root.path().to_path_buf());
        let state = store.read_state(session_id).await.expect("read state");

        assert_eq!(state.current_run_id.as_deref(), Some("run-active"));
        assert_eq!(
            store.run_registry().resolve_current_run_id(session_id).as_deref(),
            Some("run-active")
        );
    }

    #[test]
    fn format_run_stop_message_preserves_policy_blocked_detail() {
        let reason = RunStopReason::policy_blocked(
            "目标路径不在当前工作目录范围内。你可以先切换当前会话的工作目录后重试。",
        )
        .with_last_completed_step("已读取当前工作区");

        let formatted = format_run_stop_message(&reason);

        assert!(formatted.contains("本次请求触发了安全或工作区限制"));
        assert!(formatted
            .contains("目标路径不在当前工作目录范围内。你可以先切换当前会话的工作目录后重试。"));
        assert!(formatted.contains("最后完成步骤：已读取当前工作区"));
    }
}
