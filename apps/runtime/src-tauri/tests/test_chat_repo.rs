mod helpers;

use runtime_chat_app::ChatSettingsRepository;
use runtime_lib::commands::chat_repo::PoolChatSettingsRepository;

#[tokio::test]
async fn chat_repo_loads_routing_settings_and_chat_routing() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    sqlx::query(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES ('route_max_call_depth', '7')",
    )
    .execute(&pool)
    .await
    .expect("set depth");
    sqlx::query(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES ('route_node_timeout_seconds', '90')",
    )
    .execute(&pool)
    .await
    .expect("set timeout");
    sqlx::query(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES ('route_retry_count', '2')",
    )
    .execute(&pool)
    .await
    .expect("set retry");
    sqlx::query(
        "INSERT INTO routing_policies (capability, primary_provider_id, primary_model, fallback_chain_json, timeout_ms, retry_count, enabled)
         VALUES ('chat', 'provider-chat', 'gpt-4.1', '[{\"provider_id\":\"provider-fallback\",\"model\":\"claude-3-5-sonnet\"}]', 45000, 1, 1)",
    )
    .execute(&pool)
    .await
    .expect("insert chat route");

    let repo = PoolChatSettingsRepository::new(&pool);
    let settings = repo
        .load_routing_settings()
        .await
        .expect("routing settings should load");
    let chat_routing = repo
        .load_chat_routing()
        .await
        .expect("chat routing should load")
        .expect("chat policy exists");

    assert_eq!(settings.max_call_depth, 7);
    assert_eq!(settings.node_timeout_seconds, 90);
    assert_eq!(settings.retry_count, 2);
    assert_eq!(chat_routing.primary_provider_id, "provider-chat");
    assert_eq!(chat_routing.primary_model, "gpt-4.1");
    assert!(chat_routing.fallback_chain_json.contains("provider-fallback"));
    assert_eq!(chat_routing.timeout_ms, 45000);
    assert_eq!(chat_routing.retry_count, 1);
    assert!(chat_routing.enabled);
}

#[tokio::test]
async fn chat_repo_loads_capability_route_and_model_defaults() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    sqlx::query(
        "INSERT INTO routing_policies (capability, primary_provider_id, primary_model, fallback_chain_json, timeout_ms, retry_count, enabled)
         VALUES ('vision', 'provider-vision', 'qwen-vl-max', '[]', 30000, 2, 1)",
    )
    .execute(&pool)
    .await
    .expect("insert vision route");
    sqlx::query(
        "INSERT INTO model_configs (id, name, api_format, base_url, model_name, is_default, api_key)
         VALUES ('model-default', 'Default', 'openai', 'https://api.example.com/v1', 'gpt-4.1', 1, 'sk-default'),
                ('model-fallback', 'Fallback', 'openai', 'https://api.example.com/v1', 'gpt-4o-mini', 0, 'sk-fallback')",
    )
    .execute(&pool)
    .await
    .expect("insert model configs");

    let repo = PoolChatSettingsRepository::new(&pool);
    let vision_route = repo
        .load_route_policy("vision")
        .await
        .expect("vision route should load")
        .expect("vision policy exists");
    let default_model_id = repo
        .resolve_default_model_id()
        .await
        .expect("default model id should load");
    let default_usable_model_id = repo
        .resolve_default_usable_model_id()
        .await
        .expect("default usable model id should load");

    assert_eq!(vision_route.primary_provider_id, "provider-vision");
    assert_eq!(vision_route.primary_model, "qwen-vl-max");
    assert_eq!(vision_route.retry_count, 2);
    assert!(vision_route.enabled);
    assert_eq!(default_model_id.as_deref(), Some("model-default"));
    assert_eq!(default_usable_model_id.as_deref(), Some("model-default"));
}

#[tokio::test]
async fn chat_repo_loads_provider_connection_and_session_model() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    sqlx::query(
        "INSERT INTO provider_configs (id, provider_key, display_name, protocol_type, base_url, auth_type, api_key_encrypted, org_id, extra_json, enabled, created_at, updated_at)
         VALUES ('provider-1', 'openai-main', 'OpenAI Main', 'openai', 'https://api.openai.com/v1', 'api_key', 'sk-provider', '', '{}', 1, '2026-03-11T00:00:00Z', '2026-03-11T00:00:00Z')",
    )
    .execute(&pool)
    .await
    .expect("insert provider");
    sqlx::query(
        "INSERT INTO model_configs (id, name, api_format, base_url, model_name, is_default, api_key)
         VALUES ('model-1', 'Vision Model', 'anthropic', 'https://api.anthropic.com', 'claude-3-5-sonnet', 0, 'sk-model')",
    )
    .execute(&pool)
    .await
    .expect("insert session model");

    let repo = PoolChatSettingsRepository::new(&pool);
    let provider = repo
        .get_provider_connection("provider-1")
        .await
        .expect("provider should load")
        .expect("provider exists");
    let session_model = repo
        .load_session_model("model-1")
        .await
        .expect("session model should load");

    assert_eq!(provider.provider_id, "provider-1");
    assert_eq!(provider.protocol_type, "openai");
    assert_eq!(provider.base_url, "https://api.openai.com/v1");
    assert_eq!(provider.api_key, "sk-provider");
    assert_eq!(session_model.model_id, "model-1");
    assert_eq!(session_model.api_format, "anthropic");
    assert_eq!(session_model.base_url, "https://api.anthropic.com");
    assert_eq!(session_model.model_name, "claude-3-5-sonnet");
    assert_eq!(session_model.api_key, "sk-model");
}
