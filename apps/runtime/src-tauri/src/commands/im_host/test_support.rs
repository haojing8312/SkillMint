use crate::approval_bus::PendingApprovalRecord;
use sqlx::SqlitePool;

fn parse_lifecycle_phase(phase: &str) -> Result<super::ImReplyLifecyclePhase, String> {
    match phase.trim() {
        "processing_started" => Ok(super::ImReplyLifecyclePhase::ProcessingStarted),
        "ask_user_requested" => Ok(super::ImReplyLifecyclePhase::AskUserRequested),
        "ask_user_answered" => Ok(super::ImReplyLifecyclePhase::AskUserAnswered),
        "approval_requested" => Ok(super::ImReplyLifecyclePhase::ApprovalRequested),
        "approval_resolved" => Ok(super::ImReplyLifecyclePhase::ApprovalResolved),
        "resumed" => Ok(super::ImReplyLifecyclePhase::Resumed),
        other => Err(format!("unsupported lifecycle phase for test support: {other}")),
    }
}

pub async fn maybe_notify_registered_ask_user_requested_with_pool(
    pool: &SqlitePool,
    session_id: &str,
    question: &str,
    options: &[String],
    sidecar_base_url: Option<String>,
) -> Result<bool, String> {
    super::interactive_dispatch::maybe_notify_registered_ask_user_requested_with_pool(
        pool,
        session_id,
        question,
        options,
        sidecar_base_url,
    )
    .await
}

pub async fn maybe_notify_registered_approval_requested_with_pool(
    pool: &SqlitePool,
    session_id: &str,
    record: &PendingApprovalRecord,
    sidecar_base_url: Option<String>,
) -> Result<bool, String> {
    super::interactive_dispatch::maybe_notify_registered_approval_requested_with_pool(
        pool,
        session_id,
        record,
        sidecar_base_url,
    )
    .await
}

pub async fn maybe_emit_registered_host_lifecycle_phase_for_session_with_pool(
    pool: &SqlitePool,
    session_id: &str,
    logical_reply_id: Option<&str>,
    phase: &str,
    account_id: Option<&str>,
) -> Result<bool, String> {
    super::lifecycle::maybe_emit_registered_host_lifecycle_phase_for_session_with_pool(
        pool,
        session_id,
        logical_reply_id,
        parse_lifecycle_phase(phase)?,
        account_id,
    )
    .await
}

pub async fn maybe_dispatch_registered_im_session_reply_with_pool(
    pool: &SqlitePool,
    session_id: &str,
    text: &str,
) -> Result<bool, String> {
    super::lifecycle::maybe_dispatch_registered_im_session_reply_with_pool(pool, session_id, text)
        .await
}
