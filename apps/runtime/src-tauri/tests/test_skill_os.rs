use runtime_lib::agent::runtime::runtime_io::{
    list_skill_os_index_with_pool, view_skill_os_entry_with_pool,
};
use runtime_lib::agent::tools::SkillOsTool;
use runtime_lib::agent::types::{Tool, ToolContext};
use runtime_lib::commands::skills::{
    archive_skill_os_with_pool, delete_skill_os_with_pool, patch_skill_os_with_pool,
    reset_skill_os_with_pool, restore_skill_os_with_pool,
};
use serde_json::json;

fn skill_manifest(id: &str, name: &str, description: &str, tags: &[&str]) -> String {
    json!({
        "id": id,
        "name": name,
        "description": description,
        "version": "1.0.0",
        "author": "WorkClaw",
        "recommended_model": "",
        "tags": tags,
        "created_at": "2026-05-08T00:00:00Z",
        "username_hint": null,
        "encrypted_verify": ""
    })
    .to_string()
}

async fn setup_pool(with_source_type: bool) -> sqlx::SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite::memory:")
        .await
        .expect("create sqlite pool");

    if with_source_type {
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
    } else {
        sqlx::query(
            "CREATE TABLE installed_skills (
                id TEXT PRIMARY KEY,
                manifest TEXT NOT NULL,
                installed_at TEXT NOT NULL,
                username TEXT NOT NULL,
                pack_path TEXT NOT NULL DEFAULT ''
            )",
        )
        .execute(&pool)
        .await
        .expect("create legacy installed_skills");
    }

    sqlx::query(
        "CREATE TABLE sessions (
            id TEXT PRIMARY KEY,
            skill_id TEXT NOT NULL DEFAULT '',
            title TEXT,
            created_at TEXT NOT NULL DEFAULT '',
            model_id TEXT NOT NULL DEFAULT '',
            employee_id TEXT NOT NULL DEFAULT '',
            profile_id TEXT
        )",
    )
    .execute(&pool)
    .await
    .expect("create sessions");

    sqlx::query(
        "CREATE TABLE agent_profiles (
            id TEXT PRIMARY KEY,
            legacy_employee_row_id TEXT NOT NULL DEFAULT '',
            display_name TEXT NOT NULL DEFAULT '',
            route_aliases_json TEXT NOT NULL DEFAULT '[]',
            profile_home TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT '',
            updated_at TEXT NOT NULL DEFAULT ''
        )",
    )
    .execute(&pool)
    .await
    .expect("create agent_profiles");

    pool
}

#[tokio::test]
async fn skill_os_index_is_source_aware_and_legacy_schema_safe() {
    let pool = setup_pool(false).await;
    sqlx::query(
        "INSERT INTO installed_skills (id, manifest, installed_at, username, pack_path)
         VALUES ('legacy-pack', ?, '2026-05-08T00:00:00Z', 'alice', 'D:/packs/a.skillpack')",
    )
    .bind(skill_manifest(
        "legacy-pack",
        "Legacy Pack",
        "Skillpack without source_type column",
        &["legacy"],
    ))
    .execute(&pool)
    .await
    .expect("seed legacy skillpack");

    let items = list_skill_os_index_with_pool(&pool)
        .await
        .expect("list skill os index");

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].skill_id, "legacy-pack");
    assert_eq!(items[0].source.canonical, "skillpack");
    assert!(items[0].source.immutable_content);
    assert!(items[0].capabilities.can_view);
    assert!(!items[0].capabilities.can_patch);
    assert!(!items[0].capabilities.can_agent_delete);
}

#[tokio::test]
async fn skill_os_view_loads_only_requested_local_skill_and_keeps_skillpack_read_only() {
    let pool = setup_pool(true).await;
    let local_dir = tempfile::tempdir().expect("local skill dir");
    std::fs::write(
        local_dir.path().join("SKILL.md"),
        "# Local Skill\n\nUse this only when local details are requested.",
    )
    .expect("write local skill");

    sqlx::query(
        "INSERT INTO installed_skills (id, manifest, installed_at, username, pack_path, source_type)
         VALUES
         ('local-skill', ?, '2026-05-08T00:00:00Z', '', ?, 'local'),
         ('pack-skill', ?, '2026-05-08T00:00:01Z', 'alice', 'D:/packs/pack.skillpack', 'encrypted')",
    )
    .bind(skill_manifest(
        "local-skill",
        "Local Skill",
        "Local readable skill",
        &["local"],
    ))
    .bind(local_dir.path().to_string_lossy().to_string())
    .bind(skill_manifest(
        "pack-skill",
        "Pack Skill",
        "Encrypted readonly skill",
        &["pack"],
    ))
    .execute(&pool)
    .await
    .expect("seed skills");

    let local = view_skill_os_entry_with_pool(&pool, "local-skill")
        .await
        .expect("view local skill")
        .expect("local skill exists");
    assert_eq!(local.entry.skill_id, "local-skill");
    assert_eq!(local.entry.source.canonical, "local");
    assert!(!local.read_only);
    assert!(local.content.contains("Local Skill"));

    let pack = view_skill_os_entry_with_pool(&pool, "pack-skill")
        .await
        .expect("view pack skill")
        .expect("pack skill exists");
    assert_eq!(pack.entry.skill_id, "pack-skill");
    assert_eq!(pack.entry.source.canonical, "skillpack");
    assert!(pack.read_only);
    assert!(pack.derived);
    assert_eq!(pack.content, "");
}

#[tokio::test]
async fn skill_os_view_projects_toolset_policy_from_skill_frontmatter() {
    let pool = setup_pool(true).await;
    let local_dir = tempfile::tempdir().expect("local skill dir");
    std::fs::write(
        local_dir.path().join("SKILL.md"),
        "---\nname: Toolset Aware Skill\nrequires_toolsets:\n  - memory\n  - skills\noptional_toolsets: web, browser\ndenied_toolsets:\n  - mcp\n---\n# Toolset Aware Skill\n\nUse profile memory and skill creation when appropriate.",
    )
    .expect("write local skill");

    sqlx::query(
        "INSERT INTO installed_skills (id, manifest, installed_at, username, pack_path, source_type)
         VALUES ('toolset-aware', ?, '2026-05-08T00:00:00Z', '', ?, 'local')",
    )
    .bind(skill_manifest(
        "toolset-aware",
        "Toolset Aware Skill",
        "Declares toolset needs",
        &["toolset"],
    ))
    .bind(local_dir.path().to_string_lossy().to_string())
    .execute(&pool)
    .await
    .expect("seed toolset skill");

    let view = view_skill_os_entry_with_pool(&pool, "toolset-aware")
        .await
        .expect("view skill")
        .expect("skill exists");

    assert_eq!(
        view.entry.toolset_policy.requires_toolsets,
        vec!["memory".to_string(), "skills".to_string()]
    );
    assert_eq!(
        view.entry.toolset_policy.optional_toolsets,
        vec!["browser".to_string(), "web".to_string()]
    );
    assert_eq!(
        view.entry.toolset_policy.denied_toolsets,
        vec!["mcp".to_string()]
    );
}

#[test]
fn skill_os_tool_exposes_skills_list_and_skill_view() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let pool = runtime.block_on(setup_pool(true));
    let local_dir = tempfile::tempdir().expect("local skill dir");
    std::fs::write(
        local_dir.path().join("SKILL.md"),
        "# Tool Skill\n\nTool view body",
    )
    .expect("write local skill");
    runtime
        .block_on(async {
            sqlx::query(
                "INSERT INTO installed_skills (id, manifest, installed_at, username, pack_path, source_type)
                 VALUES ('tool-skill', ?, '2026-05-08T00:00:00Z', '', ?, 'local')",
            )
            .bind(skill_manifest(
                "tool-skill",
                "Tool Skill",
                "Visible through skills tool",
                &["tool"],
            ))
            .bind(local_dir.path().to_string_lossy().to_string())
            .execute(&pool)
            .await
        })
        .expect("seed skill");

    let tool = SkillOsTool::new(pool);
    let list = tool
        .execute(json!({"action": "skills_list"}), &ToolContext::default())
        .expect("list skills");
    assert!(list.contains("tool-skill"));
    assert!(list.contains("\"canonical\": \"local\""));

    let view = tool
        .execute(
            json!({
                "action": "skill_view",
                "skill_id": "tool-skill"
            }),
            &ToolContext::default(),
        )
        .expect("view skill");
    assert!(view.contains("Tool view body"));
}

#[test]
fn skill_os_tool_patches_directory_skill_with_versions_and_rollback() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let pool = runtime.block_on(setup_pool(true));
    let local_dir = tempfile::tempdir().expect("local skill dir");
    let skill_md = local_dir.path().join("SKILL.md");
    std::fs::write(&skill_md, "# Original Skill\n\nOriginal body.").expect("write skill");
    runtime
        .block_on(async {
            sqlx::query(
                "INSERT INTO installed_skills (id, manifest, installed_at, username, pack_path, source_type)
                 VALUES ('mutable-skill', ?, '2026-05-08T00:00:00Z', '', ?, 'local')",
            )
            .bind(skill_manifest(
                "mutable-skill",
                "Mutable Skill",
                "Can be improved",
                &["mutable"],
            ))
            .bind(local_dir.path().to_string_lossy().to_string())
            .execute(&pool)
            .await
        })
        .expect("seed skill");

    let query_pool = pool.clone();
    runtime
        .block_on(async {
            sqlx::query(
                "INSERT INTO sessions (id, skill_id, created_at, model_id, employee_id, profile_id)
                 VALUES ('session-growth', '', '2026-05-08T00:00:00Z', 'model', '', 'profile-growth')",
            )
            .execute(&query_pool)
            .await
        })
        .expect("seed session");

    let tool = SkillOsTool::new(pool);
    let patch = tool
        .execute(
            json!({
                "action": "skill_patch",
                "skill_id": "mutable-skill",
                "content": "# Improved Skill\n\nImproved body.",
                "summary": "tighten instructions"
            }),
            &ToolContext {
                session_id: Some("session-growth".to_string()),
                ..ToolContext::default()
            },
        )
        .expect("patch skill");
    assert!(patch.contains("skill_patch"));
    assert!(patch.contains("\"diff\""));
    assert!(patch.contains("-Original body."));
    assert!(patch.contains("+Improved body."));
    assert!(patch.contains("\"growth_event_id\""));
    assert!(std::fs::read_to_string(&skill_md)
        .expect("read patched skill")
        .contains("Improved body"));

    let growth_count: i64 = runtime
        .block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*)
                 FROM growth_events
                 WHERE profile_id = 'profile-growth'
                   AND session_id = 'session-growth'
                   AND event_type = 'skill_patch'
                   AND target_id = 'mutable-skill'",
            )
            .fetch_one(&query_pool)
            .await
        })
        .expect("query growth events");
    assert_eq!(growth_count, 1);

    let versions_raw = tool
        .execute(
            json!({
                "action": "skill_versions",
                "skill_id": "mutable-skill"
            }),
            &ToolContext::default(),
        )
        .expect("list versions");
    let versions: serde_json::Value = serde_json::from_str(&versions_raw).expect("versions json");
    let items = versions["items"].as_array().expect("versions items");
    assert_eq!(items.len(), 1);
    let version_id = items[0]["version_id"].as_str().expect("version id");
    assert_eq!(items[0]["action"], "patch");

    let view_version = tool
        .execute(
            json!({
                "action": "skill_view_version",
                "skill_id": "mutable-skill",
                "version_id": version_id
            }),
            &ToolContext::default(),
        )
        .expect("view version");
    assert!(view_version.contains("Original body"));

    let rollback_needs_confirm = tool
        .execute(
            json!({
                "action": "skill_rollback",
                "skill_id": "mutable-skill",
                "version_id": version_id
            }),
            &ToolContext::default(),
        )
        .expect("rollback requires confirm");
    assert!(rollback_needs_confirm.contains("confirm=true"));
    assert!(std::fs::read_to_string(&skill_md)
        .expect("read still patched skill")
        .contains("Improved body"));

    let rollback = tool
        .execute(
            json!({
                "action": "skill_rollback",
                "skill_id": "mutable-skill",
                "version_id": version_id,
                "confirm": true,
                "summary": "restore original"
            }),
            &ToolContext::default(),
        )
        .expect("rollback skill");
    assert!(rollback.contains("skill_rollback"));
    assert!(std::fs::read_to_string(&skill_md)
        .expect("read rolled back skill")
        .contains("Original body"));
}

#[test]
fn skill_os_tool_blocks_skillpack_mutation() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let pool = runtime.block_on(setup_pool(true));
    runtime
        .block_on(async {
            sqlx::query(
                "INSERT INTO installed_skills (id, manifest, installed_at, username, pack_path, source_type)
                 VALUES ('pack-skill', ?, '2026-05-08T00:00:00Z', 'alice', 'D:/packs/a.skillpack', 'encrypted')",
            )
            .bind(skill_manifest(
                "pack-skill",
                "Pack Skill",
                "Immutable skillpack",
                &["pack"],
            ))
            .execute(&pool)
            .await
        })
        .expect("seed pack skill");

    let tool = SkillOsTool::new(pool);
    let err = tool
        .execute(
            json!({
                "action": "skill_patch",
                "skill_id": "pack-skill",
                "content": "# Mutated"
            }),
            &ToolContext::default(),
        )
        .expect_err("skillpack mutation must fail");
    assert!(err.to_string().contains("not mutable"));

    let delete_err = tool
        .execute(
            json!({
                "action": "skill_delete",
                "skill_id": "pack-skill",
                "confirm": true
            }),
            &ToolContext::default(),
        )
        .expect_err("skillpack delete must fail");
    assert!(delete_err.to_string().contains("not mutable"));
}

#[test]
fn skill_os_tool_creates_agent_skill_in_profile_home_with_growth_event() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let pool = runtime.block_on(setup_pool(true));
    let profile_home = tempfile::tempdir().expect("profile home");
    let query_pool = pool.clone();
    runtime
        .block_on(async {
            sqlx::query(
                "INSERT INTO agent_profiles (id, profile_home, created_at, updated_at)
                 VALUES ('profile-create', ?, '2026-05-08T00:00:00Z', '2026-05-08T00:00:00Z')",
            )
            .bind(profile_home.path().to_string_lossy().to_string())
            .execute(&query_pool)
            .await
            .expect("seed profile");

            sqlx::query(
                "INSERT INTO sessions (id, skill_id, created_at, model_id, employee_id, profile_id)
                 VALUES ('session-create', '', '2026-05-08T00:00:00Z', 'model', '', 'profile-create')",
            )
            .execute(&query_pool)
            .await
            .expect("seed session");
        });

    let tool = SkillOsTool::new(pool);
    let created = tool
        .execute(
            json!({
                "action": "skill_create",
                "name": "Weekly Insight Writer",
                "description": "Write weekly insight summaries from recurring project evidence",
                "content": "---\nname: Weekly Insight Writer\ndescription: Write weekly insight summaries from recurring project evidence\ntags: [agent-created, writing]\n---\n# Weekly Insight Writer\n\nUse when recurring weekly insight writing succeeds.",
                "summary": "Promote recurring weekly insight workflow into a reusable skill"
            }),
            &ToolContext {
                session_id: Some("session-create".to_string()),
                ..ToolContext::default()
            },
        )
        .expect("create agent skill");

    assert!(created.contains("skill_create"));
    assert!(created.contains("\"growth_event_id\""));
    assert!(created.contains("\"canonical\": \"agent_created\""));

    let skill_id: String = runtime
        .block_on(async {
            sqlx::query_scalar(
                "SELECT id FROM installed_skills WHERE source_type = 'agent_created'",
            )
            .fetch_one(&query_pool)
            .await
        })
        .expect("created skill id");
    assert!(skill_id.starts_with("agent-weekly-insight-writer-"));

    let pack_path: String = runtime
        .block_on(async {
            sqlx::query_scalar("SELECT pack_path FROM installed_skills WHERE id = ?")
                .bind(&skill_id)
                .fetch_one(&query_pool)
                .await
        })
        .expect("created pack path");
    assert!(std::path::Path::new(&pack_path).join("SKILL.md").exists());
    assert!(std::path::Path::new(&pack_path)
        .starts_with(profile_home.path().join("skills").join("active")));

    let versions_raw = tool
        .execute(
            json!({
                "action": "skill_versions",
                "skill_id": skill_id
            }),
            &ToolContext::default(),
        )
        .expect("list created skill versions");
    let versions: serde_json::Value = serde_json::from_str(&versions_raw).expect("versions json");
    assert_eq!(versions["items"][0]["action"], "create");

    let growth_count: i64 = runtime
        .block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*)
                 FROM growth_events
                 WHERE profile_id = 'profile-create'
                   AND session_id = 'session-create'
                   AND event_type = 'skill_create'
                   AND target_id = ?",
            )
            .bind(&skill_id)
            .fetch_one(&query_pool)
            .await
        })
        .expect("query growth events");
    assert_eq!(growth_count, 1);
}

#[test]
fn skill_os_tool_archives_and_restores_agent_created_skill() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let pool = runtime.block_on(setup_pool(true));
    let profile_home = tempfile::tempdir().expect("profile home");
    let query_pool = pool.clone();
    runtime.block_on(async {
        sqlx::query(
            "INSERT INTO agent_profiles (id, profile_home, created_at, updated_at)
             VALUES ('profile-archive', ?, '2026-05-08T00:00:00Z', '2026-05-08T00:00:00Z')",
        )
        .bind(profile_home.path().to_string_lossy().to_string())
        .execute(&query_pool)
        .await
        .expect("seed profile");

        sqlx::query(
            "INSERT INTO sessions (id, skill_id, created_at, model_id, employee_id, profile_id)
             VALUES ('session-archive', '', '2026-05-08T00:00:00Z', 'model', '', 'profile-archive')",
        )
        .execute(&query_pool)
        .await
        .expect("seed session");
    });

    let tool = SkillOsTool::new(pool);
    let created = tool
        .execute(
            json!({
                "action": "skill_create",
                "name": "Archive Candidate",
                "description": "Temporary workflow",
                "content": "---\nname: Archive Candidate\ndescription: Temporary workflow\n---\n# Archive Candidate\n\nTemporary workflow.",
                "summary": "create archive candidate"
            }),
            &ToolContext {
                session_id: Some("session-archive".to_string()),
                ..ToolContext::default()
            },
        )
        .expect("create skill");
    let created_json: serde_json::Value = serde_json::from_str(&created).expect("created json");
    let skill_id = created_json["skill"]["entry"]["skill_id"]
        .as_str()
        .expect("skill id")
        .to_string();
    let active_path: String = runtime
        .block_on(async {
            sqlx::query_scalar("SELECT pack_path FROM installed_skills WHERE id = ?")
                .bind(&skill_id)
                .fetch_one(&query_pool)
                .await
        })
        .expect("active pack path");
    assert!(std::path::Path::new(&active_path)
        .starts_with(profile_home.path().join("skills").join("active")));

    let archive_without_confirm = tool
        .execute(
            json!({
                "action": "skill_archive",
                "skill_id": skill_id
            }),
            &ToolContext::default(),
        )
        .expect("archive asks for confirm");
    assert!(archive_without_confirm.contains("confirm=true"));

    let archived = tool
        .execute(
            json!({
                "action": "skill_archive",
                "skill_id": skill_id,
                "confirm": true,
                "summary": "archive stale workflow"
            }),
            &ToolContext {
                session_id: Some("session-archive".to_string()),
                ..ToolContext::default()
            },
        )
        .expect("archive skill");
    assert!(archived.contains("skill_archive"));
    assert!(archived.contains("\"growth_event_id\""));

    let archived_path: String = runtime
        .block_on(async {
            sqlx::query_scalar("SELECT pack_path FROM installed_skills WHERE id = ?")
                .bind(&skill_id)
                .fetch_one(&query_pool)
                .await
        })
        .expect("archived pack path");
    assert!(std::path::Path::new(&archived_path)
        .starts_with(profile_home.path().join("skills").join("archive")));
    assert!(std::path::Path::new(&archived_path)
        .join("SKILL.md")
        .exists());

    let list_after_archive = tool
        .execute(json!({"action": "skills_list"}), &ToolContext::default())
        .expect("list after archive");
    assert!(!list_after_archive.contains(&skill_id));

    let restored = tool
        .execute(
            json!({
                "action": "skill_restore",
                "skill_id": skill_id,
                "summary": "restore workflow"
            }),
            &ToolContext {
                session_id: Some("session-archive".to_string()),
                ..ToolContext::default()
            },
        )
        .expect("restore skill");
    assert!(restored.contains("skill_restore"));

    let restored_path: String = runtime
        .block_on(async {
            sqlx::query_scalar("SELECT pack_path FROM installed_skills WHERE id = ?")
                .bind(&skill_id)
                .fetch_one(&query_pool)
                .await
        })
        .expect("restored pack path");
    assert!(std::path::Path::new(&restored_path)
        .starts_with(profile_home.path().join("skills").join("active")));
    assert!(std::path::Path::new(&restored_path)
        .join("SKILL.md")
        .exists());

    let list_after_restore = tool
        .execute(json!({"action": "skills_list"}), &ToolContext::default())
        .expect("list after restore");
    assert!(list_after_restore.contains(&skill_id));

    let lifecycle_events: i64 = runtime
        .block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*)
                 FROM growth_events
                 WHERE profile_id = 'profile-archive'
                   AND session_id = 'session-archive'
                   AND target_id = ?
                   AND event_type IN ('skill_archive', 'skill_restore')",
            )
            .bind(&skill_id)
            .fetch_one(&query_pool)
            .await
        })
        .expect("query lifecycle growth events");
    assert_eq!(lifecycle_events, 2);
}

#[test]
fn skill_os_tool_deletes_agent_created_skill_with_evidence() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let pool = runtime.block_on(setup_pool(true));
    let profile_home = tempfile::tempdir().expect("profile home");
    let query_pool = pool.clone();
    runtime.block_on(async {
        sqlx::query(
            "INSERT INTO agent_profiles (id, profile_home, created_at, updated_at)
             VALUES ('profile-delete', ?, '2026-05-08T00:00:00Z', '2026-05-08T00:00:00Z')",
        )
        .bind(profile_home.path().to_string_lossy().to_string())
        .execute(&query_pool)
        .await
        .expect("seed profile");

        sqlx::query(
            "INSERT INTO sessions (id, skill_id, created_at, model_id, employee_id, profile_id)
             VALUES ('session-delete', '', '2026-05-08T00:00:00Z', 'model', '', 'profile-delete')",
        )
        .execute(&query_pool)
        .await
        .expect("seed session");
    });

    let tool = SkillOsTool::new(pool);
    let created = tool
        .execute(
            json!({
                "action": "skill_create",
                "name": "Delete Candidate",
                "description": "Temporary workflow",
                "content": "---\nname: Delete Candidate\ndescription: Temporary workflow\n---\n# Delete Candidate\n\nTemporary workflow.",
                "summary": "create delete candidate"
            }),
            &ToolContext {
                session_id: Some("session-delete".to_string()),
                ..ToolContext::default()
            },
        )
        .expect("create skill");
    let created_json: serde_json::Value = serde_json::from_str(&created).expect("created json");
    let skill_id = created_json["skill"]["entry"]["skill_id"]
        .as_str()
        .expect("skill id")
        .to_string();
    let active_path: String = runtime
        .block_on(async {
            sqlx::query_scalar("SELECT pack_path FROM installed_skills WHERE id = ?")
                .bind(&skill_id)
                .fetch_one(&query_pool)
                .await
        })
        .expect("active pack path");
    assert!(std::path::Path::new(&active_path).join("SKILL.md").exists());

    let delete_without_confirm = tool
        .execute(
            json!({
                "action": "skill_delete",
                "skill_id": skill_id
            }),
            &ToolContext::default(),
        )
        .expect("delete asks for confirm");
    assert!(delete_without_confirm.contains("confirm=true"));
    assert!(std::path::Path::new(&active_path).exists());

    let deleted = tool
        .execute(
            json!({
                "action": "skill_delete",
                "skill_id": skill_id,
                "confirm": true,
                "summary": "delete obsolete workflow"
            }),
            &ToolContext {
                session_id: Some("session-delete".to_string()),
                ..ToolContext::default()
            },
        )
        .expect("delete skill");
    assert!(deleted.contains("skill_delete"));
    assert!(deleted.contains("\"growth_event_id\""));
    assert!(deleted.contains("\"version_id\""));
    assert!(!std::path::Path::new(&active_path).exists());

    let installed_count: i64 = runtime
        .block_on(async {
            sqlx::query_scalar("SELECT COUNT(*) FROM installed_skills WHERE id = ?")
                .bind(&skill_id)
                .fetch_one(&query_pool)
                .await
        })
        .expect("query installed skill");
    assert_eq!(installed_count, 0);

    let list_after_delete = tool
        .execute(json!({"action": "skills_list"}), &ToolContext::default())
        .expect("list after delete");
    assert!(!list_after_delete.contains(&skill_id));

    let versions_raw = tool
        .execute(
            json!({
                "action": "skill_versions",
                "skill_id": skill_id
            }),
            &ToolContext::default(),
        )
        .expect("list versions");
    assert!(versions_raw.contains("\"action\": \"delete\""));

    let delete_events: i64 = runtime
        .block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*)
                 FROM growth_events
                 WHERE profile_id = 'profile-delete'
                   AND session_id = 'session-delete'
                   AND target_id = ?
                   AND event_type = 'skill_delete'",
            )
            .bind(&skill_id)
            .fetch_one(&query_pool)
            .await
        })
        .expect("query delete growth events");
    assert_eq!(delete_events, 1);
}

#[test]
fn skill_os_tool_resets_skill_to_earliest_version_snapshot() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let pool = runtime.block_on(setup_pool(true));
    let profile_home = tempfile::tempdir().expect("profile home");
    let query_pool = pool.clone();
    runtime.block_on(async {
        sqlx::query(
            "INSERT INTO agent_profiles (id, profile_home, created_at, updated_at)
             VALUES ('profile-reset', ?, '2026-05-08T00:00:00Z', '2026-05-08T00:00:00Z')",
        )
        .bind(profile_home.path().to_string_lossy().to_string())
        .execute(&query_pool)
        .await
        .expect("seed profile");

        sqlx::query(
            "INSERT INTO sessions (id, skill_id, created_at, model_id, employee_id, profile_id)
             VALUES ('session-reset', '', '2026-05-08T00:00:00Z', 'model', '', 'profile-reset')",
        )
        .execute(&query_pool)
        .await
        .expect("seed session");
    });

    let tool = SkillOsTool::new(pool);
    let created = tool
        .execute(
            json!({
                "action": "skill_create",
                "name": "Reset Candidate",
                "description": "Reset baseline",
                "content": "---\nname: Reset Candidate\ndescription: Reset baseline\n---\n# Reset Candidate\n\nBaseline body.",
                "summary": "create reset baseline"
            }),
            &ToolContext {
                session_id: Some("session-reset".to_string()),
                ..ToolContext::default()
            },
        )
        .expect("create reset skill");
    let created_json: serde_json::Value = serde_json::from_str(&created).expect("created json");
    let skill_id = created_json["skill"]["entry"]["skill_id"]
        .as_str()
        .expect("skill id")
        .to_string();

    tool.execute(
        json!({
            "action": "skill_patch",
            "skill_id": skill_id,
            "content": "---\nname: Reset Candidate\ndescription: Reset baseline\n---\n# Reset Candidate\n\nChanged body.",
            "summary": "change reset candidate"
        }),
        &ToolContext {
            session_id: Some("session-reset".to_string()),
            ..ToolContext::default()
        },
    )
    .expect("patch reset skill");

    let reset_without_confirm = tool
        .execute(
            json!({
                "action": "skill_reset",
                "skill_id": skill_id
            }),
            &ToolContext::default(),
        )
        .expect("reset asks for confirm");
    assert!(reset_without_confirm.contains("confirm=true"));

    let reset = tool
        .execute(
            json!({
                "action": "skill_reset",
                "skill_id": skill_id,
                "confirm": true,
                "summary": "reset to baseline"
            }),
            &ToolContext {
                session_id: Some("session-reset".to_string()),
                ..ToolContext::default()
            },
        )
        .expect("reset skill");
    assert!(reset.contains("skill_reset"));
    assert!(reset.contains("Baseline body."));
    assert!(reset.contains("\"growth_event_id\""));
    assert!(reset.contains("\"reset_to_version_id\""));

    let versions_raw = tool
        .execute(
            json!({
                "action": "skill_versions",
                "skill_id": skill_id,
                "limit": 10
            }),
            &ToolContext::default(),
        )
        .expect("list reset versions");
    let versions: serde_json::Value = serde_json::from_str(&versions_raw).expect("versions json");
    let items = versions["items"].as_array().expect("versions items");
    assert!(items.iter().any(|item| item["action"] == "reset"));

    let growth_count: i64 = runtime
        .block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*)
                 FROM growth_events
                 WHERE profile_id = 'profile-reset'
                   AND session_id = 'session-reset'
                   AND event_type = 'skill_reset'
                   AND target_id = ?",
            )
            .bind(&skill_id)
            .fetch_one(&query_pool)
            .await
        })
        .expect("query reset growth events");
    assert_eq!(growth_count, 1);
}

#[test]
fn skill_os_ui_reset_records_profile_growth_event() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let pool = runtime.block_on(setup_pool(true));
    let profile_home = tempfile::tempdir().expect("profile home");
    let query_pool = pool.clone();
    runtime.block_on(async {
        sqlx::query(
            "INSERT INTO agent_profiles (id, legacy_employee_row_id, profile_home, created_at, updated_at)
             VALUES ('profile-ui-reset', 'employee-ui-reset', ?, '2026-05-08T00:00:00Z', '2026-05-08T00:00:00Z')",
        )
        .bind(profile_home.path().to_string_lossy().to_string())
        .execute(&query_pool)
        .await
        .expect("seed profile");

        sqlx::query(
            "INSERT INTO sessions (id, skill_id, created_at, model_id, employee_id, profile_id)
             VALUES ('session-ui-reset', '', '2026-05-08T00:00:00Z', 'model', '', 'profile-ui-reset')",
        )
        .execute(&query_pool)
        .await
        .expect("seed session");
    });

    let tool = SkillOsTool::new(pool.clone());
    let created = tool
        .execute(
            json!({
                "action": "skill_create",
                "name": "UI Reset Candidate",
                "description": "UI reset baseline",
                "content": "---\nname: UI Reset Candidate\ndescription: UI reset baseline\n---\n# UI Reset Candidate\n\nBaseline body.",
                "summary": "create ui reset baseline"
            }),
            &ToolContext {
                session_id: Some("session-ui-reset".to_string()),
                ..ToolContext::default()
            },
        )
        .expect("create ui reset skill");
    let created_json: serde_json::Value = serde_json::from_str(&created).expect("created json");
    let skill_id = created_json["skill"]["entry"]["skill_id"]
        .as_str()
        .expect("skill id")
        .to_string();

    tool.execute(
        json!({
            "action": "skill_patch",
            "skill_id": skill_id,
            "content": "---\nname: UI Reset Candidate\ndescription: UI reset baseline\n---\n# UI Reset Candidate\n\nChanged body.",
            "summary": "change ui reset candidate"
        }),
        &ToolContext {
            session_id: Some("session-ui-reset".to_string()),
            ..ToolContext::default()
        },
    )
    .expect("patch ui reset skill");

    let confirm_error = runtime
        .block_on(reset_skill_os_with_pool(
            &pool,
            &skill_id,
            Some("employee-ui-reset"),
            "reset from employee ui",
            false,
        ))
        .expect_err("reset should require confirm");
    assert!(confirm_error.contains("confirm=true"));

    let result = runtime
        .block_on(reset_skill_os_with_pool(
            &pool,
            &skill_id,
            Some("employee-ui-reset"),
            "reset from employee ui",
            true,
        ))
        .expect("reset from employee ui");
    assert_eq!(result.action, "skill_reset");
    assert!(result.skill.content.contains("Baseline body."));
    assert!(result.reset_to_version_id.is_some());
    assert!(result.growth_event_id.is_some());

    let growth_count: i64 = runtime
        .block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*)
                 FROM growth_events
                 WHERE profile_id = 'profile-ui-reset'
                   AND session_id = ''
                   AND event_type = 'skill_reset'
                   AND target_id = ?",
            )
            .bind(&skill_id)
            .fetch_one(&query_pool)
            .await
        })
        .expect("query ui reset growth events");
    assert_eq!(growth_count, 1);
}

#[test]
fn skill_os_ui_patch_records_profile_growth_event() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let pool = runtime.block_on(setup_pool(true));
    let profile_home = tempfile::tempdir().expect("profile home");
    let query_pool = pool.clone();
    runtime.block_on(async {
        sqlx::query(
            "INSERT INTO agent_profiles (id, legacy_employee_row_id, profile_home, created_at, updated_at)
             VALUES ('profile-ui-patch', 'employee-ui-patch', ?, '2026-05-08T00:00:00Z', '2026-05-08T00:00:00Z')",
        )
        .bind(profile_home.path().to_string_lossy().to_string())
        .execute(&query_pool)
        .await
        .expect("seed profile");

        sqlx::query(
            "INSERT INTO sessions (id, skill_id, created_at, model_id, employee_id, profile_id)
             VALUES ('session-ui-patch', '', '2026-05-08T00:00:00Z', 'model', '', 'profile-ui-patch')",
        )
        .execute(&query_pool)
        .await
        .expect("seed session");
    });

    let tool = SkillOsTool::new(pool.clone());
    let created = tool
        .execute(
            json!({
                "action": "skill_create",
                "name": "UI Patch Candidate",
                "description": "UI patch baseline",
                "content": "---\nname: UI Patch Candidate\ndescription: UI patch baseline\n---\n# UI Patch Candidate\n\nBaseline body.",
                "summary": "create ui patch baseline"
            }),
            &ToolContext {
                session_id: Some("session-ui-patch".to_string()),
                ..ToolContext::default()
            },
        )
        .expect("create ui patch skill");
    let created_json: serde_json::Value = serde_json::from_str(&created).expect("created json");
    let skill_id = created_json["skill"]["entry"]["skill_id"]
        .as_str()
        .expect("skill id")
        .to_string();

    let confirm_error = runtime
        .block_on(patch_skill_os_with_pool(
            &pool,
            &skill_id,
            "---\nname: UI Patch Candidate\ndescription: UI patch baseline\n---\n# UI Patch Candidate\n\nChanged by UI.",
            Some("employee-ui-patch"),
            "patch from employee ui",
            false,
        ))
        .expect_err("patch should require confirm");
    assert!(confirm_error.contains("confirm=true"));

    let result = runtime
        .block_on(patch_skill_os_with_pool(
            &pool,
            &skill_id,
            "---\nname: UI Patch Candidate\ndescription: UI patch baseline\n---\n# UI Patch Candidate\n\nChanged by UI.",
            Some("employee-ui-patch"),
            "patch from employee ui",
            true,
        ))
        .expect("patch from employee ui");
    assert_eq!(result.action, "skill_patch");
    assert!(result.skill.content.contains("Changed by UI."));
    assert!(result.growth_event_id.is_some());
    assert!(result.diff.contains("Changed by UI."));

    let growth_count: i64 = runtime
        .block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*)
                 FROM growth_events
                 WHERE profile_id = 'profile-ui-patch'
                   AND session_id = ''
                   AND event_type = 'skill_patch'
                   AND target_id = ?",
            )
            .bind(&skill_id)
            .fetch_one(&query_pool)
            .await
        })
        .expect("query ui patch growth events");
    assert_eq!(growth_count, 1);
}

#[test]
fn skill_os_ui_archive_restore_delete_record_profile_growth_events() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let pool = runtime.block_on(setup_pool(true));
    let profile_home = tempfile::tempdir().expect("profile home");
    let query_pool = pool.clone();
    runtime.block_on(async {
        sqlx::query(
            "INSERT INTO agent_profiles (id, legacy_employee_row_id, profile_home, created_at, updated_at)
             VALUES ('profile-ui-lifecycle', 'employee-ui-lifecycle', ?, '2026-05-08T00:00:00Z', '2026-05-08T00:00:00Z')",
        )
        .bind(profile_home.path().to_string_lossy().to_string())
        .execute(&query_pool)
        .await
        .expect("seed profile");

        sqlx::query(
            "INSERT INTO sessions (id, skill_id, created_at, model_id, employee_id, profile_id)
             VALUES ('session-ui-lifecycle', '', '2026-05-08T00:00:00Z', 'model', '', 'profile-ui-lifecycle')",
        )
        .execute(&query_pool)
        .await
        .expect("seed session");
    });

    let tool = SkillOsTool::new(pool.clone());
    let created = tool
        .execute(
            json!({
                "action": "skill_create",
                "name": "UI Lifecycle Candidate",
                "description": "UI lifecycle baseline",
                "content": "---\nname: UI Lifecycle Candidate\ndescription: UI lifecycle baseline\n---\n# UI Lifecycle Candidate\n\nLifecycle body.",
                "summary": "create ui lifecycle baseline"
            }),
            &ToolContext {
                session_id: Some("session-ui-lifecycle".to_string()),
                ..ToolContext::default()
            },
        )
        .expect("create ui lifecycle skill");
    let created_json: serde_json::Value = serde_json::from_str(&created).expect("created json");
    let skill_id = created_json["skill"]["entry"]["skill_id"]
        .as_str()
        .expect("skill id")
        .to_string();

    let archive_confirm_error = runtime
        .block_on(archive_skill_os_with_pool(
            &pool,
            &skill_id,
            Some("employee-ui-lifecycle"),
            "archive from employee ui",
            false,
        ))
        .expect_err("archive should require confirm");
    assert!(archive_confirm_error.contains("confirm=true"));

    let archived = runtime
        .block_on(archive_skill_os_with_pool(
            &pool,
            &skill_id,
            Some("employee-ui-lifecycle"),
            "archive from employee ui",
            true,
        ))
        .expect("archive from employee ui");
    assert_eq!(archived.action, "skill_archive");
    assert_eq!(archived.skill.entry.lifecycle_state, "archived");
    assert!(archived.growth_event_id.is_some());

    let restored = runtime
        .block_on(restore_skill_os_with_pool(
            &pool,
            &skill_id,
            Some("employee-ui-lifecycle"),
            "restore from employee ui",
        ))
        .expect("restore from employee ui");
    assert_eq!(restored.action, "skill_restore");
    assert_eq!(restored.skill.entry.lifecycle_state, "active");
    assert!(restored.growth_event_id.is_some());

    let delete_confirm_error = runtime
        .block_on(delete_skill_os_with_pool(
            &pool,
            &skill_id,
            Some("employee-ui-lifecycle"),
            "delete from employee ui",
            false,
        ))
        .expect_err("delete should require confirm");
    assert!(delete_confirm_error.contains("confirm=true"));

    let deleted = runtime
        .block_on(delete_skill_os_with_pool(
            &pool,
            &skill_id,
            Some("employee-ui-lifecycle"),
            "delete from employee ui",
            true,
        ))
        .expect("delete from employee ui");
    assert_eq!(deleted.action, "skill_delete");
    assert!(deleted.growth_event_id.is_some());

    let installed_count: i64 = runtime
        .block_on(async {
            sqlx::query_scalar("SELECT COUNT(*) FROM installed_skills WHERE id = ?")
                .bind(&skill_id)
                .fetch_one(&query_pool)
                .await
        })
        .expect("query installed skill count");
    assert_eq!(installed_count, 0);

    let growth_count: i64 = runtime
        .block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*)
                 FROM growth_events
                 WHERE profile_id = 'profile-ui-lifecycle'
                   AND session_id = ''
                   AND target_id = ?
                   AND event_type IN ('skill_archive', 'skill_restore', 'skill_delete')",
            )
            .bind(&skill_id)
            .fetch_one(&query_pool)
            .await
        })
        .expect("query lifecycle growth events");
    assert_eq!(growth_count, 3);
}
