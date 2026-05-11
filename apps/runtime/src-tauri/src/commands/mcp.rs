use super::skills::DbState;
use crate::agent::tools::{list_native_mcp_tools, NativeMcpServerConfig, NativeMcpTool};
use crate::agent::ToolRegistry;
use chrono::Utc;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

pub async fn register_native_mcp_server_tools(
    registry: Arc<ToolRegistry>,
    server: NativeMcpServerConfig,
) -> Result<usize, String> {
    let tools = list_native_mcp_tools(&server)
        .await
        .map_err(|e| e.to_string())?;
    let mut registered = 0;

    for tool in tools {
        let full_name = format!("mcp_{}_{}", server.name, tool.name);
        registry.register(Arc::new(NativeMcpTool::new(
            full_name,
            tool.description,
            tool.input_schema,
            server.clone(),
            tool.name,
        )));
        registered += 1;
    }

    Ok(registered)
}

pub async fn add_mcp_server_with_registry(
    pool: &sqlx::SqlitePool,
    registry: Arc<ToolRegistry>,
    name: String,
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    // 保存到数据库
    sqlx::query(
        "INSERT INTO mcp_servers (id, name, command, args, env, enabled, created_at) VALUES (?, ?, ?, ?, ?, 1, ?)"
    )
    .bind(&id)
    .bind(&name)
    .bind(&command)
    .bind(serde_json::to_string(&args).unwrap_or_default())
    .bind(serde_json::to_string(&env).unwrap_or_default())
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    let server = NativeMcpServerConfig {
        name,
        command,
        args,
        env,
    };

    if let Err(error) = register_native_mcp_server_tools(registry, server).await {
        let _ = sqlx::query("DELETE FROM mcp_servers WHERE id = ?")
            .bind(&id)
            .execute(pool)
            .await;
        return Err(error);
    }

    Ok(id)
}

#[tauri::command]
pub async fn add_mcp_server(
    name: String,
    command: String,
    args: Vec<String>,
    env: std::collections::HashMap<String, String>,
    db: State<'_, DbState>,
    registry: State<'_, Arc<ToolRegistry>>,
) -> Result<String, String> {
    add_mcp_server_with_registry(
        &db.0,
        Arc::clone(&registry.inner()),
        name,
        command,
        args,
        env,
    )
    .await
}

#[tauri::command]
pub async fn list_mcp_servers(db: State<'_, DbState>) -> Result<Vec<Value>, String> {
    let rows = sqlx::query_as::<_, (String, String, String, String, String, i32, String)>(
        "SELECT id, name, command, args, env, enabled, created_at FROM mcp_servers ORDER BY created_at DESC"
    )
    .fetch_all(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .iter()
        .map(|(id, name, command, args, env, enabled, created_at)| {
            json!({
                "id": id,
                "name": name,
                "command": command,
                "args": serde_json::from_str::<Value>(args).unwrap_or(json!([])),
                "env": serde_json::from_str::<Value>(env).unwrap_or(json!({})),
                "enabled": enabled == &1,
                "created_at": created_at,
            })
        })
        .collect())
}

pub async fn remove_mcp_server_with_registry(
    pool: &sqlx::SqlitePool,
    registry: Arc<ToolRegistry>,
    id: String,
) -> Result<(), String> {
    // 获取 server name
    let (name,): (String,) = sqlx::query_as("SELECT name FROM mcp_servers WHERE id = ?")
        .bind(&id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    // 从 registry 反注册所有该服务器的工具
    let prefix = format!("mcp_{}_", name);
    let tool_names = registry.tools_with_prefix(&prefix);
    for tool_name in tool_names {
        registry.unregister(&tool_name);
    }

    // 从数据库删除
    sqlx::query("DELETE FROM mcp_servers WHERE id = ?")
        .bind(&id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn remove_mcp_server(
    id: String,
    db: State<'_, DbState>,
    registry: State<'_, Arc<ToolRegistry>>,
) -> Result<(), String> {
    remove_mcp_server_with_registry(&db.0, Arc::clone(&registry.inner()), id).await
}

pub async fn restore_saved_mcp_servers_with_registry(
    pool: &sqlx::SqlitePool,
    registry: Arc<ToolRegistry>,
) -> Result<usize, String> {
    let servers = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT name, command, args, env FROM mcp_servers WHERE enabled = 1",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut restored = 0;
    for (name, command, args_json, env_json) in servers {
        let args: Vec<String> = serde_json::from_str(&args_json).unwrap_or_default();
        let env: HashMap<String, String> = serde_json::from_str(&env_json).unwrap_or_default();
        let server = NativeMcpServerConfig {
            name: name.clone(),
            command,
            args,
            env,
        };

        match register_native_mcp_server_tools(Arc::clone(&registry), server).await {
            Ok(_) => {
                restored += 1;
                eprintln!("[mcp] restored native MCP server tool registration for {name}");
            }
            Err(error) => {
                eprintln!("[mcp] failed to restore native MCP server {name}: {error}");
            }
        }
    }

    Ok(restored)
}
