use crate::agent::types::{Tool, ToolContext};
use crate::agent::tools::tool_result;
use anyhow::{anyhow, Result};
use regex::RegexBuilder;
use serde_json::{json, Value};
use std::path::Path;

/// 单次搜索结果的最大行数，防止输出过大
const MAX_RESULT_LINES: usize = 500;

pub struct GrepTool;

impl Tool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "在文件或目录中搜索文本模式（正则表达式）。返回结构化匹配结果。支持目录递归搜索。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "正则表达式搜索模式"
                },
                "path": {
                    "type": "string",
                    "description": "要搜索的文件或目录路径"
                },
                "case_insensitive": {
                    "type": "boolean",
                    "description": "是否忽略大小写（可选，默认 false）"
                },
                "file_pattern": {
                    "type": "string",
                    "description": "文件名过滤（如 *.py, *.rs），仅在搜索目录时生效（可选）"
                }
            },
            "required": ["pattern", "path"]
        })
    }

    fn execute(&self, input: Value, ctx: &ToolContext) -> Result<String> {
        let pattern = input["pattern"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 pattern 参数"))?;
        let path = input["path"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 path 参数"))?;
        let case_insensitive = input["case_insensitive"].as_bool().unwrap_or(false);
        let file_pattern = input["file_pattern"].as_str();

        let checked = ctx.check_path(path)?;

        let re = RegexBuilder::new(pattern)
            .case_insensitive(case_insensitive)
            .build()
            .map_err(|e| anyhow!("正则表达式错误: {}", e))?;

        let metadata = std::fs::metadata(&checked).map_err(|e| anyhow!("无法访问路径: {}", e))?;

        if metadata.is_file() {
            // 单文件搜索
            search_file(&checked, path, &re)
        } else if metadata.is_dir() {
            // 目录递归搜索
            let file_glob = file_pattern.map(|p| {
                glob::Pattern::new(p).unwrap_or_else(|_| glob::Pattern::new("*").unwrap())
            });
            search_directory(&checked, path, &re, file_glob.as_ref())
        } else {
            Err(anyhow!("路径既不是文件也不是目录: {}", path))
        }
    }
}

/// 搜索单个文件
fn search_file(path: &Path, original_path: &str, re: &regex::Regex) -> Result<String> {
    let content = std::fs::read_to_string(path).map_err(|e| anyhow!("读取文件失败: {}", e))?;

    let matches: Vec<Value> = content
        .lines()
        .enumerate()
        .filter(|(_, line)| re.is_match(line))
        .map(|(i, line)| {
            json!({
                "path": original_path,
                "line": i + 1,
                "text": line,
            })
        })
        .collect();

    tool_result::success(
        "grep",
        format!("找到 {} 处匹配", matches.len()),
        json!({
            "searched_path": original_path,
            "files_searched": 1,
            "total_matches": matches.len(),
            "truncated": false,
            "matches": matches,
        }),
    )
}

/// 递归搜索目录中的所有文件
fn search_directory(
    dir: &Path,
    original_path: &str,
    re: &regex::Regex,
    file_pattern: Option<&glob::Pattern>,
) -> Result<String> {
    let mut all_matches: Vec<Value> = Vec::new();
    let mut files_searched = 0u32;

    walk_dir(dir, re, file_pattern, &mut all_matches, &mut files_searched)?;

    let total = all_matches.len();
    let truncated = total > MAX_RESULT_LINES;
    if truncated {
        all_matches.truncate(MAX_RESULT_LINES);
    }

    tool_result::success(
        "grep",
        format!("在 {} 个文件中找到 {} 处匹配", files_searched, total),
        json!({
            "searched_path": original_path,
            "files_searched": files_searched,
            "total_matches": total,
            "truncated": truncated,
            "matches": all_matches,
        }),
    )
}

/// 递归遍历目录
fn walk_dir(
    dir: &Path,
    re: &regex::Regex,
    file_pattern: Option<&glob::Pattern>,
    results: &mut Vec<Value>,
    files_count: &mut u32,
) -> Result<()> {
    let entries = std::fs::read_dir(dir).map_err(|e| anyhow!("读取目录失败: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| anyhow!("读取目录条目失败: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            // 跳过隐藏目录和常见的排除目录
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') || name == "node_modules" || name == "target" {
                    continue;
                }
            }
            walk_dir(&path, re, file_pattern, results, files_count)?;
        } else if path.is_file() {
            // 检查文件名过滤
            if let Some(pattern) = file_pattern {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if !pattern.matches(name) {
                        continue;
                    }
                }
            }

            // 尝试读取文件（跳过二进制文件）
            if let Ok(content) = std::fs::read_to_string(&path) {
                *files_count += 1;
                let relative = path.strip_prefix(dir).unwrap_or(&path).to_string_lossy();

                for (i, line) in content.lines().enumerate() {
                    if re.is_match(line) {
                        results.push(json!({
                            "path": relative.to_string(),
                            "line": i + 1,
                            "text": line,
                        }));
                    }
                }
            }
        }
    }
    Ok(())
}
