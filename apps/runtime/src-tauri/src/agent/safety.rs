use super::run_guard::RunStopReason;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn normalize_policy_blocked_detail(error_text: &str) -> String {
    error_text
        .strip_prefix("工具执行错误: ")
        .unwrap_or(error_text)
        .trim()
        .to_string()
}

pub(super) fn classify_policy_blocked_tool_error(
    _tool_name: &str,
    error_text: &str,
) -> Option<RunStopReason> {
    let normalized = normalize_policy_blocked_detail(error_text);
    if normalized.contains("不在工作目录") && normalized.contains("范围内") {
        return Some(RunStopReason::policy_blocked(format!(
            "目标路径不在当前工作目录范围内。你可以先切换当前会话的工作目录后重试。\n{normalized}"
        )));
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileDeleteTargetKind {
    File,
    Directory,
    Unknown,
}

fn resolve_delete_target_path(path: &str, work_dir: Option<&Path>) -> Option<PathBuf> {
    if path.trim().is_empty() {
        return None;
    }

    let candidate = PathBuf::from(path);
    if candidate.is_absolute() {
        return Some(candidate);
    }

    match work_dir {
        Some(dir) => Some(dir.join(candidate)),
        None => Some(candidate),
    }
}

fn detect_file_delete_target_kind(path: &str, work_dir: Option<&Path>) -> FileDeleteTargetKind {
    let Some(resolved) = resolve_delete_target_path(path, work_dir) else {
        return FileDeleteTargetKind::Unknown;
    };

    if resolved.is_file() {
        FileDeleteTargetKind::File
    } else if resolved.is_dir() {
        FileDeleteTargetKind::Directory
    } else {
        FileDeleteTargetKind::Unknown
    }
}

pub(super) fn critical_action_summary(
    tool_name: &str,
    input: &Value,
    work_dir: Option<&Path>,
) -> (String, String, String, bool) {
    let path = input
        .get("path")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    match tool_name {
        "file_delete" => {
            let recursive = input
                .get("recursive")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let target_kind = detect_file_delete_target_kind(&path, work_dir);

            match (target_kind, recursive) {
                (FileDeleteTargetKind::File, _) => (
                    "删除文件".to_string(),
                    format!(
                        "将删除文件 {}",
                        if path.is_empty() {
                            "目标文件"
                        } else {
                            &path
                        }
                    ),
                    "该操作不可逆，删除后无法自动恢复。".to_string(),
                    true,
                ),
                (FileDeleteTargetKind::Directory, false) => (
                    "删除文件夹".to_string(),
                    format!(
                        "将删除文件夹 {}",
                        if path.is_empty() {
                            "目标文件夹"
                        } else {
                            &path
                        }
                    ),
                    "该操作不可逆，删除后无法自动恢复。".to_string(),
                    true,
                ),
                (FileDeleteTargetKind::Directory, true) => (
                    "递归删除文件夹".to_string(),
                    format!(
                        "将递归删除文件夹 {}",
                        if path.is_empty() {
                            "目标文件夹"
                        } else {
                            &path
                        }
                    ),
                    "该操作不可逆，文件夹及其内容删除后无法自动恢复。".to_string(),
                    true,
                ),
                (FileDeleteTargetKind::Unknown, true) => (
                    "递归删除目标".to_string(),
                    format!(
                        "将递归删除 {}",
                        if path.is_empty() {
                            "目标文件或文件夹"
                        } else {
                            &path
                        }
                    ),
                    "该操作不可逆，目标及其内容删除后无法自动恢复。".to_string(),
                    true,
                ),
                (FileDeleteTargetKind::Unknown, false) => (
                    "删除目标".to_string(),
                    format!(
                        "将删除 {}",
                        if path.is_empty() {
                            "目标文件或文件夹"
                        } else {
                            &path
                        }
                    ),
                    "该操作不可逆，删除后无法自动恢复。".to_string(),
                    true,
                ),
            }
        }
        "write_file" => (
            "写入文件".to_string(),
            format!(
                "将写入 {}",
                if path.is_empty() {
                    "目标文件"
                } else {
                    &path
                }
            ),
            "该操作可能覆盖现有内容，请确认影响范围。".to_string(),
            false,
        ),
        "edit" => (
            "修改文件".to_string(),
            format!(
                "将修改 {}",
                if path.is_empty() {
                    "目标文件"
                } else {
                    &path
                }
            ),
            "这可能改变现有文件内容，请确认替换目标正确。".to_string(),
            false,
        ),
        "bash" => {
            let command = input
                .get("command")
                .and_then(Value::as_str)
                .unwrap_or("命令");
            (
                "执行高危命令".to_string(),
                format!("将执行命令：{}", command),
                "该命令可能删除文件、重置环境或影响系统状态。".to_string(),
                true,
            )
        }
        "browser_click" | "browser_type" | "browser_press_key" | "browser_evaluate"
        | "browser_act" => (
            "提交网页操作".to_string(),
            "将执行可能触发提交、发送或状态变更的浏览器动作".to_string(),
            "这可能在外部系统中创建、修改或删除真实数据。".to_string(),
            true,
        ),
        _ => (
            "高危操作确认".to_string(),
            format!("将执行工具 {}", tool_name),
            "该操作具有较高风险，请确认后继续。".to_string(),
            false,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{classify_policy_blocked_tool_error, critical_action_summary};
    use crate::agent::run_guard::RunStopReasonKind;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn unique_temp_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!("workclaw-{}-{}", label, Uuid::new_v4()))
    }

    #[test]
    fn workspace_boundary_error_maps_to_policy_blocked() {
        let reason = classify_policy_blocked_tool_error(
            "list_dir",
            "工具执行错误: 路径 C:\\Users\\Administrator\\Desktop 不在工作目录 C:\\Users\\Administrator\\WorkClaw\\workspace 范围内",
        )
        .expect("should classify");

        assert_eq!(reason.kind, RunStopReasonKind::PolicyBlocked);
        assert!(reason
            .detail
            .as_deref()
            .unwrap_or_default()
            .contains("切换当前会话的工作目录"));
    }

    #[test]
    fn skill_allowlist_error_is_not_policy_blocked() {
        let reason = classify_policy_blocked_tool_error("bash", "此 Skill 不允许使用工具: bash");

        assert!(reason.is_none());
    }

    #[test]
    fn ordinary_tool_failure_is_not_policy_blocked() {
        let reason = classify_policy_blocked_tool_error(
            "read_file",
            "工具执行错误: 文件不存在: missing.txt",
        );

        assert!(reason.is_none());
    }

    #[test]
    fn file_delete_confirmation_describes_file_targets() {
        let file_path = unique_temp_path("delete-file.txt");
        let path_text = file_path.display().to_string();
        fs::write(&file_path, "danger").expect("create temp file");

        let (title, summary, impact, irreversible) =
            critical_action_summary("file_delete", &json!({ "path": path_text }), None);

        assert_eq!(title, "删除文件");
        assert_eq!(summary, format!("将删除文件 {}", file_path.display()));
        assert_eq!(impact, "该操作不可逆，删除后无法自动恢复。");
        assert!(irreversible);

        fs::remove_file(&file_path).expect("cleanup temp file");
    }

    #[test]
    fn file_delete_confirmation_describes_folder_targets() {
        let dir_path = unique_temp_path("delete-folder");
        let path_text = dir_path.display().to_string();
        fs::create_dir_all(&dir_path).expect("create temp folder");

        let (title, summary, impact, irreversible) =
            critical_action_summary("file_delete", &json!({ "path": path_text }), None);

        assert_eq!(title, "删除文件夹");
        assert_eq!(summary, format!("将删除文件夹 {}", dir_path.display()));
        assert_eq!(impact, "该操作不可逆，删除后无法自动恢复。");
        assert!(irreversible);

        fs::remove_dir(&dir_path).expect("cleanup temp folder");
    }

    #[test]
    fn file_delete_confirmation_describes_recursive_folder_targets() {
        let dir_path = unique_temp_path("delete-folder-recursive");
        let nested_file = dir_path.join("nested.txt");
        let path_text = dir_path.display().to_string();
        fs::create_dir_all(&dir_path).expect("create temp folder");
        fs::write(&nested_file, "nested").expect("create nested file");

        let (title, summary, impact, irreversible) = critical_action_summary(
            "file_delete",
            &json!({ "path": path_text, "recursive": true }),
            None,
        );

        assert_eq!(title, "递归删除文件夹");
        assert_eq!(summary, format!("将递归删除文件夹 {}", dir_path.display()));
        assert_eq!(impact, "该操作不可逆，文件夹及其内容删除后无法自动恢复。");
        assert!(irreversible);

        fs::remove_dir_all(&dir_path).expect("cleanup recursive temp folder");
    }

    #[test]
    fn file_delete_confirmation_falls_back_for_unknown_targets() {
        let missing_path = unique_temp_path("missing-target");
        let path_text = missing_path.display().to_string();

        let (title, summary, impact, irreversible) =
            critical_action_summary("file_delete", &json!({ "path": path_text }), None);

        assert_eq!(title, "删除目标");
        assert_eq!(summary, format!("将删除 {}", missing_path.display()));
        assert_eq!(impact, "该操作不可逆，删除后无法自动恢复。");
        assert!(irreversible);
    }
}
