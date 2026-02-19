use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;
use super::skills::DbState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub id: String,
    pub name: String,
    pub api_format: String,
    pub base_url: String,
    pub model_name: String,
    pub is_default: bool,
}

#[tauri::command]
pub async fn save_model_config(
    config: ModelConfig,
    api_key: String,
    db: State<'_, DbState>,
) -> Result<(), String> {
    let id = if config.id.is_empty() {
        Uuid::new_v4().to_string()
    } else {
        config.id.clone()
    };

    sqlx::query(
        "INSERT OR REPLACE INTO model_configs (id, name, api_format, base_url, model_name, is_default, api_key) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&config.name)
    .bind(&config.api_format)
    .bind(&config.base_url)
    .bind(&config.model_name)
    .bind(config.is_default)
    .bind(&api_key)
    .execute(&db.0)
    .await
    .map_err(|e| format!("保存模型配置失败: {e}"))?;

    eprintln!("[models] 模型已保存: id={id}, name={}, api_key={}...{}",
        config.name,
        &api_key[..6.min(api_key.len())],
        &api_key[api_key.len().saturating_sub(4)..]);

    Ok(())
}

#[tauri::command]
pub async fn list_model_configs(db: State<'_, DbState>) -> Result<Vec<ModelConfig>, String> {
    let rows = sqlx::query_as::<_, (String, String, String, String, String, bool)>(
        "SELECT id, name, api_format, base_url, model_name, CAST(is_default AS BOOLEAN) FROM model_configs"
    )
    .fetch_all(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|(id, name, api_format, base_url, model_name, is_default)| {
        ModelConfig { id, name, api_format, base_url, model_name, is_default }
    }).collect())
}

#[tauri::command]
pub async fn delete_model_config(model_id: String, db: State<'_, DbState>) -> Result<(), String> {
    sqlx::query("DELETE FROM model_configs WHERE id = ?")
        .bind(&model_id)
        .execute(&db.0)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn test_connection_cmd(config: ModelConfig, api_key: String) -> Result<bool, String> {
    if config.api_format == "anthropic" {
        crate::adapters::anthropic::test_connection(&config.base_url, &api_key, &config.model_name)
            .await
            .map_err(|e| e.to_string())
    } else {
        crate::adapters::openai::test_connection(&config.base_url, &api_key, &config.model_name)
            .await
            .map_err(|e| e.to_string())
    }
}
