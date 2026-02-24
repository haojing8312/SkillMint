use crate::agent::types::{Tool, ToolContext};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::process::Command;

/// 在系统文件管理器中显示文件或目录
pub struct OpenInFolderTool;

impl Tool for OpenInFolderTool {
    fn name(&self) -> &str {
        "open_in_folder"
    }

    fn description(&self) -> &str {
        "在系统文件管理器中显示文件或目录。如果是文件，会在文件管理器中选中该文件。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "要在文件管理器中打开的文件或目录路径（相对或绝对）"
                }
            },
            "required": ["path"]
        })
    }

    fn execute(&self, input: Value, ctx: &ToolContext) -> Result<String> {
        let path = input["path"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 path 参数"))?;

        let checked = ctx.check_path(path)?;

        // 检查路径是否存在
        if !checked.exists() {
            return Err(anyhow!("路径不存在: {}", checked.display()));
        }

        let path_str = checked.to_string_lossy().to_string();

        // 根据平台调用文件管理器
        open_in_file_manager(&path_str, checked.is_dir())?;

        if checked.is_dir() {
            Ok(format!("已在文件管理器中打开目录: {}", path_str))
        } else {
            Ok(format!("已在文件管理器中显示文件: {}", path_str))
        }
    }
}

/// 根据操作系统打开文件管理器
#[cfg(target_os = "windows")]
fn open_in_file_manager(path: &str, is_dir: bool) -> Result<()> {
    if is_dir {
        // 打开目录
        Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| anyhow!("启动 explorer 失败: {}", e))?;
    } else {
        // 打开文件所在目录并选中文件
        Command::new("explorer")
            .args(["/select,", path])
            .spawn()
            .map_err(|e| anyhow!("启动 explorer 失败: {}", e))?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn open_in_file_manager(path: &str, is_dir: bool) -> Result<()> {
    if is_dir {
        // 在 Finder 中打开目录
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| anyhow!("启动 Finder 失败: {}", e))?;
    } else {
        // 在 Finder 中显示并选中文件
        Command::new("open")
            .args(["-R", path])
            .spawn()
            .map_err(|e| anyhow!("启动 Finder 失败: {}", e))?;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn open_in_file_manager(path: &str, is_dir: bool) -> Result<()> {
    if is_dir {
        // 直接打开目录
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| anyhow!("启动文件管理器失败: {}", e))?;
    } else {
        // Linux 下 xdg-open 不支持选中文件，打开文件的父目录
        let parent = std::path::Path::new(path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());
        Command::new("xdg-open")
            .arg(&parent)
            .spawn()
            .map_err(|e| anyhow!("启动文件管理器失败: {}", e))?;
    }
    Ok(())
}
