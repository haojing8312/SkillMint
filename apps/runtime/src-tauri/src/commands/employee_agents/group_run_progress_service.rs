use super::super::repo::{
    find_group_run_finalize_state, find_group_run_state, find_pending_review_step,
    insert_group_run_assistant_message, insert_group_run_event, list_group_run_execute_outputs,
    list_pending_execute_step_ids, load_group_run_blocking_counts, mark_group_run_finalized,
    mark_group_run_waiting_review, review_requested_event_exists,
};
use sqlx::SqlitePool;

pub(crate) async fn load_group_run_continue_state(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<(String, String), String> {
    let normalized_run_id = run_id.trim();
    if normalized_run_id.is_empty() {
        return Err("run_id is required".to_string());
    }
    let run_row = find_group_run_state(pool, normalized_run_id)
        .await?
        .ok_or_else(|| "group run not found".to_string())?;
    Ok((run_row.state, run_row.current_phase))
}

pub(crate) async fn maybe_mark_group_run_waiting_review(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<Option<String>, String> {
    let Some(review_row) = find_pending_review_step(pool, run_id).await? else {
        return Ok(None);
    };

    let review_requested_exists = review_requested_event_exists(pool, run_id, &review_row.step_id).await?;
    let default_reason = format!("等待{}审议", review_row.assignee_employee_id.trim());
    let now = chrono::Utc::now().to_rfc3339();
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    mark_group_run_waiting_review(
        &mut tx,
        run_id,
        &review_row.assignee_employee_id,
        &default_reason,
        &now,
    )
    .await?;
    if !review_requested_exists {
        insert_group_run_event(
            &mut tx,
            run_id,
            &review_row.step_id,
            "review_requested",
            &serde_json::json!({
                "assignee_employee_id": review_row.assignee_employee_id,
                "phase": "review",
            })
            .to_string(),
            &now,
        )
        .await?;
    }
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(Some(review_row.assignee_employee_id))
}

pub(crate) async fn list_pending_execute_steps_for_continue(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<Vec<String>, String> {
    list_pending_execute_step_ids(pool, run_id).await
}

pub(crate) async fn maybe_finalize_group_run_with_pool(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<(), String> {
    let (execute_blocking, review_blocking) = load_group_run_blocking_counts(pool, run_id).await?;
    if execute_blocking > 0 || review_blocking > 0 {
        return Ok(());
    }

    let run_row = find_group_run_finalize_state(pool, run_id)
        .await?
        .ok_or_else(|| "group run not found".to_string())?;
    if run_row.state == "done" {
        return Ok(());
    }

    let execute_rows = list_group_run_execute_outputs(pool, run_id).await?;
    let mut summary_lines = vec![
        format!("计划：围绕“{}”的团队执行已完成。", run_row.user_goal.trim()),
        "执行：".to_string(),
    ];
    for (assignee_employee_id, output) in execute_rows {
        summary_lines.push(format!("- {}: {}", assignee_employee_id, output.trim()));
    }
    summary_lines.push("汇报：团队协作已完成，可继续进入人工复核或直接对外回复。".to_string());
    let final_report = summary_lines.join("\n");

    let now = chrono::Utc::now().to_rfc3339();
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    insert_group_run_assistant_message(&mut tx, &run_row.session_id, &final_report, &now).await?;
    mark_group_run_finalized(&mut tx, run_id, &now).await?;
    insert_group_run_event(
        &mut tx,
        run_id,
        "",
        "run_completed",
        &serde_json::json!({
            "state": "done",
            "phase": "finalize",
            "summary": final_report,
        })
        .to_string(),
        &now,
    )
    .await?;
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}
