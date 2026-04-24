use super::{
    send_wecom_text_message_with_pool,
};
use crate::approval_bus::PendingApprovalRecord;
use crate::commands::im_host::{
    build_im_approval_request_text, build_im_ask_user_request_text,
    prepare_channel_interactive_approval_notice_with_pool,
    prepare_channel_interactive_session_thread_with_pool,
    lookup_session_delivery_route_with_pool,
};
use crate::commands::openclaw_plugins::im_host_contract::ImReplyLifecyclePhase;
use sqlx::SqlitePool;

pub(crate) async fn notify_wecom_ask_user_requested_with_pool(
    pool: &SqlitePool,
    session_id: &str,
    question: &str,
    options: &[String],
    sidecar_base_url: Option<String>,
) -> Result<(), String> {
    let Some(thread_id) = prepare_channel_interactive_session_thread_with_pool(
        pool,
        "wecom",
        session_id,
        Some("ask_user"),
        ImReplyLifecyclePhase::AskUserRequested,
    )
    .await?
    else {
        return Ok(());
    };
    send_wecom_text_message_with_pool(pool, thread_id, build_im_ask_user_request_text(question, options), None, sidecar_base_url)
        .await?;
    Ok(())
}

pub(crate) async fn notify_wecom_approval_requested_with_pool(
    pool: &SqlitePool,
    session_id: &str,
    record: &PendingApprovalRecord,
    sidecar_base_url: Option<String>,
) -> Result<(), String> {
    let Some(thread_id) = prepare_channel_interactive_session_thread_with_pool(
        pool,
        "wecom",
        session_id,
        Some("waiting_approval"),
        ImReplyLifecyclePhase::ApprovalRequested,
    )
    .await?
    else {
        return Ok(());
    };
    send_wecom_text_message_with_pool(
        pool,
        thread_id,
        build_im_approval_request_text(record),
        None,
        sidecar_base_url,
    )
    .await?;
    Ok(())
}

pub(crate) async fn notify_wecom_approval_resolved_with_pool(
    pool: &SqlitePool,
    approval_id: &str,
    sidecar_base_url: Option<String>,
) -> Result<(), String> {
    let Some(row) = prepare_channel_interactive_approval_notice_with_pool(pool, approval_id).await?
    else {
        return Ok(());
    };
    let Some(thread_id) =
        lookup_session_delivery_route_with_pool(pool, &row.session_id, Some("wecom"))
            .await?
            .map(|route| route.thread_id)
    else {
        return Ok(());
    };
    let text = crate::commands::im_host::build_im_approval_resolved_notice_text(&row);
    send_wecom_text_message_with_pool(pool, thread_id, text, None, sidecar_base_url).await?;
    Ok(())
}
