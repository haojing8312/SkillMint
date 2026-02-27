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
        "SELECT id, name, api_format, base_url, model_name, CAST(is_default AS BOOLEAN) FROM model_configs WHERE api_format NOT LIKE 'search_%'"
    )
    .fetch_all(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|(id, name, api_format, base_url, model_name, is_default)| {
        ModelConfig { id, name, api_format, base_url, model_name, is_default }
    }).collect())
}

/// 获取指定配置的 API Key（编辑时用）
#[tauri::command]
pub async fn get_model_api_key(model_id: String, db: State<'_, DbState>) -> Result<String, String> {
    let row = sqlx::query_as::<_, (String,)>(
        "SELECT api_key FROM model_configs WHERE id = ?"
    )
    .bind(&model_id)
    .fetch_optional(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    match row {
        Some((key,)) => Ok(key),
        None => Err("配置不存在".to_string()),
    }
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

/// 列出所有搜索 Provider 配置
#[tauri::command]
pub async fn list_search_configs(db: State<'_, DbState>) -> Result<Vec<ModelConfig>, String> {
    let rows = sqlx::query_as::<_, (String, String, String, String, String, bool)>(
        "SELECT id, name, api_format, base_url, model_name, CAST(is_default AS BOOLEAN) FROM model_configs WHERE api_format LIKE 'search_%'"
    )
    .fetch_all(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|(id, name, api_format, base_url, model_name, is_default)| {
        ModelConfig { id, name, api_format, base_url, model_name, is_default }
    }).collect())
}

/// 测试搜索 Provider 连接（执行一次最小化搜索请求）
#[tauri::command]
pub async fn test_search_connection(config: ModelConfig, api_key: String) -> Result<bool, String> {
    use crate::agent::tools::search_providers::{create_provider, SearchParams};

    let provider = create_provider(&config.api_format, &config.base_url, &api_key, &config.model_name)
        .map_err(|e| format!("创建 Provider 失败: {}", e))?;

    let result = tokio::task::spawn_blocking(move || {
        provider.search(&SearchParams {
            query: "test".to_string(),
            count: 1,
            freshness: None,
        })
    })
    .await
    .map_err(|e| format!("测试线程异常: {}", e))?;

    match result {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("连接测试失败: {}", e)),
    }
}

/// 设置默认搜索 Provider（同时取消同类其他配置的默认状态）
#[tauri::command]
pub async fn set_default_search(config_id: String, db: State<'_, DbState>) -> Result<(), String> {
    // 先清除所有搜索配置的默认标记
    sqlx::query("UPDATE model_configs SET is_default = 0 WHERE api_format LIKE 'search_%'")
        .execute(&db.0)
        .await
        .map_err(|e| e.to_string())?;

    // 再将指定配置设为默认
    sqlx::query("UPDATE model_configs SET is_default = 1 WHERE id = ?")
        .bind(&config_id)
        .execute(&db.0)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
