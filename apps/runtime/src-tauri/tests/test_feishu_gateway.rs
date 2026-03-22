mod helpers;

use runtime_lib::approval_bus::{ApprovalDecision, ApprovalManager, CreateApprovalRequest};
use runtime_lib::commands::feishu_gateway::{
    calculate_feishu_signature, clear_feishu_runtime_state_for_outbound,
    maybe_handle_feishu_approval_command_with_pool, notify_feishu_approval_requested_with_pool,
    parse_feishu_payload, plan_role_dispatch_requests_for_feishu, plan_role_events_for_feishu,
    remember_feishu_runtime_state_for_outbound, resolve_feishu_app_credentials,
    resolve_feishu_sidecar_base_url, send_feishu_text_message_with_pool,
    set_app_setting, set_feishu_official_runtime_outbound_send_hook_for_tests,
    validate_feishu_auth_with_pool, validate_feishu_signature_with_pool, ParsedFeishuPayload,
};
use runtime_lib::commands::openclaw_plugins::{
    OpenClawPluginFeishuOutboundDeliveryResult, OpenClawPluginFeishuOutboundSendRequest,
    OpenClawPluginFeishuOutboundSendResult, OpenClawPluginFeishuRuntimeState,
};
use runtime_lib::commands::im_config::bind_thread_roles_with_pool;
use runtime_lib::im::types::{ImEvent, ImEventType};
use std::sync::{Arc, Mutex};
use std::sync::OnceLock;
use tokio::sync::Mutex as TokioMutex;

fn feishu_runtime_test_lock() -> &'static TokioMutex<()> {
    static LOCK: OnceLock<TokioMutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| TokioMutex::new(()))
}

fn install_feishu_official_runtime_outbound_send_hook(
    captured_request: Arc<Mutex<Option<OpenClawPluginFeishuOutboundSendRequest>>>,
) {
    set_feishu_official_runtime_outbound_send_hook_for_tests(Some(Arc::new(
        move |request: &OpenClawPluginFeishuOutboundSendRequest| {
            *captured_request
                .lock()
                .expect("capture official runtime outbound request") = Some(request.clone());
            Ok(OpenClawPluginFeishuOutboundDeliveryResult {
                delivered: true,
                channel: "feishu".to_string(),
                account_id: request.account_id.clone(),
                target: request.target.clone(),
                thread_id: request.thread_id.clone(),
                text: request.text.clone(),
                mode: request.mode.clone(),
                message_id: format!("plugin_message_{}", request.request_id),
                chat_id: format!("plugin:{}", request.target),
                sequence: 1,
            })
        },
    )));
}

#[test]
fn parse_feishu_payload_supports_challenge() {
    let raw = r#"{"challenge":"abc123"}"#;
    let parsed = parse_feishu_payload(raw).expect("challenge parse");
    match parsed {
        ParsedFeishuPayload::Challenge(v) => assert_eq!(v, "abc123"),
        _ => panic!("expected challenge"),
    }
}

#[test]
fn parse_feishu_payload_maps_message_event() {
    let raw = r#"{
      "header": {
        "event_id": "evt-feishu-1",
        "event_type": "im.message.receive_v1",
        "tenant_key": "tenant-x"
      },
      "event": {
        "message": {
          "message_id": "msg-1",
          "chat_id": "chat-1",
          "content": "{\"text\":\"你好，帮我评审商机\"}"
        },
        "sender": {
          "sender_id": { "open_id": "ou_xxx" }
        }
      }
    }"#;
    let parsed = parse_feishu_payload(raw).expect("event parse");
    let evt = match parsed {
        ParsedFeishuPayload::Event(e) => e,
        _ => panic!("expected event"),
    };
    assert_eq!(evt.event_type, ImEventType::MessageCreated);
    assert_eq!(evt.thread_id, "chat-1");
    assert_eq!(evt.event_id.as_deref(), Some("evt-feishu-1"));
    assert_eq!(evt.text.as_deref(), Some("你好，帮我评审商机"));
}

#[tokio::test]
async fn validate_feishu_auth_honors_configured_token() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    sqlx::query(
        "INSERT INTO app_settings (key, value) VALUES ('feishu_ingress_token', 'feishu-secret')",
    )
    .execute(&pool)
    .await
    .expect("seed token");

    assert!(
        validate_feishu_auth_with_pool(&pool, Some("feishu-secret".to_string()))
            .await
            .is_ok()
    );
    assert!(
        validate_feishu_auth_with_pool(&pool, Some("wrong".to_string()))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn validate_feishu_signature_honors_encrypt_key() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    sqlx::query("INSERT INTO app_settings (key, value) VALUES ('feishu_encrypt_key', 'enc-key-1')")
        .execute(&pool)
        .await
        .expect("seed encrypt key");

    let payload = r#"{"header":{"event_id":"evt-1","event_type":"im.message.receive_v1"},"event":{"message":{"message_id":"m1","chat_id":"c1","content":"{\"text\":\"x\"}"}}}"#;
    let timestamp = "1700000000";
    let nonce = "abc123";
    let signature = calculate_feishu_signature(timestamp, nonce, "enc-key-1", payload);

    let ok = validate_feishu_signature_with_pool(
        &pool,
        payload,
        Some(timestamp.to_string()),
        Some(nonce.to_string()),
        Some(signature),
    )
    .await;
    assert!(ok.is_ok());

    let bad = validate_feishu_signature_with_pool(
        &pool,
        payload,
        Some(timestamp.to_string()),
        Some(nonce.to_string()),
        Some("wrong".to_string()),
    )
    .await;
    assert!(bad.is_err());
}

#[tokio::test]
async fn plan_role_events_for_feishu_uses_thread_bindings() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    bind_thread_roles_with_pool(
        &pool,
        "chat-1",
        "tenant-x",
        "opportunity_review",
        &["presales".to_string(), "architect".to_string()],
    )
    .await
    .expect("bind roles");

    let parsed = parse_feishu_payload(
        r#"{
          "header":{"event_id":"evt-2","event_type":"im.message.receive_v1"},
          "event":{"message":{"message_id":"msg-2","chat_id":"chat-1","content":"{\"text\":\"开始\"}"}}
        }"#,
    )
    .expect("parse");

    let evt = match parsed {
        ParsedFeishuPayload::Event(e) => e,
        _ => panic!("expected event"),
    };
    let planned = plan_role_events_for_feishu(&pool, &evt)
        .await
        .expect("plan role events");
    assert_eq!(planned.len(), 2);
    assert_eq!(planned[0].thread_id, "chat-1");
    assert_eq!(planned[0].status, "running");
    assert_eq!(planned[0].message_type, "system");
    assert_eq!(planned[0].sender_role, "main_agent");
    assert_eq!(planned[0].source_channel, "feishu");

    let dispatches = plan_role_dispatch_requests_for_feishu(&pool, &evt)
        .await
        .expect("plan dispatch");
    assert_eq!(dispatches.len(), 2);
    assert_eq!(dispatches[0].thread_id, "chat-1");
    assert_eq!(dispatches[0].agent_type, "plan");
    assert!(dispatches[0].prompt.contains("场景=opportunity_review"));
    assert_eq!(dispatches[0].message_type, "user_input");
    assert_eq!(dispatches[0].sender_role, "main_agent");
    assert_eq!(dispatches[0].sender_employee_id, dispatches[0].role_id);
    assert_eq!(dispatches[0].target_employee_id, dispatches[0].role_id);
    assert_eq!(dispatches[0].source_channel, "feishu");
}

#[tokio::test]
async fn plan_role_dispatch_falls_back_to_thread_roles_when_mention_role_unknown() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    bind_thread_roles_with_pool(
        &pool,
        "chat-unknown-role",
        "tenant-x",
        "opportunity_review",
        &["presales".to_string(), "architect".to_string()],
    )
    .await
    .expect("bind roles");

    let parsed = parse_feishu_payload(
        r#"{
          "header":{"event_id":"evt-unknown","event_type":"im.message.receive_v1"},
          "event":{"message":{"message_id":"msg-unknown","chat_id":"chat-unknown-role","content":"{\"text\":\"@某人 请先分析\"}"}}
        }"#,
    )
    .expect("parse");

    let mut evt = match parsed {
        ParsedFeishuPayload::Event(e) => e,
        _ => panic!("expected event"),
    };
    evt.role_id = Some("ou_unknown_mention".to_string());

    let planned = plan_role_events_for_feishu(&pool, &evt)
        .await
        .expect("plan role events");
    assert_eq!(planned.len(), 2);

    let dispatches = plan_role_dispatch_requests_for_feishu(&pool, &evt)
        .await
        .expect("plan dispatch");
    assert_eq!(dispatches.len(), 2);
}

#[tokio::test]
async fn resolve_feishu_settings_reads_from_app_settings() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    set_app_setting(&pool, "feishu_app_id", "cli_app")
        .await
        .expect("set app id");
    set_app_setting(&pool, "feishu_app_secret", "cli_secret")
        .await
        .expect("set app secret");
    set_app_setting(&pool, "feishu_sidecar_base_url", "http://127.0.0.1:9000")
        .await
        .expect("set sidecar url");

    let (app_id, app_secret) = resolve_feishu_app_credentials(&pool, None, None)
        .await
        .expect("resolve creds");
    assert_eq!(app_id.as_deref(), Some("cli_app"));
    assert_eq!(app_secret.as_deref(), Some("cli_secret"));

    let base_url = resolve_feishu_sidecar_base_url(&pool, None)
        .await
        .expect("resolve sidecar");
    assert_eq!(base_url.as_deref(), Some("http://127.0.0.1:9000"));
}

#[tokio::test]
async fn resolve_feishu_sidecar_base_url_falls_back_to_generic_im_sidecar_key() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    set_app_setting(&pool, "im_sidecar_base_url", "http://127.0.0.1:9100")
        .await
        .expect("set generic sidecar url");

    let base_url = resolve_feishu_sidecar_base_url(&pool, None)
        .await
        .expect("resolve generic sidecar");
    assert_eq!(base_url.as_deref(), Some("http://127.0.0.1:9100"));
}

#[tokio::test]
async fn feishu_outbound_send_uses_official_runtime_helper() {
    let _guard = feishu_runtime_test_lock().lock().await;
    let (pool, _tmp) = helpers::setup_test_db().await;
    clear_feishu_runtime_state_for_outbound();
    set_feishu_official_runtime_outbound_send_hook_for_tests(None);
    set_app_setting(&pool, "feishu_app_id", "demo-app")
        .await
        .expect("seed feishu app id");
    set_app_setting(&pool, "feishu_app_secret", "demo-secret")
        .await
        .expect("seed feishu app secret");
    let runtime_state = OpenClawPluginFeishuRuntimeState::default();
    remember_feishu_runtime_state_for_outbound(&runtime_state);

    let captured_request: Arc<Mutex<Option<OpenClawPluginFeishuOutboundSendRequest>>> =
        Arc::new(Mutex::new(None));
    install_feishu_official_runtime_outbound_send_hook(captured_request.clone());

    let result_json = send_feishu_text_message_with_pool(
        &pool,
        "chat-outbound-1",
        "你好，来自官方 runtime",
        Some("http://127.0.0.1:9000".to_string()),
    )
    .await
    .expect("send via official runtime");

    let result: OpenClawPluginFeishuOutboundSendResult =
        serde_json::from_str(&result_json).expect("parse outbound send result");
    assert_eq!(result.request.account_id, "default");
    assert_eq!(result.request.target, "chat-outbound-1");
    assert_eq!(result.request.thread_id.as_deref(), Some("chat-outbound-1"));
    assert_eq!(result.request.text, "你好，来自官方 runtime");
    assert_eq!(result.request.mode, "text");
    assert!(result.result.delivered);
    assert_eq!(result.result.channel, "feishu");
    assert_eq!(result.result.account_id, "default");
    assert_eq!(result.result.target, "chat-outbound-1");
    assert_eq!(result.result.chat_id, "plugin:chat-outbound-1");
    assert_eq!(result.result.text, "你好，来自官方 runtime");
    let captured = captured_request
        .lock()
        .expect("capture official runtime outbound request")
        .clone()
        .expect("captured outbound request");
    assert_eq!(captured.account_id, "default");
    assert_eq!(captured.target, "chat-outbound-1");
    assert_eq!(captured.thread_id.as_deref(), Some("chat-outbound-1"));
    assert_eq!(captured.text, "你好，来自官方 runtime");
    assert_eq!(captured.mode, "text");

    set_feishu_official_runtime_outbound_send_hook_for_tests(None);
    clear_feishu_runtime_state_for_outbound();
}

#[tokio::test]
async fn feishu_outbound_send_requires_registered_runtime() {
    let _guard = feishu_runtime_test_lock().lock().await;
    let (pool, _tmp) = helpers::setup_test_db().await;
    clear_feishu_runtime_state_for_outbound();
    set_feishu_official_runtime_outbound_send_hook_for_tests(None);

    let error = send_feishu_text_message_with_pool(&pool, "chat-outbound-2", "你好", None)
        .await
        .expect_err("runtime should be required");
    assert!(error.contains("official feishu runtime is not registered"));
}

#[tokio::test]
async fn feishu_pending_approval_notification_targets_bound_thread() {
    let _guard = feishu_runtime_test_lock().lock().await;
    let (pool, _tmp) = helpers::setup_test_db().await;
    clear_feishu_runtime_state_for_outbound();
    set_feishu_official_runtime_outbound_send_hook_for_tests(None);
    set_app_setting(&pool, "feishu_app_id", "demo-app")
        .await
        .expect("seed feishu app id");
    set_app_setting(&pool, "feishu_app_secret", "demo-secret")
        .await
        .expect("seed feishu app secret");
    let runtime_state = OpenClawPluginFeishuRuntimeState::default();
    remember_feishu_runtime_state_for_outbound(&runtime_state);

    let captured_request: Arc<Mutex<Option<OpenClawPluginFeishuOutboundSendRequest>>> =
        Arc::new(Mutex::new(None));
    install_feishu_official_runtime_outbound_send_hook(captured_request.clone());

    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO im_thread_sessions (thread_id, employee_id, session_id, route_session_key, created_at, updated_at)
         VALUES (?, ?, ?, '', ?, ?)",
    )
    .bind("chat-approval-1")
    .bind("employee-1")
    .bind("session-feishu-approval")
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .expect("seed thread session");
    sqlx::query(
        "INSERT INTO im_inbox_events (id, event_id, thread_id, message_id, text_preview, source, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("inbox-1")
    .bind("evt-approval-1")
    .bind("chat-approval-1")
    .bind("msg-approval-1")
    .bind("hello")
    .bind("feishu")
    .bind(&now)
    .execute(&pool)
    .await
    .expect("seed feishu inbox event");

    notify_feishu_approval_requested_with_pool(
        &pool,
        "session-feishu-approval",
        &runtime_lib::approval_bus::PendingApprovalRecord {
            approval_id: "approval-feishu-1".to_string(),
            session_id: "session-feishu-approval".to_string(),
            run_id: Some("run-feishu-1".to_string()),
            call_id: "call-feishu-1".to_string(),
            tool_name: "file_delete".to_string(),
            input: serde_json::json!({
                "path": "C:\\\\Users\\\\demo\\\\danger",
                "recursive": true
            }),
            summary: "删除目录 C:\\Users\\demo\\danger".to_string(),
            impact: Some("目录及其全部子文件会被永久删除".to_string()),
            irreversible: true,
            status: "pending".to_string(),
        },
        None,
    )
    .await
    .expect("notify approval request");

    let captured = captured_request
        .lock()
        .expect("capture official runtime outbound request")
        .clone()
        .expect("captured outbound request");
    assert_eq!(captured.account_id, "default");
    assert_eq!(captured.target, "chat-approval-1");
    assert_eq!(captured.thread_id.as_deref(), Some("chat-approval-1"));
    assert!(captured.text.contains("待审批 #approval-feishu-1"));

    set_feishu_official_runtime_outbound_send_hook_for_tests(None);
    clear_feishu_runtime_state_for_outbound();
}

#[tokio::test]
async fn feishu_approve_command_resolves_pending_approval() {
    let _guard = feishu_runtime_test_lock().lock().await;
    let (pool, _tmp) = helpers::setup_test_db().await;
    clear_feishu_runtime_state_for_outbound();
    set_feishu_official_runtime_outbound_send_hook_for_tests(None);
    set_app_setting(&pool, "feishu_app_id", "demo-app")
        .await
        .expect("seed feishu app id");
    set_app_setting(&pool, "feishu_app_secret", "demo-secret")
        .await
        .expect("seed feishu app secret");
    let runtime_state = OpenClawPluginFeishuRuntimeState::default();
    remember_feishu_runtime_state_for_outbound(&runtime_state);

    let captured_request: Arc<Mutex<Option<OpenClawPluginFeishuOutboundSendRequest>>> =
        Arc::new(Mutex::new(None));
    install_feishu_official_runtime_outbound_send_hook(captured_request.clone());

    let approvals = ApprovalManager::default();
    approvals
        .create_pending_with_pool(
            &pool,
            None,
            CreateApprovalRequest {
                approval_id: "approval-feishu-cmd-1".to_string(),
                session_id: "session-feishu-cmd-1".to_string(),
                run_id: Some("run-feishu-cmd-1".to_string()),
                call_id: "call-feishu-cmd-1".to_string(),
                tool_name: "file_delete".to_string(),
                input: serde_json::json!({
                    "path": "C:\\\\Users\\\\demo\\\\danger",
                    "recursive": true
                }),
                summary: "删除目录 C:\\Users\\demo\\danger".to_string(),
                impact: Some("目录及其全部子文件会被永久删除".to_string()),
                irreversible: true,
                work_dir: None,
            },
        )
        .await
        .expect("create pending approval");

    let result = maybe_handle_feishu_approval_command_with_pool(
        &pool,
        &approvals,
        &ImEvent {
            channel: "feishu".to_string(),
            event_type: ImEventType::MessageCreated,
            thread_id: "chat-approval-2".to_string(),
            event_id: Some("evt-approval-cmd-1".to_string()),
            message_id: Some("msg-approval-cmd-1".to_string()),
            text: Some("/approve approval-feishu-cmd-1 allow_once".to_string()),
            role_id: None,
            account_id: Some("ou_approver_1".to_string()),
            tenant_id: Some("ou_approver_1".to_string()),
            sender_id: Some("ou_approver_1".to_string()),
            chat_type: Some("direct".to_string()),
        },
        None,
    )
    .await
    .expect("handle approval command");

    let applied = result.expect("command should be handled");
    match applied {
        runtime_lib::approval_bus::ApprovalResolveResult::Applied {
            approval_id,
            status,
            decision,
        } => {
            assert_eq!(approval_id, "approval-feishu-cmd-1");
            assert_eq!(status, "approved");
            assert_eq!(decision, ApprovalDecision::AllowOnce);
        }
        other => panic!("expected applied resolution, got {:?}", other),
    }

    let row: (String, String, String, String) = sqlx::query_as(
        "SELECT status, decision, resolved_by_surface, resolved_by_user
         FROM approvals WHERE id = ?",
    )
    .bind("approval-feishu-cmd-1")
    .fetch_one(&pool)
    .await
    .expect("load approval row");
    assert_eq!(row.0, "approved");
    assert_eq!(row.1, "allow_once");
    assert_eq!(row.2, "feishu");
    assert_eq!(row.3, "ou_approver_1");

    let captured = captured_request
        .lock()
        .expect("capture official runtime outbound request")
        .clone()
        .expect("captured outbound request");
    assert_eq!(captured.account_id, "default");
    assert_eq!(captured.thread_id.as_deref(), Some("chat-approval-2"));
    assert!(captured.text.contains("审批 approval-feishu-cmd-1 已处理"));

    set_feishu_official_runtime_outbound_send_hook_for_tests(None);
    clear_feishu_runtime_state_for_outbound();
}
