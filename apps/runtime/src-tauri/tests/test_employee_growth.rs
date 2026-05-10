use runtime_lib::commands::employee_agents::{
    list_employee_curator_runs_with_pool, list_employee_growth_events_with_pool,
    restore_employee_curator_stale_skill_with_pool,
};

#[tokio::test]
async fn list_employee_growth_events_resolves_profile_and_orders_recent_events() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite::memory:")
        .await
        .expect("create sqlite pool");

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

    sqlx::query(
        "CREATE TABLE growth_events (
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
    .execute(&pool)
    .await
    .expect("create growth_events");

    sqlx::query(
        "INSERT INTO agent_profiles (id, legacy_employee_row_id, display_name)
         VALUES ('profile-growth-ui', 'employee-row-growth', 'Growth UI')",
    )
    .execute(&pool)
    .await
    .expect("seed profile");

    sqlx::query(
        "INSERT INTO growth_events (
            id, profile_id, session_id, event_type, target_type, target_id, summary, evidence_json, created_at
         ) VALUES
         ('old-event', 'profile-growth-ui', 'session-1', 'skill_patch', 'skill', 'skill-a', 'older patch', '{\"version_id\":\"v1\"}', '2026-05-08T00:00:00Z'),
         ('new-event', 'profile-growth-ui', 'session-2', 'skill_reset', 'skill', 'skill-a', 'newer reset', '{\"version_id\":\"v2\"}', '2026-05-08T01:00:00Z'),
         ('other-profile', 'profile-other', 'session-3', 'skill_patch', 'skill', 'skill-b', 'other', '{}', '2026-05-08T02:00:00Z')",
    )
    .execute(&pool)
    .await
    .expect("seed growth events");

    let timeline = list_employee_growth_events_with_pool(&pool, "employee-row-growth", 10)
        .await
        .expect("list growth events");

    assert_eq!(timeline.employee_id, "employee-row-growth");
    assert_eq!(timeline.profile_id.as_deref(), Some("profile-growth-ui"));
    assert_eq!(timeline.events.len(), 2);
    assert_eq!(timeline.events[0].id, "new-event");
    assert_eq!(timeline.events[0].event_type, "skill_reset");
    assert_eq!(timeline.events[0].evidence_json["version_id"], "v2");
    assert_eq!(timeline.events[1].id, "old-event");
}

#[tokio::test]
async fn list_employee_curator_runs_resolves_profile_and_parses_findings() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite::memory:")
        .await
        .expect("create sqlite pool");

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

    sqlx::query(
        "CREATE TABLE curator_runs (
            id TEXT PRIMARY KEY,
            profile_id TEXT NOT NULL DEFAULT '',
            scope TEXT NOT NULL DEFAULT 'profile',
            summary TEXT NOT NULL DEFAULT '',
            report_json TEXT NOT NULL DEFAULT '{}',
            report_path TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("create curator_runs");

    sqlx::query(
        "INSERT INTO agent_profiles (id, legacy_employee_row_id, display_name)
         VALUES ('profile-curator-ui', 'employee-row-curator', 'Curator UI')",
    )
    .execute(&pool)
    .await
    .expect("seed profile");

    sqlx::query(
        "INSERT INTO curator_runs (id, profile_id, scope, summary, report_json, report_path, created_at)
         VALUES (
            'cur-run-1',
            'profile-curator-ui',
            'profile',
            '发现 1 个可整理项',
            '{\"mode\":\"run\",\"findings\":[{\"kind\":\"stale_skill\",\"severity\":\"low\",\"target_type\":\"skill\",\"target_id\":\"agent-stale\",\"summary\":\"已标记 stale\",\"evidence\":{\"state_changed\":true},\"suggested_action\":\"curator.restore\",\"reversible\":true},{\"kind\":\"duplicate_memory\",\"severity\":\"medium\",\"target_type\":\"memory\",\"target_id\":\"MEMORY.md\",\"summary\":\"重复记忆\",\"evidence\":{\"line\":2},\"suggested_action\":\"memory.replace\",\"reversible\":true}]}',
            'D:/profiles/profile-curator-ui/curator/reports/cur-run-1.json',
            '2026-05-08T02:00:00Z'
         )",
    )
    .execute(&pool)
    .await
    .expect("seed curator run");

    let reports = list_employee_curator_runs_with_pool(&pool, "employee-row-curator", 10)
        .await
        .expect("list curator runs");

    assert_eq!(reports.employee_id, "employee-row-curator");
    assert_eq!(reports.profile_id.as_deref(), Some("profile-curator-ui"));
    assert_eq!(reports.runs.len(), 1);
    assert_eq!(reports.runs[0].id, "cur-run-1");
    assert_eq!(reports.runs[0].mode, "run");
    assert!(reports.runs[0].has_state_changes);
    assert_eq!(reports.runs[0].changed_targets[0].target_id, "agent-stale");
    assert_eq!(
        reports.runs[0].restore_candidates[0].target_id,
        "agent-stale"
    );
    assert_eq!(
        reports.runs[0].restore_candidates[0].input["skill_id"],
        "agent-stale"
    );
    assert_eq!(reports.runs[0].findings.len(), 2);
    assert_eq!(reports.runs[0].findings[1].kind, "duplicate_memory");
    assert_eq!(reports.runs[0].findings[1].evidence_json["line"], 2);
}

#[tokio::test]
async fn restore_employee_curator_stale_skill_restores_state_and_records_report() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite::memory:")
        .await
        .expect("create sqlite pool");
    let profile_home = tempfile::tempdir().expect("profile home");

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

    sqlx::query(
        "CREATE TABLE installed_skills (
            id TEXT PRIMARY KEY,
            manifest TEXT NOT NULL,
            installed_at TEXT NOT NULL,
            username TEXT NOT NULL,
            pack_path TEXT NOT NULL DEFAULT '',
            source_type TEXT NOT NULL DEFAULT 'agent_created'
        )",
    )
    .execute(&pool)
    .await
    .expect("create installed_skills");

    sqlx::query(
        "INSERT INTO agent_profiles (id, legacy_employee_row_id, display_name, profile_home)
         VALUES ('profile-curator-restore', 'employee-row-restore', 'Curator Restore', ?)",
    )
    .bind(profile_home.path().to_string_lossy().to_string())
    .execute(&pool)
    .await
    .expect("seed profile");

    sqlx::query(
        "INSERT INTO installed_skills (id, manifest, installed_at, username, pack_path, source_type)
         VALUES ('agent-stale', '{}', '2026-05-08T00:00:00Z', '', '', 'agent_created')",
    )
    .execute(&pool)
    .await
    .expect("seed installed skill");

    runtime_lib::agent::runtime::runtime_io::mark_skill_os_stale_with_pool(&pool, "agent-stale")
        .await
        .expect("mark stale");

    let run = restore_employee_curator_stale_skill_with_pool(
        &pool,
        "employee-row-restore",
        "agent-stale",
    )
    .await
    .expect("restore stale skill");

    assert_eq!(run.mode, "restore");
    assert!(run.has_state_changes);
    assert_eq!(run.changed_targets[0].target_id, "agent-stale");
    assert_eq!(run.changed_targets[0].restored_to, "active");
    assert!(run.restore_candidates.is_empty());
    assert!(std::path::Path::new(&run.report_path).exists());

    let state: String =
        sqlx::query_scalar("SELECT state FROM skill_lifecycle WHERE skill_id = 'agent-stale'")
            .fetch_one(&pool)
            .await
            .expect("query lifecycle state");
    assert_eq!(state, "active");

    let growth_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM growth_events
         WHERE profile_id = 'profile-curator-restore'
           AND event_type = 'curator_restore'",
    )
    .fetch_one(&pool)
    .await
    .expect("query growth events");
    assert_eq!(growth_count, 1);
}
