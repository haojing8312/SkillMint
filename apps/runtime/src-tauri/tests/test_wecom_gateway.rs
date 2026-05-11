mod helpers;

use runtime_lib::commands::feishu_gateway::set_app_setting;
use runtime_lib::commands::wecom_gateway::{
    get_wecom_connector_status_with_pool, resolve_wecom_credentials,
    resolve_wecom_sidecar_base_url, send_wecom_text_message_with_pool,
    start_wecom_connector_with_pool, stop_wecom_connector_with_pool,
    test_support as wecom_test_support,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

async fn spawn_mock_wecom_sidecar(
    expected_requests: usize,
) -> (
    String,
    tokio::task::JoinHandle<Vec<(String, serde_json::Value)>>,
) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mock sidecar");
    let addr = listener.local_addr().expect("local addr");
    let handle = tokio::spawn(async move {
        let mut requests = Vec::new();
        for _ in 0..expected_requests {
            let (mut socket, _) = listener.accept().await.expect("accept");
            let mut buf = vec![0u8; 64 * 1024];
            let n = socket.read(&mut buf).await.expect("read request");
            let raw = String::from_utf8_lossy(&buf[..n]).to_string();
            let request_line = raw.lines().next().unwrap_or_default().to_string();
            let path = request_line
                .split_whitespace()
                .nth(1)
                .unwrap_or("/")
                .to_string();
            let body = raw.split("\r\n\r\n").nth(1).unwrap_or("{}");
            let body_json: serde_json::Value =
                serde_json::from_str(body).unwrap_or_else(|_| serde_json::json!({}));
            requests.push((path.clone(), body_json.clone()));

            let output = match path.as_str() {
                "/api/channels/send-message" => serde_json::json!({
                    "message_id": "wecom-msg-1",
                    "delivered_at": "2026-03-10T10:00:01Z"
                }),
                _ => serde_json::json!({}),
            };

            let response_body = serde_json::json!({
                "output": output.to_string()
            })
            .to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            socket
                .write_all(response.as_bytes())
                .await
                .expect("write response");
        }
        requests
    });
    (format!("http://{}", addr), handle)
}

#[tokio::test]
async fn resolve_wecom_sidecar_base_url_falls_back_to_generic_im_sidecar_key() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    set_app_setting(&pool, "im_sidecar_base_url", "http://127.0.0.1:9200")
        .await
        .expect("set generic sidecar url");

    let base_url = resolve_wecom_sidecar_base_url(&pool, None)
        .await
        .expect("resolve generic sidecar");
    assert_eq!(base_url.as_deref(), Some("http://127.0.0.1:9200"));
}

#[tokio::test]
async fn resolve_wecom_credentials_reads_from_app_settings() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    set_app_setting(&pool, "wecom_corp_id", "wwcorp")
        .await
        .expect("set corp id");
    set_app_setting(&pool, "wecom_agent_id", "1000002")
        .await
        .expect("set agent id");
    set_app_setting(&pool, "wecom_agent_secret", "secret-x")
        .await
        .expect("set agent secret");

    let creds = resolve_wecom_credentials(&pool, None, None, None)
        .await
        .expect("resolve wecom creds");
    assert_eq!(
        creds,
        (
            "wwcorp".to_string(),
            "1000002".to_string(),
            "secret-x".to_string()
        )
    );
}

#[tokio::test]
async fn wecom_connector_start_status_stop_uses_native_runtime_state_without_sidecar() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    let host_runtime_state = wecom_test_support::new_wecom_host_runtime_state_for_tests();
    set_app_setting(&pool, "wecom_corp_id", "wwcorp")
        .await
        .expect("set corp id");
    set_app_setting(&pool, "wecom_agent_id", "1000002")
        .await
        .expect("set agent id");
    set_app_setting(&pool, "wecom_agent_secret", "secret-x")
        .await
        .expect("set agent secret");

    let instance_id = start_wecom_connector_with_pool(
        &pool,
        Some("http://127.0.0.1:1".to_string()),
        None,
        None,
        None,
        Some(&host_runtime_state),
    )
    .await
    .expect("start connector without sidecar");
    assert_eq!(instance_id, "wecom:wecom-main");

    let started_status = get_wecom_connector_status_with_pool(
        &pool,
        Some("http://127.0.0.1:1".to_string()),
        Some(&host_runtime_state),
    )
    .await
    .expect("get native runtime status");
    assert!(started_status.running);
    assert_eq!(started_status.state, "running");
    assert_eq!(started_status.instance_id, "wecom:wecom-main");

    let runtime_status = wecom_test_support::wecom_runtime_status_for_tests(&host_runtime_state)
        .expect("runtime status persisted")
        .expect("wecom runtime status");
    assert_eq!(runtime_status["running"], true);
    assert_eq!(runtime_status["instance_id"], "wecom:wecom-main");

    stop_wecom_connector_with_pool(
        &pool,
        Some("http://127.0.0.1:1".to_string()),
        Some(&host_runtime_state),
    )
    .await
    .expect("stop connector without sidecar");
    let stopped_status =
        get_wecom_connector_status_with_pool(&pool, None, Some(&host_runtime_state))
            .await
            .expect("get stopped native status");
    assert!(!stopped_status.running);
    assert_eq!(stopped_status.state, "stopped");
    assert_eq!(stopped_status.instance_id, "wecom:wecom-main");
}

#[tokio::test]
async fn wecom_text_send_still_uses_channel_neutral_sidecar_endpoint() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    let (base_url, server_task) = spawn_mock_wecom_sidecar(1).await;
    set_app_setting(&pool, "im_sidecar_base_url", &base_url)
        .await
        .expect("set generic sidecar");

    let send_result = send_wecom_text_message_with_pool(
        &pool,
        "conversation-1".to_string(),
        "hello".to_string(),
        None,
        None,
    )
    .await
    .expect("send text");
    assert!(send_result.contains("wecom-msg-1"));

    let requests = server_task.await.expect("mock sidecar task");
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].0, "/api/channels/send-message");
    assert_eq!(requests[0].1["instance_id"], "wecom:wecom-main");
    assert_eq!(requests[0].1["request"]["thread_id"], "conversation-1");
    assert_eq!(requests[0].1["request"]["reply_target"], "conversation-1");
    assert_eq!(requests[0].1["request"]["text"], "hello");
}
