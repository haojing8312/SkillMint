#![recursion_limit = "256"]

mod helpers;

use runtime_lib::commands::employee_agents::test_support::{
    create_employee_group_with_pool, run_group_step_with_pool, start_employee_group_run_with_pool,
};
use runtime_lib::commands::employee_agents::{
    get_employee_group_run_snapshot_with_pool, reassign_group_run_step_with_pool,
    review_group_run_step_with_pool, upsert_agent_employee_with_pool, CreateEmployeeGroupInput,
    StartEmployeeGroupRunInput, UpsertAgentEmployeeInput,
};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

async fn rebuild_sessions_without_profile_id(pool: &SqlitePool) {
    sqlx::query("DROP TABLE sessions")
        .execute(pool)
        .await
        .expect("drop current sessions table");
    sqlx::query(
        "CREATE TABLE sessions (
            id TEXT PRIMARY KEY,
            skill_id TEXT NOT NULL,
            title TEXT,
            created_at TEXT NOT NULL,
            model_id TEXT NOT NULL,
            permission_mode TEXT NOT NULL DEFAULT 'accept_edits',
            work_dir TEXT NOT NULL DEFAULT '',
            employee_id TEXT NOT NULL DEFAULT '',
            session_mode TEXT NOT NULL DEFAULT 'general',
            team_id TEXT NOT NULL DEFAULT ''
        )",
    )
    .execute(pool)
    .await
    .expect("create legacy sessions without profile_id");
}

async fn rebuild_group_run_steps_without_profile_columns(pool: &SqlitePool) {
    sqlx::query("DROP TABLE group_run_steps")
        .execute(pool)
        .await
        .expect("drop current group_run_steps table");
    sqlx::query(
        "CREATE TABLE group_run_steps (
            id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            round_no INTEGER NOT NULL DEFAULT 0,
            parent_step_id TEXT NOT NULL DEFAULT '',
            assignee_employee_id TEXT NOT NULL DEFAULT '',
            dispatch_source_employee_id TEXT NOT NULL DEFAULT '',
            phase TEXT NOT NULL DEFAULT '',
            step_type TEXT NOT NULL DEFAULT 'execute',
            step_kind TEXT NOT NULL DEFAULT 'execute',
            input TEXT NOT NULL DEFAULT '',
            input_summary TEXT NOT NULL DEFAULT '',
            output TEXT NOT NULL DEFAULT '',
            output_summary TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'pending',
            requires_review INTEGER NOT NULL DEFAULT 0,
            review_status TEXT NOT NULL DEFAULT 'not_required',
            attempt_no INTEGER NOT NULL DEFAULT 0,
            session_id TEXT NOT NULL DEFAULT '',
            visibility TEXT NOT NULL DEFAULT 'internal',
            started_at TEXT NOT NULL DEFAULT '',
            finished_at TEXT NOT NULL DEFAULT ''
        )",
    )
    .execute(pool)
    .await
    .expect("create legacy group_run_steps without profile columns");
}

async fn recreate_agent_profiles_without_legacy_mapping(pool: &SqlitePool) {
    sqlx::query("DROP TABLE agent_profiles")
        .execute(pool)
        .await
        .expect("drop current agent_profiles table");
    sqlx::query(
        "CREATE TABLE agent_profiles (
            id TEXT PRIMARY KEY,
            profile_home TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT '',
            updated_at TEXT NOT NULL DEFAULT ''
        )",
    )
    .execute(pool)
    .await
    .expect("create legacy agent_profiles without legacy mapping column");
}

async fn seed_default_model_config(pool: &SqlitePool) {
    sqlx::query(
        "INSERT INTO model_configs (id, name, api_format, base_url, model_name, is_default, api_key)
         VALUES ('m1', 'default', 'openai', 'http://mock', 'gpt-4o-mini', 1, 'k')",
    )
    .execute(pool)
    .await
    .expect("seed model config");
}

async fn seed_group_run_employee(
    pool: &SqlitePool,
    employee_id: &str,
    skill_id: &str,
) -> (String, String) {
    upsert_agent_employee_with_pool(
        pool,
        UpsertAgentEmployeeInput {
            id: None,
            employee_id: employee_id.to_string(),
            name: employee_id.to_string(),
            role_id: employee_id.to_string(),
            persona: "".to_string(),
            feishu_open_id: "".to_string(),
            feishu_app_id: "".to_string(),
            feishu_app_secret: "".to_string(),
            primary_skill_id: skill_id.to_string(),
            default_work_dir: format!("E:/workspace/{employee_id}"),
            openclaw_agent_id: employee_id.to_string(),
            routing_priority: 100,
            enabled_scopes: vec!["feishu".to_string(), "app".to_string()],
            enabled: true,
            is_default: employee_id == "project_manager",
            skill_ids: vec![skill_id.to_string()],
        },
    )
    .await
    .expect("seed team employee");

    let (employee_row_id,): (String,) =
        sqlx::query_as("SELECT id FROM agent_employees WHERE employee_id = ?")
            .bind(employee_id)
            .fetch_one(pool)
            .await
            .expect("load employee row id");
    let (profile_id,): (String,) =
        sqlx::query_as("SELECT id FROM agent_profiles WHERE legacy_employee_row_id = ? LIMIT 1")
            .bind(&employee_row_id)
            .fetch_one(pool)
            .await
            .expect("load created real profile id");

    (employee_row_id, profile_id)
}

async fn drop_legacy_profile_binding_columns(pool: &SqlitePool) {
    sqlx::query("DROP INDEX IF EXISTS idx_sessions_profile_id")
        .execute(pool)
        .await
        .expect("drop sessions profile index");
    sqlx::query("ALTER TABLE sessions DROP COLUMN profile_id")
        .execute(pool)
        .await
        .expect("drop sessions.profile_id");
    sqlx::query("ALTER TABLE group_run_steps DROP COLUMN assignee_profile_id")
        .execute(pool)
        .await
        .expect("drop group_run_steps.assignee_profile_id");
    sqlx::query("ALTER TABLE group_run_steps DROP COLUMN dispatch_source_profile_id")
        .execute(pool)
        .await
        .expect("drop group_run_steps.dispatch_source_profile_id");
}

async fn drop_agent_profiles_legacy_employee_column(pool: &SqlitePool) {
    sqlx::query("DROP INDEX IF EXISTS idx_agent_profiles_employee_row_id")
        .execute(pool)
        .await
        .expect("drop agent profile legacy employee index");
    sqlx::query("ALTER TABLE agent_profiles DROP COLUMN legacy_employee_row_id")
        .execute(pool)
        .await
        .expect("drop agent_profiles.legacy_employee_row_id");
}

#[tokio::test]
async fn group_run_persists_real_profile_ids_for_steps_and_sessions() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    seed_default_model_config(&pool).await;

    let (_pm_row_id, pm_profile_id) =
        seed_group_run_employee(&pool, "project_manager", "builtin-general").await;
    let (_dev_row_id, dev_profile_id) =
        seed_group_run_employee(&pool, "dev_team", "delivery-skill").await;
    let (_qa_row_id, _qa_profile_id) = seed_group_run_employee(&pool, "qa_team", "qa-skill").await;

    let group_id = create_employee_group_with_pool(
        &pool,
        CreateEmployeeGroupInput {
            name: "Profile 绑定战队".to_string(),
            coordinator_employee_id: "project_manager".to_string(),
            member_employee_ids: vec![
                "project_manager".to_string(),
                "dev_team".to_string(),
                "qa_team".to_string(),
            ],
        },
    )
    .await
    .expect("create profile-bound group");

    let outcome = start_employee_group_run_with_pool(
        &pool,
        StartEmployeeGroupRunInput {
            group_id,
            user_goal: "完成 profile 绑定验证".to_string(),
            execution_window: 3,
            max_retry_per_step: 1,
            timeout_employee_ids: vec![],
        },
    )
    .await
    .expect("start profile-bound group run");

    let dev_step = outcome
        .steps
        .iter()
        .find(|step| step.step_type == "execute" && step.assignee_employee_id == "dev_team")
        .expect("dev execute step");
    assert_eq!(
        dev_step.assignee_profile_id.as_deref(),
        Some(dev_profile_id.as_str())
    );
    assert_eq!(
        dev_step.dispatch_source_profile_id.as_deref(),
        Some(pm_profile_id.as_str())
    );

    let (assignee_profile_id, dispatch_source_profile_id): (Option<String>, Option<String>) =
        sqlx::query_as(
            "SELECT assignee_profile_id, dispatch_source_profile_id
             FROM group_run_steps
             WHERE id = ?",
        )
        .bind(&dev_step.id)
        .fetch_one(&pool)
        .await
        .expect("load persisted step profile binding");
    assert_eq!(
        assignee_profile_id.as_deref(),
        Some(dev_profile_id.as_str())
    );
    assert_eq!(
        dispatch_source_profile_id.as_deref(),
        Some(pm_profile_id.as_str())
    );

    let (coordinator_session_profile_id,): (Option<String>,) =
        sqlx::query_as("SELECT profile_id FROM sessions WHERE id = ?")
            .bind(&outcome.session_id)
            .fetch_one(&pool)
            .await
            .expect("load coordinator session profile binding");
    assert_eq!(
        coordinator_session_profile_id.as_deref(),
        Some(pm_profile_id.as_str())
    );

    let step_created_payload_json: (String,) = sqlx::query_as(
        "SELECT payload_json
         FROM group_run_events
         WHERE run_id = ? AND step_id = ? AND event_type = 'step_created'
         LIMIT 1",
    )
    .bind(&outcome.run_id)
    .bind(&dev_step.id)
    .fetch_one(&pool)
    .await
    .expect("load profile-bound step_created event");
    assert!(step_created_payload_json
        .0
        .contains(&format!("\"assignee_profile_id\":\"{dev_profile_id}\"")));
    assert!(step_created_payload_json.0.contains(&format!(
        "\"dispatch_source_profile_id\":\"{pm_profile_id}\""
    )));

    let executed = run_group_step_with_pool(&pool, &dev_step.id)
        .await
        .expect("run profile-bound dev step");
    let (step_session_profile_id,): (Option<String>,) =
        sqlx::query_as("SELECT profile_id FROM sessions WHERE id = ?")
            .bind(&executed.session_id)
            .fetch_one(&pool)
            .await
            .expect("load step session profile binding");
    assert_eq!(
        step_session_profile_id.as_deref(),
        Some(dev_profile_id.as_str())
    );
}

#[tokio::test]
async fn legacy_group_run_start_without_profile_tables_or_columns_still_seeds_run() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    seed_default_model_config(&pool).await;
    seed_group_run_employee(&pool, "project_manager", "builtin-general").await;
    seed_group_run_employee(&pool, "dev_team", "delivery-skill").await;
    sqlx::query("DROP TABLE agent_profiles")
        .execute(&pool)
        .await
        .expect("drop agent_profiles");
    drop_legacy_profile_binding_columns(&pool).await;

    let group_id = create_employee_group_with_pool(
        &pool,
        CreateEmployeeGroupInput {
            name: "Legacy no profile schema team".to_string(),
            coordinator_employee_id: "project_manager".to_string(),
            member_employee_ids: vec!["project_manager".to_string(), "dev_team".to_string()],
        },
    )
    .await
    .expect("create legacy no-profile group");

    let outcome = start_employee_group_run_with_pool(
        &pool,
        StartEmployeeGroupRunInput {
            group_id,
            user_goal: "legacy profile fallback".to_string(),
            execution_window: 2,
            max_retry_per_step: 1,
            timeout_employee_ids: vec![],
        },
    )
    .await
    .expect("start legacy no-profile group run");

    assert!(outcome
        .steps
        .iter()
        .all(|step| step.assignee_profile_id.is_none()));
    assert!(outcome
        .steps
        .iter()
        .all(|step| step.dispatch_source_profile_id.is_none()));

    let (session_employee_id,): (String,) =
        sqlx::query_as("SELECT employee_id FROM sessions WHERE id = ?")
            .bind(&outcome.session_id)
            .fetch_one(&pool)
            .await
            .expect("load legacy seeded group run session");
    assert_eq!(session_employee_id, "project_manager");

    let (step_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM group_run_steps WHERE run_id = ?")
            .bind(&outcome.run_id)
            .fetch_one(&pool)
            .await
            .expect("count legacy profile-free group run steps");
    assert!(step_count >= 2);
}

#[tokio::test]
async fn partial_agent_profiles_without_legacy_employee_column_still_seeds_session() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    seed_default_model_config(&pool).await;
    seed_group_run_employee(&pool, "project_manager", "builtin-general").await;
    seed_group_run_employee(&pool, "dev_team", "delivery-skill").await;
    drop_agent_profiles_legacy_employee_column(&pool).await;

    let group_id = create_employee_group_with_pool(
        &pool,
        CreateEmployeeGroupInput {
            name: "Partial profile schema team".to_string(),
            coordinator_employee_id: "project_manager".to_string(),
            member_employee_ids: vec!["project_manager".to_string(), "dev_team".to_string()],
        },
    )
    .await
    .expect("create partial profile schema group");

    let outcome = start_employee_group_run_with_pool(
        &pool,
        StartEmployeeGroupRunInput {
            group_id,
            user_goal: "partial legacy_employee_row_id fallback".to_string(),
            execution_window: 2,
            max_retry_per_step: 1,
            timeout_employee_ids: vec![],
        },
    )
    .await
    .expect("start partial profile schema group run");

    let (coordinator_session_profile_id,): (Option<String>,) =
        sqlx::query_as("SELECT profile_id FROM sessions WHERE id = ?")
            .bind(&outcome.session_id)
            .fetch_one(&pool)
            .await
            .expect("load partial profile schema session profile binding");
    assert!(coordinator_session_profile_id.is_none());
    assert!(outcome
        .steps
        .iter()
        .all(|step| step.assignee_profile_id.is_none()));
}

#[tokio::test]
async fn group_run_review_reject_keeps_revision_plan_profile_binding() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    seed_default_model_config(&pool).await;

    let (_pm_row_id, pm_profile_id) =
        seed_group_run_employee(&pool, "project_manager", "builtin-general").await;
    let (_reviewer_row_id, reviewer_profile_id) =
        seed_group_run_employee(&pool, "review_team", "review-skill").await;

    let group_id = create_employee_group_with_pool(
        &pool,
        CreateEmployeeGroupInput {
            name: "Profile 复审战队".to_string(),
            coordinator_employee_id: "project_manager".to_string(),
            member_employee_ids: vec!["project_manager".to_string(), "review_team".to_string()],
        },
    )
    .await
    .expect("create reviewable profile-bound group");

    sqlx::query(
        "UPDATE employee_groups
         SET review_mode = 'hard', entry_employee_id = 'project_manager'
         WHERE id = ?",
    )
    .bind(&group_id)
    .execute(&pool)
    .await
    .expect("enable hard review");
    sqlx::query(
        "INSERT INTO employee_group_rules (
            id, group_id, from_employee_id, to_employee_id, relation_type, phase_scope, required, priority, created_at
         ) VALUES (?, ?, ?, ?, 'review', 'plan', 1, 100, ?)",
    )
    .bind("rule-profile-review-reject")
    .bind(&group_id)
    .bind("project_manager")
    .bind("review_team")
    .bind("2026-03-09T00:00:00Z")
    .execute(&pool)
    .await
    .expect("seed review rule");

    let outcome = start_employee_group_run_with_pool(
        &pool,
        StartEmployeeGroupRunInput {
            group_id,
            user_goal: "验证退回后的 profile 绑定".to_string(),
            execution_window: 2,
            max_retry_per_step: 1,
            timeout_employee_ids: vec![],
        },
    )
    .await
    .expect("start reviewable profile-bound run");

    let (review_step_id, review_assignee_profile_id, review_dispatch_profile_id): (
        String,
        Option<String>,
        Option<String>,
    ) = sqlx::query_as(
        "SELECT id, assignee_profile_id, dispatch_source_profile_id
         FROM group_run_steps
         WHERE run_id = ? AND step_type = 'review'
         LIMIT 1",
    )
    .bind(&outcome.run_id)
    .fetch_one(&pool)
    .await
    .expect("load review step profile binding");
    assert_eq!(
        review_assignee_profile_id.as_deref(),
        Some(reviewer_profile_id.as_str())
    );
    assert!(review_dispatch_profile_id.is_none());

    review_group_run_step_with_pool(&pool, &outcome.run_id, "reject", "补充风险说明")
        .await
        .expect("reject review");

    let (
        revision_step_id,
        revision_assignee_employee_id,
        revision_assignee_profile_id,
        revision_dispatch_profile_id,
    ): (String, String, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT id, assignee_employee_id, assignee_profile_id, dispatch_source_profile_id
         FROM group_run_steps
         WHERE run_id = ? AND step_type = 'plan' AND parent_step_id = ?
         LIMIT 1",
    )
    .bind(&outcome.run_id)
    .bind(&review_step_id)
    .fetch_one(&pool)
    .await
    .expect("load rejected review revision plan step profile binding");
    assert_eq!(revision_assignee_employee_id, "project_manager");
    assert_eq!(
        revision_assignee_profile_id.as_deref(),
        Some(pm_profile_id.as_str())
    );
    assert!(revision_dispatch_profile_id.is_none());

    let (revision_event_payload_json,): (String,) = sqlx::query_as(
        "SELECT payload_json
         FROM group_run_events
         WHERE run_id = ? AND step_id = ? AND event_type = 'step_created'
         LIMIT 1",
    )
    .bind(&outcome.run_id)
    .bind(&revision_step_id)
    .fetch_one(&pool)
    .await
    .expect("load profile-bound revision step event");
    assert!(revision_event_payload_json
        .contains(&format!("\"assignee_profile_id\":\"{pm_profile_id}\"")));
}

#[tokio::test]
async fn legacy_review_reject_without_profile_step_columns_still_creates_revision_step() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    seed_default_model_config(&pool).await;

    seed_group_run_employee(&pool, "project_manager", "builtin-general").await;
    seed_group_run_employee(&pool, "review_team", "review-skill").await;

    let group_id = create_employee_group_with_pool(
        &pool,
        CreateEmployeeGroupInput {
            name: "Legacy review reject team".to_string(),
            coordinator_employee_id: "project_manager".to_string(),
            member_employee_ids: vec!["project_manager".to_string(), "review_team".to_string()],
        },
    )
    .await
    .expect("create legacy review group");

    sqlx::query(
        "UPDATE employee_groups
         SET review_mode = 'hard', entry_employee_id = 'project_manager'
         WHERE id = ?",
    )
    .bind(&group_id)
    .execute(&pool)
    .await
    .expect("enable hard review");
    sqlx::query(
        "INSERT INTO employee_group_rules (
            id, group_id, from_employee_id, to_employee_id, relation_type, phase_scope, required, priority, created_at
         ) VALUES ('rule-legacy-review', ?, 'project_manager', 'review_team', 'review', 'plan', 1, 100, '2026-03-09T00:00:00Z')",
    )
    .bind(&group_id)
    .execute(&pool)
    .await
    .expect("seed legacy review rule");

    let outcome = start_employee_group_run_with_pool(
        &pool,
        StartEmployeeGroupRunInput {
            group_id,
            user_goal: "legacy reject fallback".to_string(),
            execution_window: 2,
            max_retry_per_step: 1,
            timeout_employee_ids: vec![],
        },
    )
    .await
    .expect("start legacy review group run");

    let (review_step_id,): (String,) = sqlx::query_as(
        "SELECT id
         FROM group_run_steps
         WHERE run_id = ? AND step_type = 'review'
         LIMIT 1",
    )
    .bind(&outcome.run_id)
    .fetch_one(&pool)
    .await
    .expect("load legacy review step");

    drop_agent_profiles_legacy_employee_column(&pool).await;
    sqlx::query("ALTER TABLE group_run_steps DROP COLUMN assignee_profile_id")
        .execute(&pool)
        .await
        .expect("drop step assignee profile column before reject");
    sqlx::query("ALTER TABLE group_run_steps DROP COLUMN dispatch_source_profile_id")
        .execute(&pool)
        .await
        .expect("drop step dispatch profile column before reject");

    review_group_run_step_with_pool(&pool, &outcome.run_id, "reject", "补充风险说明")
        .await
        .expect("reject review on legacy profile step schema");

    let (revision_assignee_employee_id, revision_status): (String, String) = sqlx::query_as(
        "SELECT assignee_employee_id, status
         FROM group_run_steps
         WHERE run_id = ? AND step_type = 'plan' AND parent_step_id = ?
         LIMIT 1",
    )
    .bind(&outcome.run_id)
    .bind(&review_step_id)
    .fetch_one(&pool)
    .await
    .expect("load legacy rejected review revision step");
    assert_eq!(revision_assignee_employee_id, "project_manager");
    assert_eq!(revision_status, "pending");
}

#[tokio::test]
async fn group_run_reassign_updates_step_and_session_profile_binding() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    seed_default_model_config(&pool).await;

    let (_pm_row_id, pm_profile_id) =
        seed_group_run_employee(&pool, "project_manager", "builtin-general").await;
    let (_dev_row_id, dev_profile_id) =
        seed_group_run_employee(&pool, "dev_team", "delivery-skill").await;
    let (_qa_row_id, qa_profile_id) = seed_group_run_employee(&pool, "qa_team", "qa-skill").await;

    let group_id = create_employee_group_with_pool(
        &pool,
        CreateEmployeeGroupInput {
            name: "Profile 改派战队".to_string(),
            coordinator_employee_id: "project_manager".to_string(),
            member_employee_ids: vec![
                "project_manager".to_string(),
                "dev_team".to_string(),
                "qa_team".to_string(),
            ],
        },
    )
    .await
    .expect("create reassignable profile-bound group");

    let outcome = start_employee_group_run_with_pool(
        &pool,
        StartEmployeeGroupRunInput {
            group_id,
            user_goal: "验证改派后的 profile 绑定".to_string(),
            execution_window: 3,
            max_retry_per_step: 1,
            timeout_employee_ids: vec![],
        },
    )
    .await
    .expect("start reassignable profile-bound run");

    let (step_id, original_assignee_profile_id, original_dispatch_profile_id): (
        String,
        Option<String>,
        Option<String>,
    ) = sqlx::query_as(
        "SELECT id, assignee_profile_id, dispatch_source_profile_id
         FROM group_run_steps
         WHERE run_id = ? AND step_type = 'execute' AND assignee_employee_id = 'dev_team'
         LIMIT 1",
    )
    .bind(&outcome.run_id)
    .fetch_one(&pool)
    .await
    .expect("load dev execute step profile binding");
    assert_eq!(
        original_assignee_profile_id.as_deref(),
        Some(dev_profile_id.as_str())
    );
    assert_eq!(
        original_dispatch_profile_id.as_deref(),
        Some(pm_profile_id.as_str())
    );

    sqlx::query(
        "UPDATE group_run_steps
         SET status = 'failed',
             output = '原负责人失败',
             output_summary = '原负责人失败',
             session_id = 'session-old'
         WHERE id = ?",
    )
    .bind(&step_id)
    .execute(&pool)
    .await
    .expect("mark profile-bound step failed");

    reassign_group_run_step_with_pool(&pool, &step_id, "qa_team")
        .await
        .expect("reassign profile-bound step");

    let (assignee_employee_id, assignee_profile_id, dispatch_profile_id, status, session_id): (
        String,
        Option<String>,
        Option<String>,
        String,
        String,
    ) = sqlx::query_as(
        "SELECT assignee_employee_id, assignee_profile_id, dispatch_source_profile_id, status, session_id
         FROM group_run_steps
         WHERE id = ?",
    )
    .bind(&step_id)
    .fetch_one(&pool)
    .await
    .expect("reload reassigned profile-bound step");
    assert_eq!(assignee_employee_id, "qa_team");
    assert_eq!(assignee_profile_id.as_deref(), Some(qa_profile_id.as_str()));
    assert_eq!(dispatch_profile_id.as_deref(), Some(pm_profile_id.as_str()));
    assert_eq!(status, "pending");
    assert_eq!(session_id, "");

    let (reassign_event_payload_json,): (String,) = sqlx::query_as(
        "SELECT payload_json
         FROM group_run_events
         WHERE run_id = ? AND step_id = ? AND event_type = 'step_reassigned'
         LIMIT 1",
    )
    .bind(&outcome.run_id)
    .bind(&step_id)
    .fetch_one(&pool)
    .await
    .expect("load profile-bound reassign event");
    assert!(reassign_event_payload_json
        .contains(&format!("\"assignee_profile_id\":\"{qa_profile_id}\"")));

    let executed = run_group_step_with_pool(&pool, &step_id)
        .await
        .expect("run reassigned profile-bound step");
    let (step_session_profile_id,): (Option<String>,) =
        sqlx::query_as("SELECT profile_id FROM sessions WHERE id = ?")
            .bind(&executed.session_id)
            .fetch_one(&pool)
            .await
            .expect("load reassigned step session profile binding");
    assert_eq!(
        step_session_profile_id.as_deref(),
        Some(qa_profile_id.as_str())
    );
}

#[tokio::test]
async fn partial_group_run_steps_without_assignee_profile_column_still_reassigns() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    seed_default_model_config(&pool).await;

    seed_group_run_employee(&pool, "project_manager", "builtin-general").await;
    seed_group_run_employee(&pool, "dev_team", "delivery-skill").await;
    seed_group_run_employee(&pool, "qa_team", "qa-skill").await;

    let group_id = create_employee_group_with_pool(
        &pool,
        CreateEmployeeGroupInput {
            name: "Partial reassign team".to_string(),
            coordinator_employee_id: "project_manager".to_string(),
            member_employee_ids: vec![
                "project_manager".to_string(),
                "dev_team".to_string(),
                "qa_team".to_string(),
            ],
        },
    )
    .await
    .expect("create partial reassign group");

    let outcome = start_employee_group_run_with_pool(
        &pool,
        StartEmployeeGroupRunInput {
            group_id,
            user_goal: "partial reassign fallback".to_string(),
            execution_window: 3,
            max_retry_per_step: 1,
            timeout_employee_ids: vec![],
        },
    )
    .await
    .expect("start partial reassign run");

    let (step_id,): (String,) = sqlx::query_as(
        "SELECT id
         FROM group_run_steps
         WHERE run_id = ? AND step_type = 'execute' AND assignee_employee_id = 'dev_team'
         LIMIT 1",
    )
    .bind(&outcome.run_id)
    .fetch_one(&pool)
    .await
    .expect("load partial reassign execute step");

    sqlx::query(
        "UPDATE group_run_steps
         SET status = 'failed',
             output = '原负责人失败',
             output_summary = '原负责人失败',
             session_id = 'session-old'
         WHERE id = ?",
    )
    .bind(&step_id)
    .execute(&pool)
    .await
    .expect("mark partial reassign step failed");
    sqlx::query("ALTER TABLE group_run_steps DROP COLUMN assignee_profile_id")
        .execute(&pool)
        .await
        .expect("drop assignee profile column before reassignment");

    reassign_group_run_step_with_pool(&pool, &step_id, "qa_team")
        .await
        .expect("reassign without assignee profile column");

    let (assignee_employee_id, status, session_id): (String, String, String) = sqlx::query_as(
        "SELECT assignee_employee_id, status, session_id
         FROM group_run_steps
         WHERE id = ?",
    )
    .bind(&step_id)
    .fetch_one(&pool)
    .await
    .expect("load reassigned partial profile step");
    assert_eq!(assignee_employee_id, "qa_team");
    assert_eq!(status, "pending");
    assert_eq!(session_id, "");
}

#[tokio::test]
async fn group_run_keeps_profile_ids_empty_without_real_profile_rows() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    seed_default_model_config(&pool).await;
    seed_group_run_employee(&pool, "project_manager", "builtin-general").await;
    seed_group_run_employee(&pool, "dev_team", "delivery-skill").await;
    sqlx::query("DELETE FROM agent_profiles")
        .execute(&pool)
        .await
        .expect("remove profile rows to simulate legacy employees without real profiles");

    let group_id = create_employee_group_with_pool(
        &pool,
        CreateEmployeeGroupInput {
            name: "Legacy alias 战队".to_string(),
            coordinator_employee_id: "project_manager".to_string(),
            member_employee_ids: vec!["project_manager".to_string(), "dev_team".to_string()],
        },
    )
    .await
    .expect("create legacy group");

    let outcome = start_employee_group_run_with_pool(
        &pool,
        StartEmployeeGroupRunInput {
            group_id,
            user_goal: "验证无真实 profile 时不伪造 profile_id".to_string(),
            execution_window: 2,
            max_retry_per_step: 1,
            timeout_employee_ids: vec![],
        },
    )
    .await
    .expect("start legacy group run");

    assert!(outcome
        .steps
        .iter()
        .all(|step| step.assignee_profile_id.is_none()));
    assert!(outcome
        .steps
        .iter()
        .all(|step| step.dispatch_source_profile_id.is_none()));

    let (bound_profile_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM group_run_steps
         WHERE run_id = ?
           AND (COALESCE(assignee_profile_id, '') != ''
                OR COALESCE(dispatch_source_profile_id, '') != '')",
    )
    .bind(&outcome.run_id)
    .fetch_one(&pool)
    .await
    .expect("count persisted synthetic profile bindings");
    assert_eq!(bound_profile_count, 0);

    let (coordinator_session_profile_id,): (Option<String>,) =
        sqlx::query_as("SELECT profile_id FROM sessions WHERE id = ?")
            .bind(&outcome.session_id)
            .fetch_one(&pool)
            .await
            .expect("load coordinator session profile binding");
    assert!(coordinator_session_profile_id
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty());
}

#[tokio::test]
async fn legacy_missing_agent_profiles_table_starts_group_run_without_profile_binding() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    seed_default_model_config(&pool).await;
    seed_group_run_employee(&pool, "project_manager", "builtin-general").await;
    seed_group_run_employee(&pool, "dev_team", "delivery-skill").await;

    let group_id = create_employee_group_with_pool(
        &pool,
        CreateEmployeeGroupInput {
            name: "缺 agent_profiles 旧库战队".to_string(),
            coordinator_employee_id: "project_manager".to_string(),
            member_employee_ids: vec!["project_manager".to_string(), "dev_team".to_string()],
        },
    )
    .await
    .expect("create legacy missing-profile-table group");
    sqlx::query("DROP TABLE agent_profiles")
        .execute(&pool)
        .await
        .expect("simulate legacy db without agent_profiles table");

    let outcome = start_employee_group_run_with_pool(
        &pool,
        StartEmployeeGroupRunInput {
            group_id,
            user_goal: "验证缺 agent_profiles 表时仍能启动".to_string(),
            execution_window: 2,
            max_retry_per_step: 1,
            timeout_employee_ids: vec![],
        },
    )
    .await
    .expect("start should not require agent_profiles table");

    assert!(outcome
        .steps
        .iter()
        .all(|step| step.assignee_profile_id.is_none()));
    assert!(outcome
        .steps
        .iter()
        .all(|step| step.dispatch_source_profile_id.is_none()));
}

#[tokio::test]
async fn legacy_agent_profiles_without_mapping_column_starts_and_rejects_review() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    seed_default_model_config(&pool).await;
    seed_group_run_employee(&pool, "project_manager", "builtin-general").await;
    seed_group_run_employee(&pool, "review_team", "review-skill").await;

    let group_id = create_employee_group_with_pool(
        &pool,
        CreateEmployeeGroupInput {
            name: "缺 legacy 映射列复审战队".to_string(),
            coordinator_employee_id: "project_manager".to_string(),
            member_employee_ids: vec!["project_manager".to_string(), "review_team".to_string()],
        },
    )
    .await
    .expect("create legacy missing-mapping-column group");
    sqlx::query(
        "UPDATE employee_groups
         SET review_mode = 'hard', entry_employee_id = 'project_manager'
         WHERE id = ?",
    )
    .bind(&group_id)
    .execute(&pool)
    .await
    .expect("enable hard review");
    sqlx::query(
        "INSERT INTO employee_group_rules (
            id, group_id, from_employee_id, to_employee_id, relation_type, phase_scope, required, priority, created_at
         ) VALUES (?, ?, ?, ?, 'review', 'plan', 1, 100, ?)",
    )
    .bind("rule-legacy-profile-review-reject")
    .bind(&group_id)
    .bind("project_manager")
    .bind("review_team")
    .bind("2026-03-09T00:00:00Z")
    .execute(&pool)
    .await
    .expect("seed legacy review rule");
    recreate_agent_profiles_without_legacy_mapping(&pool).await;

    let outcome = start_employee_group_run_with_pool(
        &pool,
        StartEmployeeGroupRunInput {
            group_id,
            user_goal: "验证缺 legacy_employee_row_id 时复审退回不崩溃".to_string(),
            execution_window: 2,
            max_retry_per_step: 1,
            timeout_employee_ids: vec![],
        },
    )
    .await
    .expect("start should not require agent_profiles.legacy_employee_row_id");
    review_group_run_step_with_pool(&pool, &outcome.run_id, "reject", "继续补充")
        .await
        .expect("reject should not require agent_profiles.legacy_employee_row_id");

    let snapshot = get_employee_group_run_snapshot_with_pool(&pool, &outcome.session_id)
        .await
        .expect("snapshot after legacy reject should not error")
        .expect("snapshot exists");
    assert!(snapshot
        .steps
        .iter()
        .all(|step| step.assignee_profile_id.is_none()));
}

#[tokio::test]
async fn legacy_sessions_without_profile_id_column_still_start_group_run() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    seed_default_model_config(&pool).await;
    seed_group_run_employee(&pool, "project_manager", "builtin-general").await;
    seed_group_run_employee(&pool, "dev_team", "delivery-skill").await;

    let group_id = create_employee_group_with_pool(
        &pool,
        CreateEmployeeGroupInput {
            name: "缺 sessions.profile_id 旧库战队".to_string(),
            coordinator_employee_id: "project_manager".to_string(),
            member_employee_ids: vec!["project_manager".to_string(), "dev_team".to_string()],
        },
    )
    .await
    .expect("create legacy sessions group");
    rebuild_sessions_without_profile_id(&pool).await;

    start_employee_group_run_with_pool(
        &pool,
        StartEmployeeGroupRunInput {
            group_id,
            user_goal: "验证 sessions 无 profile_id 时仍能启动".to_string(),
            execution_window: 2,
            max_retry_per_step: 1,
            timeout_employee_ids: vec![],
        },
    )
    .await
    .expect("start should not require sessions.profile_id");
}

#[tokio::test]
async fn legacy_group_run_step_profile_columns_missing_still_start_reassign_and_reject() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    seed_default_model_config(&pool).await;
    seed_group_run_employee(&pool, "project_manager", "builtin-general").await;
    seed_group_run_employee(&pool, "dev_team", "delivery-skill").await;
    seed_group_run_employee(&pool, "qa_team", "qa-skill").await;

    let group_id = create_employee_group_with_pool(
        &pool,
        CreateEmployeeGroupInput {
            name: "缺步骤 profile 列旧库战队".to_string(),
            coordinator_employee_id: "project_manager".to_string(),
            member_employee_ids: vec![
                "project_manager".to_string(),
                "dev_team".to_string(),
                "qa_team".to_string(),
            ],
        },
    )
    .await
    .expect("create legacy step-column group");
    sqlx::query(
        "UPDATE employee_groups
         SET review_mode = 'hard', entry_employee_id = 'project_manager'
         WHERE id = ?",
    )
    .bind(&group_id)
    .execute(&pool)
    .await
    .expect("enable hard review for legacy step-column group");
    sqlx::query(
        "INSERT INTO employee_group_rules (
            id, group_id, from_employee_id, to_employee_id, relation_type, phase_scope, required, priority, created_at
         ) VALUES (?, ?, ?, ?, 'review', 'plan', 1, 100, ?)",
    )
    .bind("rule-legacy-step-column-review")
    .bind(&group_id)
    .bind("project_manager")
    .bind("qa_team")
    .bind("2026-03-09T00:00:00Z")
    .execute(&pool)
    .await
    .expect("seed legacy step-column review rule");
    rebuild_group_run_steps_without_profile_columns(&pool).await;

    let outcome = start_employee_group_run_with_pool(
        &pool,
        StartEmployeeGroupRunInput {
            group_id,
            user_goal: "验证 group_run_steps 无 profile 列时写路径兼容".to_string(),
            execution_window: 3,
            max_retry_per_step: 1,
            timeout_employee_ids: vec![],
        },
    )
    .await
    .expect("start should not require group_run_steps profile columns");

    let (step_id,): (String,) = sqlx::query_as(
        "SELECT id
         FROM group_run_steps
         WHERE run_id = ? AND step_type = 'execute' AND assignee_employee_id = 'dev_team'
         LIMIT 1",
    )
    .bind(&outcome.run_id)
    .fetch_one(&pool)
    .await
    .expect("load legacy execute step");
    sqlx::query(
        "UPDATE group_run_steps
         SET status = 'failed', output = '失败', output_summary = '失败', session_id = 'session-old'
         WHERE id = ?",
    )
    .bind(&step_id)
    .execute(&pool)
    .await
    .expect("mark legacy step failed");
    reassign_group_run_step_with_pool(&pool, &step_id, "qa_team")
        .await
        .expect("reassign should not require assignee_profile_id column");
    review_group_run_step_with_pool(&pool, &outcome.run_id, "reject", "补充计划")
        .await
        .expect("reject should not require group_run_steps profile columns");
}

#[tokio::test]
async fn legacy_group_run_steps_without_profile_columns_still_load_snapshot() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("create legacy sqlite pool");

    sqlx::query(
        "CREATE TABLE group_runs (
            id TEXT PRIMARY KEY,
            group_id TEXT NOT NULL,
            session_id TEXT NOT NULL DEFAULT '',
            user_goal TEXT NOT NULL DEFAULT '',
            state TEXT NOT NULL DEFAULT 'planning',
            current_round INTEGER NOT NULL DEFAULT 0,
            current_phase TEXT NOT NULL DEFAULT 'plan',
            review_round INTEGER NOT NULL DEFAULT 0,
            status_reason TEXT NOT NULL DEFAULT '',
            waiting_for_employee_id TEXT NOT NULL DEFAULT '',
            waiting_for_user INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("create legacy group_runs");
    sqlx::query(
        "CREATE TABLE group_run_steps (
            id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            round_no INTEGER NOT NULL DEFAULT 1,
            step_type TEXT NOT NULL,
            assignee_employee_id TEXT NOT NULL,
            dispatch_source_employee_id TEXT NOT NULL DEFAULT '',
            session_id TEXT NOT NULL DEFAULT '',
            attempt_no INTEGER NOT NULL DEFAULT 1,
            status TEXT NOT NULL DEFAULT 'pending',
            output_summary TEXT NOT NULL DEFAULT '',
            output TEXT NOT NULL DEFAULT '',
            started_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("create legacy group_run_steps");
    sqlx::query(
        "CREATE TABLE group_run_events (
            id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            step_id TEXT NOT NULL DEFAULT '',
            event_type TEXT NOT NULL,
            payload_json TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("create legacy group_run_events");
    sqlx::query(
        "CREATE TABLE messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("create messages");

    sqlx::query(
        "INSERT INTO group_runs (
            id, group_id, session_id, user_goal, state, current_round, current_phase,
            review_round, status_reason, waiting_for_employee_id, waiting_for_user, created_at
         ) VALUES (
            'run-legacy', 'group-legacy', 'session-legacy', '兼容旧步骤', 'running', 1,
            'execute', 0, '', 'project_manager', 0, '2026-03-09T00:00:00Z'
         )",
    )
    .execute(&pool)
    .await
    .expect("insert legacy run");
    sqlx::query(
        "INSERT INTO group_run_steps (
            id, run_id, round_no, step_type, assignee_employee_id,
            dispatch_source_employee_id, session_id, attempt_no, status,
            output_summary, output, started_at
         ) VALUES (
            'step-legacy', 'run-legacy', 1, 'execute', 'dev_team',
            'project_manager', '', 1, 'pending', '', '', '2026-03-09T00:00:01Z'
         )",
    )
    .execute(&pool)
    .await
    .expect("insert legacy step");

    let snapshot = get_employee_group_run_snapshot_with_pool(&pool, "session-legacy")
        .await
        .expect("legacy snapshot should not error")
        .expect("legacy snapshot exists");
    assert_eq!(snapshot.run_id, "run-legacy");
    assert_eq!(snapshot.steps.len(), 1);
    assert_eq!(snapshot.steps[0].assignee_employee_id, "dev_team");
    assert_eq!(
        snapshot.steps[0].dispatch_source_employee_id,
        "project_manager"
    );
    assert!(snapshot.steps[0].assignee_profile_id.is_none());
    assert!(snapshot.steps[0].dispatch_source_profile_id.is_none());
}
