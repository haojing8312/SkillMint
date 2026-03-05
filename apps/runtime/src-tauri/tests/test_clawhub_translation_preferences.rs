mod helpers;

use chrono::Utc;
use runtime_lib::commands::clawhub::translate_texts_with_preferences_with_pool;
use sha2::{Digest, Sha256};

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

async fn seed_default_model_for_cache(pool: &sqlx::SqlitePool) {
    sqlx::query(
        "INSERT INTO model_configs (id, name, api_format, base_url, model_name, is_default, api_key) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("model-default")
    .bind("Default Model")
    .bind("openai")
    .bind("https://example.invalid/v1")
    .bind("gpt-test")
    .bind(1)
    .bind("dummy-key")
    .execute(pool)
    .await
    .expect("insert default model");
}

#[tokio::test]
async fn translate_respects_disabled_flag_and_returns_source() {
    let (pool, _tmp) = helpers::setup_test_db().await;

    sqlx::query("INSERT OR REPLACE INTO app_settings (key, value) VALUES (?, ?)")
        .bind("runtime_immersive_translation_enabled")
        .bind("false")
        .execute(&pool)
        .await
        .expect("disable immersive translation");

    let out = translate_texts_with_preferences_with_pool(
        &pool,
        vec!["Video Maker".to_string(), "  ".to_string()],
    )
    .await
    .expect("translate texts with disabled flag");

    assert_eq!(out, vec!["Video Maker".to_string(), "".to_string()]);
}

#[tokio::test]
async fn translate_cache_key_includes_target_language() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    seed_default_model_for_cache(&pool).await;

    sqlx::query("INSERT OR REPLACE INTO app_settings (key, value) VALUES (?, ?)")
        .bind("runtime_immersive_translation_enabled")
        .bind("true")
        .execute(&pool)
        .await
        .expect("enable immersive translation");

    let source = "Video Maker";
    let hash = sha256_hex(source);
    let engine = "model:openai:gpt-test";
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO skill_i18n_cache (cache_key, source_text, translated_text, updated_at) VALUES (?, ?, ?, ?)",
    )
    .bind(format!("zh-CN:{engine}:{hash}"))
    .bind(source)
    .bind("视频制作器")
    .bind(&now)
    .execute(&pool)
    .await
    .expect("insert zh cache");

    sqlx::query(
        "INSERT INTO skill_i18n_cache (cache_key, source_text, translated_text, updated_at) VALUES (?, ?, ?, ?)",
    )
    .bind(format!("en-US:{engine}:{hash}"))
    .bind(source)
    .bind("Video Maker EN")
    .bind(&now)
    .execute(&pool)
    .await
    .expect("insert en cache");

    sqlx::query("INSERT OR REPLACE INTO app_settings (key, value) VALUES (?, ?)")
        .bind("runtime_default_language")
        .bind("zh-CN")
        .execute(&pool)
        .await
        .expect("set zh language");
    let zh = translate_texts_with_preferences_with_pool(&pool, vec![source.to_string()])
        .await
        .expect("translate zh");
    assert_eq!(zh, vec!["视频制作器".to_string()]);

    sqlx::query("INSERT OR REPLACE INTO app_settings (key, value) VALUES (?, ?)")
        .bind("runtime_default_language")
        .bind("en-US")
        .execute(&pool)
        .await
        .expect("set en language");
    let en = translate_texts_with_preferences_with_pool(&pool, vec![source.to_string()])
        .await
        .expect("translate en");
    assert_eq!(en, vec!["Video Maker EN".to_string()]);
}

#[tokio::test]
async fn translate_returns_results_in_input_order() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    seed_default_model_for_cache(&pool).await;

    sqlx::query("INSERT OR REPLACE INTO app_settings (key, value) VALUES (?, ?)")
        .bind("runtime_immersive_translation_enabled")
        .bind("true")
        .execute(&pool)
        .await
        .expect("enable immersive translation");
    sqlx::query("INSERT OR REPLACE INTO app_settings (key, value) VALUES (?, ?)")
        .bind("runtime_default_language")
        .bind("zh-CN")
        .execute(&pool)
        .await
        .expect("set zh language");

    let now = Utc::now().to_rfc3339();
    let engine = "model:openai:gpt-test";
    for (src, translated) in [("Alpha", "阿尔法"), ("Beta", "贝塔")] {
        sqlx::query(
            "INSERT INTO skill_i18n_cache (cache_key, source_text, translated_text, updated_at) VALUES (?, ?, ?, ?)",
        )
        .bind(format!("zh-CN:{engine}:{}", sha256_hex(src)))
        .bind(src)
        .bind(translated)
        .bind(&now)
        .execute(&pool)
        .await
        .expect("seed cache row");
    }

    let out = translate_texts_with_preferences_with_pool(
        &pool,
        vec!["Alpha".to_string(), "Beta".to_string(), "Alpha".to_string()],
    )
    .await
    .expect("translate with cache");

    assert_eq!(
        out,
        vec![
            "阿尔法".to_string(),
            "贝塔".to_string(),
            "阿尔法".to_string()
        ]
    );
}
