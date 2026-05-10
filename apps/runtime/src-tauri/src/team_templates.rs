use crate::commands::agent_profile::{apply_agent_profile_draft_with_pool, AgentProfileDraft};
use crate::commands::employee_agents::{
    create_employee_group_with_pool, upsert_agent_employee_with_pool, CreateEmployeeGroupInput,
    UpsertAgentEmployeeInput,
};
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const SANSHENG_LIUBU_TEMPLATE_JSON: &str =
    include_str!("../builtin-team-templates/sansheng-liubu.json");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TeamTemplate {
    pub template_id: String,
    pub template_version: String,
    pub seed_on_first_run: bool,
    pub name: String,
    pub description: String,
    pub default_entry_employee_key: String,
    #[serde(default)]
    pub roles: Vec<TeamTemplateRole>,
    #[serde(default)]
    pub employees: Vec<TeamTemplateEmployee>,
    #[serde(default)]
    pub rules: Vec<TeamTemplateRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TeamTemplateRole {
    pub role_type: String,
    pub employee_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TeamTemplateEmployee {
    pub employee_key: String,
    pub employee_id: String,
    pub name: String,
    #[serde(default)]
    pub persona: String,
    #[serde(default)]
    pub primary_skill_id: String,
    #[serde(default)]
    pub default_work_dir: String,
    #[serde(default)]
    pub enabled_scopes: Vec<String>,
    #[serde(default)]
    pub outward_facing: bool,
    #[serde(default)]
    pub profile_templates: TeamTemplateProfileTemplates,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TeamTemplateProfileTemplates {
    #[serde(default)]
    pub agents_md_template: String,
    #[serde(default)]
    pub soul_md_template: String,
    #[serde(default)]
    pub user_md_template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TeamTemplateRule {
    pub from_employee_id: String,
    pub to_employee_id: String,
    pub relation_type: String,
    #[serde(default)]
    pub phase_scope: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default = "default_rule_priority")]
    pub priority: i32,
}

const fn default_rule_priority() -> i32 {
    100
}

pub fn load_builtin_template(template_id: &str) -> Result<TeamTemplate> {
    let raw = builtin_template_json(template_id)
        .ok_or_else(|| anyhow!("unknown builtin team template: {}", template_id))?;
    serde_json::from_str(raw)
        .with_context(|| format!("failed to parse builtin team template {}", template_id))
}

pub fn list_builtin_templates() -> Result<Vec<TeamTemplate>> {
    builtin_template_ids()
        .iter()
        .map(|template_id| load_builtin_template(template_id))
        .collect()
}

pub fn list_seedable_builtin_templates() -> Result<Vec<TeamTemplate>> {
    Ok(list_builtin_templates()?
        .into_iter()
        .filter(|template| template.seed_on_first_run)
        .collect())
}

pub fn builtin_template_ids() -> &'static [&'static str] {
    &["sansheng-liubu"]
}

fn builtin_template_json(template_id: &str) -> Option<&'static str> {
    match template_id {
        "sansheng-liubu" => Some(SANSHENG_LIUBU_TEMPLATE_JSON),
        _ => None,
    }
}

pub async fn seed_builtin_team_templates_with_root(
    pool: &SqlitePool,
    seed_root: &Path,
) -> Result<()> {
    std::fs::create_dir_all(seed_root).with_context(|| {
        format!(
            "failed to create team template seed root {}",
            seed_root.display()
        )
    })?;

    for template in list_seedable_builtin_templates()? {
        if template_seed_record_exists(pool, &template.template_id).await? {
            continue;
        }
        seed_team_template_with_root(pool, seed_root, &template).await?;
    }

    Ok(())
}

async fn template_seed_record_exists(pool: &SqlitePool, template_id: &str) -> Result<bool> {
    let existing = sqlx::query_as::<_, (String,)>(
        "SELECT template_id FROM seeded_team_templates WHERE template_id = ? LIMIT 1",
    )
    .bind(template_id)
    .fetch_optional(pool)
    .await?;
    Ok(existing.is_some())
}

async fn seed_team_template_with_root(
    pool: &SqlitePool,
    seed_root: &Path,
    template: &TeamTemplate,
) -> Result<()> {
    let member_employee_ids = template
        .employees
        .iter()
        .map(|employee| employee.employee_id.clone())
        .collect::<Vec<_>>();
    let entry_employee_id = employee_id_for_key(template, &template.default_entry_employee_key)?;
    let coordinator_employee_id =
        employee_id_for_role(template, "coordinator").unwrap_or_else(|_| entry_employee_id.clone());

    upsert_template_employees(pool, seed_root, template).await?;
    let group_id = upsert_template_group(
        pool,
        template,
        &coordinator_employee_id,
        &entry_employee_id,
        &member_employee_ids,
    )
    .await?;
    sync_template_rules(pool, &group_id, template).await?;
    upsert_template_seed_record(pool, template, &group_id, &member_employee_ids).await?;

    Ok(())
}

async fn upsert_template_employees(
    pool: &SqlitePool,
    seed_root: &Path,
    template: &TeamTemplate,
) -> Result<()> {
    for employee in &template.employees {
        let primary_skill_id = default_primary_skill_id(employee);
        let employee_db_id = upsert_agent_employee_with_pool(
            pool,
            UpsertAgentEmployeeInput {
                id: None,
                employee_id: employee.employee_id.clone(),
                name: employee.name.clone(),
                role_id: employee.employee_id.clone(),
                persona: employee.persona.clone(),
                feishu_open_id: String::new(),
                feishu_app_id: String::new(),
                feishu_app_secret: String::new(),
                primary_skill_id: primary_skill_id.clone(),
                default_work_dir: resolve_employee_work_dir(seed_root, employee),
                openclaw_agent_id: employee.employee_id.clone(),
                routing_priority: default_routing_priority_for_employee(employee),
                enabled_scopes: normalize_enabled_scopes(&employee.enabled_scopes),
                enabled: true,
                is_default: employee.employee_key == template.default_entry_employee_key,
                skill_ids: vec![primary_skill_id],
            },
        )
        .await
        .map_err(|error| anyhow!(error))?;

        apply_agent_profile_draft_with_pool(pool, &employee_db_id, build_profile_draft(employee))
            .await
            .map_err(|error| anyhow!(error))?;
    }

    Ok(())
}

async fn upsert_template_group(
    pool: &SqlitePool,
    template: &TeamTemplate,
    coordinator_employee_id: &str,
    entry_employee_id: &str,
    member_employee_ids: &[String],
) -> Result<String> {
    let now = chrono::Utc::now().to_rfc3339();
    let member_employee_ids_json = serde_json::to_string(member_employee_ids)?;
    let group_config_json = serde_json::to_string(&serde_json::json!({
        "template_id": template.template_id,
        "template_version": template.template_version,
        "roles": template.roles,
    }))?;

    let existing_group_id = sqlx::query_as::<_, (String,)>(
        "SELECT id FROM employee_groups WHERE template_id = ? LIMIT 1",
    )
    .bind(&template.template_id)
    .fetch_optional(pool)
    .await?
    .map(|(group_id,)| group_id);

    let group_id = if let Some(group_id) = existing_group_id {
        sqlx::query(
            "UPDATE employee_groups
             SET name = ?,
                 coordinator_employee_id = ?,
                 member_employee_ids_json = ?,
                 member_count = ?,
                 template_id = ?,
                 entry_employee_id = ?,
                 review_mode = ?,
                 execution_mode = ?,
                 visibility_mode = ?,
                 is_bootstrap_seeded = 1,
                 config_json = ?,
                 updated_at = ?
             WHERE id = ?",
        )
        .bind(&template.name)
        .bind(coordinator_employee_id)
        .bind(&member_employee_ids_json)
        .bind(member_employee_ids.len() as i64)
        .bind(&template.template_id)
        .bind(entry_employee_id)
        .bind(default_review_mode(template))
        .bind("staged")
        .bind(default_visibility_mode(template))
        .bind(&group_config_json)
        .bind(&now)
        .bind(&group_id)
        .execute(pool)
        .await?;
        group_id
    } else {
        let group_id = create_employee_group_with_pool(
            pool,
            CreateEmployeeGroupInput {
                name: template.name.clone(),
                coordinator_employee_id: coordinator_employee_id.to_string(),
                member_employee_ids: member_employee_ids.to_vec(),
            },
        )
        .await
        .map_err(|error| anyhow!(error))?;
        sqlx::query(
            "UPDATE employee_groups
             SET template_id = ?,
                 entry_employee_id = ?,
                 review_mode = ?,
                 execution_mode = ?,
                 visibility_mode = ?,
                 is_bootstrap_seeded = 1,
                 config_json = ?,
                 updated_at = ?
             WHERE id = ?",
        )
        .bind(&template.template_id)
        .bind(entry_employee_id)
        .bind(default_review_mode(template))
        .bind("staged")
        .bind(default_visibility_mode(template))
        .bind(&group_config_json)
        .bind(&now)
        .bind(&group_id)
        .execute(pool)
        .await?;
        group_id
    };

    Ok(group_id)
}

async fn sync_template_rules(
    pool: &SqlitePool,
    group_id: &str,
    template: &TeamTemplate,
) -> Result<()> {
    sqlx::query("DELETE FROM employee_group_rules WHERE group_id = ?")
        .bind(group_id)
        .execute(pool)
        .await?;

    let now = chrono::Utc::now().to_rfc3339();
    for rule in &template.rules {
        sqlx::query(
            "INSERT INTO employee_group_rules (
                id, group_id, from_employee_id, to_employee_id, relation_type, phase_scope, required, priority, created_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(group_id)
        .bind(&rule.from_employee_id)
        .bind(&rule.to_employee_id)
        .bind(&rule.relation_type)
        .bind(&rule.phase_scope)
        .bind(if rule.required { 1 } else { 0 })
        .bind(rule.priority)
        .bind(&now)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn upsert_template_seed_record(
    pool: &SqlitePool,
    template: &TeamTemplate,
    group_id: &str,
    member_employee_ids: &[String],
) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    let employee_ids_json = serde_json::to_string(member_employee_ids)?;
    sqlx::query(
        "INSERT INTO seeded_team_templates (
            template_id, template_version, instance_group_id, instance_employee_ids_json, seed_mode, seeded_at
         ) VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(template_id) DO UPDATE SET
            template_version = excluded.template_version,
            instance_group_id = excluded.instance_group_id,
            instance_employee_ids_json = excluded.instance_employee_ids_json,
            seed_mode = excluded.seed_mode,
            seeded_at = excluded.seeded_at",
    )
    .bind(&template.template_id)
    .bind(&template.template_version)
    .bind(group_id)
    .bind(employee_ids_json)
    .bind("first_run")
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(())
}

fn employee_id_for_key(template: &TeamTemplate, employee_key: &str) -> Result<String> {
    template
        .employees
        .iter()
        .find(|employee| employee.employee_key == employee_key)
        .map(|employee| employee.employee_id.clone())
        .ok_or_else(|| {
            anyhow!(
                "template {} missing employee key {}",
                template.template_id,
                employee_key
            )
        })
}

fn employee_id_for_role(template: &TeamTemplate, role_type: &str) -> Result<String> {
    let employee_key = template
        .roles
        .iter()
        .find(|role| role.role_type == role_type)
        .map(|role| role.employee_key.clone())
        .ok_or_else(|| {
            anyhow!(
                "template {} missing role {}",
                template.template_id,
                role_type
            )
        })?;
    employee_id_for_key(template, &employee_key)
}

fn default_primary_skill_id(employee: &TeamTemplateEmployee) -> String {
    if employee.primary_skill_id.trim().is_empty() {
        "builtin-general".to_string()
    } else {
        employee.primary_skill_id.trim().to_string()
    }
}

fn normalize_enabled_scopes(enabled_scopes: &[String]) -> Vec<String> {
    if enabled_scopes.is_empty() {
        vec!["app".to_string()]
    } else {
        enabled_scopes.to_vec()
    }
}

fn resolve_employee_work_dir(seed_root: &Path, employee: &TeamTemplateEmployee) -> String {
    if !employee.default_work_dir.trim().is_empty() {
        return employee.default_work_dir.trim().to_string();
    }

    build_employee_seed_root(seed_root, &employee.employee_id)
        .to_string_lossy()
        .to_string()
}

fn build_employee_seed_root(seed_root: &Path, employee_id: &str) -> PathBuf {
    seed_root.join("employees").join(employee_id.trim())
}

fn default_review_mode(template: &TeamTemplate) -> &'static str {
    if template
        .rules
        .iter()
        .any(|rule| rule.relation_type.eq_ignore_ascii_case("review"))
    {
        "hard"
    } else {
        "none"
    }
}

fn default_visibility_mode(template: &TeamTemplate) -> &'static str {
    if template
        .employees
        .iter()
        .any(|employee| employee.outward_facing)
    {
        "entry_only"
    } else {
        "internal"
    }
}

fn default_routing_priority_for_employee(employee: &TeamTemplateEmployee) -> i64 {
    if employee.outward_facing {
        100
    } else {
        80
    }
}

fn build_profile_draft(employee: &TeamTemplateEmployee) -> AgentProfileDraft {
    let agents_md = non_empty_or_else(&employee.profile_templates.agents_md_template, || {
        format!(
            "# RULES\n\n## Agent\n- 名称: {name}\n- 员工编号: {employee_id}\n\n## Mission\n{persona}\n",
            name = employee.name,
            employee_id = employee.employee_id,
            persona = fallback_persona(employee),
        )
    });
    let soul_md = non_empty_or_else(&employee.profile_templates.soul_md_template, || {
        format!(
            "# PERSONA\n\n## Tone\n专业、简洁、可执行。\n\n## Working Style\n{persona}\n",
            persona = fallback_persona(employee),
        )
    });
    let user_md = non_empty_or_else(&employee.profile_templates.user_md_template, || {
        format!(
            "# USER_CONTEXT\n\n## External Role\n{name}\n\n## Collaboration Contract\n{persona}\n",
            name = employee.name,
            persona = fallback_persona(employee),
        )
    });

    AgentProfileDraft {
        employee_id: employee.employee_id.clone(),
        employee_name: employee.name.clone(),
        agents_md,
        soul_md,
        user_md,
    }
}

fn fallback_persona(employee: &TeamTemplateEmployee) -> &str {
    if employee.persona.trim().is_empty() {
        "负责在团队中完成分配给自己的工作。"
    } else {
        employee.persona.trim()
    }
}

fn non_empty_or_else<F>(value: &str, fallback: F) -> String
where
    F: FnOnce() -> String,
{
    if value.trim().is_empty() {
        fallback()
    } else {
        value.trim().to_string()
    }
}
