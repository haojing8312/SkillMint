use super::types::{DesktopCleanupResult, DesktopLifecyclePaths};
use crate::commands::runtime_preferences::resolve_default_work_dir_with_pool;
use crate::diagnostics;
use sqlx::SqlitePool;
use std::fs;
use std::path::Path;
use std::process::Command;
use tauri::{AppHandle, Manager};

pub(crate) async fn resolve_desktop_lifecycle_paths(
    app: &AppHandle,
    pool: &SqlitePool,
) -> Result<DesktopLifecyclePaths, String> {
    let default_work_dir = resolve_default_work_dir_with_pool(pool)
        .await
        .unwrap_or_default();
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let cache_dir = app.path().app_cache_dir().map_err(|e| e.to_string())?;
    let log_dir = app.path().app_log_dir().map_err(|e| e.to_string())?;
    let diagnostics_dir = diagnostics::DiagnosticsPaths::from_app(app).root;

    Ok(DesktopLifecyclePaths {
        app_data_dir: app_data_dir.to_string_lossy().to_string(),
        cache_dir: cache_dir.to_string_lossy().to_string(),
        log_dir: log_dir.to_string_lossy().to_string(),
        diagnostics_dir: diagnostics_dir.to_string_lossy().to_string(),
        default_work_dir,
    })
}

pub(crate) fn clear_directory_contents(path: &Path) -> Result<DesktopCleanupResult, String> {
    if !path.exists() {
        return Ok(DesktopCleanupResult::default());
    }

    let mut result = DesktopCleanupResult::default();
    let entries =
        fs::read_dir(path).map_err(|e| format!("读取目录失败 {}: {}", path.display(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录项失败 {}: {}", path.display(), e))?;
        let target = entry.path();
        if target.is_dir() {
            fs::remove_dir_all(&target)
                .map_err(|e| format!("删除目录失败 {}: {}", target.display(), e))?;
            result.removed_dirs += 1;
        } else {
            fs::remove_file(&target)
                .map_err(|e| format!("删除文件失败 {}: {}", target.display(), e))?;
            result.removed_files += 1;
        }
    }
    Ok(result)
}

pub(crate) fn merge_cleanup_result(acc: &mut DesktopCleanupResult, next: DesktopCleanupResult) {
    acc.removed_files += next.removed_files;
    acc.removed_dirs += next.removed_dirs;
}

pub(crate) fn open_path_with_system(target: &Path) -> Result<(), String> {
    if !target.exists() {
        return Err(format!("目录不存在: {}", target.display()));
    }

    #[cfg(target_os = "windows")]
    let status = Command::new("explorer").arg(target).status();

    #[cfg(target_os = "macos")]
    let status = Command::new("open").arg(target).status();

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    let status = Command::new("xdg-open").arg(target).status();

    let status = status.map_err(|e| format!("打开目录失败 {}: {}", target.display(), e))?;
    if !status.success() {
        return Err(format!("打开目录失败: {}", target.display()));
    }
    Ok(())
}
