use runtime_lib::agent::tools::CuratorTool;
use runtime_lib::agent::types::{Tool, ToolContext};
use serde_json::json;

fn skill_manifest(id: &str, name: &str, description: &str) -> String {
    json!({
        "id": id,
        "name": name,
        "description": description,
        "version": "1.0.0",
        "author": "WorkClaw",
        "recommended_model": "",
        "tags": [],
        "created_at": "2026-05-08T00:00:00Z",
        "username_hint": null,
        "encrypted_verify": ""
    })
    .to_string()
}

#[test]
fn curator_scan_reports_memory_and_skill_growth_opportunities() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let pool = runtime
        .block_on(async {
            sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(5)
                .connect("sqlite::memory:")
                .await
        })
        .expect("create sqlite pool");
    let profile_home = tempfile::tempdir().expect("profile home");
    let memory_dir = profile_home.path().join("memories");
    let skill_dir = profile_home
        .path()
        .join("skills")
        .join("active")
        .join("agent-draft");
    std::fs::create_dir_all(&memory_dir).expect("create memory dir");
    std::fs::create_dir_all(&skill_dir).expect("create skill dir");
    std::fs::write(
        memory_dir.join("MEMORY.md"),
        "- 每周复盘流程：收集日报、提炼风险、输出行动项。\n- 每周复盘流程：收集日报、提炼风险、输出行动项。\n- ok\n",
    )
    .expect("write memory");
    std::fs::write(skill_dir.join("SKILL.md"), "# Draft\n").expect("write skill");

    runtime
        .block_on(async {
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

            sqlx::query(
                "INSERT INTO installed_skills (id, manifest, installed_at, username, pack_path, source_type)
                 VALUES ('agent-draft', ?, '2026-05-08T00:00:00Z', '', ?, 'agent_created')",
            )
            .bind(skill_manifest("agent-draft", "Draft Skill", "todo"))
            .bind(skill_dir.to_string_lossy().to_string())
            .execute(&pool)
            .await
            .expect("seed skill");
        });

    let tool = CuratorTool::new(
        pool.clone(),
        "profile-curator".to_string(),
        memory_dir.clone(),
    );
    let raw = tool
        .execute(json!({"action": "scan"}), &ToolContext::default())
        .expect("scan curator");
    let output: serde_json::Value = serde_json::from_str(&raw).expect("curator json");

    assert_eq!(output["action"], "scan");
    assert!(output["run_id"]
        .as_str()
        .unwrap_or_default()
        .starts_with("cur_"));
    assert!(raw.contains("duplicate_memory"));
    assert!(raw.contains("reusable_skill_candidate"));
    assert!(raw.contains("low_value_debris"));
    let report_path = output["report_path"].as_str().expect("report path");
    assert!(std::path::Path::new(report_path).exists());

    let run_count: i64 = runtime
        .block_on(async {
            sqlx::query_scalar("SELECT COUNT(*) FROM curator_runs WHERE profile_id = ?")
                .bind("profile-curator")
                .fetch_one(&pool)
                .await
        })
        .expect("query curator runs");
    assert_eq!(run_count, 1);

    let growth_count: i64 = runtime
        .block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM growth_events
                 WHERE profile_id = ?
                   AND event_type = 'curator_scan'
                   AND target_type = 'curator'",
            )
            .bind("profile-curator")
            .fetch_one(&pool)
            .await
        })
        .expect("query growth events");
    assert_eq!(growth_count, 1);

    let history = tool
        .execute(json!({"action": "history"}), &ToolContext::default())
        .expect("curator history");
    assert!(history.contains("profile-curator"));
    assert!(history.contains("curator_runs"));
}

#[test]
fn curator_run_marks_unpinned_agent_skill_stale_and_skips_pinned_skill() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let pool = runtime
        .block_on(async {
            sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(5)
                .connect("sqlite::memory:")
                .await
        })
        .expect("create sqlite pool");
    let profile_home = tempfile::tempdir().expect("profile home");
    let memory_dir = profile_home.path().join("memories");
    let stale_dir = profile_home
        .path()
        .join("skills")
        .join("active")
        .join("agent-stale");
    let pinned_dir = profile_home
        .path()
        .join("skills")
        .join("active")
        .join("agent-pinned");
    std::fs::create_dir_all(&memory_dir).expect("create memory dir");
    std::fs::create_dir_all(&stale_dir).expect("create stale skill dir");
    std::fs::create_dir_all(&pinned_dir).expect("create pinned skill dir");
    std::fs::write(memory_dir.join("MEMORY.md"), "").expect("write memory");
    std::fs::write(stale_dir.join("SKILL.md"), "# Draft\n").expect("write stale skill");
    std::fs::write(pinned_dir.join("SKILL.md"), "# Draft\n").expect("write pinned skill");

    runtime
        .block_on(async {
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

            sqlx::query(
                "INSERT INTO installed_skills (id, manifest, installed_at, username, pack_path, source_type)
                 VALUES ('agent-stale', ?, '2026-05-08T00:00:00Z', '', ?, 'agent_created')",
            )
            .bind(skill_manifest("agent-stale", "Draft Skill", "todo"))
            .bind(stale_dir.to_string_lossy().to_string())
            .execute(&pool)
            .await
            .expect("seed stale skill");

            sqlx::query(
                "INSERT INTO installed_skills (id, manifest, installed_at, username, pack_path, source_type)
                 VALUES ('agent-pinned', ?, '2026-05-08T00:00:01Z', '', ?, 'agent_created')",
            )
            .bind(skill_manifest("agent-pinned", "Pinned Draft Skill", "todo"))
            .bind(pinned_dir.to_string_lossy().to_string())
            .execute(&pool)
            .await
            .expect("seed pinned skill");

            runtime_lib::agent::runtime::runtime_io::set_skill_os_pinned_with_pool(
                &pool,
                "agent-pinned",
                true,
            )
            .await
            .expect("pin skill");
        });

    let tool = CuratorTool::new(
        pool.clone(),
        "profile-curator".to_string(),
        memory_dir.clone(),
    );
    let raw = tool
        .execute(json!({"action": "run"}), &ToolContext::default())
        .expect("run curator");
    assert!(raw.contains("\"action\": \"run\""));
    assert!(raw.contains("stale_skill"));
    assert!(raw.contains("pinned_skill_protected"));

    let (stale_state, pinned_state): (String, String) = runtime.block_on(async {
        let stale_state =
            sqlx::query_scalar("SELECT state FROM skill_lifecycle WHERE skill_id = 'agent-stale'")
                .fetch_one(&pool)
                .await
                .expect("query stale state");
        let pinned_state =
            sqlx::query_scalar("SELECT state FROM skill_lifecycle WHERE skill_id = 'agent-pinned'")
                .fetch_one(&pool)
                .await
                .expect("query pinned state");
        (stale_state, pinned_state)
    });
    assert_eq!(stale_state, "stale");
    assert_eq!(pinned_state, "active");

    let history_after_run = tool
        .execute(
            json!({"action": "history", "limit": 1}),
            &ToolContext::default(),
        )
        .expect("curator history after run");
    let history_json: serde_json::Value =
        serde_json::from_str(&history_after_run).expect("history json");
    let run_item = &history_json["items"][0];
    assert_eq!(run_item["mode"], "run");
    assert_eq!(run_item["has_state_changes"], true);
    assert_eq!(run_item["changed_targets"][0]["target_id"], "agent-stale");
    assert_eq!(
        run_item["restore_candidates"][0]["input"]["skill_id"],
        "agent-stale"
    );

    let restore_raw = tool
        .execute(
            json!({"action": "restore", "skill_id": "agent-stale"}),
            &ToolContext::default(),
        )
        .expect("restore stale skill");
    assert!(restore_raw.contains("\"action\": \"restore\""));
    assert!(restore_raw.contains("\"restored\": true"));
    assert!(restore_raw.contains("curator_restore"));

    let restored_state: String = runtime
        .block_on(async {
            sqlx::query_scalar("SELECT state FROM skill_lifecycle WHERE skill_id = 'agent-stale'")
                .fetch_one(&pool)
                .await
        })
        .expect("query restored state");
    assert_eq!(restored_state, "active");

    let restore_growth_count: i64 = runtime
        .block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM growth_events
                 WHERE profile_id = ?
                   AND event_type = 'curator_restore'
                   AND target_type = 'curator'",
            )
            .bind("profile-curator")
            .fetch_one(&pool)
            .await
        })
        .expect("query restore growth events");
    assert_eq!(restore_growth_count, 1);

    let history_after_restore = tool
        .execute(
            json!({"action": "history", "limit": 1}),
            &ToolContext::default(),
        )
        .expect("curator history after restore");
    let restore_history_json: serde_json::Value =
        serde_json::from_str(&history_after_restore).expect("restore history json");
    let restore_item = &restore_history_json["items"][0];
    assert_eq!(restore_item["mode"], "restore");
    assert_eq!(restore_item["has_state_changes"], true);
    assert_eq!(restore_item["changed_targets"][0]["restored_to"], "active");
    assert!(restore_item["restore_candidates"]
        .as_array()
        .expect("restore candidates")
        .is_empty());
}

#[test]
fn curator_run_suggests_patch_for_used_draft_without_marking_stale() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let pool = runtime
        .block_on(async {
            sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(5)
                .connect("sqlite::memory:")
                .await
        })
        .expect("create sqlite pool");
    let profile_home = tempfile::tempdir().expect("profile home");
    let memory_dir = profile_home.path().join("memories");
    let active_dir = profile_home
        .path()
        .join("skills")
        .join("active")
        .join("agent-active-draft");
    std::fs::create_dir_all(&memory_dir).expect("create memory dir");
    std::fs::create_dir_all(&active_dir).expect("create active skill dir");
    std::fs::write(memory_dir.join("MEMORY.md"), "").expect("write memory");
    std::fs::write(active_dir.join("SKILL.md"), "# Draft\n").expect("write active draft skill");

    runtime.block_on(async {
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

        sqlx::query(
            "INSERT INTO installed_skills (id, manifest, installed_at, username, pack_path, source_type)
             VALUES ('agent-active-draft', ?, '2026-05-08T00:00:00Z', '', ?, 'agent_created')",
        )
        .bind(skill_manifest("agent-active-draft", "Active Draft Skill", "todo"))
        .bind(active_dir.to_string_lossy().to_string())
        .execute(&pool)
        .await
        .expect("seed active draft skill");

        runtime_lib::agent::runtime::runtime_io::record_skill_os_usage_with_pool(
            &pool,
            "agent-active-draft",
            "use",
        )
        .await
        .expect("record skill use");
    });

    let tool = CuratorTool::new(
        pool.clone(),
        "profile-curator".to_string(),
        memory_dir.clone(),
    );
    let raw = tool
        .execute(json!({"action": "run"}), &ToolContext::default())
        .expect("run curator");
    assert!(raw.contains("skill_improvement_candidate"));
    assert!(raw.contains("\"use_count\": 1"));
    assert!(!raw.contains("stale_skill"));

    let state: String = runtime
        .block_on(async {
            sqlx::query_scalar(
                "SELECT state FROM skill_lifecycle WHERE skill_id = 'agent-active-draft'",
            )
            .fetch_one(&pool)
            .await
        })
        .expect("query active draft state");
    assert_eq!(state, "active");
}
