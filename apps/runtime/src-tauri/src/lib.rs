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
            app.manage(DbState(pool));

            // 初始化 AgentExecutor（包含文件工具）
            let registry = Arc::new(ToolRegistry::with_file_tools());
            let agent_executor = Arc::new(AgentExecutor::new(registry));
            app.manage(agent_executor);

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
