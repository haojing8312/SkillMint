use sqlx::{Row, Sqlite, SqlitePool, Transaction};

pub(crate) struct GroupRunStateRow {
    pub state: String,
    pub current_phase: String,
}

pub(crate) struct FailedGroupRunStepRow {
    pub step_id: String,
    pub output: String,
}

pub(crate) struct GroupRunStepReassignRow {
    pub run_id: String,
    pub status: String,
    pub step_type: String,
    pub dispatch_source_employee_id: String,
    pub previous_assignee_employee_id: String,
    pub previous_output_summary: String,
    pub previous_output: String,
}

pub(crate) struct GroupRunExecuteStepContextRow {
    pub step_id: String,
    pub run_id: String,
    pub assignee_employee_id: String,
    pub dispatch_source_employee_id: String,
    pub step_type: String,
    pub existing_session_id: String,
    pub step_input: String,
    pub user_goal: String,
}

pub(crate) struct GroupStepSessionRow {
    pub skill_id: String,
    pub model_id: String,
    pub work_dir: String,
}

pub(crate) struct EmployeeSessionSeedRow {
    pub primary_skill_id: String,
    pub default_work_dir: String,
}

pub(crate) struct GroupRunStartConfigRow {
    pub name: String,
    pub coordinator_employee_id: String,
    pub member_employee_ids_json: String,
    pub review_mode: String,
    pub entry_employee_id: String,
}

pub(crate) struct ModelConfigRow {
    pub api_format: String,
    pub base_url: String,
    pub model_name: String,
    pub api_key: String,
}

pub(crate) struct SessionMessageRow {
    pub role: String,
    pub content: String,
}

pub(crate) struct PendingReviewStepRow {
    pub step_id: String,
    pub assignee_employee_id: String,
}

pub(crate) struct GroupRunReviewStateRow {
    pub main_employee_id: String,
    pub review_round: i64,
    pub review_step_id: String,
}

pub(crate) struct PlanRevisionSeedRow {
    pub input: String,
    pub assignee_employee_id: String,
}

pub(crate) struct GroupRunFinalizeStateRow {
    pub session_id: String,
    pub user_goal: String,
    pub state: String,
}

pub(crate) struct GroupRunSnapshotRow {
    pub run_id: String,
    pub group_id: String,
    pub session_id: String,
    pub state: String,
    pub current_round: i64,
    pub user_goal: String,
    pub current_phase: String,
    pub review_round: i64,
    pub status_reason: String,
    pub waiting_for_employee_id: String,
    pub waiting_for_user: bool,
}

pub(crate) struct GroupRunStepSnapshotRow {
    pub id: String,
    pub round_no: i64,
    pub step_type: String,
    pub assignee_employee_id: String,
    pub dispatch_source_employee_id: String,
    pub session_id: String,
    pub attempt_no: i64,
    pub status: String,
    pub output_summary: String,
    pub output: String,
}

pub(crate) struct GroupRunEventSnapshotRow {
    pub id: String,
    pub step_id: String,
    pub event_type: String,
    pub payload_json: String,
    pub created_at: String,
}

pub(crate) async fn find_group_step_session_row(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Option<GroupStepSessionRow>, String> {
    let row = sqlx::query(
        "SELECT skill_id, model_id, COALESCE(work_dir, '')
         FROM sessions
         WHERE id = ?",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|record| GroupStepSessionRow {
        skill_id: record.try_get(0).expect("group step session skill_id"),
        model_id: record.try_get(1).expect("group step session model_id"),
        work_dir: record.try_get(2).expect("group step session work_dir"),
    }))
}

pub(crate) async fn find_group_run_start_config(
    pool: &SqlitePool,
    group_id: &str,
) -> Result<Option<GroupRunStartConfigRow>, String> {
    let row = sqlx::query(
        "SELECT name,
                coordinator_employee_id,
                member_employee_ids_json,
                COALESCE(review_mode, 'none'),
                COALESCE(entry_employee_id, '')
         FROM employee_groups WHERE id = ?",
    )
    .bind(group_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|record| GroupRunStartConfigRow {
        name: record.try_get("name").expect("group start name"),
        coordinator_employee_id: record
            .try_get("coordinator_employee_id")
            .expect("group start coordinator_employee_id"),
        member_employee_ids_json: record
            .try_get("member_employee_ids_json")
            .expect("group start member_employee_ids_json"),
        review_mode: record.try_get(3).expect("group start review_mode"),
        entry_employee_id: record.try_get(4).expect("group start entry_employee_id"),
    }))
}

pub(crate) async fn find_existing_session_skill_id(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Option<String>, String> {
    let row = sqlx::query_as::<_, (String,)>(
        "SELECT COALESCE(skill_id, '') FROM sessions WHERE id = ?",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(row.map(|(skill_id,)| skill_id))
}

pub(crate) async fn find_employee_session_seed_row(
    pool: &SqlitePool,
    employee_id: &str,
) -> Result<Option<EmployeeSessionSeedRow>, String> {
    let row = sqlx::query(
        "SELECT primary_skill_id, default_work_dir
         FROM agent_employees
         WHERE lower(employee_id) = lower(?) OR lower(role_id) = lower(?)
         ORDER BY is_default DESC, updated_at DESC
         LIMIT 1",
    )
    .bind(employee_id)
    .bind(employee_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|record| EmployeeSessionSeedRow {
        primary_skill_id: record
            .try_get("primary_skill_id")
            .expect("employee session seed primary_skill_id"),
        default_work_dir: record
            .try_get("default_work_dir")
            .expect("employee session seed default_work_dir"),
    }))
}

pub(crate) async fn find_recent_group_step_session_id(
    pool: &SqlitePool,
    run_id: &str,
    assignee_employee_id: &str,
) -> Result<Option<String>, String> {
    let row = sqlx::query_as::<_, (String,)>(
        "SELECT session_id
         FROM group_run_steps
         WHERE run_id = ? AND assignee_employee_id = ? AND TRIM(session_id) <> ''
         ORDER BY finished_at DESC, started_at DESC
         LIMIT 1",
    )
    .bind(run_id)
    .bind(assignee_employee_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(row.map(|(session_id,)| session_id))
}

pub(crate) async fn find_model_config_row(
    pool: &SqlitePool,
    model_id: &str,
) -> Result<Option<ModelConfigRow>, String> {
    let row = sqlx::query(
        "SELECT api_format, base_url, model_name, api_key
         FROM model_configs
         WHERE id = ?",
    )
    .bind(model_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|record| ModelConfigRow {
        api_format: record.try_get(0).expect("model config api_format"),
        base_url: record.try_get(1).expect("model config base_url"),
        model_name: record.try_get(2).expect("model config model_name"),
        api_key: record.try_get(3).expect("model config api_key"),
    }))
}

pub(crate) async fn insert_session_message(
    pool: &SqlitePool,
    session_id: &str,
    role: &str,
    content: &str,
    created_at: &str,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO messages (id, session_id, role, content, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(session_id)
    .bind(role)
    .bind(content)
    .bind(created_at)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn list_session_message_rows(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Vec<SessionMessageRow>, String> {
    let rows = sqlx::query_as::<_, (String, String)>(
        "SELECT role, content FROM messages WHERE session_id = ? ORDER BY created_at ASC",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|(role, content)| SessionMessageRow { role, content })
        .collect())
}

pub(crate) async fn pause_group_run(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    reason: &str,
    now: &str,
) -> Result<u64, String> {
    let result = sqlx::query(
        "UPDATE group_runs
         SET state = 'paused',
             status_reason = ?,
             updated_at = ?
         WHERE id = ? AND state NOT IN ('done', 'failed', 'cancelled', 'paused')",
    )
    .bind(reason)
    .bind(now)
    .bind(run_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(result.rows_affected())
}

pub(crate) async fn find_group_run_state(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<Option<GroupRunStateRow>, String> {
    let row = sqlx::query(
        "SELECT state, COALESCE(current_phase, 'plan')
         FROM group_runs
         WHERE id = ?",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|record| GroupRunStateRow {
        state: record.try_get(0).expect("group run state"),
        current_phase: record.try_get(1).expect("group run current_phase"),
    }))
}

pub(crate) async fn find_group_run_review_state(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<Option<GroupRunReviewStateRow>, String> {
    let run_row = sqlx::query(
        "SELECT COALESCE(main_employee_id, ''), COALESCE(review_round, 0)
         FROM group_runs
         WHERE id = ?",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    let Some(run_row) = run_row else {
        return Ok(None);
    };

    let review_step_row = sqlx::query_as::<_, (String,)>(
        "SELECT id
         FROM group_run_steps
         WHERE run_id = ? AND step_type = 'review'
         ORDER BY round_no DESC, id DESC
         LIMIT 1",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(review_step_row.map(|(review_step_id,)| GroupRunReviewStateRow {
        main_employee_id: run_row
            .try_get(0)
            .expect("group run review state main_employee_id"),
        review_round: run_row
            .try_get(1)
            .expect("group run review state review_round"),
        review_step_id,
    }))
}

pub(crate) async fn mark_review_step_completed(
    tx: &mut Transaction<'_, Sqlite>,
    review_step_id: &str,
    comment: &str,
    review_status: &str,
    now: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE group_run_steps
         SET status = 'completed',
             output = ?,
             output_summary = ?,
             review_status = ?,
             finished_at = ?
         WHERE id = ?",
    )
    .bind(comment)
    .bind(comment)
    .bind(review_status)
    .bind(now)
    .bind(review_step_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn find_plan_revision_seed(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
) -> Result<Option<PlanRevisionSeedRow>, String> {
    let row = sqlx::query_as::<_, (String, String)>(
        "SELECT COALESCE(input, ''), COALESCE(assignee_employee_id, '')
         FROM group_run_steps
         WHERE run_id = ? AND step_type = 'plan'
         ORDER BY round_no DESC, id DESC
         LIMIT 1",
    )
    .bind(run_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|(input, assignee_employee_id)| PlanRevisionSeedRow {
        input,
        assignee_employee_id,
    }))
}

pub(crate) async fn insert_plan_revision_step(
    tx: &mut Transaction<'_, Sqlite>,
    revision_step_id: &str,
    run_id: &str,
    review_step_id: &str,
    revision_assignee_employee_id: &str,
    revision_input: &str,
    comment: &str,
    next_review_round: i64,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO group_run_steps (
            id, run_id, round_no, parent_step_id, assignee_employee_id, phase, step_type, step_kind,
            input, input_summary, output, output_summary, status, requires_review, review_status,
            attempt_no, session_id, visibility, started_at, finished_at
         ) VALUES (?, ?, ?, ?, ?, 'plan', 'plan', 'plan', ?, ?, '', '', 'pending', 1, 'pending', ?, '', 'internal', '', '')",
    )
    .bind(revision_step_id)
    .bind(run_id)
    .bind(0_i64)
    .bind(review_step_id)
    .bind(revision_assignee_employee_id)
    .bind(revision_input)
    .bind(comment)
    .bind(next_review_round)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn mark_group_run_review_rejected(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    next_review_round: i64,
    comment: &str,
    revision_assignee_employee_id: &str,
    now: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE group_runs
         SET state = 'planning',
             current_phase = 'plan',
             review_round = ?,
             status_reason = ?,
             waiting_for_employee_id = ?,
             updated_at = ?
         WHERE id = ?",
    )
    .bind(next_review_round)
    .bind(comment)
    .bind(revision_assignee_employee_id)
    .bind(now)
    .bind(run_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn mark_group_run_review_approved(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    now: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE group_runs
         SET state = 'planning',
             current_phase = 'execute',
             status_reason = '',
             waiting_for_employee_id = '',
             updated_at = ?
         WHERE id = ?",
    )
    .bind(now)
    .bind(run_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn resume_group_run(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    resumed_state: &str,
    now: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE group_runs
         SET state = ?,
             status_reason = '',
             updated_at = ?
         WHERE id = ?",
    )
    .bind(resumed_state)
    .bind(now)
    .bind(run_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn insert_group_run_event(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    step_id: &str,
    event_type: &str,
    payload_json: &str,
    created_at: &str,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO group_run_events (id, run_id, step_id, event_type, payload_json, created_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(run_id)
    .bind(step_id)
    .bind(event_type)
    .bind(payload_json)
    .bind(created_at)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn insert_group_run_record(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    group_id: &str,
    session_id: &str,
    user_goal: &str,
    initial_state: &str,
    initial_round: i64,
    current_phase: &str,
    coordinator_employee_id: &str,
    waiting_for_employee_id: &str,
    now: &str,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO group_runs (
            id, group_id, session_id, user_goal, state, current_round, current_phase, entry_session_id,
            main_employee_id, review_round, status_reason, template_version, waiting_for_employee_id, waiting_for_user,
            created_at, updated_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(run_id)
    .bind(group_id)
    .bind(session_id)
    .bind(user_goal)
    .bind(initial_state)
    .bind(initial_round)
    .bind(current_phase)
    .bind(session_id)
    .bind(coordinator_employee_id)
    .bind(0_i64)
    .bind("")
    .bind("")
    .bind(waiting_for_employee_id)
    .bind(0_i64)
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn insert_tx_session_message(
    tx: &mut Transaction<'_, Sqlite>,
    session_id: &str,
    role: &str,
    content: &str,
    created_at: &str,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO messages (id, session_id, role, content, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(session_id)
    .bind(role)
    .bind(content)
    .bind(created_at)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn insert_group_run_step_seed(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    step_id: &str,
    round_no: i64,
    assignee_employee_id: &str,
    dispatch_source_employee_id: &str,
    phase: &str,
    step_type: &str,
    user_goal: &str,
    output: &str,
    status: &str,
    requires_review: bool,
    review_status: &str,
    now: &str,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO group_run_steps (
            id, run_id, round_no, parent_step_id, assignee_employee_id, dispatch_source_employee_id,
            phase, step_type, step_kind, input, input_summary, output, output_summary, status,
            requires_review, review_status, attempt_no, session_id, visibility, started_at, finished_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(step_id)
    .bind(run_id)
    .bind(round_no)
    .bind("")
    .bind(assignee_employee_id)
    .bind(dispatch_source_employee_id)
    .bind(phase)
    .bind(step_type)
    .bind(step_type)
    .bind(user_goal)
    .bind(if step_type == "plan" {
        "已生成结构化计划"
    } else {
        ""
    })
    .bind(output)
    .bind(output)
    .bind(status)
    .bind(if requires_review { 1_i64 } else { 0_i64 })
    .bind(review_status)
    .bind(0_i64)
    .bind("")
    .bind("internal")
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn cancel_group_run(
    pool: &SqlitePool,
    run_id: &str,
    now: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE group_runs
         SET state = 'cancelled', updated_at = ?
         WHERE id = ? AND state NOT IN ('done', 'failed', 'cancelled')",
    )
    .bind(now)
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn list_failed_group_run_steps(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<Vec<FailedGroupRunStepRow>, String> {
    let rows = sqlx::query("SELECT id, output FROM group_run_steps WHERE run_id = ? AND status = 'failed'")
        .bind(run_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|row| FailedGroupRunStepRow {
            step_id: row.try_get("id").expect("failed step id"),
            output: row.try_get("output").expect("failed step output"),
        })
        .collect())
}

pub(crate) async fn complete_failed_group_run_step(
    tx: &mut Transaction<'_, Sqlite>,
    step_id: &str,
    output: &str,
    now: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE group_run_steps
         SET status = 'completed', output = ?, finished_at = ?
         WHERE id = ?",
    )
    .bind(output)
    .bind(now)
    .bind(step_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn mark_group_run_done_after_retry(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    now: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE group_runs
         SET state = 'done', current_round = current_round + 1, updated_at = ?
         WHERE id = ?",
    )
    .bind(now)
    .bind(run_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn find_group_run_step_reassign_row(
    pool: &SqlitePool,
    step_id: &str,
) -> Result<Option<GroupRunStepReassignRow>, String> {
    let row = sqlx::query(
        "SELECT run_id, status, step_type, COALESCE(dispatch_source_employee_id, ''), COALESCE(assignee_employee_id, ''),
                COALESCE(output_summary, ''), COALESCE(output, '')
         FROM group_run_steps
         WHERE id = ?",
    )
    .bind(step_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|record| GroupRunStepReassignRow {
        run_id: record.try_get(0).expect("reassign row run_id"),
        status: record.try_get(1).expect("reassign row status"),
        step_type: record.try_get(2).expect("reassign row step_type"),
        dispatch_source_employee_id: record
            .try_get(3)
            .expect("reassign row dispatch_source_employee_id"),
        previous_assignee_employee_id: record
            .try_get(4)
            .expect("reassign row previous_assignee_employee_id"),
        previous_output_summary: record
            .try_get(5)
            .expect("reassign row previous_output_summary"),
        previous_output: record.try_get(6).expect("reassign row previous_output"),
    }))
}

pub(crate) async fn employee_exists_for_reassignment(
    pool: &SqlitePool,
    employee_id: &str,
) -> Result<bool, String> {
    let row = sqlx::query("SELECT id FROM agent_employees WHERE lower(employee_id) = lower(?) OR lower(role_id) = lower(?) LIMIT 1")
        .bind(employee_id)
        .bind(employee_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(row.is_some())
}

pub(crate) async fn reset_group_run_step_for_reassignment(
    tx: &mut Transaction<'_, Sqlite>,
    step_id: &str,
    assignee_employee_id: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE group_run_steps
         SET assignee_employee_id = ?,
             status = 'pending',
             output = '',
             output_summary = '',
             session_id = '',
             started_at = '',
             finished_at = '',
             attempt_no = attempt_no + 1
         WHERE id = ?",
    )
    .bind(assignee_employee_id)
    .bind(step_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn list_failed_execute_assignees(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
) -> Result<Vec<String>, String> {
    sqlx::query_scalar::<_, String>(
        "SELECT assignee_employee_id
         FROM group_run_steps
         WHERE run_id = ? AND step_type = 'execute' AND status = 'failed'
         ORDER BY round_no ASC, id ASC",
    )
    .bind(run_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(|e| e.to_string())
}

pub(crate) async fn update_group_run_after_reassignment(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    state: &str,
    waiting_for_employee_id: &str,
    status_reason: &str,
    now: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE group_runs
         SET state = ?,
             current_phase = 'execute',
             waiting_for_employee_id = ?,
             status_reason = ?,
             updated_at = ?
         WHERE id = ?",
    )
    .bind(state)
    .bind(waiting_for_employee_id)
    .bind(status_reason)
    .bind(now)
    .bind(run_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn find_group_run_execute_step_context(
    pool: &SqlitePool,
    step_id: &str,
) -> Result<Option<GroupRunExecuteStepContextRow>, String> {
    let row = sqlx::query(
        "SELECT s.id, s.run_id, s.assignee_employee_id, COALESCE(s.dispatch_source_employee_id, ''),
                s.step_type, COALESCE(s.session_id, ''), COALESCE(s.input, ''), COALESCE(r.user_goal, '')
         FROM group_run_steps s
         INNER JOIN group_runs r ON r.id = s.run_id
         WHERE s.id = ?",
    )
    .bind(step_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|record| GroupRunExecuteStepContextRow {
        step_id: record.try_get(0).expect("execute step row step_id"),
        run_id: record.try_get(1).expect("execute step row run_id"),
        assignee_employee_id: record
            .try_get(2)
            .expect("execute step row assignee_employee_id"),
        dispatch_source_employee_id: record
            .try_get(3)
            .expect("execute step row dispatch_source_employee_id"),
        step_type: record.try_get(4).expect("execute step row step_type"),
        existing_session_id: record
            .try_get(5)
            .expect("execute step row existing_session_id"),
        step_input: record.try_get(6).expect("execute step row step_input"),
        user_goal: record.try_get(7).expect("execute step row user_goal"),
    }))
}

pub(crate) async fn mark_group_run_step_dispatched(
    tx: &mut Transaction<'_, Sqlite>,
    step_id: &str,
    session_id: &str,
    now: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE group_run_steps
         SET status = 'running',
             session_id = ?,
             started_at = CASE WHEN TRIM(started_at) = '' THEN ? ELSE started_at END,
             phase = CASE WHEN TRIM(phase) = '' THEN 'execute' ELSE phase END
         WHERE id = ?",
    )
    .bind(session_id)
    .bind(now)
    .bind(step_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn mark_group_run_executing(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    waiting_for_employee_id: &str,
    now: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE group_runs
         SET state = 'executing',
             current_phase = 'execute',
             waiting_for_employee_id = ?,
             status_reason = '',
             updated_at = ?
         WHERE id = ?",
    )
    .bind(waiting_for_employee_id)
    .bind(now)
    .bind(run_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn mark_group_run_step_failed(
    tx: &mut Transaction<'_, Sqlite>,
    step_id: &str,
    output: &str,
    output_summary: &str,
    session_id: &str,
    now: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE group_run_steps
         SET status = 'failed',
             output = ?,
             output_summary = ?,
             session_id = ?,
             finished_at = ?,
             phase = CASE WHEN TRIM(phase) = '' THEN 'execute' ELSE phase END
         WHERE id = ?",
    )
    .bind(output)
    .bind(output_summary)
    .bind(session_id)
    .bind(now)
    .bind(step_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn mark_group_run_failed(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    waiting_for_employee_id: &str,
    status_reason: &str,
    now: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE group_runs
         SET state = 'failed',
             current_phase = 'execute',
             waiting_for_employee_id = ?,
             status_reason = ?,
             updated_at = ?
         WHERE id = ?",
    )
    .bind(waiting_for_employee_id)
    .bind(status_reason)
    .bind(now)
    .bind(run_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn mark_group_run_step_completed(
    tx: &mut Transaction<'_, Sqlite>,
    step_id: &str,
    output: &str,
    output_summary: &str,
    session_id: &str,
    now: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE group_run_steps
         SET status = 'completed',
             output = ?,
             output_summary = ?,
             session_id = ?,
             finished_at = ?,
             phase = CASE WHEN TRIM(phase) = '' THEN 'execute' ELSE phase END
         WHERE id = ?",
    )
    .bind(output)
    .bind(output_summary)
    .bind(session_id)
    .bind(now)
    .bind(step_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn clear_group_run_execute_waiting_state(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    now: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE group_runs
         SET state = 'executing',
             current_phase = 'execute',
             status_reason = '',
             waiting_for_employee_id = '',
             updated_at = ?
         WHERE id = ?",
    )
    .bind(now)
    .bind(run_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn find_pending_review_step(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<Option<PendingReviewStepRow>, String> {
    let row = sqlx::query(
        "SELECT id, assignee_employee_id
         FROM group_run_steps
         WHERE run_id = ? AND step_type = 'review' AND status IN ('pending', 'running', 'blocked')
         ORDER BY round_no DESC, id DESC
         LIMIT 1",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|record| PendingReviewStepRow {
        step_id: record.try_get(0).expect("pending review step_id"),
        assignee_employee_id: record
            .try_get(1)
            .expect("pending review assignee_employee_id"),
    }))
}

pub(crate) async fn review_requested_event_exists(
    pool: &SqlitePool,
    run_id: &str,
    step_id: &str,
) -> Result<bool, String> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)
         FROM group_run_events
         WHERE run_id = ? AND step_id = ? AND event_type = 'review_requested'",
    )
    .bind(run_id)
    .bind(step_id)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(count > 0)
}

pub(crate) async fn mark_group_run_waiting_review(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    waiting_for_employee_id: &str,
    default_reason: &str,
    now: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE group_runs
         SET state = 'waiting_review',
             current_phase = 'review',
             waiting_for_employee_id = ?,
             status_reason = CASE
               WHEN TRIM(status_reason) = '' THEN ?
               ELSE status_reason
             END,
             updated_at = ?
         WHERE id = ?",
    )
    .bind(waiting_for_employee_id)
    .bind(default_reason)
    .bind(now)
    .bind(run_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn list_pending_execute_step_ids(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<Vec<String>, String> {
    sqlx::query_scalar::<_, String>(
        "SELECT id
         FROM group_run_steps
         WHERE run_id = ? AND step_type = 'execute' AND status = 'pending'
         ORDER BY round_no ASC, id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())
}

pub(crate) async fn load_group_run_blocking_counts(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<(i64, i64), String> {
    let row = sqlx::query(
        "SELECT
            SUM(CASE WHEN step_type = 'execute' AND status IN ('pending', 'running', 'failed') THEN 1 ELSE 0 END) AS execute_blocking,
            SUM(CASE WHEN step_type = 'review' AND status IN ('pending', 'running') THEN 1 ELSE 0 END) AS review_blocking
         FROM group_run_steps
         WHERE run_id = ?",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok((
        row.try_get::<Option<i64>, _>("execute_blocking")
            .map_err(|e| e.to_string())?
            .unwrap_or(0),
        row.try_get::<Option<i64>, _>("review_blocking")
            .map_err(|e| e.to_string())?
            .unwrap_or(0),
    ))
}

pub(crate) async fn find_group_run_finalize_state(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<Option<GroupRunFinalizeStateRow>, String> {
    let row = sqlx::query(
        "SELECT session_id, user_goal, state
         FROM group_runs
         WHERE id = ?",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(row.map(|record| GroupRunFinalizeStateRow {
        session_id: record.try_get(0).expect("finalize state session_id"),
        user_goal: record.try_get(1).expect("finalize state user_goal"),
        state: record.try_get(2).expect("finalize state state"),
    }))
}

pub(crate) async fn list_group_run_execute_outputs(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<Vec<(String, String)>, String> {
    let rows = sqlx::query(
        "SELECT assignee_employee_id, output
         FROM group_run_steps
         WHERE run_id = ? AND step_type = 'execute'
         ORDER BY round_no ASC, finished_at ASC, id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(rows
        .into_iter()
        .map(|row| {
            (
                row.try_get(0).expect("execute output assignee"),
                row.try_get(1).expect("execute output content"),
            )
        })
        .collect())
}

pub(crate) async fn insert_group_run_assistant_message(
    tx: &mut Transaction<'_, Sqlite>,
    session_id: &str,
    content: &str,
    now: &str,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO messages (id, session_id, role, content, created_at)
         VALUES (?, ?, 'assistant', ?, ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(session_id)
    .bind(content)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn mark_group_run_finalized(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    now: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE group_runs
         SET state = 'done',
             current_phase = 'finalize',
             waiting_for_employee_id = '',
             waiting_for_user = 0,
             status_reason = '',
             updated_at = ?
         WHERE id = ?",
    )
    .bind(now)
    .bind(run_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn get_group_run_session_id(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<Option<String>, String> {
    let row = sqlx::query("SELECT session_id FROM group_runs WHERE id = ?")
        .bind(run_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(row.map(|record| record.try_get(0).expect("group run session id")))
}

pub(crate) async fn find_group_run_snapshot_row(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Option<GroupRunSnapshotRow>, String> {
    let row = sqlx::query(
        "SELECT id, group_id, session_id, state, current_round, user_goal,
                COALESCE(current_phase, 'plan'), COALESCE(review_round, 0),
                COALESCE(status_reason, ''), COALESCE(waiting_for_employee_id, ''),
                COALESCE(waiting_for_user, 0)
         FROM group_runs
         WHERE session_id = ?
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(row.map(|record| GroupRunSnapshotRow {
        run_id: record.try_get("id").expect("snapshot run_id"),
        group_id: record.try_get("group_id").expect("snapshot group_id"),
        session_id: record.try_get("session_id").expect("snapshot session_id"),
        state: record.try_get("state").expect("snapshot state"),
        current_round: record.try_get("current_round").expect("snapshot current_round"),
        user_goal: record.try_get("user_goal").expect("snapshot user_goal"),
        current_phase: record.try_get(6).expect("snapshot current_phase"),
        review_round: record.try_get(7).expect("snapshot review_round"),
        status_reason: record.try_get(8).expect("snapshot status_reason"),
        waiting_for_employee_id: record
            .try_get(9)
            .expect("snapshot waiting_for_employee_id"),
        waiting_for_user: record
            .try_get::<i64, _>(10)
            .expect("snapshot waiting_for_user")
            != 0,
    }))
}

pub(crate) async fn list_group_run_step_snapshot_rows(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<Vec<GroupRunStepSnapshotRow>, String> {
    let rows = sqlx::query(
        "SELECT id, round_no, step_type, assignee_employee_id,
                COALESCE(dispatch_source_employee_id, ''), COALESCE(session_id, ''),
                COALESCE(attempt_no, 1), status, COALESCE(output_summary, ''), output
         FROM group_run_steps
         WHERE run_id = ?
         ORDER BY round_no ASC, started_at ASC, id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(rows
        .into_iter()
        .map(|row| GroupRunStepSnapshotRow {
            id: row.try_get("id").expect("step snapshot id"),
            round_no: row.try_get("round_no").expect("step snapshot round_no"),
            step_type: row.try_get("step_type").expect("step snapshot step_type"),
            assignee_employee_id: row
                .try_get("assignee_employee_id")
                .expect("step snapshot assignee"),
            dispatch_source_employee_id: row
                .try_get(4)
                .expect("step snapshot dispatch_source"),
            session_id: row.try_get(5).expect("step snapshot session_id"),
            attempt_no: row.try_get(6).expect("step snapshot attempt_no"),
            status: row.try_get(7).expect("step snapshot status"),
            output_summary: row.try_get(8).expect("step snapshot output_summary"),
            output: row.try_get(9).expect("step snapshot output"),
        })
        .collect())
}

pub(crate) async fn list_group_run_event_snapshot_rows(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<Vec<GroupRunEventSnapshotRow>, String> {
    let rows = sqlx::query(
        "SELECT id, COALESCE(step_id, ''), event_type, COALESCE(payload_json, '{}'), created_at
         FROM group_run_events
         WHERE run_id = ?
         ORDER BY created_at ASC, id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(rows
        .into_iter()
        .map(|row| GroupRunEventSnapshotRow {
            id: row.try_get("id").expect("event snapshot id"),
            step_id: row.try_get(1).expect("event snapshot step_id"),
            event_type: row.try_get("event_type").expect("event snapshot event_type"),
            payload_json: row.try_get(3).expect("event snapshot payload_json"),
            created_at: row.try_get("created_at").expect("event snapshot created_at"),
        })
        .collect())
}

pub(crate) async fn find_latest_assistant_message_content(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Option<String>, String> {
    let row = sqlx::query(
        "SELECT content
         FROM messages
         WHERE session_id = ? AND role = 'assistant'
         ORDER BY created_at DESC, id DESC
         LIMIT 1",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(row.map(|record| record.try_get(0).expect("latest assistant content")))
}
