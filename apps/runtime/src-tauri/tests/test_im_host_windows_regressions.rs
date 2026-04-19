mod helpers;

use runtime_lib::approval_bus::PendingApprovalRecord;
use runtime_lib::commands::im_host::test_support::{
    maybe_dispatch_registered_im_session_reply_with_pool,
    maybe_emit_registered_host_lifecycle_phase_for_session_with_pool,
    maybe_notify_registered_approval_requested_with_pool,
    maybe_notify_registered_ask_user_requested_with_pool,
};
use runtime_lib::commands::wecom_gateway::test_support::{
    clear_wecom_test_hooks, install_recording_wecom_interactive_lifecycle_hooks,
    install_recording_wecom_lifecycle_hook, install_recording_wecom_send_hook,
};
use serde_json::json;
use sqlx::SqlitePool;

async fn seed_session_channel(
    pool: &SqlitePool,
    session_id: &str,
    thread_id: &str,
    source: &str,
    message_id: &str,
) {
    sqlx::query(
        "INSERT INTO im_thread_sessions (thread_id, employee_id, session_id, route_session_key, created_at, updated_at)
         VALUES (?, '', ?, '', '2026-04-19T00:00:00Z', '2026-04-19T00:00:01Z')",
    )
    .bind(thread_id)
    .bind(session_id)
    .execute(pool)
    .await
    .expect("seed im_thread_sessions");

    sqlx::query(
        "INSERT INTO im_inbox_events (id, event_id, thread_id, message_id, text_preview, source, created_at)
         VALUES (?, ?, ?, ?, 'hello', ?, '2026-04-19T00:00:02Z')",
    )
    .bind(format!("evt-{thread_id}"))
    .bind(format!("evt-{thread_id}"))
    .bind(thread_id)
    .bind(message_id)
    .bind(source)
    .execute(pool)
    .await
    .expect("seed im_inbox_events");
}

#[tokio::test]
async fn wecom_unified_host_regressions_run_in_windows_safe_target() {
    let (pool, _tmp) = helpers::setup_test_db().await;

    seed_session_channel(
        &pool,
        "session-wecom-ask-user",
        "wecom_chat_ask_user",
        "wecom",
        "wm_parent_ask_user",
    )
    .await;
    let ask_user_sent = install_recording_wecom_send_hook();
    let ask_user_lifecycle = install_recording_wecom_interactive_lifecycle_hooks();

    let ask_user_result = maybe_notify_registered_ask_user_requested_with_pool(
        &pool,
        "session-wecom-ask-user",
        "请确认企微方案",
        &["方案一".to_string(), "方案二".to_string()],
        None,
    )
    .await
    .expect("notify wecom ask_user");

    assert!(ask_user_result);
    let ask_user_texts = ask_user_sent.lock().expect("lock ask_user sent");
    assert_eq!(ask_user_texts.len(), 1);
    assert!(ask_user_texts[0].contains("请确认企微方案"));
    assert!(ask_user_texts[0].contains("可选项：方案一 / 方案二"));
    assert_eq!(
        ask_user_lifecycle
            .lock()
            .expect("lock ask_user lifecycle")
            .as_slice(),
        [
            "processing_stop:wm_parent_ask_user:ask_user",
            "lifecycle:wm_parent_ask_user:\"ask_user_requested\"",
        ]
    );
    clear_wecom_test_hooks();

    seed_session_channel(
        &pool,
        "session-wecom-approval-request",
        "wecom_chat_approval_request",
        "wecom",
        "wm_parent_approval_request",
    )
    .await;
    let approval_sent = install_recording_wecom_send_hook();
    let approval_lifecycle = install_recording_wecom_interactive_lifecycle_hooks();
    let approval_record = PendingApprovalRecord {
        approval_id: "approval-wecom-1".to_string(),
        session_id: "session-wecom-approval-request".to_string(),
        run_id: None,
        call_id: "call-wecom-1".to_string(),
        tool_name: "shell".to_string(),
        input: json!({"command": "rm -rf /tmp/wecom-demo"}),
        summary: "执行企微高风险命令".to_string(),
        impact: Some("可能修改企微关联工作目录内容".to_string()),
        irreversible: true,
        status: "pending".to_string(),
    };

    let approval_result = maybe_notify_registered_approval_requested_with_pool(
        &pool,
        "session-wecom-approval-request",
        &approval_record,
        None,
    )
    .await
    .expect("notify wecom approval requested");

    assert!(approval_result);
    let approval_sent_text = approval_sent.lock().expect("lock approval sent");
    assert_eq!(approval_sent_text.len(), 1);
    assert!(approval_sent_text[0].contains("待审批 #approval-wecom-1"));
    assert!(approval_sent_text[0].contains("/approve approval-wecom-1 allow_once | allow_always | deny"));
    assert_eq!(
        approval_lifecycle
            .lock()
            .expect("lock approval lifecycle")
            .as_slice(),
        [
            "processing_stop:wm_parent_approval_request:waiting_approval",
            "lifecycle:wm_parent_approval_request:\"approval_requested\"",
        ]
    );
    clear_wecom_test_hooks();

    seed_session_channel(
        &pool,
        "session-wecom-lifecycle",
        "wecom_chat_lifecycle",
        "wecom",
        "wm_parent_lifecycle",
    )
    .await;
    let lifecycle_records = install_recording_wecom_lifecycle_hook();

    let answered = maybe_emit_registered_host_lifecycle_phase_for_session_with_pool(
        &pool,
        "session-wecom-lifecycle",
        None,
        "ask_user_answered",
        None,
    )
    .await
    .expect("emit wecom ask_user_answered");
    let resolved = maybe_emit_registered_host_lifecycle_phase_for_session_with_pool(
        &pool,
        "session-wecom-lifecycle",
        Some("reply-wecom-approval"),
        "approval_resolved",
        None,
    )
    .await
    .expect("emit wecom approval_resolved");
    let resumed = maybe_emit_registered_host_lifecycle_phase_for_session_with_pool(
        &pool,
        "session-wecom-lifecycle",
        Some("reply-wecom-resumed"),
        "resumed",
        None,
    )
    .await
    .expect("emit wecom resumed");

    assert!(answered);
    assert!(resolved);
    assert!(resumed);
    assert_eq!(
        lifecycle_records
            .lock()
            .expect("lock lifecycle records")
            .as_slice(),
        [
            "lifecycle:wm_parent_lifecycle:\"ask_user_answered\"",
            "lifecycle:wm_parent_lifecycle:\"approval_resolved\"",
            "lifecycle:wm_parent_lifecycle:\"resumed\"",
        ]
    );
    clear_wecom_test_hooks();

    seed_session_channel(
        &pool,
        "session-wecom-dispatch",
        "wecom_chat_dispatch",
        "wecom",
        "wm_parent_dispatch",
    )
    .await;
    let dispatch_sent = install_recording_wecom_send_hook();

    let dispatch_result = maybe_dispatch_registered_im_session_reply_with_pool(
        &pool,
        "session-wecom-dispatch",
        "企微 unified host 最终回复",
    )
    .await
    .expect("dispatch wecom reply");

    assert!(dispatch_result);
    assert_eq!(
        dispatch_sent.lock().expect("lock dispatch sent").as_slice(),
        ["企微 unified host 最终回复"]
    );
    clear_wecom_test_hooks();
}
