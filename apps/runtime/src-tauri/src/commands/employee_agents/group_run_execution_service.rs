use super::super::repo::{
    find_employee_session_seed_row, find_existing_session_skill_id, find_group_run_start_config,
    find_group_step_session_row, find_model_config_row, find_recent_group_step_session_id,
    insert_group_run_event, insert_group_run_record, insert_group_run_step_seed,
    insert_session_message, insert_session_seed, insert_tx_session_message,
    list_session_message_rows, SessionSeedInput,
};
use super::super::{EmployeeGroupRunResult, StartEmployeeGroupRunInput};
use super::{get_employee_group_run_snapshot_by_run_id_with_pool, list_agent_employees_with_pool};
use crate::agent::runtime::kernel::execution_plan::{ExecutionOutcome, SessionEngineError};
use crate::agent::runtime::kernel::session_engine::SessionEngine;
use crate::agent::runtime::kernel::turn_preparation::prepare_employee_step_turn;
use crate::agent::tools::{EmployeeManageTool, MemoryTool};
use crate::agent::{AgentExecutor, ToolRegistry};
use crate::commands::chat_runtime_io::extract_assistant_text_content;
use crate::commands::models::resolve_default_model_id_with_pool;
use serde_json::Value;
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

pub(crate) async fn execute_group_step_in_employee_context_with_pool(
    pool: &SqlitePool,
    run_id: &str,
    step_id: &str,
    session_id: &str,
    assignee_employee_id: &str,
    user_goal: &str,
    step_input: &str,
) -> Result<String, String> {
    let session_row = find_group_step_session_row(pool, session_id)
        .await?
        .ok_or_else(|| "group step session not found".to_string())?;

    let employee = list_agent_employees_with_pool(pool)
        .await?
        .into_iter()
        .find(|item| {
            item.employee_id.eq_ignore_ascii_case(assignee_employee_id)
                || item.role_id.eq_ignore_ascii_case(assignee_employee_id)
                || item.id.eq_ignore_ascii_case(assignee_employee_id)
        })
        .ok_or_else(|| "assignee employee not found".to_string())?;

    let model_row = find_model_config_row(pool, &session_row.model_id)
        .await?
        .ok_or_else(|| "model config not found".to_string())?;

    let (system_prompt, allowed_tools, max_iterations) =
        super::super::build_group_step_system_prompt(&employee, &session_row.skill_id);
    let user_prompt = super::super::build_group_step_user_prompt(
        run_id, step_id, user_goal, step_input, &employee,
    );

    let now = chrono::Utc::now().to_rfc3339();
    insert_session_message(pool, session_id, "user", &user_prompt, &now).await?;

    let messages: Vec<Value> = list_session_message_rows(pool, session_id)
        .await?
        .into_iter()
        .map(|row| {
            let normalized_content = if row.role == "assistant" {
                extract_assistant_text_content(&row.content)
            } else {
                row.content
            };
            serde_json::json!({ "role": row.role, "content": normalized_content })
        })
        .collect();

    let registry = Arc::new(ToolRegistry::with_standard_tools());
    let memory_root = if session_row.work_dir.trim().is_empty() {
        std::env::temp_dir().join("workclaw-group-run-memory")
    } else {
        PathBuf::from(session_row.work_dir.trim())
            .join("openclaw")
            .join(employee.employee_id.trim())
            .join("memory")
    };
    let memory_dir = memory_root.join(if session_row.skill_id.trim().is_empty() {
        "builtin-general"
    } else {
        session_row.skill_id.trim()
    });
    std::fs::create_dir_all(&memory_dir).map_err(|e| e.to_string())?;
    registry.register(Arc::new(MemoryTool::new(memory_dir)));
    registry.register(Arc::new(EmployeeManageTool::new(pool.clone())));

    let executor = Arc::new(AgentExecutor::with_max_iterations(
        Arc::clone(&registry),
        max_iterations,
    ));
    let (mut turn_context, execution_context) = prepare_employee_step_turn(
        &executor,
        &user_prompt,
        &system_prompt,
        &model_row.api_format,
        &model_row.base_url,
        &model_row.api_key,
        &model_row.model_name,
        allowed_tools,
        max_iterations,
        if session_row.work_dir.trim().is_empty() {
            None
        } else {
            Some(session_row.work_dir.clone())
        },
    );
    turn_context.messages = messages;

    let assistant_output = match SessionEngine::run_employee_step_turn(
        None,
        &executor,
        session_id,
        &turn_context,
        &execution_context,
        |_| {},
    )
    .await
    {
        Ok(ExecutionOutcome::RouteExecution {
            route_execution, ..
        }) => {
            if let Some(final_messages) = route_execution.final_messages {
                let assistant_output = super::super::extract_assistant_text(&final_messages);
                if assistant_output.trim().is_empty() {
                    return Err(
                        "employee step execution returned empty assistant output".to_string()
                    );
                }
                assistant_output
            } else if let Some(stop_reason) = route_execution.last_stop_reason {
                if !stop_reason
                    .kind
                    .eq(&crate::agent::run_guard::RunStopReasonKind::MaxTurns)
                {
                    return Err(route_execution
                        .last_error
                        .unwrap_or_else(|| stop_reason.message.clone()));
                }
                let fallback_output = super::super::build_group_step_iteration_fallback_output(
                    &employee,
                    user_goal,
                    step_input,
                    stop_reason
                        .detail
                        .as_deref()
                        .unwrap_or(stop_reason.message.as_str()),
                );
                let finished_at = chrono::Utc::now().to_rfc3339();
                insert_session_message(
                    pool,
                    session_id,
                    "assistant",
                    &fallback_output,
                    &finished_at,
                )
                .await?;
                return Ok(fallback_output);
            } else {
                return Err(route_execution
                    .last_error
                    .unwrap_or_else(|| "employee step execution failed".to_string()));
            }
        }
        Ok(ExecutionOutcome::DirectDispatch { output, .. }) => output,
        Ok(ExecutionOutcome::SkillCommandFailed { error, .. })
        | Ok(ExecutionOutcome::SkillCommandStopped { error, .. }) => return Err(error),
        Err(SessionEngineError::Generic(message)) => return Err(message),
    };

    let finished_at = chrono::Utc::now().to_rfc3339();
    insert_session_message(
        pool,
        session_id,
        "assistant",
        &assistant_output,
        &finished_at,
    )
    .await?;

    Ok(assistant_output)
}

pub(crate) async fn ensure_group_run_session_with_pool(
    pool: &SqlitePool,
    coordinator_employee_id: &str,
    group_name: &str,
    now: &str,
    preferred_session_id: Option<&str>,
) -> Result<(String, String), String> {
    let employee_row = find_employee_session_seed_row(pool, coordinator_employee_id)
        .await?
        .ok_or_else(|| "coordinator employee not found".to_string())?;

    let session_skill_id = if employee_row.primary_skill_id.trim().is_empty() {
        "builtin-general".to_string()
    } else {
        employee_row.primary_skill_id.trim().to_string()
    };

    if let Some(existing_session_id) = preferred_session_id
        .map(str::trim)
        .filter(|session_id| !session_id.is_empty())
    {
        let existing_skill_id = find_existing_session_skill_id(pool, existing_session_id)
            .await?
            .ok_or_else(|| "preferred group run session not found".to_string())?;
        let existing_skill_id = if existing_skill_id.trim().is_empty() {
            session_skill_id.clone()
        } else {
            existing_skill_id.trim().to_string()
        };
        return Ok((existing_session_id.to_string(), existing_skill_id));
    }

    let model_id = resolve_default_model_id_with_pool(pool)
        .await?
        .ok_or_else(|| "model config not found".to_string())?;

    let session_id = Uuid::new_v4().to_string();
    insert_session_seed(
        pool,
        &SessionSeedInput {
            id: &session_id,
            skill_id: &session_skill_id,
            title: &format!("群组协作：{}", group_name.trim()),
            created_at: now,
            model_id: &model_id,
            work_dir: &employee_row.default_work_dir,
            employee_id: coordinator_employee_id,
        },
    )
    .await?;

    Ok((session_id, session_skill_id))
}

pub(crate) async fn append_group_run_assistant_message_with_pool(
    pool: &SqlitePool,
    session_id: &str,
    content: &str,
) -> Result<(), String> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    let now = chrono::Utc::now().to_rfc3339();
    insert_session_message(pool, session_id, "assistant", trimmed, &now).await
}

pub(crate) async fn ensure_group_step_session_with_pool(
    pool: &SqlitePool,
    run_id: &str,
    assignee_employee_id: &str,
    now: &str,
) -> Result<String, String> {
    if let Some(session_id) =
        find_recent_group_step_session_id(pool, run_id, assignee_employee_id).await?
    {
        return Ok(session_id);
    }

    let employee_row = find_employee_session_seed_row(pool, assignee_employee_id)
        .await?
        .ok_or_else(|| "assignee employee not found".to_string())?;

    let session_skill_id = if employee_row.primary_skill_id.trim().is_empty() {
        "builtin-general".to_string()
    } else {
        employee_row.primary_skill_id.trim().to_string()
    };

    let model_id = resolve_default_model_id_with_pool(pool)
        .await?
        .ok_or_else(|| "model config not found".to_string())?;

    let session_id = Uuid::new_v4().to_string();
    insert_session_seed(
        pool,
        &SessionSeedInput {
            id: &session_id,
            skill_id: &session_skill_id,
            title: &format!("群组执行:{}@{}", run_id, assignee_employee_id),
            created_at: now,
            model_id: &model_id,
            work_dir: &employee_row.default_work_dir,
            employee_id: assignee_employee_id,
        },
    )
    .await?;

    Ok(session_id)
}

pub(crate) async fn start_employee_group_run_internal_with_pool(
    pool: &SqlitePool,
    input: StartEmployeeGroupRunInput,
    preferred_session_id: Option<&str>,
    persist_user_message: bool,
) -> Result<EmployeeGroupRunResult, String> {
    let group_id = input.group_id.trim().to_string();
    if group_id.is_empty() {
        return Err("group_id is required".to_string());
    }
    let user_goal = input.user_goal.trim().to_string();
    if user_goal.is_empty() {
        return Err("user_goal is required".to_string());
    }

    let config = find_group_run_start_config(pool, &group_id)
        .await?
        .ok_or_else(|| "employee group not found".to_string())?;

    let member_employee_ids =
        serde_json::from_str::<Vec<String>>(&config.member_employee_ids_json).unwrap_or_default();
    let rules = super::super::list_employee_group_rules_with_pool(pool, &group_id).await?;
    let planner_employee_id = super::super::resolve_group_planner_employee_id(
        &config.entry_employee_id,
        &config.coordinator_employee_id,
        &rules,
    );
    let reviewer_employee_id = super::super::resolve_group_reviewer_employee_id(
        &config.review_mode,
        &planner_employee_id,
        &rules,
    );
    let (execute_targets, _) = super::super::select_group_execute_dispatch_targets(
        &rules,
        &member_employee_ids,
        &[
            config.coordinator_employee_id.clone(),
            planner_employee_id.clone(),
            config.entry_employee_id.clone(),
        ],
    );

    let plan = crate::agent::group_orchestrator::build_group_run_plan(
        crate::agent::group_orchestrator::GroupRunRequest {
            group_id: group_id.clone(),
            coordinator_employee_id: config.coordinator_employee_id.clone(),
            planner_employee_id: Some(planner_employee_id.clone()),
            reviewer_employee_id: reviewer_employee_id.clone(),
            member_employee_ids,
            execute_targets,
            user_goal: user_goal.clone(),
            execution_window: input.execution_window,
            timeout_employee_ids: input.timeout_employee_ids,
            max_retry_per_step: input.max_retry_per_step,
        },
    );
    let initial_report = plan.final_report.clone();
    let initial_state = plan.state.clone();
    let initial_round = plan.current_round;
    let now = chrono::Utc::now().to_rfc3339();
    let run_id = Uuid::new_v4().to_string();
    let (session_id, session_skill_id) = ensure_group_run_session_with_pool(
        pool,
        &config.coordinator_employee_id,
        &config.name,
        &now,
        preferred_session_id,
    )
    .await?;

    let waiting_for_employee_id = reviewer_employee_id
        .as_deref()
        .filter(|employee_id| !employee_id.trim().is_empty())
        .unwrap_or(config.coordinator_employee_id.as_str())
        .to_string();

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    insert_group_run_record(
        &mut tx,
        &run_id,
        &group_id,
        &session_id,
        &user_goal,
        &initial_state,
        initial_round,
        &plan.current_phase,
        &config.coordinator_employee_id,
        &waiting_for_employee_id,
        &now,
    )
    .await?;

    if persist_user_message {
        insert_tx_session_message(&mut tx, &session_id, "user", &user_goal, &now).await?;
    }

    for event in &plan.events {
        insert_group_run_event(
            &mut tx,
            &run_id,
            "",
            &event.event_type,
            &event.payload_json,
            &now,
        )
        .await?;
    }

    for step in plan.steps {
        let step_id = Uuid::new_v4().to_string();
        let dispatch_source_employee_id = step.dispatch_source_employee_id.clone();
        insert_group_run_step_seed(
            &mut tx,
            &run_id,
            &step_id,
            step.round_no,
            &step.assignee_employee_id,
            &dispatch_source_employee_id,
            &step.phase,
            &step.step_type,
            &user_goal,
            &step.output,
            &step.status,
            step.requires_review,
            &step.review_status,
            &now,
        )
        .await?;
        insert_group_run_event(
            &mut tx,
            &run_id,
            &step_id,
            "step_created",
            &serde_json::json!({
                "phase": step.phase,
                "step_type": step.step_type,
                "assignee_employee_id": step.assignee_employee_id,
                "dispatch_source_employee_id": dispatch_source_employee_id,
                "status": step.status
            })
            .to_string(),
            &now,
        )
        .await?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    let snapshot = super::super::continue_employee_group_run_with_pool(pool, &run_id).await?;
    if snapshot.state != "done" {
        append_group_run_assistant_message_with_pool(pool, &session_id, &initial_report).await?;
    }
    let final_snapshot = get_employee_group_run_snapshot_by_run_id_with_pool(pool, &run_id).await?;

    Ok(EmployeeGroupRunResult {
        run_id,
        group_id,
        session_id,
        session_skill_id,
        state: final_snapshot.state,
        current_round: final_snapshot.current_round,
        final_report: final_snapshot.final_report,
        steps: final_snapshot.steps,
    })
}
