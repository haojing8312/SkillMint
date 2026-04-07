use super::events::ToolConfirmResponder;
use crate::agent::context::build_tool_context;
use crate::agent::run_guard::{RunBudgetPolicy, RunBudgetScope};
use crate::agent::runtime::attempt_runner::RouteExecutionOutcome;
use crate::agent::runtime::kernel::execution_plan::{
    ExecutionContext, ExecutionOutcome, SessionEngineError,
};
use crate::agent::runtime::kernel::outcome_commit::{OutcomeCommitter, TerminalOutcome};
use crate::agent::runtime::kernel::session_engine::SessionEngine;
use crate::agent::runtime::kernel::turn_preparation::parse_user_skill_command;
use crate::agent::AgentExecutor;
use crate::session_journal::SessionJournalStore;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::AppHandle;
use uuid::Uuid;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SessionRuntime;

enum SessionTurnCompletion {
    DirectDispatch(String),
    RouteExecution {
        route_execution: RouteExecutionOutcome,
        reconstructed_history_len: usize,
    },
    SkillCommandFailed(String),
    SkillCommandStopped {
        stop_reason: crate::agent::run_guard::RunStopReason,
        error: String,
    },
    GenericError(String),
}

impl SessionRuntime {
    pub fn new() -> Self {
        Self
    }

    pub(crate) async fn maybe_execute_user_skill_command(
        app: &AppHandle,
        agent_executor: &Arc<AgentExecutor>,
        session_id: &str,
        run_id: &str,
        user_message: &str,
        execution_context: &ExecutionContext,
        cancel_flag: Arc<AtomicBool>,
        tool_confirm_responder: ToolConfirmResponder,
    ) -> Result<Option<String>, String> {
        let Some((command_name, raw_args)) = parse_user_skill_command(user_message) else {
            return Ok(None);
        };
        let Some(spec) = execution_context
            .skill_command_specs()
            .iter()
            .find(|spec| spec.name.eq_ignore_ascii_case(&command_name) && spec.dispatch.is_some())
        else {
            return Ok(None);
        };

        let tool_ctx = build_tool_context(
            Some(session_id),
            execution_context
                .executor_work_dir
                .as_ref()
                .map(PathBuf::from),
            execution_context.allowed_tools(),
        )
        .map_err(|err| err.to_string())?;
        let dispatch_context = crate::agent::runtime::tool_dispatch::ToolDispatchContext {
            registry: agent_executor.registry(),
            app_handle: Some(app),
            session_id: Some(session_id),
            persisted_run_id: Some(run_id),
            allowed_tools: execution_context.allowed_tools(),
            permission_mode: execution_context.permission_mode,
            tool_ctx: &tool_ctx,
            tool_confirm_tx: Some(&tool_confirm_responder),
            cancel_flag: Some(cancel_flag),
            route_run_id: run_id,
            route_node_timeout_secs: execution_context.node_timeout_seconds,
            route_retry_count: 0,
            iteration: 1,
            run_budget_policy: RunBudgetPolicy::for_scope(RunBudgetScope::Skill),
        };

        crate::agent::runtime::tool_dispatch::dispatch_skill_command(
            &dispatch_context,
            spec,
            &raw_args,
        )
        .await
        .map(Some)
        .map_err(|err| err.to_string())
    }

    pub(crate) async fn run_send_message(
        app: &AppHandle,
        agent_executor: &Arc<AgentExecutor>,
        db: &sqlx::SqlitePool,
        journal: &SessionJournalStore,
        session_id: &str,
        user_message_id: &str,
        user_message: &str,
        user_message_parts: &[Value],
        max_iterations_override: Option<usize>,
        cancel_flag: Arc<AtomicBool>,
        tool_confirm_responder: ToolConfirmResponder,
    ) -> Result<(), String> {
        let run_id = Uuid::new_v4().to_string();
        let session_engine_result = SessionEngine::run_local_turn(
            app,
            agent_executor,
            db,
            journal,
            session_id,
            &run_id,
            user_message_id,
            user_message,
            user_message_parts,
            max_iterations_override,
            cancel_flag.clone(),
            tool_confirm_responder.clone(),
        )
        .await;

        match Self::classify_session_engine_result(session_engine_result) {
            SessionTurnCompletion::DirectDispatch(output) => {
                OutcomeCommitter::commit_terminal_outcome(
                    app,
                    db,
                    journal,
                    session_id,
                    &run_id,
                    TerminalOutcome::DirectDispatch(output),
                )
                .await
            }
            SessionTurnCompletion::RouteExecution {
                route_execution,
                reconstructed_history_len,
            } => {
                OutcomeCommitter::commit_terminal_outcome(
                    app,
                    db,
                    journal,
                    session_id,
                    &run_id,
                    TerminalOutcome::RouteExecution {
                        route_execution,
                        reconstructed_history_len,
                    },
                )
                .await
            }
            SessionTurnCompletion::SkillCommandFailed(error) => {
                OutcomeCommitter::commit_terminal_outcome(
                    app,
                    db,
                    journal,
                    session_id,
                    &run_id,
                    TerminalOutcome::SkillCommandFailed(error),
                )
                .await
            }
            SessionTurnCompletion::SkillCommandStopped { stop_reason, error } => {
                OutcomeCommitter::commit_terminal_outcome(
                    app,
                    db,
                    journal,
                    session_id,
                    &run_id,
                    TerminalOutcome::SkillCommandStopped { stop_reason, error },
                )
                .await
            }
            SessionTurnCompletion::GenericError(error) => {
                return Err(error);
            }
        }
    }

    fn classify_session_engine_result(
        result: Result<ExecutionOutcome, SessionEngineError>,
    ) -> SessionTurnCompletion {
        match result {
            Ok(ExecutionOutcome::DirectDispatch(output)) => {
                SessionTurnCompletion::DirectDispatch(output)
            }
            Ok(ExecutionOutcome::RouteExecution {
                route_execution,
                reconstructed_history_len,
            }) => SessionTurnCompletion::RouteExecution {
                route_execution,
                reconstructed_history_len,
            },
            Ok(ExecutionOutcome::SkillCommandFailed(error)) => {
                SessionTurnCompletion::SkillCommandFailed(error)
            }
            Ok(ExecutionOutcome::SkillCommandStopped { stop_reason, error }) => {
                SessionTurnCompletion::SkillCommandStopped { stop_reason, error }
            }
            Err(SessionEngineError::Generic(error)) => SessionTurnCompletion::GenericError(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SessionRuntime, SessionTurnCompletion};
    use crate::agent::run_guard::RunStopReason;
    use crate::agent::runtime::kernel::execution_plan::{ExecutionOutcome, SessionEngineError};

    #[test]
    fn classify_session_engine_result_keeps_generic_errors_out_of_terminal_handling() {
        let classification = SessionRuntime::classify_session_engine_result(Err(
            SessionEngineError::Generic("db failed".to_string()),
        ));

        assert!(matches!(
            classification,
            SessionTurnCompletion::GenericError(error) if error == "db failed"
        ));
    }

    #[test]
    fn classify_session_engine_result_keeps_explicit_skill_command_terminals() {
        let failure = SessionRuntime::classify_session_engine_result(Ok(
            ExecutionOutcome::SkillCommandFailed("dispatch failed".to_string()),
        ));
        assert!(matches!(
            failure,
            SessionTurnCompletion::SkillCommandFailed(error) if error == "dispatch failed"
        ));

        let stop_reason = RunStopReason::max_turns(12);
        let stopped = SessionRuntime::classify_session_engine_result(Ok(
            ExecutionOutcome::SkillCommandStopped {
                stop_reason: stop_reason.clone(),
                error: "max turns".to_string(),
            },
        ));
        assert!(matches!(
            stopped,
            SessionTurnCompletion::SkillCommandStopped {
                stop_reason: reason,
                error
            } if reason == stop_reason && error == "max turns"
        ));
    }
}
