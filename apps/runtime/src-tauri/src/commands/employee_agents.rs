use crate::commands::{chat_runtime_io, skills::DbState};
use crate::employee_runtime_adapter::employee_adapter::{
    build_group_run_execute_targets, build_team_runtime_view,
};
use crate::im::types::ImEvent;
use crate::im::{
    AgentInboundDispatchSession, EnsuredAgentSession,
    link_inbound_event_to_agent_session_with_pool as link_inbound_event_to_agent_session_binding_with_pool,
    list_ensured_agent_sessions_for_event_with_pool,
};
use serde_json::Value;
use sqlx::{Row, SqlitePool};
use std::path::Path;
use tauri::State;

#[path = "employee_agents/curator_scheduler.rs"]
pub(crate) mod curator_scheduler;
#[path = "employee_agents/group_management.rs"]
mod group_management;
#[path = "employee_agents/group_run_entry.rs"]
mod group_run_entry;
#[path = "employee_agents/memory_commands.rs"]
mod memory_commands;
#[path = "employee_agents/repo.rs"]
pub(crate) mod repo;
#[path = "employee_agents/service.rs"]
mod service;
#[path = "employee_agents/tauri_commands.rs"]
mod tauri_commands;
#[path = "employee_agents/team_rules.rs"]
mod team_rules;
#[path = "employee_agents/test_support.rs"]
#[doc(hidden)]
pub mod test_support;
#[path = "employee_agents/types.rs"]
mod types;

pub(crate) use group_management::{
    clone_employee_group_template_with_pool, create_employee_group_with_pool,
    create_employee_team_with_pool, delete_employee_group_with_pool,
    list_employee_group_rules_with_pool, list_employee_group_runs_with_pool,
    list_employee_groups_with_pool,
};
pub(crate) use group_run_entry::{
    build_group_step_iteration_fallback_output, build_group_step_system_prompt,
    build_group_step_user_prompt, continue_employee_group_run_with_pool_and_journal,
    extract_assistant_text, maybe_handle_team_entry_session_message_with_pool,
    run_group_step_with_pool_and_journal, start_employee_group_run_with_pool_and_journal,
};
use team_rules::{group_rule_matches_relation_types, normalize_member_employee_ids};
pub use types::{
    AgentEmployee, CloneEmployeeGroupTemplateInput, CreateEmployeeGroupInput,
    CreateEmployeeTeamInput, CreateEmployeeTeamRuleInput, EmployeeCuratorChangedTarget,
    EmployeeCuratorFinding, EmployeeCuratorReports, EmployeeCuratorRestoreCandidate,
    EmployeeCuratorRun, EmployeeCuratorSchedulerStatus, EmployeeGroup, EmployeeGroupRule,
    EmployeeGroupRunEvent, EmployeeGroupRunResult, EmployeeGroupRunSnapshot, EmployeeGroupRunStep,
    EmployeeGroupRunSummary, EmployeeGrowthEvent, EmployeeGrowthTimeline,
    EmployeeInboundDispatchSession, EmployeeProfileMemoryStatus, EnsuredEmployeeSession,
    GroupStepExecutionResult, SaveFeishuEmployeeAssociationInput, StartEmployeeGroupRunInput,
    UpsertAgentEmployeeInput,
};
use types::{default_group_execution_window, default_group_max_retry};

async fn load_execute_reassignment_targets_with_pool(
    pool: &SqlitePool,
    run_id: &str,
    dispatch_source_override: Option<&str>,
) -> Result<(Vec<String>, bool), String> {
    let row = sqlx::query(
        "SELECT g.id,
                COALESCE(g.member_employee_ids_json, '[]'),
                COALESCE(r.main_employee_id, ''),
                COALESCE(g.coordinator_employee_id, ''),
                COALESCE(g.entry_employee_id, ''),
                COALESCE(g.review_mode, 'none')
         FROM group_runs r
         INNER JOIN employee_groups g ON g.id = r.group_id
         WHERE r.id = ?",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "group run not found".to_string())?;

    let group_id: String = row.try_get(0).map_err(|e| e.to_string())?;
    let member_employee_ids_json: String = row.try_get(1).map_err(|e| e.to_string())?;
    let main_employee_id: String = row.try_get(2).map_err(|e| e.to_string())?;
    let coordinator_employee_id: String = row.try_get(3).map_err(|e| e.to_string())?;
    let entry_employee_id: String = row.try_get(4).map_err(|e| e.to_string())?;
    let review_mode: String = row.try_get(5).map_err(|e| e.to_string())?;
    let member_employee_ids =
        serde_json::from_str::<Vec<String>>(&member_employee_ids_json).unwrap_or_default();
    let normalized_member_ids = normalize_member_employee_ids(&member_employee_ids);
    let run_dispatch_source_employee_id = if main_employee_id.trim().is_empty() {
        coordinator_employee_id.trim().to_lowercase()
    } else {
        main_employee_id.trim().to_lowercase()
    };
    let dispatch_source_employee_id = if let Some(dispatch_source_override) =
        dispatch_source_override
            .map(str::trim)
            .filter(|value| !value.is_empty())
    {
        dispatch_source_override.to_lowercase()
    } else {
        run_dispatch_source_employee_id.clone()
    };

    let rules = list_employee_group_rules_with_pool(pool, &group_id).await?;
    let employees = service::list_agent_employees_with_pool(pool).await?;
    let team_runtime_view = build_team_runtime_view(
        &employees,
        &coordinator_employee_id,
        &entry_employee_id,
        &member_employee_ids,
        &review_mode,
        &rules,
        &[dispatch_source_employee_id, run_dispatch_source_employee_id],
    );
    let targets = build_group_run_execute_targets(&team_runtime_view);
    let has_execute_rules = !team_runtime_view.delegation_policy.targets.is_empty();
    if !has_execute_rules {
        return Ok((normalized_member_ids, false));
    }
    Ok((
        targets
            .into_iter()
            .map(|target| target.assignee_employee_id)
            .collect(),
        true,
    ))
}

fn normalize_employee_id(employee_id: &str) -> Result<String, String> {
    let normalized = employee_id.trim().to_string();
    if normalized.is_empty() {
        return Err("employee_id is required".to_string());
    }
    Ok(normalized)
}

fn normalize_memory_skill_scope(skill_id: Option<&str>) -> Result<Option<String>, String> {
    let normalized = skill_id.map(|v| v.trim()).unwrap_or_default();
    if normalized.is_empty() {
        return Ok(None);
    }
    if normalized.contains("..") || normalized.contains('/') || normalized.contains('\\') {
        return Err("invalid skill_id".to_string());
    }
    Ok(Some(normalized.to_string()))
}

fn memory_status_path_payload(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub(crate) fn collect_employee_profile_memory_status_from_root(
    runtime_root: &std::path::Path,
    memory_root: &std::path::Path,
    work_dir: Option<&std::path::Path>,
    skill_id: &str,
    employee_id: &str,
    profile_id: Option<&str>,
    im_role_id: Option<&str>,
) -> Result<EmployeeProfileMemoryStatus, String> {
    let employee_id = normalize_employee_id(employee_id)?;
    let skill_id = normalize_memory_skill_scope(Some(skill_id))?
        .ok_or_else(|| "skill_id is required".to_string())?;
    let profile_id = profile_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);

    let locator = chat_runtime_io::build_profile_memory_locator(
        runtime_root,
        memory_root,
        work_dir,
        &skill_id,
        &employee_id,
        profile_id.as_deref(),
        im_role_id,
    );
    let status = chat_runtime_io::collect_profile_memory_status(&locator);

    Ok(EmployeeProfileMemoryStatus {
        employee_id,
        profile_id,
        skill_id,
        profile_memory_dir: status
            .profile_memory_dir
            .as_deref()
            .map(memory_status_path_payload),
        profile_memory_file_path: status
            .profile_memory_file_path
            .as_deref()
            .map(memory_status_path_payload),
        profile_memory_file_exists: status.profile_memory_file_exists,
        active_source: status.active_source.to_string(),
        active_source_path: status
            .active_source_path
            .as_deref()
            .map(memory_status_path_payload),
    })
}

pub async fn list_agent_employees_with_pool(
    pool: &SqlitePool,
) -> Result<Vec<AgentEmployee>, String> {
    service::list_agent_employees_with_pool(pool).await
}

#[cfg(test)]
fn normalize_enabled_scopes_for_storage(enabled_scopes: &[String]) -> Vec<String> {
    service::normalize_enabled_scopes_for_storage(enabled_scopes)
}

pub async fn save_feishu_employee_association_with_pool(
    pool: &SqlitePool,
    input: SaveFeishuEmployeeAssociationInput,
) -> Result<(), String> {
    service::save_feishu_employee_association_with_pool(pool, input).await
}

pub async fn upsert_agent_employee_with_pool(
    pool: &SqlitePool,
    input: UpsertAgentEmployeeInput,
) -> Result<String, String> {
    service::upsert_agent_employee_with_pool(pool, input).await
}

pub async fn delete_agent_employee_with_pool(
    pool: &SqlitePool,
    employee_id: &str,
) -> Result<(), String> {
    service::delete_agent_employee_with_pool(pool, employee_id).await
}

pub async fn review_group_run_step_with_pool(
    pool: &SqlitePool,
    run_id: &str,
    action: &str,
    comment: &str,
) -> Result<(), String> {
    service::review_group_run_step_with_pool(pool, run_id, action, comment).await
}

pub async fn pause_employee_group_run_with_pool(
    pool: &SqlitePool,
    run_id: &str,
    reason: &str,
) -> Result<(), String> {
    service::pause_employee_group_run_with_pool(pool, run_id, reason).await
}

pub async fn resume_employee_group_run_with_pool(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<(), String> {
    service::resume_employee_group_run_with_pool(pool, run_id).await
}

pub async fn reassign_group_run_step_with_pool(
    pool: &SqlitePool,
    step_id: &str,
    assignee_employee_id: &str,
) -> Result<(), String> {
    service::reassign_group_run_step_with_pool(pool, step_id, assignee_employee_id).await
}

pub async fn get_employee_group_run_snapshot_with_pool(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Option<EmployeeGroupRunSnapshot>, String> {
    service::get_employee_group_run_snapshot_with_pool(pool, session_id).await
}

pub async fn cancel_employee_group_run_with_pool(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<(), String> {
    service::cancel_employee_group_run_with_pool(pool, run_id).await
}

pub async fn retry_employee_group_run_failed_steps_with_pool(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<(), String> {
    service::retry_employee_group_run_failed_steps_with_pool(pool, run_id).await
}

fn employee_scope_matches_event(employee: &AgentEmployee, event: &ImEvent) -> bool {
    let event_channel = event.channel.trim().to_lowercase();
    let normalized_event_channel = if event_channel.is_empty() {
        "app"
    } else {
        event_channel.as_str()
    };
    let normalized_scopes = if employee.enabled_scopes.is_empty() {
        vec!["app".to_string()]
    } else {
        employee
            .enabled_scopes
            .iter()
            .map(|scope| scope.trim().to_lowercase())
            .filter(|scope| !scope.is_empty())
            .collect::<Vec<_>>()
    };
    normalized_scopes.iter().any(|scope| {
        scope == normalized_event_channel || (scope == "app" && normalized_event_channel == "app")
    })
}

pub async fn resolve_target_employees_for_event(
    pool: &SqlitePool,
    event: &ImEvent,
) -> Result<Vec<AgentEmployee>, String> {
    service::resolve_target_employees_for_event(pool, event).await
}

fn im_binding_matches_event(
    binding: &crate::commands::im_routing::ImRoutingBinding,
    event: &ImEvent,
) -> bool {
    if !binding.enabled {
        return false;
    }
    if !binding.channel.trim().is_empty()
        && !binding.channel.eq_ignore_ascii_case(event.channel.trim())
    {
        return false;
    }
    if !binding.account_id.trim().is_empty() {
        let tenant_id = event.tenant_id.as_deref().unwrap_or_default().trim();
        if !binding.account_id.trim().eq_ignore_ascii_case(tenant_id) {
            return false;
        }
    }
    if !binding.peer_kind.trim().is_empty() && !binding.peer_kind.eq_ignore_ascii_case("group") {
        return false;
    }
    if !binding.peer_id.trim().is_empty() && binding.peer_id.trim() != event.thread_id.trim() {
        return false;
    }
    true
}

pub async fn ensure_employee_sessions_for_event_with_pool(
    pool: &SqlitePool,
    event: &ImEvent,
) -> Result<Vec<EnsuredEmployeeSession>, String> {
    service::ensure_employee_sessions_for_event_with_pool(pool, event).await
}

pub async fn ensure_agent_sessions_for_event_with_pool(
    pool: &SqlitePool,
    event: &ImEvent,
) -> Result<Vec<EnsuredAgentSession>, String> {
    list_ensured_agent_sessions_for_event_with_pool(pool, event).await
}

pub(crate) fn build_route_session_key(event: &ImEvent, employee: &AgentEmployee) -> String {
    let channel = event.channel.trim().to_lowercase();
    let tenant = event
        .tenant_id
        .as_ref()
        .map(|v| v.trim().to_lowercase())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "default".to_string());
    let agent_id = if employee.openclaw_agent_id.trim().is_empty() {
        employee.role_id.trim().to_lowercase()
    } else {
        employee.openclaw_agent_id.trim().to_lowercase()
    };
    let conversation = event
        .conversation_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| event.thread_id.trim());
    format!(
        "{}:{}:{}:{}",
        if channel.is_empty() {
            "app"
        } else {
            channel.as_str()
        },
        tenant,
        agent_id,
        conversation
    )
}

pub async fn link_inbound_event_to_session_with_pool(
    pool: &SqlitePool,
    event: &ImEvent,
    employee_id: &str,
    session_id: &str,
) -> Result<(), String> {
    service::link_inbound_event_to_session_with_pool(pool, event, employee_id, session_id).await
}

pub async fn link_inbound_event_to_agent_session_with_pool(
    pool: &SqlitePool,
    event: &ImEvent,
    agent_id: &str,
    session_id: &str,
) -> Result<(), String> {
    link_inbound_event_to_agent_session_binding_with_pool(pool, event, agent_id, session_id).await
}

pub async fn bridge_inbound_event_to_employee_sessions_with_pool(
    pool: &SqlitePool,
    event: &ImEvent,
    route_decision: Option<&Value>,
) -> Result<Vec<EmployeeInboundDispatchSession>, String> {
    service::bridge_inbound_event_to_employee_sessions_with_pool(pool, event, route_decision).await
}

pub async fn resolve_agent_session_dispatches_for_event_with_pool(
    pool: &SqlitePool,
    event: &ImEvent,
    route_decision: Option<&Value>,
) -> Result<Vec<AgentInboundDispatchSession>, String> {
    service::bridge_inbound_event_to_employee_sessions_with_pool(pool, event, route_decision)
        .await
        .map(|dispatches| {
            dispatches
                .into_iter()
                .map(AgentInboundDispatchSession::from)
                .collect()
        })
}

async fn table_exists(pool: &SqlitePool, table_name: &str) -> Result<bool, String> {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
    )
    .bind(table_name)
    .fetch_one(pool)
    .await
    .map(|count| count > 0)
    .map_err(|e| e.to_string())
}

async fn resolve_employee_profile_id_for_growth_with_pool(
    pool: &SqlitePool,
    employee_id: &str,
) -> Result<Option<String>, String> {
    if !table_exists(pool, "agent_profiles").await? {
        return Ok(None);
    }
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
}

pub async fn list_employee_growth_events_with_pool(
    pool: &SqlitePool,
    employee_id: &str,
    limit: i64,
) -> Result<EmployeeGrowthTimeline, String> {
    let employee_id = employee_id.trim();
    if employee_id.is_empty() {
        return Err("employee_id 不能为空".to_string());
    }
    let profile_id = resolve_employee_profile_id_for_growth_with_pool(pool, employee_id).await?;
    if !table_exists(pool, "growth_events").await? {
        return Ok(EmployeeGrowthTimeline {
            employee_id: employee_id.to_string(),
            profile_id,
            events: Vec::new(),
        });
    }
    let query_profile_id = profile_id.as_deref().unwrap_or(employee_id);
    let limit = limit.clamp(1, 100);
    let rows = if table_exists(pool, "sessions").await? {
        sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                String,
            ),
        >(
            "SELECT ge.id,
                    ge.profile_id,
                    ge.session_id,
                    COALESCE(s.title, '') AS session_title,
                    ge.event_type,
                    ge.target_type,
                    ge.target_id,
                    ge.summary,
                    ge.evidence_json,
                    ge.created_at
             FROM growth_events ge
             LEFT JOIN sessions s ON s.id = ge.session_id
             WHERE ge.profile_id = ?
             ORDER BY ge.created_at DESC, ge.id DESC
             LIMIT ?",
        )
        .bind(query_profile_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?
    } else {
        sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                String,
            ),
        >(
            "SELECT id,
                    profile_id,
                    session_id,
                    '' AS session_title,
                    event_type,
                    target_type,
                    target_id,
                    summary,
                    evidence_json,
                    created_at
             FROM growth_events
             WHERE profile_id = ?
             ORDER BY created_at DESC, id DESC
             LIMIT ?",
        )
        .bind(query_profile_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?
    };
    let events = rows
        .into_iter()
        .map(
            |(
                id,
                profile_id,
                session_id,
                session_title,
                event_type,
                target_type,
                target_id,
                summary,
                evidence_json,
                created_at,
            )| {
                let evidence_json: Value =
                    serde_json::from_str(&evidence_json).unwrap_or_else(|_| serde_json::json!({}));
                let display_summary = human_growth_summary(
                    &event_type,
                    &target_type,
                    &target_id,
                    &summary,
                    &evidence_json,
                );
                let target_label = human_growth_target_label(&target_type, &target_id);
                let evidence_label = human_growth_evidence_label(&evidence_json);
                EmployeeGrowthEvent {
                    id,
                    profile_id,
                    session_id,
                    session_title,
                    event_type,
                    target_type,
                    target_id,
                    summary,
                    display_summary,
                    target_label,
                    evidence_label,
                    evidence_json,
                    created_at,
                }
            },
        )
        .collect();

    Ok(EmployeeGrowthTimeline {
        employee_id: employee_id.to_string(),
        profile_id,
        events,
    })
}

fn human_growth_summary(
    event_type: &str,
    target_type: &str,
    target_id: &str,
    summary: &str,
    evidence: &Value,
) -> String {
    let trimmed = summary.trim();
    if event_type.starts_with("memory_") {
        if let Some(preview) = read_growth_memory_preview(evidence) {
            return match event_type {
                "memory_add" => format!("记住：{preview}"),
                "memory_replace" => format!("更新记忆：{preview}"),
                "memory_remove" => format!("删除记忆：{preview}"),
                "memory_rollback" => format!("回滚记忆：{preview}"),
                _ => format!("更新 Profile Memory：{preview}"),
            };
        }
        if !trimmed.is_empty() && !matches!(trimmed, "add" | "replace" | "remove" | "update") {
            return trimmed.to_string();
        }
        return match event_type {
            "memory_add" => "写入 Profile Memory".to_string(),
            "memory_replace" => "更新 Profile Memory".to_string(),
            "memory_remove" => "删除 Profile Memory".to_string(),
            "memory_rollback" => "回滚 Profile Memory".to_string(),
            _ => "Profile Memory 已更新".to_string(),
        };
    }

    if !trimmed.is_empty() && !matches!(trimmed, "add" | "replace" | "remove" | "update") {
        return trimmed.to_string();
    }

    if event_type.starts_with("skill_") {
        return match event_type {
            "skill_create" => format!("创建技能：{target_id}"),
            "skill_patch" => format!("优化技能：{target_id}"),
            "skill_archive" => format!("归档技能：{target_id}"),
            "skill_restore" => format!("恢复技能：{target_id}"),
            "skill_delete" => format!("删除技能：{target_id}"),
            "skill_rollback" => format!("回滚技能：{target_id}"),
            "skill_reset" => format!("重置技能：{target_id}"),
            _ => format!("技能已更新：{target_id}"),
        };
    }

    if event_type.starts_with("curator_") {
        return match event_type {
            "curator_scan" => "Curator 完成扫描".to_string(),
            "curator_restore" => "Curator 恢复 stale skill".to_string(),
            _ => "Curator 已更新".to_string(),
        };
    }

    if target_type.is_empty() && target_id.is_empty() {
        event_type.to_string()
    } else {
        format!("{event_type}: {target_type}/{target_id}")
    }
}

fn human_growth_target_label(target_type: &str, target_id: &str) -> String {
    match target_type {
        "profile_memory" => "Profile Memory".to_string(),
        "skill" => {
            if target_id.trim().is_empty() {
                "Skill OS".to_string()
            } else {
                format!("Skill: {target_id}")
            }
        }
        "curator" => "Curator".to_string(),
        _ if target_id.trim().is_empty() => target_type.to_string(),
        _ => format!("{target_type}: {target_id}"),
    }
}

fn human_growth_evidence_label(evidence: &Value) -> String {
    let version_id = evidence
        .get("version_id")
        .and_then(Value::as_str)
        .or_else(|| {
            evidence
                .pointer("/memory_version/version_id")
                .and_then(Value::as_str)
        })
        .unwrap_or_default();
    if version_id.trim().is_empty() {
        "有审计记录".to_string()
    } else {
        "已保存版本，可审计/回滚".to_string()
    }
}

fn read_growth_memory_preview(evidence: &Value) -> Option<String> {
    [
        evidence.get("content_preview").and_then(Value::as_str),
        evidence.get("diff_summary").and_then(Value::as_str),
        evidence
            .pointer("/memory_version/content_preview")
            .and_then(Value::as_str),
        evidence
            .pointer("/memory_version/diff_summary")
            .and_then(Value::as_str),
    ]
    .into_iter()
    .flatten()
    .map(str::trim)
    .find(|value| !value.is_empty())
    .map(|value| value.chars().take(120).collect::<String>())
}

fn parse_curator_findings(report_json: &str) -> Vec<EmployeeCuratorFinding> {
    let report =
        serde_json::from_str::<Value>(report_json).unwrap_or_else(|_| serde_json::json!({}));
    report["findings"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .map(|item| EmployeeCuratorFinding {
                    kind: item["kind"].as_str().unwrap_or_default().to_string(),
                    severity: item["severity"].as_str().unwrap_or_default().to_string(),
                    target_type: item["target_type"].as_str().unwrap_or_default().to_string(),
                    target_id: item["target_id"].as_str().unwrap_or_default().to_string(),
                    summary: item["summary"].as_str().unwrap_or_default().to_string(),
                    evidence_json: item
                        .get("evidence")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!({})),
                    suggested_action: item["suggested_action"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                    reversible: item["reversible"].as_bool().unwrap_or(false),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_curator_changed_targets(report: &Value) -> Vec<EmployeeCuratorChangedTarget> {
    report["findings"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let evidence = item.get("evidence").unwrap_or(&Value::Null);
                    let state_changed = evidence["state_changed"].as_bool().unwrap_or(false);
                    let restored_to = evidence["restored_to"].as_str().unwrap_or_default();
                    if !state_changed && restored_to.is_empty() {
                        return None;
                    }
                    Some(EmployeeCuratorChangedTarget {
                        kind: item["kind"].as_str().unwrap_or_default().to_string(),
                        target_type: item["target_type"].as_str().unwrap_or_default().to_string(),
                        target_id: item["target_id"].as_str().unwrap_or_default().to_string(),
                        state_changed,
                        restored_to: restored_to.to_string(),
                        suggested_action: item["suggested_action"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        reversible: item["reversible"].as_bool().unwrap_or(false),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_curator_restore_candidates(report: &Value) -> Vec<EmployeeCuratorRestoreCandidate> {
    report["findings"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let kind = item["kind"].as_str().unwrap_or_default();
                    let target_type = item["target_type"].as_str().unwrap_or_default();
                    let target_id = item["target_id"].as_str().unwrap_or_default();
                    let evidence = item.get("evidence").unwrap_or(&Value::Null);
                    let state_changed = evidence["state_changed"].as_bool().unwrap_or(false);
                    let reversible = item["reversible"].as_bool().unwrap_or(false);
                    if kind != "stale_skill"
                        || target_type != "skill"
                        || target_id.is_empty()
                        || !state_changed
                        || !reversible
                    {
                        return None;
                    }
                    Some(EmployeeCuratorRestoreCandidate {
                        target_type: "skill".to_string(),
                        target_id: target_id.to_string(),
                        tool: "curator".to_string(),
                        action: "restore".to_string(),
                        input: serde_json::json!({
                            "action": "restore",
                            "skill_id": target_id
                        }),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn project_employee_curator_run(
    id: String,
    profile_id: String,
    scope: String,
    summary: String,
    report_json: String,
    report_path: String,
    created_at: String,
) -> EmployeeCuratorRun {
    let report =
        serde_json::from_str::<Value>(&report_json).unwrap_or_else(|_| serde_json::json!({}));
    let changed_targets = parse_curator_changed_targets(&report);
    let restore_candidates = parse_curator_restore_candidates(&report);
    EmployeeCuratorRun {
        id,
        profile_id,
        scope,
        summary,
        report_path,
        mode: report["mode"].as_str().unwrap_or("scan").to_string(),
        has_state_changes: !changed_targets.is_empty(),
        changed_targets,
        restore_candidates,
        findings: parse_curator_findings(&report_json),
        created_at,
    }
}

pub async fn list_employee_curator_runs_with_pool(
    pool: &SqlitePool,
    employee_id: &str,
    limit: i64,
) -> Result<EmployeeCuratorReports, String> {
    let employee_id = employee_id.trim();
    if employee_id.is_empty() {
        return Err("employee_id 不能为空".to_string());
    }
    let profile_id = resolve_employee_profile_id_for_growth_with_pool(pool, employee_id).await?;
    if !table_exists(pool, "curator_runs").await? {
        return Ok(EmployeeCuratorReports {
            employee_id: employee_id.to_string(),
            profile_id,
            runs: Vec::new(),
        });
    }
    let query_profile_id = profile_id.as_deref().unwrap_or(employee_id);
    let limit = limit.clamp(1, 50);
    let rows = sqlx::query_as::<_, (String, String, String, String, String, String, String)>(
        "SELECT id, profile_id, scope, summary, report_json, report_path, created_at
         FROM curator_runs
         WHERE profile_id = ?
         ORDER BY created_at DESC, id DESC
         LIMIT ?",
    )
    .bind(query_profile_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("读取 curator_runs 失败: {e}"))?;
    let runs = rows
        .into_iter()
        .map(
            |(id, profile_id, scope, summary, report_json, report_path, created_at)| {
                project_employee_curator_run(
                    id,
                    profile_id,
                    scope,
                    summary,
                    report_json,
                    report_path,
                    created_at,
                )
            },
        )
        .collect();

    Ok(EmployeeCuratorReports {
        employee_id: employee_id.to_string(),
        profile_id,
        runs,
    })
}

async fn resolve_employee_profile_for_curator_with_pool(
    pool: &SqlitePool,
    employee_id: &str,
) -> Result<(String, String), String> {
    if !table_exists(pool, "agent_profiles").await? {
        return Ok((employee_id.to_string(), String::new()));
    }
    sqlx::query_as::<_, (String, String)>(
        "SELECT id, COALESCE(profile_home, '')
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
    .map(|row| row.unwrap_or_else(|| (employee_id.to_string(), String::new())))
}

fn sanitize_curator_profile_path_component(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "." || trimmed == ".." {
        return None;
    }
    let mut out = String::new();
    let mut prev_sep = false;
    for ch in trimmed.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
            prev_sep = false;
        } else if !prev_sep {
            out.push('_');
            prev_sep = true;
        }
    }
    let normalized = out.trim_matches('_').to_string();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

pub(crate) fn resolve_profile_home_for_curator(
    profile_id: &str,
    profile_home: &str,
    runtime_root: Option<&Path>,
) -> Option<String> {
    let trimmed_home = profile_home.trim();
    if !trimmed_home.is_empty() {
        return Some(trimmed_home.to_string());
    }
    let profile_id = sanitize_curator_profile_path_component(profile_id)?;
    let root = runtime_root
        .map(Path::to_path_buf)
        .unwrap_or_else(crate::runtime_paths::resolve_runtime_root);
    Some(
        root.join("profiles")
            .join(profile_id)
            .to_string_lossy()
            .to_string(),
    )
}

async fn ensure_curator_runs_schema_with_pool(pool: &SqlitePool) -> Result<(), String> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS curator_runs (
            id TEXT PRIMARY KEY,
            profile_id TEXT NOT NULL DEFAULT '',
            scope TEXT NOT NULL DEFAULT 'profile',
            summary TEXT NOT NULL DEFAULT '',
            report_json TEXT NOT NULL DEFAULT '{}',
            report_path TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| format!("创建 curator_runs 表失败: {e}"))?;
    Ok(())
}

async fn ensure_growth_events_schema_for_curator_with_pool(
    pool: &SqlitePool,
) -> Result<(), String> {
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
    Ok(())
}

pub async fn restore_employee_curator_stale_skill_with_pool(
    pool: &SqlitePool,
    employee_id: &str,
    skill_id: &str,
    runtime_root: Option<&Path>,
) -> Result<EmployeeCuratorRun, String> {
    let employee_id = employee_id.trim();
    let skill_id = skill_id.trim();
    if employee_id.is_empty() {
        return Err("employee_id 不能为空".to_string());
    }
    if skill_id.is_empty() {
        return Err("skill_id 不能为空".to_string());
    }
    let (profile_id, profile_home) =
        resolve_employee_profile_for_curator_with_pool(pool, employee_id).await?;
    let profile_home = resolve_profile_home_for_curator(&profile_id, &profile_home, runtime_root)
        .unwrap_or_default();
    let restored =
        crate::agent::runtime::runtime_io::restore_stale_skill_os_with_pool(pool, skill_id).await?;
    let run_id = format!("cur_{}", uuid::Uuid::new_v4().simple());
    let created_at = chrono::Utc::now().to_rfc3339();
    let summary = if restored {
        format!("已将 stale skill 恢复为 active: {skill_id}")
    } else {
        format!("未执行恢复：Skill 当前不是 stale: {skill_id}")
    };
    let report = serde_json::json!({
        "run_id": run_id,
        "profile_id": profile_id,
        "scope": "profile",
        "mode": "restore",
        "summary": summary,
        "created_at": created_at,
        "findings": [{
            "kind": "curator_restore",
            "severity": "low",
            "target_type": "skill",
            "target_id": skill_id,
            "summary": summary,
            "evidence": {
                "state_changed": restored,
                "restored_to": if restored { "active" } else { "" }
            },
            "suggested_action": if restored {
                "继续观察该技能的 use_count、patch_count 和后续任务表现"
            } else {
                "无需恢复；如需恢复 archived skill，请使用 skills.skill_restore"
            },
            "reversible": true
        }]
    });
    let report_path = if profile_home.trim().is_empty() {
        String::new()
    } else {
        let path = std::path::PathBuf::from(profile_home)
            .join("curator")
            .join("reports")
            .join(format!("{run_id}.json"));
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建 curator report 目录失败: {e}"))?;
        }
        std::fs::write(
            &path,
            serde_json::to_string_pretty(&report)
                .map_err(|e| format!("序列化 curator report 失败: {e}"))?,
        )
        .map_err(|e| format!("写入 curator report 失败: {e}"))?;
        path.to_string_lossy().to_string()
    };
    let report_json =
        serde_json::to_string(&report).map_err(|e| format!("序列化 curator report 失败: {e}"))?;
    ensure_curator_runs_schema_with_pool(pool).await?;
    ensure_growth_events_schema_for_curator_with_pool(pool).await?;
    sqlx::query(
        "INSERT INTO curator_runs (id, profile_id, scope, summary, report_json, report_path, created_at)
         VALUES (?, ?, 'profile', ?, ?, ?, ?)",
    )
    .bind(&run_id)
    .bind(&profile_id)
    .bind(&summary)
    .bind(&report_json)
    .bind(&report_path)
    .bind(&created_at)
    .execute(pool)
    .await
    .map_err(|e| format!("写入 curator_runs 失败: {e}"))?;
    sqlx::query(
        "INSERT INTO growth_events (
            id, profile_id, session_id, event_type, target_type, target_id, summary, evidence_json, created_at
         ) VALUES (?, ?, '', 'curator_restore', 'curator', ?, ?, ?, ?)",
    )
    .bind(format!("grw_{}", uuid::Uuid::new_v4().simple()))
    .bind(&profile_id)
    .bind(&run_id)
    .bind(&summary)
    .bind(&report_json)
    .bind(&created_at)
    .execute(pool)
    .await
    .map_err(|e| format!("写入 curator growth event 失败: {e}"))?;

    Ok(project_employee_curator_run(
        run_id,
        profile_id,
        "profile".to_string(),
        summary,
        report_json,
        report_path,
        created_at,
    ))
}

pub async fn scan_employee_curator_profile_with_pool(
    pool: &SqlitePool,
    employee_id: &str,
    mode: Option<String>,
    runtime_root: Option<&Path>,
) -> Result<EmployeeCuratorRun, String> {
    let employee_id = employee_id.trim();
    if employee_id.is_empty() {
        return Err("employee_id 不能为空".to_string());
    }
    let normalized_mode = mode.unwrap_or_else(|| "scan".to_string());
    let mutate = match normalized_mode.trim() {
        "" | "scan" => false,
        "run" => true,
        other => return Err(format!("未知 Curator 模式: {other}")),
    };
    let (profile_id, profile_home) =
        resolve_employee_profile_for_curator_with_pool(pool, employee_id).await?;
    let profile_home = resolve_profile_home_for_curator(&profile_id, &profile_home, runtime_root)
        .unwrap_or_default();
    let memory_dir = if profile_home.trim().is_empty() {
        std::path::PathBuf::new()
    } else {
        std::path::PathBuf::from(&profile_home).join("memories")
    };
    let result = crate::agent::tools::CuratorTool::scan_profile_with_pool(
        pool.clone(),
        profile_id.clone(),
        memory_dir,
        mutate,
    )
    .await?;
    let run_id = result["run_id"]
        .as_str()
        .ok_or_else(|| "Curator scan 未返回 run_id".to_string())?;
    let row = sqlx::query_as::<_, (String, String, String, String, String, String, String)>(
        "SELECT id, profile_id, scope, summary, report_json, report_path, created_at
         FROM curator_runs
         WHERE id = ?
         LIMIT 1",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("读取 Curator scan 结果失败: {e}"))?;

    Ok(project_employee_curator_run(
        row.0, row.1, row.2, row.3, row.4, row.5, row.6,
    ))
}

#[tauri::command]
pub async fn get_employee_profile_memory_status(
    employee_id: String,
    skill_id: String,
    profile_id: Option<String>,
    work_dir: Option<String>,
    im_role_id: Option<String>,
    app: tauri::AppHandle,
    db: State<'_, DbState>,
) -> Result<EmployeeProfileMemoryStatus, String> {
    memory_commands::get_employee_profile_memory_status(
        employee_id,
        skill_id,
        profile_id,
        work_dir,
        im_role_id,
        app,
        db,
    )
    .await
}

#[tauri::command]
pub async fn scan_employee_curator_profile(
    employee_id: String,
    mode: Option<String>,
    app: tauri::AppHandle,
    db: State<'_, DbState>,
) -> Result<EmployeeCuratorRun, String> {
    let runtime_paths = crate::runtime_environment::runtime_paths_from_app(&app)?;
    scan_employee_curator_profile_with_pool(&db.0, &employee_id, mode, Some(&runtime_paths.root))
        .await
}

#[tauri::command]
pub async fn list_employee_growth_events(
    employee_id: String,
    limit: Option<i64>,
    db: State<'_, DbState>,
) -> Result<EmployeeGrowthTimeline, String> {
    list_employee_growth_events_with_pool(&db.0, &employee_id, limit.unwrap_or(20)).await
}

#[tauri::command]
pub async fn list_employee_curator_runs(
    employee_id: String,
    limit: Option<i64>,
    db: State<'_, DbState>,
) -> Result<EmployeeCuratorReports, String> {
    list_employee_curator_runs_with_pool(&db.0, &employee_id, limit.unwrap_or(10)).await
}

#[tauri::command]
pub async fn restore_employee_curator_stale_skill(
    employee_id: String,
    skill_id: String,
    app: tauri::AppHandle,
    db: State<'_, DbState>,
) -> Result<EmployeeCuratorRun, String> {
    let runtime_paths = crate::runtime_environment::runtime_paths_from_app(&app)?;
    restore_employee_curator_stale_skill_with_pool(
        &db.0,
        &employee_id,
        &skill_id,
        Some(&runtime_paths.root),
    )
    .await
}

#[tauri::command]
pub async fn create_employee_group(
    input: CreateEmployeeGroupInput,
    db: State<'_, DbState>,
) -> Result<String, String> {
    tauri_commands::create_employee_group(input, db).await
}

#[tauri::command]
pub async fn create_employee_team(
    input: CreateEmployeeTeamInput,
    db: State<'_, DbState>,
) -> Result<String, String> {
    tauri_commands::create_employee_team(input, db).await
}

#[tauri::command]
pub async fn clone_employee_group_template(
    input: CloneEmployeeGroupTemplateInput,
    db: State<'_, DbState>,
) -> Result<String, String> {
    tauri_commands::clone_employee_group_template(input, db).await
}

#[tauri::command]
pub async fn list_employee_groups(db: State<'_, DbState>) -> Result<Vec<EmployeeGroup>, String> {
    tauri_commands::list_employee_groups(db).await
}

#[tauri::command]
pub async fn list_employee_group_runs(
    limit: Option<i64>,
    db: State<'_, DbState>,
) -> Result<Vec<EmployeeGroupRunSummary>, String> {
    tauri_commands::list_employee_group_runs(limit, db).await
}

#[tauri::command]
pub async fn list_employee_group_rules(
    group_id: String,
    db: State<'_, DbState>,
) -> Result<Vec<EmployeeGroupRule>, String> {
    tauri_commands::list_employee_group_rules(group_id, db).await
}

#[tauri::command]
pub async fn delete_employee_group(group_id: String, db: State<'_, DbState>) -> Result<(), String> {
    tauri_commands::delete_employee_group(group_id, db).await
}

#[tauri::command]
pub async fn start_employee_group_run(
    input: StartEmployeeGroupRunInput,
    db: State<'_, DbState>,
    journal: State<'_, crate::session_journal::SessionJournalStateHandle>,
) -> Result<EmployeeGroupRunResult, String> {
    tauri_commands::start_employee_group_run(input, db, journal).await
}

#[tauri::command]
pub async fn continue_employee_group_run(
    run_id: String,
    db: State<'_, DbState>,
    journal: State<'_, crate::session_journal::SessionJournalStateHandle>,
) -> Result<EmployeeGroupRunSnapshot, String> {
    tauri_commands::continue_employee_group_run(run_id, db, journal).await
}

#[tauri::command]
pub async fn run_group_step(
    step_id: String,
    db: State<'_, DbState>,
    journal: State<'_, crate::session_journal::SessionJournalStateHandle>,
) -> Result<GroupStepExecutionResult, String> {
    tauri_commands::run_group_step(step_id, db, journal).await
}

#[tauri::command]
pub async fn get_employee_group_run_snapshot(
    session_id: String,
    db: State<'_, DbState>,
) -> Result<Option<EmployeeGroupRunSnapshot>, String> {
    tauri_commands::get_employee_group_run_snapshot(session_id, db).await
}

#[tauri::command]
pub async fn cancel_employee_group_run(
    run_id: String,
    db: State<'_, DbState>,
) -> Result<(), String> {
    tauri_commands::cancel_employee_group_run(run_id, db).await
}

#[tauri::command]
pub async fn retry_employee_group_run_failed_steps(
    run_id: String,
    db: State<'_, DbState>,
) -> Result<(), String> {
    tauri_commands::retry_employee_group_run_failed_steps(run_id, db).await
}

#[tauri::command]
pub async fn review_group_run_step(
    run_id: String,
    action: String,
    comment: String,
    db: State<'_, DbState>,
) -> Result<(), String> {
    tauri_commands::review_group_run_step(run_id, action, comment, db).await
}

#[tauri::command]
pub async fn pause_employee_group_run(
    run_id: String,
    reason: Option<String>,
    db: State<'_, DbState>,
) -> Result<(), String> {
    tauri_commands::pause_employee_group_run(run_id, reason, db).await
}

#[tauri::command]
pub async fn resume_employee_group_run(
    run_id: String,
    db: State<'_, DbState>,
) -> Result<(), String> {
    tauri_commands::resume_employee_group_run(run_id, db).await
}

#[tauri::command]
pub async fn reassign_group_run_step(
    step_id: String,
    assignee_employee_id: String,
    db: State<'_, DbState>,
) -> Result<(), String> {
    tauri_commands::reassign_group_run_step(step_id, assignee_employee_id, db).await
}

#[tauri::command]
pub async fn list_agent_employees(db: State<'_, DbState>) -> Result<Vec<AgentEmployee>, String> {
    tauri_commands::list_agent_employees(db).await
}

#[tauri::command]
pub async fn upsert_agent_employee(
    input: UpsertAgentEmployeeInput,
    db: State<'_, DbState>,
    relay: State<'_, crate::commands::feishu_gateway::FeishuEventRelayState>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    tauri_commands::upsert_agent_employee(input, db, relay, app).await
}

#[tauri::command]
pub async fn save_feishu_employee_association(
    input: SaveFeishuEmployeeAssociationInput,
    db: State<'_, DbState>,
    relay: State<'_, crate::commands::feishu_gateway::FeishuEventRelayState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    tauri_commands::save_feishu_employee_association(input, db, relay, app).await
}

#[tauri::command]
pub async fn delete_agent_employee(
    employee_id: String,
    db: State<'_, DbState>,
    relay: State<'_, crate::commands::feishu_gateway::FeishuEventRelayState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    tauri_commands::delete_agent_employee(employee_id, db, relay, app).await
}

#[cfg(test)]
mod tests {
    #[test]
    fn im_binding_matches_event_respects_non_feishu_channels() {
        let binding = crate::commands::im_routing::ImRoutingBinding {
            id: "binding-1".to_string(),
            agent_id: "architect".to_string(),
            channel: "wecom".to_string(),
            account_id: "tenant-wecom".to_string(),
            peer_kind: "group".to_string(),
            peer_id: "wecom-room-1".to_string(),
            guild_id: String::new(),
            team_id: String::new(),
            role_ids: Vec::new(),
            connector_meta: serde_json::json!({}),
            priority: 100,
            enabled: true,
            created_at: "2026-03-11T00:00:00Z".to_string(),
            updated_at: "2026-03-11T00:00:00Z".to_string(),
        };

        let wecom_event = crate::im::types::ImEvent {
            channel: "wecom".to_string(),
            event_type: crate::im::types::ImEventType::MessageCreated,
            thread_id: "wecom-room-1".to_string(),
            event_id: Some("evt-wecom".to_string()),
            message_id: Some("msg-wecom".to_string()),
            text: Some("企业微信消息".to_string()),
            role_id: None,
            account_id: Some("tenant-wecom".to_string()),
            tenant_id: Some("tenant-wecom".to_string()),
            sender_id: None,
            chat_type: Some("group".to_string()),
            conversation_id: Some("wecom:tenant-wecom:group:wecom-room-1".to_string()),
            base_conversation_id: Some("wecom:tenant-wecom:group:wecom-room-1".to_string()),
            parent_conversation_candidates: Vec::new(),
            conversation_scope: Some("peer".to_string()),
        };
        assert!(super::im_binding_matches_event(&binding, &wecom_event));

        let feishu_event = crate::im::types::ImEvent {
            channel: "feishu".to_string(),
            ..wecom_event.clone()
        };
        assert!(!super::im_binding_matches_event(&binding, &feishu_event));
    }

    #[test]
    fn build_route_session_key_uses_event_channel_namespace() {
        let employee = super::AgentEmployee {
            id: "emp-1".to_string(),
            employee_id: "main".to_string(),
            name: "主员工".to_string(),
            role_id: "main".to_string(),
            persona: String::new(),
            feishu_open_id: String::new(),
            feishu_app_id: String::new(),
            feishu_app_secret: String::new(),
            primary_skill_id: "builtin-general".to_string(),
            default_work_dir: String::new(),
            openclaw_agent_id: "main-agent".to_string(),
            routing_priority: 100,
            enabled_scopes: vec!["app".to_string()],
            enabled: true,
            is_default: true,
            skill_ids: Vec::new(),
            created_at: "2026-03-11T00:00:00Z".to_string(),
            updated_at: "2026-03-11T00:00:00Z".to_string(),
        };

        let wecom_event = crate::im::types::ImEvent {
            channel: "wecom".to_string(),
            event_type: crate::im::types::ImEventType::MessageCreated,
            thread_id: "wecom-room-1".to_string(),
            event_id: Some("evt-wecom".to_string()),
            message_id: Some("msg-wecom".to_string()),
            text: Some("企业微信消息".to_string()),
            role_id: None,
            account_id: Some("tenant-wecom".to_string()),
            tenant_id: Some("tenant-wecom".to_string()),
            sender_id: None,
            chat_type: Some("group".to_string()),
            conversation_id: Some("wecom:tenant-wecom:group:wecom-room-1".to_string()),
            base_conversation_id: Some("wecom:tenant-wecom:group:wecom-room-1".to_string()),
            parent_conversation_candidates: Vec::new(),
            conversation_scope: Some("peer".to_string()),
        };

        assert_eq!(
            super::build_route_session_key(&wecom_event, &employee),
            "wecom:tenant-wecom:main-agent:wecom:tenant-wecom:group:wecom-room-1"
        );
    }

    #[test]
    fn build_route_session_key_defaults_empty_channel_to_app_namespace() {
        let employee = super::AgentEmployee {
            id: "emp-1".to_string(),
            employee_id: "main".to_string(),
            name: "主员工".to_string(),
            role_id: "main".to_string(),
            persona: String::new(),
            feishu_open_id: String::new(),
            feishu_app_id: String::new(),
            feishu_app_secret: String::new(),
            primary_skill_id: "builtin-general".to_string(),
            default_work_dir: String::new(),
            openclaw_agent_id: "main-agent".to_string(),
            routing_priority: 100,
            enabled_scopes: vec!["app".to_string()],
            enabled: true,
            is_default: true,
            skill_ids: Vec::new(),
            created_at: "2026-03-11T00:00:00Z".to_string(),
            updated_at: "2026-03-11T00:00:00Z".to_string(),
        };

        let app_event = crate::im::types::ImEvent {
            channel: String::new(),
            event_type: crate::im::types::ImEventType::MessageCreated,
            thread_id: "room-1".to_string(),
            event_id: Some("evt-app".to_string()),
            message_id: Some("msg-app".to_string()),
            text: Some("本地消息".to_string()),
            role_id: None,
            account_id: None,
            tenant_id: None,
            sender_id: None,
            chat_type: None,
            conversation_id: None,
            base_conversation_id: None,
            parent_conversation_candidates: Vec::new(),
            conversation_scope: None,
        };

        assert_eq!(
            super::build_route_session_key(&app_event, &employee),
            "app:default:main-agent:room-1"
        );
    }

    #[test]
    fn normalize_enabled_scopes_defaults_to_app_scope() {
        assert_eq!(
            super::normalize_enabled_scopes_for_storage(&[]),
            vec!["app".to_string()]
        );
        assert_eq!(
            super::normalize_enabled_scopes_for_storage(&["wecom".to_string()]),
            vec!["wecom".to_string()]
        );
    }
}
