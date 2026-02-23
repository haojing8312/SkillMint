pub mod agent;
pub mod sidecar;
mod adapters;
mod commands;
mod db;

use agent::{AgentExecutor, ToolRegistry};
use commands::skills::DbState;
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 初始化数据库
            let pool = tauri::async_runtime::block_on(db::init_db(app.handle()))
                .expect("failed to init db");
            let pool_for_mcp = pool.clone();
            app.manage(DbState(pool));

            // 初始化 AgentExecutor（包含文件工具）
            let registry = Arc::new(ToolRegistry::with_file_tools());
            let agent_executor = Arc::new(AgentExecutor::new(Arc::clone(&registry)));
            app.manage(agent_executor);
            app.manage(Arc::clone(&registry));

            // 恢复已保存的 MCP 服务器连接
            let registry_for_mcp = Arc::clone(&registry);
            tauri::async_runtime::spawn(async move {
                let servers = sqlx::query_as::<_, (String, String, String, String)>(
                    "SELECT name, command, args, env FROM mcp_servers WHERE enabled = 1"
                )
                .fetch_all(&pool_for_mcp)
                .await
                .unwrap_or_default();

                if servers.is_empty() {
                    return;
                }

                let client = reqwest::Client::new();
                for (name, command, args_json, env_json) in servers {
                    let args: Vec<String> = serde_json::from_str(&args_json).unwrap_or_default();
                    let env: std::collections::HashMap<String, String> =
                        serde_json::from_str(&env_json).unwrap_or_default();

                    // 连接 MCP 服务器
                    let connect_result = client.post("http://localhost:8765/api/mcp/add-server")
                        .json(&serde_json::json!({
                            "name": name,
                            "config": { "command": command, "args": args, "env": env }
                        }))
                        .send()
                        .await;

                    if connect_result.is_err() {
                        eprintln!("[mcp] 连接 MCP 服务器 {} 失败（Sidecar 可能未启动）", name);
                        continue;
                    }

                    // 获取工具列表并注册
                    if let Ok(resp) = client.post("http://localhost:8765/api/mcp/list-tools")
                        .json(&serde_json::json!({ "serverName": name }))
                        .send()
                        .await
                    {
                        if let Ok(body) = resp.json::<serde_json::Value>().await {
                            if let Some(tool_list) = body["tools"].as_array() {
                                for tool in tool_list {
                                    let tool_name = tool["name"].as_str().unwrap_or_default();
                                    let tool_desc = tool["description"].as_str().unwrap_or_default();
                                    let schema = tool.get("inputSchema").cloned()
                                        .unwrap_or(serde_json::json!({"type": "object", "properties": {}}));

                                    let full_name = format!("mcp_{}_{}", name, tool_name);
                                    registry_for_mcp.register(Arc::new(
                                        agent::tools::SidecarBridgeTool::new_mcp(
                                            "http://localhost:8765".to_string(),
                                            full_name,
                                            tool_desc.to_string(),
                                            schema,
                                            name.clone(),
                                            tool_name.to_string(),
                                        )
                                    ));
                                }
                                eprintln!("[mcp] 已恢复 MCP 服务器 {} 的工具注册", name);
                            }
                        }
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::skills::install_skill,
            commands::skills::list_skills,
            commands::skills::delete_skill,
            commands::models::save_model_config,
            commands::models::list_model_configs,
            commands::models::delete_model_config,
            commands::models::test_connection_cmd,
            commands::chat::create_session,
            commands::chat::send_message,
            commands::chat::get_messages,
            commands::chat::get_sessions,
            commands::chat::delete_session,
            commands::chat::answer_user_question,
            commands::chat::confirm_tool_execution,
            commands::mcp::add_mcp_server,
            commands::mcp::list_mcp_servers,
            commands::mcp::remove_mcp_server,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
