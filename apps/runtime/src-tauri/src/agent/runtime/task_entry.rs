use crate::agent::run_guard::RunStopReason;
use crate::agent::runtime::events::ToolConfirmResponder;
use crate::agent::runtime::task_active_run::{
    run_task_backend_with_task_state, ActiveTaskBackendRunRequest, StartedRunContext,
};
use crate::agent::runtime::task_backend::{
    InteractiveChatTaskBackendPreparationRequest, PreparedTaskBackendSurface,
    TaskBackendExecutionContext, TaskBackendPreparationRequest, TaskBackendTokenCallback,
};
use crate::agent::runtime::task_execution::TaskExecutionOutcome;
use crate::agent::runtime::task_lifecycle::TaskBeginParentContext;
use crate::agent::runtime::task_state::TaskState;
use crate::agent::runtime::task_terminal::{
    finalize_delegated_task_execution_outcome, finalize_primary_task_execution_outcome,
    DelegatedTaskTerminalFinalizeRequest, DelegatedTaskTerminalOutcome,
};
use crate::agent::AgentExecutor;
use crate::session_journal::SessionJournalStore;
use serde_json::Value;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::AppHandle;

pub(crate) struct DelegatedTaskBackendRunAndFinalizeRequest<'a, F>
where
    F: FnOnce(&mut crate::agent::runtime::task_backend::PreparedTaskBackendSurface),
{
    pub db: &'a sqlx::SqlitePool,
    pub journal: &'a SessionJournalStore,
    pub task_state: TaskState,
    pub parent_context: Option<TaskBeginParentContext<'a>>,
    pub preparation_request: TaskBackendPreparationRequest<'a>,
    pub app_handle: Option<AppHandle>,
    pub agent_executor: Arc<AgentExecutor>,
    pub on_token: TaskBackendTokenCallback,
    pub prepare_surface: F,
}

pub(crate) struct DelegatedTaskTerminalFinalizeEntryRequest<'a> {
    pub db: &'a sqlx::SqlitePool,
    pub journal: &'a SessionJournalStore,
    pub task_execution_outcome: TaskExecutionOutcome,
}

#[derive(Debug, Clone)]
pub(crate) enum DelegatedTaskEntryOutcome {
    Completed {
        output: String,
    },
    Stopped {
        stop_reason: RunStopReason,
        error: String,
    },
    Failed {
        error: String,
    },
}

pub(crate) struct PrimaryLocalChatTaskRunAndFinalizeRequest<'a> {
    pub app: &'a AppHandle,
    pub agent_executor: &'a Arc<AgentExecutor>,
    pub db: &'a sqlx::SqlitePool,
    pub journal: &'a SessionJournalStore,
    pub session_id: &'a str,
    pub run_id: &'a str,
    pub user_message_id: &'a str,
    pub user_message: &'a str,
    pub user_message_parts: &'a [Value],
    pub max_iterations_override: Option<usize>,
    pub cancel_flag: Arc<AtomicBool>,
    pub tool_confirm_responder: ToolConfirmResponder,
}

pub(crate) async fn run_and_finalize_primary_local_chat_task(
    request: PrimaryLocalChatTaskRunAndFinalizeRequest<'_>,
) -> Result<(), String> {
    let PrimaryLocalChatTaskRunAndFinalizeRequest {
        app,
        agent_executor,
        db,
        journal,
        session_id,
        run_id,
        user_message_id,
        user_message,
        user_message_parts,
        max_iterations_override,
        cancel_flag,
        tool_confirm_responder,
    } = request;

    let task_state = TaskState::new_primary_local_chat(session_id, user_message_id, run_id);
    let task_execution_outcome = run_task_backend_with_task_state(ActiveTaskBackendRunRequest {
        db,
        journal,
        task_state,
        parent_context: None,
        started_run: StartedRunContext {
            run_id,
            user_message_id,
        },
        preparation_request: TaskBackendPreparationRequest::InteractiveChat(
            InteractiveChatTaskBackendPreparationRequest {
                app,
                agent_executor,
                db,
                session_id,
                user_message,
                user_message_parts,
                max_iterations_override,
            },
        ),
        prepare_surface: |_| {},
        execution_context: TaskBackendExecutionContext::InteractiveChat {
            app: app.clone(),
            agent_executor: Arc::clone(agent_executor),
            db,
            journal,
            session_id,
            run_id,
            user_message,
            cancel_flag,
            tool_confirm_responder,
        },
    })
    .await?;

    finalize_primary_task_execution_outcome(
        app,
        db,
        journal,
        session_id,
        run_id,
        task_execution_outcome,
    )
    .await
}

pub(crate) async fn finalize_delegated_task_execution_outcome_entry(
    request: DelegatedTaskTerminalFinalizeEntryRequest<'_>,
) -> Result<DelegatedTaskEntryOutcome, String> {
    finalize_delegated_task_execution_outcome(DelegatedTaskTerminalFinalizeRequest {
        db: request.db,
        journal: request.journal,
        task_execution_outcome: request.task_execution_outcome,
    })
    .await
    .map(map_delegated_terminal_outcome_to_entry_outcome)
}

pub(crate) async fn run_and_finalize_delegated_task_backend<F>(
    request: DelegatedTaskBackendRunAndFinalizeRequest<'_, F>,
) -> Result<DelegatedTaskEntryOutcome, String>
where
    F: FnOnce(&mut PreparedTaskBackendSurface),
{
    let DelegatedTaskBackendRunAndFinalizeRequest {
        db,
        journal,
        task_state,
        parent_context,
        preparation_request,
        app_handle,
        agent_executor,
        on_token,
        prepare_surface,
    } = request;

    let session_id = task_state.session_id.clone();
    let run_id = task_state.run_id.clone();
    let user_message_id = task_state.user_message_id.clone();
    let task_execution_outcome = run_task_backend_with_task_state(ActiveTaskBackendRunRequest {
        db,
        journal,
        task_state,
        parent_context,
        started_run: StartedRunContext {
            run_id: &run_id,
            user_message_id: &user_message_id,
        },
        preparation_request,
        prepare_surface,
        execution_context: TaskBackendExecutionContext::Delegated {
            app_handle,
            agent_executor: Arc::clone(&agent_executor),
            session_id: &session_id,
            on_token,
        },
    })
    .await?;

    finalize_delegated_task_execution_outcome(DelegatedTaskTerminalFinalizeRequest {
        db,
        journal,
        task_execution_outcome,
    })
    .await
    .map(map_delegated_terminal_outcome_to_entry_outcome)
}

fn map_delegated_terminal_outcome_to_entry_outcome(
    outcome: DelegatedTaskTerminalOutcome,
) -> DelegatedTaskEntryOutcome {
    match outcome {
        DelegatedTaskTerminalOutcome::Completed { output } => {
            DelegatedTaskEntryOutcome::Completed { output }
        }
        DelegatedTaskTerminalOutcome::Stopped { stop_reason, error } => {
            DelegatedTaskEntryOutcome::Stopped { stop_reason, error }
        }
        DelegatedTaskTerminalOutcome::Failed { error } => {
            DelegatedTaskEntryOutcome::Failed { error }
        }
    }
}
