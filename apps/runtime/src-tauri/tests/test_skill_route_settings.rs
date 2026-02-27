mod helpers;

use runtime_lib::commands::models::load_routing_settings_from_pool;

#[tokio::test]
async fn route_settings_use_defaults_when_empty() {
    let (pool, _tmp) = helpers::setup_test_db().await;

    let settings = load_routing_settings_from_pool(&pool).await.expect("load settings");
    assert_eq!(settings.max_call_depth, 4);
    assert_eq!(settings.node_timeout_seconds, 60);
    assert_eq!(settings.retry_count, 0);
}

#[tokio::test]
async fn route_settings_parse_from_app_settings_table() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    sqlx::query("INSERT OR REPLACE INTO app_settings (key, value) VALUES ('route_max_call_depth', '7')")
        .execute(&pool)
        .await
        .expect("set depth");
    sqlx::query("INSERT OR REPLACE INTO app_settings (key, value) VALUES ('route_node_timeout_seconds', '120')")
        .execute(&pool)
        .await
        .expect("set timeout");
    sqlx::query("INSERT OR REPLACE INTO app_settings (key, value) VALUES ('route_retry_count', '2')")
        .execute(&pool)
        .await
        .expect("set retry");

    let settings = load_routing_settings_from_pool(&pool).await.expect("load settings");
    assert_eq!(settings.max_call_depth, 7);
    assert_eq!(settings.node_timeout_seconds, 120);
    assert_eq!(settings.retry_count, 2);
}
