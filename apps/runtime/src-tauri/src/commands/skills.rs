use chrono::Utc;
use serde_json::{Value, json};
use skillpack_rs::SkillManifest;
use sqlx::SqlitePool;
use tauri::State;

#[path = "skills/types.rs"]
mod types;

#[path = "skills/helpers.rs"]
mod helpers;

#[path = "skills/local_skill_service.rs"]
mod local_skill_service;

#[path = "skills/industry_bundle_service.rs"]
mod industry_bundle_service;

#[path = "skills/runtime_status_service.rs"]
mod runtime_status_service;

pub use industry_bundle_service::{
    check_industry_bundle_update_from_pool, install_industry_bundle_to_pool,
};
pub use local_skill_service::{
    create_local_skill_in_dir, ensure_skill_display_name_available, import_local_skill_to_pool,
    import_local_skills_to_pool, render_local_skill_preview_in_dir,
};
pub use runtime_status_service::get_skill_runtime_environment_status_with_pool;
pub use types::{
    DbState, ImportResult, IndustryBundleUpdateCheck, IndustryInstallResult,
    InstalledSkillListItem, InstalledSkillSummary, LocalImportBatchResult, LocalImportFailedItem,
    LocalImportInstalledItem, LocalSkillPreview, SkillRuntimeDependencyCheck,
    SkillRuntimeEnvironmentStatus,
};

#[derive(Debug, serde::Serialize)]
pub struct SkillOsMutationResult {
    pub action: String,
    pub skill: crate::agent::runtime::runtime_io::SkillOsView,
    pub version_id: String,
    pub rollback_to_version_id: Option<String>,
    pub reset_to_version_id: Option<String>,
    pub growth_event_id: Option<String>,
    pub diff: String,
}

#[tauri::command]
pub async fn render_local_skill_preview(
    name: String,
    description: String,
    when_to_use: String,
    target_dir: Option<String>,
    app: tauri::AppHandle,
) -> Result<LocalSkillPreview, String> {
    local_skill_service::render_local_skill_preview(
        name,
        description,
        when_to_use,
        target_dir,
        &app,
    )
    .await
}

#[tauri::command]
pub async fn create_local_skill(
    name: String,
    description: String,
    when_to_use: String,
    target_dir: Option<String>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    local_skill_service::create_local_skill(name, description, when_to_use, target_dir, &app).await
}

#[tauri::command]
pub async fn install_skill(
    pack_path: String,
    username: String,
    db: State<'_, DbState>,
) -> Result<SkillManifest, String> {
    local_skill_service::install_skill(pack_path, username, &db.0).await
}

#[tauri::command]
pub async fn import_local_skill(
    dir_path: String,
    db: State<'_, DbState>,
) -> Result<LocalImportBatchResult, String> {
    local_skill_service::import_local_skill(dir_path, &db.0).await
}

#[tauri::command]
pub async fn refresh_local_skill(
    skill_id: String,
    db: State<'_, DbState>,
) -> Result<SkillManifest, String> {
    local_skill_service::refresh_local_skill(skill_id, &db.0).await
}

#[tauri::command]
pub async fn install_industry_bundle(
    bundle_path: String,
    install_root: Option<String>,
    db: State<'_, DbState>,
) -> Result<IndustryInstallResult, String> {
    industry_bundle_service::install_industry_bundle(bundle_path, install_root, &db.0).await
}

#[tauri::command]
pub async fn check_industry_bundle_update(
    bundle_path: String,
    db: State<'_, DbState>,
) -> Result<IndustryBundleUpdateCheck, String> {
    industry_bundle_service::check_industry_bundle_update(bundle_path, &db.0).await
}

#[tauri::command]
pub async fn list_skills(db: State<'_, DbState>) -> Result<Vec<InstalledSkillListItem>, String> {
    let rows = sqlx::query_as::<_, (String, String)>(
        "SELECT manifest, COALESCE(source_type, 'encrypted') FROM installed_skills ORDER BY installed_at DESC",
    )
    .fetch_all(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    rows.iter()
        .map(|(json, source_type)| {
            serde_json::from_str::<SkillManifest>(json)
                .map(|manifest| InstalledSkillListItem {
                    manifest,
                    source_type: source_type.clone(),
                })
                .map_err(|e| e.to_string())
        })
        .collect()
}

#[tauri::command]
pub async fn list_skill_os_index(
    db: State<'_, DbState>,
) -> Result<Vec<crate::agent::runtime::runtime_io::SkillOsIndexEntry>, String> {
    crate::agent::runtime::runtime_io::list_skill_os_index_with_pool(&db.0).await
}

#[tauri::command]
pub async fn get_skill_os_view(
    skill_id: String,
    db: State<'_, DbState>,
) -> Result<Option<crate::agent::runtime::runtime_io::SkillOsView>, String> {
    let view =
        crate::agent::runtime::runtime_io::view_skill_os_entry_with_pool(&db.0, &skill_id).await?;
    if view.is_some() {
        crate::agent::runtime::runtime_io::record_skill_os_usage_with_pool(
            &db.0, &skill_id, "view",
        )
        .await?;
    }
    crate::agent::runtime::runtime_io::view_skill_os_entry_with_pool(&db.0, &skill_id).await
}

#[tauri::command]
pub async fn list_skill_os_versions(
    skill_id: String,
    limit: Option<i64>,
    db: State<'_, DbState>,
) -> Result<Vec<crate::agent::runtime::runtime_io::SkillOsVersionEntry>, String> {
    crate::agent::runtime::runtime_io::list_skill_os_versions_with_pool(
        &db.0,
        &skill_id,
        limit.unwrap_or(8),
    )
    .await
}

async fn skill_table_exists(pool: &SqlitePool, table_name: &str) -> Result<bool, String> {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
    )
    .bind(table_name)
    .fetch_one(pool)
    .await
    .map(|count| count > 0)
    .map_err(|e| e.to_string())
}

async fn skill_table_has_column(
    pool: &SqlitePool,
    table_name: &str,
    column_name: &str,
) -> Result<bool, String> {
    let query = format!("SELECT name FROM pragma_table_info('{table_name}')");
    let rows: Vec<String> = sqlx::query_scalar(&query)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(rows.iter().any(|name| name == column_name))
}

async fn ensure_skill_growth_events_schema(pool: &SqlitePool) -> Result<(), String> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS growth_events (
            id TEXT PRIMARY KEY,
            profile_id TEXT NOT NULL DEFAULT '',
            session_id TEXT NOT NULL DEFAULT '',
            event_type TEXT NOT NULL,
            target_type TEXT NOT NULL,
            target_id TEXT NOT NULL,
            summary TEXT NOT NULL DEFAULT '',
            evidence_json TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| format!("创建 growth_events 表失败: {e}"))?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_growth_events_profile_created
         ON growth_events(profile_id, created_at DESC)",
    )
    .execute(pool)
    .await
    .map_err(|e| format!("创建 growth_events profile 索引失败: {e}"))?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_growth_events_target
         ON growth_events(target_type, target_id, created_at DESC)",
    )
    .execute(pool)
    .await
    .map_err(|e| format!("创建 growth_events target 索引失败: {e}"))?;

    Ok(())
}

async fn resolve_skill_growth_profile_id(
    pool: &SqlitePool,
    employee_id: Option<&str>,
) -> Result<String, String> {
    let Some(employee_id) = employee_id.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(String::new());
    };
    if !skill_table_exists(pool, "agent_profiles").await? {
        return Ok(String::new());
    }
    let has_legacy_column =
        skill_table_has_column(pool, "agent_profiles", "legacy_employee_row_id").await?;
    if has_legacy_column {
        sqlx::query_scalar::<_, String>(
            "SELECT id
             FROM agent_profiles
             WHERE legacy_employee_row_id = ? OR id = ?
             ORDER BY CASE WHEN legacy_employee_row_id = ? THEN 0 ELSE 1 END
             LIMIT 1",
        )
        .bind(employee_id)
        .bind(employee_id)
        .bind(employee_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())
        .map(|value| value.unwrap_or_default())
    } else {
        sqlx::query_scalar::<_, String>("SELECT id FROM agent_profiles WHERE id = ? LIMIT 1")
            .bind(employee_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())
            .map(|value| value.unwrap_or_default())
    }
}

async fn record_skill_ui_growth_event(
    pool: &SqlitePool,
    employee_id: Option<&str>,
    event_type: &str,
    skill_id: &str,
    summary: &str,
    evidence: Value,
) -> Result<Option<String>, String> {
    let profile_id = resolve_skill_growth_profile_id(pool, employee_id).await?;
    if profile_id.trim().is_empty() {
        return Ok(None);
    }
    ensure_skill_growth_events_schema(pool).await?;
    let id = format!("grw_{}", uuid::Uuid::new_v4().simple());
    sqlx::query(
        "INSERT INTO growth_events (
            id, profile_id, session_id, event_type, target_type, target_id, summary, evidence_json, created_at
         ) VALUES (?, ?, '', ?, 'skill', ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(profile_id)
    .bind(event_type)
    .bind(skill_id)
    .bind(summary)
    .bind(serde_json::to_string(&evidence).unwrap_or_else(|_| "{}".to_string()))
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await
    .map_err(|e| format!("写入 skill growth event 失败: {e}"))?;
    Ok(Some(id))
}

fn skill_line_diff(before: &str, after: &str) -> String {
    if before == after {
        return String::new();
    }
    let before_lines = before.lines().collect::<Vec<_>>();
    let after_lines = after.lines().collect::<Vec<_>>();
    let max_len = before_lines.len().max(after_lines.len());
    let mut out = Vec::new();
    for index in 0..max_len {
        match (before_lines.get(index), after_lines.get(index)) {
            (Some(left), Some(right)) if left == right => {}
            (Some(left), Some(right)) => {
                out.push(format!("-{left}"));
                out.push(format!("+{right}"));
            }
            (Some(left), None) => out.push(format!("-{left}")),
            (None, Some(right)) => out.push(format!("+{right}")),
            (None, None) => {}
        }
    }
    out.join("\n")
}

pub async fn rollback_skill_os_with_pool(
    pool: &SqlitePool,
    skill_id: &str,
    version_id: &str,
    employee_id: Option<&str>,
    summary: &str,
    confirm: bool,
) -> Result<SkillOsMutationResult, String> {
    if !confirm {
        return Err("skill_rollback 是高风险操作，需要 confirm=true".to_string());
    }
    let before = crate::agent::runtime::runtime_io::view_skill_os_entry_with_pool(pool, skill_id)
        .await?
        .ok_or_else(|| format!("Skill 不存在: {skill_id}"))?;
    let skill = crate::agent::runtime::runtime_io::rollback_skill_os_entry_with_pool(
        pool, skill_id, version_id, summary,
    )
    .await?;
    let diff = skill_line_diff(&before.content, &skill.content);
    let latest_version_id =
        crate::agent::runtime::runtime_io::list_skill_os_versions_with_pool(pool, skill_id, 1)
            .await?
            .into_iter()
            .next()
            .map(|version| version.version_id)
            .unwrap_or_default();
    let growth_event_id = record_skill_ui_growth_event(
        pool,
        employee_id,
        "skill_rollback",
        skill_id,
        summary,
        json!({
            "version_id": latest_version_id,
            "rollback_to_version_id": version_id,
            "source_type": skill.entry.source.raw_source_type,
            "diff": diff
        }),
    )
    .await?;
    Ok(SkillOsMutationResult {
        action: "skill_rollback".to_string(),
        skill,
        version_id: latest_version_id,
        rollback_to_version_id: Some(version_id.to_string()),
        reset_to_version_id: None,
        growth_event_id,
        diff,
    })
}

pub async fn patch_skill_os_with_pool(
    pool: &SqlitePool,
    skill_id: &str,
    content: &str,
    employee_id: Option<&str>,
    summary: &str,
    confirm: bool,
) -> Result<SkillOsMutationResult, String> {
    if !confirm {
        return Err("skill_patch 是高风险操作，需要 confirm=true".to_string());
    }
    let before = crate::agent::runtime::runtime_io::view_skill_os_entry_with_pool(pool, skill_id)
        .await?
        .ok_or_else(|| format!("Skill 不存在: {skill_id}"))?;
    let skill = crate::agent::runtime::runtime_io::patch_skill_os_entry_with_pool(
        pool, skill_id, content, summary,
    )
    .await?;
    let diff = skill_line_diff(&before.content, &skill.content);
    let latest_version_id =
        crate::agent::runtime::runtime_io::list_skill_os_versions_with_pool(pool, skill_id, 1)
            .await?
            .into_iter()
            .next()
            .map(|version| version.version_id)
            .unwrap_or_default();
    let growth_event_id = record_skill_ui_growth_event(
        pool,
        employee_id,
        "skill_patch",
        skill_id,
        summary,
        json!({
            "version_id": latest_version_id,
            "source_type": skill.entry.source.raw_source_type,
            "diff": diff
        }),
    )
    .await?;
    Ok(SkillOsMutationResult {
        action: "skill_patch".to_string(),
        skill,
        version_id: latest_version_id,
        rollback_to_version_id: None,
        reset_to_version_id: None,
        growth_event_id,
        diff,
    })
}

pub async fn reset_skill_os_with_pool(
    pool: &SqlitePool,
    skill_id: &str,
    employee_id: Option<&str>,
    summary: &str,
    confirm: bool,
) -> Result<SkillOsMutationResult, String> {
    if !confirm {
        return Err("skill_reset 是高风险操作，需要 confirm=true".to_string());
    }
    let before = crate::agent::runtime::runtime_io::view_skill_os_entry_with_pool(pool, skill_id)
        .await?
        .ok_or_else(|| format!("Skill 不存在: {skill_id}"))?;
    let (skill, reset_to_version_id) =
        crate::agent::runtime::runtime_io::reset_skill_os_entry_with_pool(pool, skill_id, summary)
            .await?;
    let diff = skill_line_diff(&before.content, &skill.content);
    let latest_version_id =
        crate::agent::runtime::runtime_io::list_skill_os_versions_with_pool(pool, skill_id, 1)
            .await?
            .into_iter()
            .next()
            .map(|version| version.version_id)
            .unwrap_or_default();
    let growth_event_id = record_skill_ui_growth_event(
        pool,
        employee_id,
        "skill_reset",
        skill_id,
        summary,
        json!({
            "version_id": latest_version_id,
            "reset_to_version_id": reset_to_version_id,
            "source_type": skill.entry.source.raw_source_type,
            "diff": diff
        }),
    )
    .await?;
    Ok(SkillOsMutationResult {
        action: "skill_reset".to_string(),
        skill,
        version_id: latest_version_id,
        rollback_to_version_id: None,
        reset_to_version_id: Some(reset_to_version_id),
        growth_event_id,
        diff,
    })
}

pub async fn archive_skill_os_with_pool(
    pool: &SqlitePool,
    skill_id: &str,
    employee_id: Option<&str>,
    summary: &str,
    confirm: bool,
) -> Result<SkillOsMutationResult, String> {
    if !confirm {
        return Err("skill_archive 是高风险操作，需要 confirm=true".to_string());
    }
    let skill = crate::agent::runtime::runtime_io::archive_skill_os_entry_with_pool(
        pool, skill_id, summary,
    )
    .await?;
    let latest_version_id =
        crate::agent::runtime::runtime_io::list_skill_os_versions_with_pool(pool, skill_id, 1)
            .await?
            .into_iter()
            .next()
            .map(|version| version.version_id)
            .unwrap_or_default();
    let growth_event_id = record_skill_ui_growth_event(
        pool,
        employee_id,
        "skill_archive",
        skill_id,
        summary,
        json!({
            "version_id": latest_version_id,
            "source_type": skill.entry.source.raw_source_type
        }),
    )
    .await?;
    Ok(SkillOsMutationResult {
        action: "skill_archive".to_string(),
        skill,
        version_id: latest_version_id,
        rollback_to_version_id: None,
        reset_to_version_id: None,
        growth_event_id,
        diff: String::new(),
    })
}

pub async fn restore_skill_os_with_pool(
    pool: &SqlitePool,
    skill_id: &str,
    employee_id: Option<&str>,
    summary: &str,
) -> Result<SkillOsMutationResult, String> {
    let skill = crate::agent::runtime::runtime_io::restore_skill_os_entry_with_pool(
        pool, skill_id, summary,
    )
    .await?;
    let latest_version_id =
        crate::agent::runtime::runtime_io::list_skill_os_versions_with_pool(pool, skill_id, 1)
            .await?
            .into_iter()
            .next()
            .map(|version| version.version_id)
            .unwrap_or_default();
    let growth_event_id = record_skill_ui_growth_event(
        pool,
        employee_id,
        "skill_restore",
        skill_id,
        summary,
        json!({
            "version_id": latest_version_id,
            "source_type": skill.entry.source.raw_source_type
        }),
    )
    .await?;
    Ok(SkillOsMutationResult {
        action: "skill_restore".to_string(),
        skill,
        version_id: latest_version_id,
        rollback_to_version_id: None,
        reset_to_version_id: None,
        growth_event_id,
        diff: String::new(),
    })
}

pub async fn delete_skill_os_with_pool(
    pool: &SqlitePool,
    skill_id: &str,
    employee_id: Option<&str>,
    summary: &str,
    confirm: bool,
) -> Result<SkillOsMutationResult, String> {
    if !confirm {
        return Err("skill_delete 是高风险操作，需要 confirm=true".to_string());
    }
    let (skill, version, removed_path, removed_files) =
        crate::agent::runtime::runtime_io::delete_skill_os_entry_with_pool(pool, skill_id, summary)
            .await?;
    let growth_event_id = record_skill_ui_growth_event(
        pool,
        employee_id,
        "skill_delete",
        skill_id,
        summary,
        json!({
            "version_id": version.version_id,
            "source_type": skill.entry.source.raw_source_type,
            "removed_path": removed_path,
            "removed_files": removed_files
        }),
    )
    .await?;
    Ok(SkillOsMutationResult {
        action: "skill_delete".to_string(),
        skill,
        version_id: version.version_id,
        rollback_to_version_id: None,
        reset_to_version_id: None,
        growth_event_id,
        diff: String::new(),
    })
}

#[tauri::command]
pub async fn rollback_skill_os(
    skill_id: String,
    version_id: String,
    employee_id: Option<String>,
    summary: Option<String>,
    confirm: bool,
    db: State<'_, DbState>,
) -> Result<SkillOsMutationResult, String> {
    rollback_skill_os_with_pool(
        &db.0,
        &skill_id,
        &version_id,
        employee_id.as_deref(),
        summary.as_deref().unwrap_or("UI rollback skill"),
        confirm,
    )
    .await
}

#[tauri::command]
pub async fn patch_skill_os(
    skill_id: String,
    content: String,
    employee_id: Option<String>,
    summary: Option<String>,
    confirm: bool,
    db: State<'_, DbState>,
) -> Result<SkillOsMutationResult, String> {
    patch_skill_os_with_pool(
        &db.0,
        &skill_id,
        &content,
        employee_id.as_deref(),
        summary.as_deref().unwrap_or("UI patch skill"),
        confirm,
    )
    .await
}

#[tauri::command]
pub async fn reset_skill_os(
    skill_id: String,
    employee_id: Option<String>,
    summary: Option<String>,
    confirm: bool,
    db: State<'_, DbState>,
) -> Result<SkillOsMutationResult, String> {
    reset_skill_os_with_pool(
        &db.0,
        &skill_id,
        employee_id.as_deref(),
        summary.as_deref().unwrap_or("UI reset skill"),
        confirm,
    )
    .await
}

#[tauri::command]
pub async fn archive_skill_os(
    skill_id: String,
    employee_id: Option<String>,
    summary: Option<String>,
    confirm: bool,
    db: State<'_, DbState>,
) -> Result<SkillOsMutationResult, String> {
    archive_skill_os_with_pool(
        &db.0,
        &skill_id,
        employee_id.as_deref(),
        summary.as_deref().unwrap_or("UI archive skill"),
        confirm,
    )
    .await
}

#[tauri::command]
pub async fn restore_skill_os(
    skill_id: String,
    employee_id: Option<String>,
    summary: Option<String>,
    db: State<'_, DbState>,
) -> Result<SkillOsMutationResult, String> {
    restore_skill_os_with_pool(
        &db.0,
        &skill_id,
        employee_id.as_deref(),
        summary.as_deref().unwrap_or("UI restore skill"),
    )
    .await
}

#[tauri::command]
pub async fn delete_skill_os(
    skill_id: String,
    employee_id: Option<String>,
    summary: Option<String>,
    confirm: bool,
    db: State<'_, DbState>,
) -> Result<SkillOsMutationResult, String> {
    delete_skill_os_with_pool(
        &db.0,
        &skill_id,
        employee_id.as_deref(),
        summary.as_deref().unwrap_or("UI delete skill"),
        confirm,
    )
    .await
}

#[tauri::command]
pub async fn pin_skill_os(
    skill_id: String,
    pinned: bool,
    db: State<'_, DbState>,
) -> Result<(), String> {
    crate::agent::runtime::runtime_io::set_skill_os_pinned_with_pool(&db.0, &skill_id, pinned).await
}

#[tauri::command]
pub async fn get_skill_runtime_environment_status(
    skill_id: String,
    db: State<'_, DbState>,
) -> Result<SkillRuntimeEnvironmentStatus, String> {
    runtime_status_service::get_skill_runtime_environment_status_with_pool(&db.0, &skill_id).await
}

async fn ensure_skill_can_be_deleted(
    pool: &sqlx::SqlitePool,
    skill_id: &str,
) -> Result<(), String> {
    let source_type = sqlx::query_scalar::<_, String>(
        "SELECT COALESCE(source_type, 'encrypted') FROM installed_skills WHERE id = ?",
    )
    .bind(skill_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("Skill 不存在 (skill_id={skill_id})"))?;

    let policy =
        crate::agent::runtime::runtime_io::skill_source_policy::resolve_skill_source_policy(
            &source_type,
        );
    if !policy.can_delete_installed_row {
        return Err(format!(
            "Skill source '{}' is not deletable by this runtime",
            source_type
        ));
    }
    Ok(())
}

#[tauri::command]
pub async fn delete_skill(skill_id: String, db: State<'_, DbState>) -> Result<(), String> {
    ensure_skill_can_be_deleted(&db.0, &skill_id).await?;
    sqlx::query("DELETE FROM installed_skills WHERE id = ?")
        .bind(&skill_id)
        .execute(&db.0)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::ensure_skill_can_be_deleted;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_pool() -> sqlx::SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("create sqlite memory pool");

        sqlx::query(
            "CREATE TABLE installed_skills (
                id TEXT PRIMARY KEY,
                manifest TEXT NOT NULL,
                installed_at TEXT NOT NULL,
                username TEXT NOT NULL,
                pack_path TEXT NOT NULL DEFAULT '',
                source_type TEXT NOT NULL DEFAULT 'encrypted'
            )",
        )
        .execute(&pool)
        .await
        .expect("create installed_skills");

        pool
    }

    #[tokio::test]
    async fn delete_guard_blocks_unknown_source_type() {
        let pool = setup_pool().await;

        sqlx::query(
            "INSERT INTO installed_skills (id, manifest, installed_at, username, pack_path, source_type)
             VALUES ('future-skill', '{}', '2026-05-06T00:00:00Z', '', '', 'future-source')",
        )
        .execute(&pool)
        .await
        .expect("seed skill");

        let err = ensure_skill_can_be_deleted(&pool, "future-skill")
            .await
            .expect_err("unknown source should be blocked");
        assert!(err.contains("not deletable"));
    }

    #[tokio::test]
    async fn delete_guard_allows_skillpack_installed_row_without_mutating_pack_content() {
        let pool = setup_pool().await;

        sqlx::query(
            "INSERT INTO installed_skills (id, manifest, installed_at, username, pack_path, source_type)
             VALUES ('encrypted-skill', '{}', '2026-05-06T00:00:00Z', 'alice', 'D:/packs/a.skillpack', 'encrypted')",
        )
        .execute(&pool)
        .await
        .expect("seed skill");

        ensure_skill_can_be_deleted(&pool, "encrypted-skill")
            .await
            .expect("skillpack installed row deletion is explicit user uninstall");
    }
}
