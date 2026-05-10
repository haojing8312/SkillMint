use chrono::Utc;
use serde::Serialize;
use skillpack_rs::SkillManifest;
use sqlx::SqlitePool;
use std::collections::BTreeSet;
use std::path::PathBuf;

use super::skill_source_policy::{SkillSourceKind, resolve_skill_source_policy};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkillOsSourceProjection {
    pub raw_source_type: String,
    pub canonical: String,
    pub immutable_content: bool,
    pub directory_backed: bool,
    pub requires_unpack_for_view: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkillOsCapabilities {
    pub can_list: bool,
    pub can_view: bool,
    pub can_patch: bool,
    pub can_archive: bool,
    pub can_reset: bool,
    pub can_agent_delete: bool,
    pub can_user_uninstall: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct SkillOsToolsetPolicy {
    pub requires_toolsets: Vec<String>,
    pub optional_toolsets: Vec<String>,
    pub denied_toolsets: Vec<String>,
    pub unknown_toolsets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct SkillOsUsageTelemetry {
    pub view_count: i64,
    pub use_count: i64,
    pub patch_count: i64,
    pub last_viewed_at: String,
    pub last_used_at: String,
    pub last_patched_at: String,
    pub pinned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkillOsIndexEntry {
    pub skill_id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub tags: Vec<String>,
    pub source: SkillOsSourceProjection,
    pub capabilities: SkillOsCapabilities,
    pub toolset_policy: SkillOsToolsetPolicy,
    pub lifecycle_state: String,
    pub usage: SkillOsUsageTelemetry,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkillOsView {
    pub entry: SkillOsIndexEntry,
    pub content: String,
    pub read_only: bool,
    pub derived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkillOsVersionEntry {
    pub version_id: String,
    pub skill_id: String,
    pub source_type: String,
    pub action: String,
    pub summary: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkillOsVersionView {
    pub metadata: SkillOsVersionEntry,
    pub content: String,
}

async fn installed_skills_has_column(pool: &SqlitePool, column_name: &str) -> Result<bool, String> {
    let rows: Vec<String> =
        sqlx::query_scalar("SELECT name FROM pragma_table_info('installed_skills')")
            .fetch_all(pool)
            .await
            .map_err(|e| format!("读取 installed_skills schema 失败: {e}"))?;
    Ok(rows.iter().any(|name| name == column_name))
}

fn entry_from_manifest(manifest: SkillManifest, source_type: String) -> SkillOsIndexEntry {
    let policy = resolve_skill_source_policy(&source_type);
    let immutable = policy.immutable_content;
    let directory_backed = policy.directory_backed;
    SkillOsIndexEntry {
        skill_id: manifest.id,
        name: manifest.name,
        description: manifest.description,
        version: manifest.version,
        tags: manifest.tags,
        source: SkillOsSourceProjection {
            raw_source_type: source_type,
            canonical: policy.canonical_label.to_string(),
            immutable_content: immutable,
            directory_backed,
            requires_unpack_for_view: immutable && !directory_backed,
        },
        capabilities: SkillOsCapabilities {
            can_list: true,
            can_view: true,
            can_patch: !immutable && directory_backed,
            can_archive: !immutable && directory_backed,
            can_reset: !immutable && directory_backed,
            can_agent_delete: !immutable && directory_backed,
            can_user_uninstall: policy.can_delete_installed_row,
        },
        toolset_policy: SkillOsToolsetPolicy::default(),
        lifecycle_state: "active".to_string(),
        usage: SkillOsUsageTelemetry::default(),
    }
}

const KNOWN_SKILL_TOOLSETS: &[&str] = &[
    "core", "memory", "skills", "web", "browser", "im", "desktop", "media", "mcp",
];

fn normalize_skill_toolset_policy_list(
    raw: Option<Vec<String>>,
    unknown_toolsets: &mut BTreeSet<String>,
) -> Vec<String> {
    let known = KNOWN_SKILL_TOOLSETS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let mut out = BTreeSet::new();
    for value in raw.unwrap_or_default() {
        let normalized = value.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            continue;
        }
        if known.contains(normalized.as_str()) {
            out.insert(normalized);
        } else {
            unknown_toolsets.insert(normalized);
        }
    }
    out.into_iter().collect()
}

fn parse_skill_toolset_policy(content: &str) -> SkillOsToolsetPolicy {
    let config = crate::agent::skill_config::SkillConfig::parse(content);
    let mut unknown_toolsets = BTreeSet::new();
    let requires_toolsets =
        normalize_skill_toolset_policy_list(config.requires_toolsets, &mut unknown_toolsets);
    let optional_toolsets =
        normalize_skill_toolset_policy_list(config.optional_toolsets, &mut unknown_toolsets);
    let denied_toolsets =
        normalize_skill_toolset_policy_list(config.denied_toolsets, &mut unknown_toolsets);
    SkillOsToolsetPolicy {
        requires_toolsets,
        optional_toolsets,
        denied_toolsets,
        unknown_toolsets: unknown_toolsets.into_iter().collect(),
    }
}

pub async fn ensure_skill_os_versions_schema_with_pool(pool: &SqlitePool) -> Result<(), String> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS skill_versions (
            version_id TEXT PRIMARY KEY,
            skill_id TEXT NOT NULL,
            source_type TEXT NOT NULL DEFAULT '',
            action TEXT NOT NULL,
            summary TEXT NOT NULL DEFAULT '',
            content TEXT NOT NULL,
            metadata_json TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| format!("创建 Skill OS versions 表失败: {e}"))?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_skill_versions_skill_created
         ON skill_versions(skill_id, created_at DESC)",
    )
    .execute(pool)
    .await
    .map_err(|e| format!("创建 Skill OS versions 索引失败: {e}"))?;

    Ok(())
}

pub async fn ensure_skill_os_lifecycle_schema_with_pool(pool: &SqlitePool) -> Result<(), String> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS skill_lifecycle (
            skill_id TEXT PRIMARY KEY,
            state TEXT NOT NULL DEFAULT 'active',
            active_pack_path TEXT NOT NULL DEFAULT '',
            archived_pack_path TEXT NOT NULL DEFAULT '',
            updated_at TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| format!("创建 Skill OS lifecycle 表失败: {e}"))?;

    for (column, definition) in [
        ("view_count", "INTEGER NOT NULL DEFAULT 0"),
        ("use_count", "INTEGER NOT NULL DEFAULT 0"),
        ("patch_count", "INTEGER NOT NULL DEFAULT 0"),
        ("last_viewed_at", "TEXT NOT NULL DEFAULT ''"),
        ("last_used_at", "TEXT NOT NULL DEFAULT ''"),
        ("last_patched_at", "TEXT NOT NULL DEFAULT ''"),
        ("pinned", "INTEGER NOT NULL DEFAULT 0"),
    ] {
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM pragma_table_info('skill_lifecycle') WHERE name = ?",
        )
        .bind(column)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("读取 Skill lifecycle schema 失败: {e}"))?;
        if exists == 0 {
            sqlx::query(&format!(
                "ALTER TABLE skill_lifecycle ADD COLUMN {column} {definition}"
            ))
            .execute(pool)
            .await
            .map_err(|e| format!("扩展 Skill lifecycle 字段失败: {e}"))?;
        }
    }

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_skill_lifecycle_state
         ON skill_lifecycle(state)",
    )
    .execute(pool)
    .await
    .map_err(|e| format!("创建 Skill OS lifecycle 索引失败: {e}"))?;

    Ok(())
}

fn apply_lifecycle_projection(
    entry: &mut SkillOsIndexEntry,
    state: String,
    view_count: i64,
    use_count: i64,
    patch_count: i64,
    last_viewed_at: String,
    last_used_at: String,
    last_patched_at: String,
    pinned: i64,
) {
    entry.lifecycle_state = if state.trim().is_empty() {
        "active".to_string()
    } else {
        state
    };
    entry.usage = SkillOsUsageTelemetry {
        view_count,
        use_count,
        patch_count,
        last_viewed_at,
        last_used_at,
        last_patched_at,
        pinned: pinned != 0,
    };
}

fn make_skill_version_id() -> String {
    format!(
        "skv_{}_{}",
        Utc::now().timestamp_millis(),
        uuid::Uuid::new_v4().simple()
    )
}

fn sanitize_agent_skill_slug(raw: &str) -> String {
    let mut out = String::new();
    let mut last_sep = false;
    for ch in raw.trim().chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_sep = false;
        } else if !last_sep {
            out.push('-');
            last_sep = true;
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "skill".to_string()
    } else {
        trimmed
    }
}

fn sanitize_skill_version_id(version_id: &str) -> Result<String, String> {
    let trimmed = version_id.trim();
    if trimmed.is_empty() {
        return Err("version_id 不能为空".to_string());
    }
    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
    {
        return Err("version_id 包含非法字符".to_string());
    }
    Ok(trimmed.to_string())
}

async fn load_installed_skill_for_mutation(
    pool: &SqlitePool,
    skill_id: &str,
) -> Result<(SkillManifest, String, String, String), String> {
    let row = sqlx::query_as::<_, (String, String, String)>(
        "SELECT manifest, pack_path, COALESCE(source_type, 'encrypted')
         FROM installed_skills
         WHERE id = ?",
    )
    .bind(skill_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("读取 Skill 失败: {e}"))?;
    let Some((manifest_json, pack_path, source_type)) = row else {
        return Err(format!("Skill 不存在: {skill_id}"));
    };
    let manifest = serde_json::from_str::<SkillManifest>(&manifest_json)
        .map_err(|e| format!("解析 skill manifest 失败: {e}"))?;
    Ok((manifest, pack_path, source_type, manifest_json))
}

fn mutable_skill_markdown_path(pack_path: &str, source_type: &str) -> Result<PathBuf, String> {
    let policy = resolve_skill_source_policy(source_type);
    if policy.kind == SkillSourceKind::Skillpack || policy.immutable_content {
        return Err(format!("Skill source '{}' is not mutable", source_type));
    }
    if !policy.directory_backed {
        return Err(format!(
            "Skill source '{}' is not directory-backed",
            source_type
        ));
    }
    let skill_dir = PathBuf::from(pack_path);
    if !skill_dir.exists() {
        return Err(format!("Skill 目录不存在: {}", skill_dir.to_string_lossy()));
    }
    Ok(skill_dir.join("SKILL.md"))
}

async fn record_skill_version_snapshot_with_pool(
    pool: &SqlitePool,
    skill_id: &str,
    source_type: &str,
    action: &str,
    summary: &str,
    content: &str,
) -> Result<SkillOsVersionEntry, String> {
    ensure_skill_os_versions_schema_with_pool(pool).await?;
    let version_id = make_skill_version_id();
    let created_at = Utc::now().to_rfc3339();
    let metadata = SkillOsVersionEntry {
        version_id: version_id.clone(),
        skill_id: skill_id.to_string(),
        source_type: source_type.to_string(),
        action: action.to_string(),
        summary: summary.to_string(),
        created_at: created_at.clone(),
    };
    let metadata_json = serde_json::to_string(&metadata)
        .map_err(|e| format!("序列化 skill version metadata 失败: {e}"))?;
    sqlx::query(
        "INSERT INTO skill_versions (
            version_id, skill_id, source_type, action, summary, content, metadata_json, created_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&version_id)
    .bind(skill_id)
    .bind(source_type)
    .bind(action)
    .bind(summary)
    .bind(content)
    .bind(metadata_json)
    .bind(&created_at)
    .execute(pool)
    .await
    .map_err(|e| format!("写入 skill version 失败: {e}"))?;
    Ok(metadata)
}

fn update_manifest_from_content(mut manifest: SkillManifest, content: &str) -> SkillManifest {
    let config = crate::agent::skill_config::SkillConfig::parse(content);
    if let Some(name) = config.name.filter(|name| !name.trim().is_empty()) {
        manifest.name = name;
    }
    if let Some(description) = config.description {
        manifest.description = description;
    }
    if let Some(model) = config.model {
        manifest.recommended_model = model;
    }
    let tags = crate::commands::packaging::parse_skill_tags(content);
    if !tags.is_empty() {
        manifest.tags = tags;
    }
    manifest
}

async fn write_skill_content_and_refresh_manifest_with_pool(
    pool: &SqlitePool,
    skill_id: &str,
    manifest: SkillManifest,
    skill_md_path: &PathBuf,
    content: &str,
) -> Result<SkillManifest, String> {
    std::fs::write(skill_md_path, content).map_err(|e| format!("写入 SKILL.md 失败: {e}"))?;
    let next_manifest = update_manifest_from_content(manifest, content);
    let manifest_json = serde_json::to_string(&next_manifest)
        .map_err(|e| format!("序列化 skill manifest 失败: {e}"))?;
    sqlx::query("UPDATE installed_skills SET manifest = ? WHERE id = ?")
        .bind(manifest_json)
        .bind(skill_id)
        .execute(pool)
        .await
        .map_err(|e| format!("更新 skill manifest 失败: {e}"))?;
    Ok(next_manifest)
}

pub async fn list_skill_os_index_with_pool(
    pool: &SqlitePool,
) -> Result<Vec<SkillOsIndexEntry>, String> {
    ensure_skill_os_lifecycle_schema_with_pool(pool).await?;
    let has_source_type = installed_skills_has_column(pool, "source_type").await?;
    let source_select = if has_source_type {
        "COALESCE(s.source_type, 'encrypted')"
    } else {
        "'encrypted'"
    };
    let query = format!(
        "SELECT manifest, {source_select},
                COALESCE(l.state, 'active'),
                COALESCE(l.view_count, 0),
                COALESCE(l.use_count, 0),
                COALESCE(l.patch_count, 0),
                COALESCE(l.last_viewed_at, ''),
                COALESCE(l.last_used_at, ''),
                COALESCE(l.last_patched_at, ''),
                COALESCE(l.pinned, 0)
         FROM installed_skills s
         LEFT JOIN skill_lifecycle l ON l.skill_id = s.id
         WHERE COALESCE(l.state, 'active') <> 'archived'
         ORDER BY s.installed_at DESC"
    );
    let rows = sqlx::query_as::<
        _,
        (
            String,
            String,
            String,
            i64,
            i64,
            i64,
            String,
            String,
            String,
            i64,
        ),
    >(&query)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("读取 Skill OS index 失败: {e}"))?;

    rows.into_iter()
        .map(
            |(
                manifest_json,
                source_type,
                state,
                view_count,
                use_count,
                patch_count,
                last_viewed_at,
                last_used_at,
                last_patched_at,
                pinned,
            )| {
                let manifest = serde_json::from_str::<SkillManifest>(&manifest_json)
                    .map_err(|e| format!("解析 skill manifest 失败: {e}"))?;
                let mut entry = entry_from_manifest(manifest, source_type);
                apply_lifecycle_projection(
                    &mut entry,
                    state,
                    view_count,
                    use_count,
                    patch_count,
                    last_viewed_at,
                    last_used_at,
                    last_patched_at,
                    pinned,
                );
                Ok(entry)
            },
        )
        .collect()
}

pub async fn view_skill_os_entry_with_pool(
    pool: &SqlitePool,
    skill_id: &str,
) -> Result<Option<SkillOsView>, String> {
    let skill_id = skill_id.trim();
    if skill_id.is_empty() {
        return Ok(None);
    }
    ensure_skill_os_lifecycle_schema_with_pool(pool).await?;
    let has_source_type = installed_skills_has_column(pool, "source_type").await?;
    let source_select = if has_source_type {
        "COALESCE(s.source_type, 'encrypted')"
    } else {
        "'encrypted'"
    };
    let query = format!(
        "SELECT s.manifest, s.pack_path, {source_select},
                COALESCE(l.state, 'active'),
                COALESCE(l.view_count, 0),
                COALESCE(l.use_count, 0),
                COALESCE(l.patch_count, 0),
                COALESCE(l.last_viewed_at, ''),
                COALESCE(l.last_used_at, ''),
                COALESCE(l.last_patched_at, ''),
                COALESCE(l.pinned, 0)
         FROM installed_skills s
         LEFT JOIN skill_lifecycle l ON l.skill_id = s.id
         WHERE s.id = ?"
    );
    let row = sqlx::query_as::<
        _,
        (
            String,
            String,
            String,
            String,
            i64,
            i64,
            i64,
            String,
            String,
            String,
            i64,
        ),
    >(&query)
    .bind(skill_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("读取 Skill OS view 失败: {e}"))?;
    let Some((
        manifest_json,
        pack_path,
        source_type,
        state,
        view_count,
        use_count,
        patch_count,
        last_viewed_at,
        last_used_at,
        last_patched_at,
        pinned,
    )) = row
    else {
        return Ok(None);
    };
    let manifest = serde_json::from_str::<SkillManifest>(&manifest_json)
        .map_err(|e| format!("解析 skill manifest 失败: {e}"))?;
    let mut entry = entry_from_manifest(manifest, source_type);
    apply_lifecycle_projection(
        &mut entry,
        state,
        view_count,
        use_count,
        patch_count,
        last_viewed_at,
        last_used_at,
        last_patched_at,
        pinned,
    );

    let content = if entry.source.directory_backed {
        let skill_md = PathBuf::from(pack_path).join("SKILL.md");
        std::fs::read_to_string(skill_md).unwrap_or_default()
    } else {
        String::new()
    };
    if !content.trim().is_empty() {
        entry.toolset_policy = parse_skill_toolset_policy(&content);
    }
    let read_only = entry.source.immutable_content;
    let derived = entry.source.requires_unpack_for_view;

    Ok(Some(SkillOsView {
        entry,
        content,
        read_only,
        derived,
    }))
}

fn move_agent_created_skill_dir(
    skill_id: &str,
    pack_path: &str,
    source_type: &str,
    target_state: &str,
) -> Result<String, String> {
    let policy = resolve_skill_source_policy(source_type);
    if policy.kind != SkillSourceKind::AgentCreated {
        return Ok(pack_path.to_string());
    }

    let current = PathBuf::from(pack_path);
    let parent = current
        .parent()
        .ok_or_else(|| format!("Skill 路径缺少父目录: {}", current.to_string_lossy()))?;
    let skills_dir = parent.parent().ok_or_else(|| {
        format!(
            "Agent-created Skill 路径不在 profile skills 目录下: {}",
            current.to_string_lossy()
        )
    })?;
    let target_bucket = match target_state {
        "archived" => "archive",
        "active" => "active",
        _ => return Ok(pack_path.to_string()),
    };
    let target = skills_dir.join(target_bucket).join(skill_id);
    if current == target {
        return Ok(target.to_string_lossy().to_string());
    }
    if target.exists() {
        return Err(format!(
            "目标 Skill 目录已存在: {}",
            target.to_string_lossy()
        ));
    }
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建 Skill 生命周期目录失败: {e}"))?;
    }
    std::fs::rename(&current, &target).map_err(|e| format!("移动 Skill 目录失败: {e}"))?;
    Ok(target.to_string_lossy().to_string())
}

async fn upsert_skill_lifecycle_with_pool(
    pool: &SqlitePool,
    skill_id: &str,
    state: &str,
    active_pack_path: &str,
    archived_pack_path: &str,
) -> Result<(), String> {
    ensure_skill_os_lifecycle_schema_with_pool(pool).await?;
    sqlx::query(
        "INSERT INTO skill_lifecycle (
            skill_id, state, active_pack_path, archived_pack_path, updated_at
         ) VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(skill_id) DO UPDATE SET
            state = excluded.state,
            active_pack_path = excluded.active_pack_path,
            archived_pack_path = excluded.archived_pack_path,
            updated_at = excluded.updated_at",
    )
    .bind(skill_id)
    .bind(state)
    .bind(active_pack_path)
    .bind(archived_pack_path)
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await
    .map_err(|e| format!("写入 Skill lifecycle 失败: {e}"))?;
    Ok(())
}

async fn skill_lifecycle_state_with_pool(
    pool: &SqlitePool,
    skill_id: &str,
) -> Result<String, String> {
    ensure_skill_os_lifecycle_schema_with_pool(pool).await?;
    sqlx::query_scalar::<_, String>(
        "SELECT COALESCE(state, 'active') FROM skill_lifecycle WHERE skill_id = ?",
    )
    .bind(skill_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("读取 Skill lifecycle 失败: {e}"))
    .map(|state| state.unwrap_or_else(|| "active".to_string()))
}

pub async fn record_skill_os_usage_with_pool(
    pool: &SqlitePool,
    skill_id: &str,
    action: &str,
) -> Result<(), String> {
    ensure_skill_os_lifecycle_schema_with_pool(pool).await?;
    let skill_id = skill_id.trim();
    if skill_id.is_empty() {
        return Err("skill_id 不能为空".to_string());
    }
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT OR IGNORE INTO skill_lifecycle (skill_id, state, updated_at)
         VALUES (?, 'active', ?)",
    )
    .bind(skill_id)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| format!("初始化 Skill usage 失败: {e}"))?;

    let (count_column, time_column) = match action {
        "view" | "viewed" => ("view_count", "last_viewed_at"),
        "use" | "used" => ("use_count", "last_used_at"),
        "patch" | "patched" => ("patch_count", "last_patched_at"),
        other => return Err(format!("未知 Skill usage action: {other}")),
    };
    let query = format!(
        "UPDATE skill_lifecycle
         SET {count_column} = COALESCE({count_column}, 0) + 1,
             {time_column} = ?,
             updated_at = ?
         WHERE skill_id = ?"
    );
    sqlx::query(&query)
        .bind(&now)
        .bind(&now)
        .bind(skill_id)
        .execute(pool)
        .await
        .map_err(|e| format!("写入 Skill usage 失败: {e}"))?;
    Ok(())
}

pub async fn set_skill_os_pinned_with_pool(
    pool: &SqlitePool,
    skill_id: &str,
    pinned: bool,
) -> Result<(), String> {
    ensure_skill_os_lifecycle_schema_with_pool(pool).await?;
    let skill_id = skill_id.trim();
    if skill_id.is_empty() {
        return Err("skill_id 不能为空".to_string());
    }
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT OR IGNORE INTO skill_lifecycle (skill_id, state, updated_at)
         VALUES (?, 'active', ?)",
    )
    .bind(skill_id)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| format!("初始化 Skill pin 状态失败: {e}"))?;
    sqlx::query(
        "UPDATE skill_lifecycle
         SET pinned = ?, updated_at = ?
         WHERE skill_id = ?",
    )
    .bind(if pinned { 1 } else { 0 })
    .bind(&now)
    .bind(skill_id)
    .execute(pool)
    .await
    .map_err(|e| format!("写入 Skill pin 状态失败: {e}"))?;
    Ok(())
}

pub async fn mark_skill_os_stale_with_pool(
    pool: &SqlitePool,
    skill_id: &str,
) -> Result<bool, String> {
    ensure_skill_os_lifecycle_schema_with_pool(pool).await?;
    let skill_id = skill_id.trim();
    if skill_id.is_empty() {
        return Err("skill_id 不能为空".to_string());
    }
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT OR IGNORE INTO skill_lifecycle (skill_id, state, updated_at)
         VALUES (?, 'active', ?)",
    )
    .bind(skill_id)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| format!("初始化 Skill stale 状态失败: {e}"))?;
    let result = sqlx::query(
        "UPDATE skill_lifecycle
         SET state = 'stale', updated_at = ?
         WHERE skill_id = ?
           AND COALESCE(state, 'active') = 'active'
           AND COALESCE(pinned, 0) = 0",
    )
    .bind(&now)
    .bind(skill_id)
    .execute(pool)
    .await
    .map_err(|e| format!("标记 Skill stale 失败: {e}"))?;
    Ok(result.rows_affected() > 0)
}

pub async fn restore_stale_skill_os_with_pool(
    pool: &SqlitePool,
    skill_id: &str,
) -> Result<bool, String> {
    ensure_skill_os_lifecycle_schema_with_pool(pool).await?;
    let skill_id = skill_id.trim();
    if skill_id.is_empty() {
        return Err("skill_id 不能为空".to_string());
    }
    let installed_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM installed_skills WHERE id = ?")
            .bind(skill_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("检查 Skill 安装状态失败: {e}"))?;
    if installed_count == 0 {
        return Err(format!("Skill 不存在: {skill_id}"));
    }
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "UPDATE skill_lifecycle
         SET state = 'active', updated_at = ?
         WHERE skill_id = ?
           AND COALESCE(state, 'active') = 'stale'",
    )
    .bind(&now)
    .bind(skill_id)
    .execute(pool)
    .await
    .map_err(|e| format!("恢复 stale Skill 失败: {e}"))?;
    Ok(result.rows_affected() > 0)
}

pub async fn archive_skill_os_entry_with_pool(
    pool: &SqlitePool,
    skill_id: &str,
    summary: &str,
) -> Result<SkillOsView, String> {
    let skill_id = skill_id.trim();
    if skill_id.is_empty() {
        return Err("skill_id 不能为空".to_string());
    }
    if skill_lifecycle_state_with_pool(pool, skill_id).await? == "archived" {
        return Err(format!("Skill 已归档: {skill_id}"));
    }

    let (_manifest, pack_path, source_type, _) =
        load_installed_skill_for_mutation(pool, skill_id).await?;
    let skill_md_path = mutable_skill_markdown_path(&pack_path, &source_type)?;
    let current_content = std::fs::read_to_string(&skill_md_path).unwrap_or_default();
    let archived_pack_path =
        move_agent_created_skill_dir(skill_id, &pack_path, &source_type, "archived")?;
    sqlx::query("UPDATE installed_skills SET pack_path = ? WHERE id = ?")
        .bind(&archived_pack_path)
        .bind(skill_id)
        .execute(pool)
        .await
        .map_err(|e| format!("更新 Skill 归档路径失败: {e}"))?;
    upsert_skill_lifecycle_with_pool(pool, skill_id, "archived", &pack_path, &archived_pack_path)
        .await?;
    record_skill_version_snapshot_with_pool(
        pool,
        skill_id,
        &source_type,
        "archive",
        summary,
        &current_content,
    )
    .await?;

    view_skill_os_entry_with_pool(pool, skill_id)
        .await?
        .ok_or_else(|| format!("Skill 不存在: {skill_id}"))
}

pub async fn restore_skill_os_entry_with_pool(
    pool: &SqlitePool,
    skill_id: &str,
    summary: &str,
) -> Result<SkillOsView, String> {
    let skill_id = skill_id.trim();
    if skill_id.is_empty() {
        return Err("skill_id 不能为空".to_string());
    }
    if skill_lifecycle_state_with_pool(pool, skill_id).await? != "archived" {
        return Err(format!("Skill 未归档: {skill_id}"));
    }

    let (_manifest, pack_path, source_type, _) =
        load_installed_skill_for_mutation(pool, skill_id).await?;
    let skill_md_path = mutable_skill_markdown_path(&pack_path, &source_type)?;
    let current_content = std::fs::read_to_string(&skill_md_path).unwrap_or_default();
    let active_pack_path =
        move_agent_created_skill_dir(skill_id, &pack_path, &source_type, "active")?;
    sqlx::query("UPDATE installed_skills SET pack_path = ? WHERE id = ?")
        .bind(&active_pack_path)
        .bind(skill_id)
        .execute(pool)
        .await
        .map_err(|e| format!("更新 Skill 恢复路径失败: {e}"))?;
    upsert_skill_lifecycle_with_pool(pool, skill_id, "active", &active_pack_path, &pack_path)
        .await?;
    record_skill_version_snapshot_with_pool(
        pool,
        skill_id,
        &source_type,
        "restore",
        summary,
        &current_content,
    )
    .await?;

    view_skill_os_entry_with_pool(pool, skill_id)
        .await?
        .ok_or_else(|| format!("Skill 不存在: {skill_id}"))
}

pub async fn delete_skill_os_entry_with_pool(
    pool: &SqlitePool,
    skill_id: &str,
    summary: &str,
) -> Result<(SkillOsView, SkillOsVersionEntry, String, bool), String> {
    let skill_id = skill_id.trim();
    if skill_id.is_empty() {
        return Err("skill_id 不能为空".to_string());
    }
    let (manifest, pack_path, source_type, _) =
        load_installed_skill_for_mutation(pool, skill_id).await?;
    let skill_md_path = mutable_skill_markdown_path(&pack_path, &source_type)?;
    let current_content = std::fs::read_to_string(&skill_md_path).unwrap_or_default();
    let view = SkillOsView {
        entry: entry_from_manifest(manifest, source_type.clone()),
        content: current_content.clone(),
        read_only: false,
        derived: false,
    };
    let version = record_skill_version_snapshot_with_pool(
        pool,
        skill_id,
        &source_type,
        "delete",
        summary,
        &current_content,
    )
    .await?;

    let policy = resolve_skill_source_policy(&source_type);
    let removed_files = policy.kind == SkillSourceKind::AgentCreated;
    if removed_files {
        let skill_dir = PathBuf::from(&pack_path);
        if skill_dir.exists() {
            std::fs::remove_dir_all(&skill_dir)
                .map_err(|e| format!("删除 agent-created Skill 目录失败: {e}"))?;
        }
    }

    sqlx::query("DELETE FROM installed_skills WHERE id = ?")
        .bind(skill_id)
        .execute(pool)
        .await
        .map_err(|e| format!("删除 Skill 安装记录失败: {e}"))?;
    ensure_skill_os_lifecycle_schema_with_pool(pool).await?;
    sqlx::query("DELETE FROM skill_lifecycle WHERE skill_id = ?")
        .bind(skill_id)
        .execute(pool)
        .await
        .map_err(|e| format!("删除 Skill lifecycle 失败: {e}"))?;

    Ok((view, version, pack_path, removed_files))
}

pub async fn patch_skill_os_entry_with_pool(
    pool: &SqlitePool,
    skill_id: &str,
    content: &str,
    summary: &str,
) -> Result<SkillOsView, String> {
    let skill_id = skill_id.trim();
    if skill_id.is_empty() {
        return Err("skill_id 不能为空".to_string());
    }
    if content.trim().is_empty() {
        return Err("content 不能为空".to_string());
    }
    let (manifest, pack_path, source_type, _) =
        load_installed_skill_for_mutation(pool, skill_id).await?;
    let skill_md_path = mutable_skill_markdown_path(&pack_path, &source_type)?;
    let previous_content = std::fs::read_to_string(&skill_md_path).unwrap_or_default();
    record_skill_version_snapshot_with_pool(
        pool,
        skill_id,
        &source_type,
        "patch",
        summary,
        &previous_content,
    )
    .await?;

    write_skill_content_and_refresh_manifest_with_pool(
        pool,
        skill_id,
        manifest,
        &skill_md_path,
        content,
    )
    .await?;
    record_skill_os_usage_with_pool(pool, skill_id, "patch").await?;
    view_skill_os_entry_with_pool(pool, skill_id)
        .await?
        .ok_or_else(|| format!("Skill 不存在: {skill_id}"))
}

pub async fn create_agent_skill_os_entry_with_pool(
    pool: &SqlitePool,
    profile_id: &str,
    name: &str,
    description: &str,
    content: &str,
    summary: &str,
) -> Result<SkillOsView, String> {
    let profile_id = profile_id.trim();
    if profile_id.is_empty() {
        return Err("skill_create 需要当前 session 绑定 profile_id".to_string());
    }
    let name = name.trim();
    if name.is_empty() {
        return Err("skill_create 需要 name".to_string());
    }
    if content.trim().is_empty() {
        return Err("skill_create 需要 content".to_string());
    }

    let profile_home = sqlx::query_scalar::<_, String>(
        "SELECT COALESCE(profile_home, '') FROM agent_profiles WHERE id = ?",
    )
    .bind(profile_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("读取 profile home 失败: {e}"))?
    .unwrap_or_default();
    if profile_home.trim().is_empty() {
        return Err(format!("profile {} 缺少 profile_home", profile_id));
    }

    let slug = sanitize_agent_skill_slug(name);
    let skill_id = format!(
        "agent-{slug}-{}",
        uuid::Uuid::new_v4().simple().to_string()[..8].to_string()
    );
    let skill_dir = PathBuf::from(profile_home)
        .join("skills")
        .join("active")
        .join(&skill_id);
    std::fs::create_dir_all(&skill_dir).map_err(|e| format!("创建 agent skill 目录失败: {e}"))?;
    let skill_md_path = skill_dir.join("SKILL.md");
    std::fs::write(&skill_md_path, content).map_err(|e| format!("写入 SKILL.md 失败: {e}"))?;

    let config = crate::agent::skill_config::SkillConfig::parse(content);
    let mut tags = crate::commands::packaging::parse_skill_tags(content);
    if !tags.iter().any(|tag| tag == "agent-created") {
        tags.push("agent-created".to_string());
    }
    let manifest = SkillManifest {
        id: skill_id.clone(),
        name: config
            .name
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| name.to_string()),
        description: config
            .description
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| description.trim().to_string()),
        version: "agent-created".to_string(),
        author: "WorkClaw Agent".to_string(),
        recommended_model: config.model.unwrap_or_default(),
        tags,
        created_at: Utc::now(),
        username_hint: None,
        encrypted_verify: String::new(),
    };
    let manifest_json =
        serde_json::to_string(&manifest).map_err(|e| format!("序列化 skill manifest 失败: {e}"))?;
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO installed_skills (id, manifest, installed_at, username, pack_path, source_type)
         VALUES (?, ?, ?, '', ?, 'agent_created')",
    )
    .bind(&skill_id)
    .bind(manifest_json)
    .bind(now)
    .bind(skill_dir.to_string_lossy().to_string())
    .execute(pool)
    .await
    .map_err(|e| format!("登记 agent skill 失败: {e}"))?;

    record_skill_version_snapshot_with_pool(
        pool,
        &skill_id,
        "agent_created",
        "create",
        summary,
        content,
    )
    .await?;

    view_skill_os_entry_with_pool(pool, &skill_id)
        .await?
        .ok_or_else(|| format!("Skill 不存在: {skill_id}"))
}

pub async fn list_skill_os_versions_with_pool(
    pool: &SqlitePool,
    skill_id: &str,
    limit: i64,
) -> Result<Vec<SkillOsVersionEntry>, String> {
    ensure_skill_os_versions_schema_with_pool(pool).await?;
    let skill_id = skill_id.trim();
    if skill_id.is_empty() {
        return Err("skill_id 不能为空".to_string());
    }
    let limit = limit.clamp(1, 50);
    sqlx::query_as::<_, (String, String, String, String, String, String)>(
        "SELECT version_id, skill_id, source_type, action, summary, created_at
         FROM skill_versions
         WHERE skill_id = ?
         ORDER BY created_at DESC, version_id DESC
         LIMIT ?",
    )
    .bind(skill_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("读取 skill versions 失败: {e}"))
    .map(|rows| {
        rows.into_iter()
            .map(
                |(version_id, skill_id, source_type, action, summary, created_at)| {
                    SkillOsVersionEntry {
                        version_id,
                        skill_id,
                        source_type,
                        action,
                        summary,
                        created_at,
                    }
                },
            )
            .collect()
    })
}

pub async fn view_skill_os_version_with_pool(
    pool: &SqlitePool,
    skill_id: &str,
    version_id: &str,
) -> Result<Option<SkillOsVersionView>, String> {
    ensure_skill_os_versions_schema_with_pool(pool).await?;
    let version_id = sanitize_skill_version_id(version_id)?;
    let row = sqlx::query_as::<_, (String, String, String, String, String, String, String)>(
        "SELECT version_id, skill_id, source_type, action, summary, created_at, content
         FROM skill_versions
         WHERE skill_id = ? AND version_id = ?",
    )
    .bind(skill_id.trim())
    .bind(&version_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("读取 skill version 失败: {e}"))?;

    Ok(row.map(
        |(version_id, skill_id, source_type, action, summary, created_at, content)| {
            SkillOsVersionView {
                metadata: SkillOsVersionEntry {
                    version_id,
                    skill_id,
                    source_type,
                    action,
                    summary,
                    created_at,
                },
                content,
            }
        },
    ))
}

async fn earliest_skill_os_version_with_pool(
    pool: &SqlitePool,
    skill_id: &str,
) -> Result<Option<SkillOsVersionView>, String> {
    ensure_skill_os_versions_schema_with_pool(pool).await?;
    let row = sqlx::query_as::<_, (String, String, String, String, String, String, String)>(
        "SELECT version_id, skill_id, source_type, action, summary, created_at, content
         FROM skill_versions
         WHERE skill_id = ?
         ORDER BY created_at ASC, version_id ASC
         LIMIT 1",
    )
    .bind(skill_id.trim())
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("读取 skill reset baseline 失败: {e}"))?;

    Ok(row.map(
        |(version_id, skill_id, source_type, action, summary, created_at, content)| {
            SkillOsVersionView {
                metadata: SkillOsVersionEntry {
                    version_id,
                    skill_id,
                    source_type,
                    action,
                    summary,
                    created_at,
                },
                content,
            }
        },
    ))
}

pub async fn rollback_skill_os_entry_with_pool(
    pool: &SqlitePool,
    skill_id: &str,
    version_id: &str,
    summary: &str,
) -> Result<SkillOsView, String> {
    let skill_id = skill_id.trim();
    if skill_id.is_empty() {
        return Err("skill_id 不能为空".to_string());
    }
    let version = view_skill_os_version_with_pool(pool, skill_id, version_id)
        .await?
        .ok_or_else(|| format!("skill version 不存在: {version_id}"))?;
    let (manifest, pack_path, source_type, _) =
        load_installed_skill_for_mutation(pool, skill_id).await?;
    let skill_md_path = mutable_skill_markdown_path(&pack_path, &source_type)?;
    let current_content = std::fs::read_to_string(&skill_md_path).unwrap_or_default();
    record_skill_version_snapshot_with_pool(
        pool,
        skill_id,
        &source_type,
        "rollback",
        summary,
        &current_content,
    )
    .await?;

    write_skill_content_and_refresh_manifest_with_pool(
        pool,
        skill_id,
        manifest,
        &skill_md_path,
        &version.content,
    )
    .await?;
    view_skill_os_entry_with_pool(pool, skill_id)
        .await?
        .ok_or_else(|| format!("Skill 不存在: {skill_id}"))
}

pub async fn reset_skill_os_entry_with_pool(
    pool: &SqlitePool,
    skill_id: &str,
    summary: &str,
) -> Result<(SkillOsView, String), String> {
    let skill_id = skill_id.trim();
    if skill_id.is_empty() {
        return Err("skill_id 不能为空".to_string());
    }
    let baseline = earliest_skill_os_version_with_pool(pool, skill_id)
        .await?
        .ok_or_else(|| format!("Skill 没有可重置的版本基线: {skill_id}"))?;
    let (manifest, pack_path, source_type, _) =
        load_installed_skill_for_mutation(pool, skill_id).await?;
    let skill_md_path = mutable_skill_markdown_path(&pack_path, &source_type)?;
    let current_content = std::fs::read_to_string(&skill_md_path).unwrap_or_default();
    record_skill_version_snapshot_with_pool(
        pool,
        skill_id,
        &source_type,
        "reset",
        summary,
        &current_content,
    )
    .await?;

    write_skill_content_and_refresh_manifest_with_pool(
        pool,
        skill_id,
        manifest,
        &skill_md_path,
        &baseline.content,
    )
    .await?;
    let view = view_skill_os_entry_with_pool(pool, skill_id)
        .await?
        .ok_or_else(|| format!("Skill 不存在: {skill_id}"))?;
    Ok((view, baseline.metadata.version_id))
}
