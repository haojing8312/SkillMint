mod helpers;

use runtime_lib::commands::feishu_gateway::set_app_setting;
use runtime_lib::commands::wecom_gateway::{
    get_wecom_connector_status_with_pool, resolve_wecom_credentials,
    resolve_wecom_sidecar_base_url, send_wecom_text_message_with_pool,
    start_wecom_connector_with_pool, stop_wecom_connector_with_pool,
    test_support as wecom_test_support,
};

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
async fn wecom_text_send_uses_native_noop_adapter_without_sidecar() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    let host_runtime_state = wecom_test_support::new_wecom_host_runtime_state_for_tests();
    wecom_test_support::clear_wecom_test_hooks();

    let send_result = send_wecom_text_message_with_pool(
        &pool,
        "conversation-1".to_string(),
        "hello".to_string(),
        Some(&host_runtime_state),
        Some("http://127.0.0.1:1".to_string()),
    )
    .await
    .expect("send text without sidecar");
    let send_result: serde_json::Value =
        serde_json::from_str(&send_result).expect("parse native send result");
    assert_eq!(send_result["accepted"], true);
    assert_eq!(send_result["message_id"], "wecom:wecom-main");
    assert_eq!(send_result["msgid"], "wecom:wecom-main");
    assert_eq!(send_result["transport"], "native-wecom-noop");
    assert!(send_result["delivered_at"]
        .as_str()
        .expect("delivered_at timestamp")
        .contains('T'));

    let runtime_status = wecom_test_support::wecom_runtime_status_for_tests(&host_runtime_state)
        .expect("runtime status persisted")
        .expect("wecom runtime status");
    let logs = runtime_status["recent_logs"]
        .as_array()
        .expect("recent logs")
        .iter()
        .map(|entry| entry.as_str().unwrap_or_default())
        .collect::<Vec<_>>();
    assert!(logs
        .iter()
        .any(|entry| *entry == "[wecom] reply_lifecycle phase=processing_started"));
    assert!(logs.iter().any(|entry| *entry == "[wecom] send_result"));
    assert!(logs
        .iter()
        .any(|entry| *entry == "[wecom] reply_lifecycle phase=fully_complete"));
}
