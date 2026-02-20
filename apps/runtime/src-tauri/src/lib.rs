pub mod agent;
mod adapters;
mod commands;
mod db;

use commands::skills::DbState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let pool = tauri::async_runtime::block_on(db::init_db(app.handle()))
                .expect("failed to init db");
            app.manage(DbState(pool));
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
