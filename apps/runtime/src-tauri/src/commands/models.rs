use keyring::Entry;
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

fn keyring_entry(model_id: &str) -> keyring::Result<Entry> {
    Entry::new("skillhub-runtime", &format!("model-{}", model_id))
}

pub fn get_api_key(model_id: &str) -> Option<String> {
    keyring_entry(model_id).ok()?.get_password().ok()
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
        "INSERT OR REPLACE INTO model_configs (id, name, api_format, base_url, model_name, is_default) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&config.name)
    .bind(&config.api_format)
    .bind(&config.base_url)
    .bind(&config.model_name)
    .bind(config.is_default)
    .execute(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    if !api_key.is_empty() {
        keyring_entry(&id)
            .and_then(|e| e.set_password(&api_key))
            .map_err(|e| e.to_string())?;
    }
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
    let _ = keyring_entry(&model_id).and_then(|e| e.delete_credential());
    Ok(())
}

#[tauri::command]
pub async fn test_connection_cmd(config: ModelConfig, api_key: String) -> Result<bool, String> {
    if config.api_format == "anthropic" {
        crate::adapters::anthropic::test_connection(&api_key, &config.model_name)
            .await
            .map_err(|e| e.to_string())
    } else {
        crate::adapters::openai::test_connection(&config.base_url, &api_key, &config.model_name)
            .await
            .map_err(|e| e.to_string())
    }
}
