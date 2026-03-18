use crate::agent::tools::tool_result;
use crate::agent::types::{Tool, ToolContext};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::path::PathBuf;

pub struct ListDirTool;

/// 将字节数转换为人类可读的大小格式
fn format_size(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

impl Tool for ListDirTool {
    fn name(&self) -> &str {
        "list_dir"
    }

    fn description(&self) -> &str {
        "列出目录内容。先返回可读列表，再追加结构化 entries JSON。后续 file_move/file_copy/file_delete 等文件工具应优先复用 entries 中的原始 path，不要手写或改写文件名。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "要列出内容的目录路径（相对或绝对）"
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

        if !checked.is_dir() {
            return Err(anyhow!("路径不是目录: {}", checked.display()));
        }

        let mut entries: Vec<String> = Vec::new();
        let mut structured_entries: Vec<Value> = Vec::new();

        let read_dir = std::fs::read_dir(&checked).map_err(|e| anyhow!("读取目录失败: {}", e))?;

        // 收集并排序条目
        let mut dir_entries: Vec<_> = read_dir.filter_map(|entry| entry.ok()).collect();
        dir_entries.sort_by_key(|e| e.file_name());

        for entry in dir_entries {
            let metadata = entry.metadata();
            let name = entry.file_name().to_string_lossy().to_string();
            let entry_path: PathBuf = entry.path();
            let absolute_path = entry_path.to_string_lossy().to_string();

            if let Ok(meta) = metadata {
                if meta.is_dir() {
                    entries.push(format!("[DIR]  {}", name));
                    structured_entries.push(json!({
                        "name": name,
                        "path": absolute_path,
                        "kind": "dir",
                        "size_bytes": Value::Null,
                    }));
                } else {
                    let size = format_size(meta.len());
                    entries.push(format!("[FILE] {} ({})", name, size));
                    structured_entries.push(json!({
                        "name": name,
                        "path": absolute_path,
                        "kind": "file",
                        "size_bytes": meta.len(),
                    }));
                }
            } else {
                // 元数据读取失败时仍然列出条目名称
                entries.push(format!("[?]    {}", name));
                structured_entries.push(json!({
                    "name": name,
                    "path": absolute_path,
                    "kind": "unknown",
                    "size_bytes": Value::Null,
                }));
            }
        }

        let summary = if entries.is_empty() {
            "空目录".to_string()
        } else {
            entries.join("\n")
        };

        tool_result::success(
            self.name(),
            summary,
            json!({
                "path": path,
                "absolute_path": checked.to_string_lossy().to_string(),
                "entry_count": structured_entries.len(),
                "entries": structured_entries,
            }),
        )
    }
}
