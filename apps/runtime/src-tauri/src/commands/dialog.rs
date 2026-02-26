use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub async fn select_directory(
    app: AppHandle,
    default_path: Option<String>,
) -> Result<Option<String>, String> {
    let mut builder = app.dialog().file();

    if let Some(path) = default_path {
        builder = builder.set_directory(&path);
    }

    let result = builder.blocking_pick_folder();

    Ok(result.map(|p| p.to_string()))
}
