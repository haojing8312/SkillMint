use crate::agent::run_guard::{RunBudgetPolicy, RunBudgetScope};
use crate::agent::runtime::kernel::execution_plan::ExecutionContext;
use crate::agent::runtime::tool_setup::{prepare_runtime_tools, ToolSetupParams};
use crate::agent::AgentExecutor;
use runtime_chat_app::ChatExecutionPreparationService;
use std::sync::Arc;
use tauri::AppHandle;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedRoutedPrompt {
    pub allowed_tools: Option<Vec<String>>,
    pub system_prompt: String,
    pub max_iterations: usize,
}

#[derive(Clone)]
pub(crate) struct RoutedPromptPreparationParams<'a> {
    pub app: &'a AppHandle,
    pub db: &'a sqlx::SqlitePool,
    pub agent_executor: &'a Arc<AgentExecutor>,
    pub session_id: &'a str,
    pub execution_context: &'a ExecutionContext,
    pub api_format: &'a str,
    pub base_url: &'a str,
    pub model_name: &'a str,
    pub api_key: &'a str,
    pub skill_id: &'a str,
    pub skill_system_prompt: &'a str,
    pub skill_allowed_tools: Option<Vec<String>>,
    pub skill_max_iterations: Option<usize>,
    pub source_type: &'a str,
    pub pack_path: &'a str,
}

pub(crate) fn resolve_routed_prompt_max_iterations(
    skill_id: &str,
    configured_max_iterations: Option<usize>,
) -> usize {
    RunBudgetPolicy::resolve(
        if skill_id.eq_ignore_ascii_case("builtin-general") {
            RunBudgetScope::GeneralChat
        } else {
            RunBudgetScope::Skill
        },
        configured_max_iterations,
    )
    .max_turns
}

pub(crate) async fn prepare_routed_prompt(
    params: RoutedPromptPreparationParams<'_>,
) -> Result<PreparedRoutedPrompt, String> {
    let execution_preparation_service = ChatExecutionPreparationService::new();
    let max_iterations =
        resolve_routed_prompt_max_iterations(params.skill_id, params.skill_max_iterations);

    let prepared_runtime_tools = prepare_runtime_tools(ToolSetupParams {
        app: params.app,
        db: params.db,
        agent_executor: params.agent_executor,
        workspace_skill_entries: &params.execution_context.workspace_skill_entries,
        session_id: params.session_id,
        api_format: params.api_format,
        base_url: params.base_url,
        model_name: params.model_name,
        api_key: params.api_key,
        skill_id: params.skill_id,
        source_type: params.source_type,
        pack_path: params.pack_path,
        skill_system_prompt: params.skill_system_prompt,
        skill_allowed_tools: params.skill_allowed_tools,
        max_iter: max_iterations,
        max_call_depth: params.execution_context.max_call_depth,
        suppress_workspace_skills_prompt: false,
        execution_preparation_service: &execution_preparation_service,
        execution_guidance: &params.execution_context.execution_guidance,
        memory_bucket_employee_id: &params.execution_context.memory_bucket_employee_id,
        employee_collaboration_guidance: params
            .execution_context
            .employee_collaboration_guidance
            .as_deref(),
    })
    .await?;

    Ok(PreparedRoutedPrompt {
        allowed_tools: prepared_runtime_tools.allowed_tools,
        system_prompt: prepared_runtime_tools.system_prompt,
        max_iterations,
    })
}

#[cfg(test)]
mod tests {
    use super::resolve_routed_prompt_max_iterations;
    use crate::agent::run_guard::{RunBudgetPolicy, RunBudgetScope};

    #[test]
    fn resolve_routed_prompt_max_iterations_uses_general_chat_budget_for_builtin_general() {
        let resolved = resolve_routed_prompt_max_iterations("builtin-general", Some(7));

        assert_eq!(
            resolved,
            RunBudgetPolicy::resolve(RunBudgetScope::GeneralChat, Some(7)).max_turns
        );
    }

    #[test]
    fn resolve_routed_prompt_max_iterations_uses_skill_budget_for_regular_skills() {
        let resolved = resolve_routed_prompt_max_iterations("feishu-pm-hub", Some(7));

        assert_eq!(
            resolved,
            RunBudgetPolicy::resolve(RunBudgetScope::Skill, Some(7)).max_turns
        );
    }
}
