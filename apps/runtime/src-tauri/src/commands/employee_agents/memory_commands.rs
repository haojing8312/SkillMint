use super::{collect_employee_profile_memory_status_from_root, EmployeeProfileMemoryStatus};
use crate::{commands::skills::DbState, runtime_environment::runtime_paths_from_app};
use std::path::PathBuf;

pub async fn get_employee_profile_memory_status(
    employee_id: String,
    skill_id: String,
    profile_id: Option<String>,
    work_dir: Option<String>,
    im_role_id: Option<String>,
    app: tauri::AppHandle,
    db: tauri::State<'_, DbState>,
) -> Result<EmployeeProfileMemoryStatus, String> {
    let runtime_paths = runtime_paths_from_app(&app)?;
    let resolved_profile_id = match profile_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(value) => Some(value.to_string()),
        None => crate::profile_runtime::resolve_profile_for_alias_with_pool(&db.0, &employee_id)
            .await?
            .map(|resolution| resolution.profile_id),
    };
    let work_dir_path = work_dir
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);

    collect_employee_profile_memory_status_from_root(
        &runtime_paths.root,
        &runtime_paths.memory_dir,
        work_dir_path.as_deref(),
        &skill_id,
        &employee_id,
        resolved_profile_id.as_deref(),
        im_role_id.as_deref(),
    )
}

#[cfg(test)]
mod tests {
    use super::super::UpsertAgentEmployeeInput;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn collect_employee_profile_memory_status_reports_profile_home_only() {
        let tmp = TempDir::new().expect("tmp");
        let runtime_root = tmp.path().join("runtime-root");
        let memory_root = runtime_root.join("memory");
        let profile_memory_dir = runtime_root
            .join("profiles")
            .join("profile-1")
            .join("memories");
        fs::create_dir_all(&profile_memory_dir).expect("profile dir");
        fs::write(profile_memory_dir.join("MEMORY.md"), "profile memory").expect("profile write");

        let status = super::collect_employee_profile_memory_status_from_root(
            &runtime_root,
            &memory_root,
            None,
            "builtin-general",
            "sales_lead",
            Some("profile-1"),
            None,
        )
        .expect("status");

        assert_eq!(status.employee_id, "sales_lead");
        assert_eq!(status.profile_id.as_deref(), Some("profile-1"));
        assert_eq!(status.active_source, "profile");
        assert!(status.profile_memory_file_exists);
        assert!(status
            .active_source_path
            .as_ref()
            .is_some_and(|path| path.ends_with("MEMORY.md")));
    }

    #[test]
    fn upsert_input_defaults_routing_priority_when_missing() {
        let payload = serde_json::json!({
            "employee_id": "project_manager",
            "name": "项目经理",
            "role_id": "project_manager",
            "persona": "负责推进交付",
            "feishu_open_id": "",
            "feishu_app_id": "",
            "feishu_app_secret": "",
            "primary_skill_id": "builtin-general",
            "default_work_dir": "",
            "openclaw_agent_id": "project_manager",
            "enabled_scopes": ["app"],
            "enabled": true,
            "is_default": false,
            "skill_ids": []
        });
        let parsed: UpsertAgentEmployeeInput =
            serde_json::from_value(payload).expect("deserialize upsert input");
        assert_eq!(parsed.routing_priority, 100);
    }
}
