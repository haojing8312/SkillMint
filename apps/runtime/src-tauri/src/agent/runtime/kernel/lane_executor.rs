use crate::agent::runtime::attempt_runner::{execute_route_candidates, RouteExecutionParams};
use crate::agent::runtime::events::ToolConfirmResponder;
use crate::agent::runtime::kernel::execution_plan::{
    ExecutionContext, ExecutionOutcome, ExecutionPlan, TurnContext,
};
use crate::agent::runtime::kernel::route_lane::RouteRunOutcome;
use crate::agent::runtime::skill_routing::runner::execute_planned_route;
use crate::agent::AgentExecutor;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::AppHandle;

pub(crate) struct LaneExecutionParams<'a> {
    pub app: &'a AppHandle,
    pub agent_executor: &'a Arc<AgentExecutor>,
    pub db: &'a sqlx::SqlitePool,
    pub session_id: &'a str,
    pub run_id: &'a str,
    pub turn_context: &'a TurnContext,
    pub execution_context: &'a ExecutionContext,
    pub execution_plan: &'a ExecutionPlan,
    pub cancel_flag: Arc<AtomicBool>,
    pub tool_confirm_responder: ToolConfirmResponder,
}

pub(crate) async fn execute_execution_lane(
    params: LaneExecutionParams<'_>,
) -> Result<ExecutionOutcome, String> {
    match execute_planned_route(
        params.app,
        params.agent_executor,
        params.db,
        params.session_id,
        params.run_id,
        params.turn_context,
        params.execution_context,
        params.execution_plan,
        params.cancel_flag.clone(),
        params.tool_confirm_responder.clone(),
    )
    .await?
    {
        RouteRunOutcome::OpenTask => {
            let route_execution = execute_route_candidates(RouteExecutionParams {
                app: params.app,
                agent_executor: params.agent_executor.as_ref(),
                db: params.db,
                session_id: params.session_id,
                requested_capability: &params.turn_context.requested_capability,
                route_candidates: &params.turn_context.route_candidates,
                per_candidate_retry_count: params.turn_context.per_candidate_retry_count,
                system_prompt: &params.execution_context.system_prompt,
                messages: &params.turn_context.messages,
                allowed_tools: params.execution_context.allowed_tools(),
                permission_mode: params.execution_context.permission_mode,
                tool_confirm_responder: params.tool_confirm_responder,
                executor_work_dir: params.execution_context.executor_work_dir.clone(),
                max_iterations: params.execution_context.max_iterations,
                cancel_flag: params.cancel_flag,
                node_timeout_seconds: params.execution_context.node_timeout_seconds,
                route_retry_count: params.execution_context.route_retry_count,
            })
            .await;

            Ok(ExecutionOutcome::RouteExecution {
                route_execution,
                reconstructed_history_len: params.turn_context.messages.len(),
            })
        }
        RouteRunOutcome::DirectDispatch(output) => Ok(ExecutionOutcome::DirectDispatch(output)),
        RouteRunOutcome::Prompt {
            route_execution,
            reconstructed_history_len,
        } => Ok(ExecutionOutcome::RouteExecution {
            route_execution,
            reconstructed_history_len,
        }),
    }
}
