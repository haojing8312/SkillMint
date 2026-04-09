use crate::agent::runtime::events::ToolConfirmResponder;
use crate::agent::runtime::kernel::execution_plan::{
    ExecutionContext, ExecutionOutcome, SessionEngineError, TurnContext,
};
use crate::agent::runtime::kernel::session_engine::SessionEngine;
use crate::agent::runtime::kernel::session_profile::SessionSurfaceKind;
use crate::agent::runtime::kernel::turn_preparation::{
    prepare_employee_step_turn, prepare_hidden_child_turn,
};
use crate::agent::runtime::task_state::TaskBackendKind;
use crate::agent::types::StreamDelta;
use crate::agent::AgentExecutor;
use serde_json::Value;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::AppHandle;

pub(crate) type TaskBackendTokenCallback = Arc<dyn Fn(StreamDelta) + Send + Sync + 'static>;

pub(crate) struct InteractiveChatTaskBackendPreparationRequest<'a> {
    pub app: &'a AppHandle,
    pub agent_executor: &'a Arc<AgentExecutor>,
    pub db: &'a sqlx::SqlitePool,
    pub session_id: &'a str,
    pub user_message: &'a str,
    pub user_message_parts: &'a [Value],
    pub max_iterations_override: Option<usize>,
}

pub(crate) struct InteractiveChatTaskBackendRequest<'a> {
    pub app: AppHandle,
    pub agent_executor: Arc<AgentExecutor>,
    pub db: &'a sqlx::SqlitePool,
    pub journal: &'a crate::session_journal::SessionJournalStore,
    pub session_id: &'a str,
    pub run_id: &'a str,
    pub user_message_id: &'a str,
    pub user_message: &'a str,
    pub turn_context: &'a TurnContext,
    pub execution_context: &'a ExecutionContext,
    pub cancel_flag: Arc<AtomicBool>,
    pub tool_confirm_responder: ToolConfirmResponder,
}

pub(crate) struct PreparedSurfaceTaskBackendRequest<'a> {
    pub app_handle: Option<AppHandle>,
    pub agent_executor: Arc<AgentExecutor>,
    pub session_id: &'a str,
    pub turn_context: &'a TurnContext,
    pub execution_context: &'a ExecutionContext,
    pub on_token: TaskBackendTokenCallback,
}

pub(crate) struct HiddenChildTaskBackendPreparationRequest<'a> {
    pub agent_executor: &'a Arc<AgentExecutor>,
    pub prompt: &'a str,
    pub agent_type: &'a str,
    pub delegate_display_name: &'a str,
    pub api_format: &'a str,
    pub base_url: &'a str,
    pub api_key: &'a str,
    pub model: &'a str,
    pub allowed_tools: Option<Vec<String>>,
    pub max_iterations: usize,
    pub work_dir: Option<String>,
}

pub(crate) struct EmployeeStepTaskBackendPreparationRequest<'a> {
    pub agent_executor: &'a Arc<AgentExecutor>,
    pub user_prompt: &'a str,
    pub employee_step_system_prompt: &'a str,
    pub api_format: &'a str,
    pub base_url: &'a str,
    pub api_key: &'a str,
    pub model: &'a str,
    pub allowed_tools: Option<Vec<String>>,
    pub max_iterations: usize,
    pub work_dir: Option<String>,
}

pub(crate) enum TaskBackendPreparationRequest<'a> {
    InteractiveChat(InteractiveChatTaskBackendPreparationRequest<'a>),
    HiddenChild(HiddenChildTaskBackendPreparationRequest<'a>),
    EmployeeStep(EmployeeStepTaskBackendPreparationRequest<'a>),
}

pub(crate) struct PreparedTaskBackendSurface {
    pub contract: TaskBackendContract,
    pub turn_context: TurnContext,
    pub execution_context: ExecutionContext,
}

pub(crate) struct InteractiveChatPreparedTaskBackendExecution<'a> {
    pub prepared_surface: &'a PreparedTaskBackendSurface,
    pub app: AppHandle,
    pub agent_executor: Arc<AgentExecutor>,
    pub db: &'a sqlx::SqlitePool,
    pub journal: &'a crate::session_journal::SessionJournalStore,
    pub session_id: &'a str,
    pub run_id: &'a str,
    pub user_message_id: &'a str,
    pub user_message: &'a str,
    pub cancel_flag: Arc<AtomicBool>,
    pub tool_confirm_responder: ToolConfirmResponder,
}

pub(crate) struct DelegatedPreparedTaskBackendExecution<'a> {
    pub prepared_surface: &'a PreparedTaskBackendSurface,
    pub app_handle: Option<AppHandle>,
    pub agent_executor: Arc<AgentExecutor>,
    pub session_id: &'a str,
    pub on_token: TaskBackendTokenCallback,
}

pub(crate) enum PreparedTaskBackendExecutionRequest<'a> {
    InteractiveChat(InteractiveChatPreparedTaskBackendExecution<'a>),
    Delegated(DelegatedPreparedTaskBackendExecution<'a>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TaskBackendContract {
    InteractiveChat,
    HiddenChild,
    EmployeeStep,
}

impl TaskBackendContract {
    pub(crate) fn backend_kind(self) -> TaskBackendKind {
        match self {
            TaskBackendContract::InteractiveChat => TaskBackendKind::InteractiveChatBackend,
            TaskBackendContract::HiddenChild => TaskBackendKind::HiddenChildBackend,
            TaskBackendContract::EmployeeStep => TaskBackendKind::EmployeeStepBackend,
        }
    }

    pub(crate) fn session_surface(self) -> SessionSurfaceKind {
        match self {
            TaskBackendContract::InteractiveChat => SessionSurfaceKind::LocalChat,
            TaskBackendContract::HiddenChild => SessionSurfaceKind::HiddenChildSession,
            TaskBackendContract::EmployeeStep => SessionSurfaceKind::EmployeeStepSession,
        }
    }
}

pub(crate) enum TaskBackendRunRequest<'a> {
    InteractiveChat(InteractiveChatTaskBackendRequest<'a>),
    HiddenChild(PreparedSurfaceTaskBackendRequest<'a>),
    EmployeeStep(PreparedSurfaceTaskBackendRequest<'a>),
}

impl TaskBackendRunRequest<'_> {
    pub(crate) fn contract(&self) -> TaskBackendContract {
        match self {
            TaskBackendRunRequest::InteractiveChat(_) => TaskBackendContract::InteractiveChat,
            TaskBackendRunRequest::HiddenChild(_) => TaskBackendContract::HiddenChild,
            TaskBackendRunRequest::EmployeeStep(_) => TaskBackendContract::EmployeeStep,
        }
    }

    pub(crate) fn backend_kind(&self) -> TaskBackendKind {
        self.contract().backend_kind()
    }

    pub(crate) fn session_surface(&self) -> SessionSurfaceKind {
        self.contract().session_surface()
    }
}

impl TaskBackendPreparationRequest<'_> {
    pub(crate) fn contract(&self) -> TaskBackendContract {
        match self {
            TaskBackendPreparationRequest::InteractiveChat(_) => {
                TaskBackendContract::InteractiveChat
            }
            TaskBackendPreparationRequest::HiddenChild(_) => TaskBackendContract::HiddenChild,
            TaskBackendPreparationRequest::EmployeeStep(_) => TaskBackendContract::EmployeeStep,
        }
    }
}

pub(crate) async fn prepare_task_backend(
    request: TaskBackendPreparationRequest<'_>,
) -> Result<PreparedTaskBackendSurface, String> {
    let contract = request.contract();
    let (turn_context, execution_context) = match request {
        TaskBackendPreparationRequest::InteractiveChat(request) => {
            SessionEngine::prepare_local_turn_context(
                request.app,
                request.agent_executor,
                request.db,
                request.session_id,
                request.user_message,
                request.user_message_parts,
                request.max_iterations_override,
            )
            .await
            .map_err(|error| match error {
                SessionEngineError::Generic(message) => message,
            })?
        }
        TaskBackendPreparationRequest::HiddenChild(request) => prepare_hidden_child_turn(
            request.agent_executor,
            request.prompt,
            request.agent_type,
            request.delegate_display_name,
            request.api_format,
            request.base_url,
            request.api_key,
            request.model,
            request.allowed_tools,
            request.max_iterations,
            request.work_dir,
        ),
        TaskBackendPreparationRequest::EmployeeStep(request) => prepare_employee_step_turn(
            request.agent_executor,
            request.user_prompt,
            request.employee_step_system_prompt,
            request.api_format,
            request.base_url,
            request.api_key,
            request.model,
            request.allowed_tools,
            request.max_iterations,
            request.work_dir,
        ),
    };

    Ok(PreparedTaskBackendSurface {
        contract,
        turn_context,
        execution_context,
    })
}

pub(crate) async fn run_task_backend(
    request: TaskBackendRunRequest<'_>,
) -> Result<ExecutionOutcome, SessionEngineError> {
    debug_assert_eq!(request.contract().backend_kind(), request.backend_kind());
    debug_assert_eq!(
        request.contract().session_surface(),
        request.session_surface()
    );

    match request {
        TaskBackendRunRequest::InteractiveChat(request) => {
            SessionEngine::execute_prepared_local_turn(
                &request.app,
                &request.agent_executor,
                request.db,
                request.journal,
                request.session_id,
                request.run_id,
                request.user_message_id,
                request.user_message,
                request.turn_context,
                request.execution_context,
                request.cancel_flag,
                request.tool_confirm_responder,
            )
            .await
        }
        TaskBackendRunRequest::HiddenChild(request) => {
            let callback = Arc::clone(&request.on_token);
            SessionEngine::run_hidden_child_turn(
                request.app_handle.as_ref(),
                &request.agent_executor,
                request.session_id,
                request.turn_context,
                request.execution_context,
                move |delta| (callback)(delta),
            )
            .await
        }
        TaskBackendRunRequest::EmployeeStep(request) => {
            let callback = Arc::clone(&request.on_token);
            SessionEngine::run_employee_step_turn(
                request.app_handle.as_ref(),
                &request.agent_executor,
                request.session_id,
                request.turn_context,
                request.execution_context,
                move |delta| (callback)(delta),
            )
            .await
        }
    }
}

pub(crate) async fn execute_prepared_task_backend(
    request: PreparedTaskBackendExecutionRequest<'_>,
) -> Result<ExecutionOutcome, SessionEngineError> {
    match request {
        PreparedTaskBackendExecutionRequest::InteractiveChat(request) => {
            debug_assert_eq!(
                request.prepared_surface.contract,
                TaskBackendContract::InteractiveChat
            );
            run_task_backend(TaskBackendRunRequest::InteractiveChat(
                InteractiveChatTaskBackendRequest {
                    app: request.app,
                    agent_executor: request.agent_executor,
                    db: request.db,
                    journal: request.journal,
                    session_id: request.session_id,
                    run_id: request.run_id,
                    user_message_id: request.user_message_id,
                    user_message: request.user_message,
                    turn_context: &request.prepared_surface.turn_context,
                    execution_context: &request.prepared_surface.execution_context,
                    cancel_flag: request.cancel_flag,
                    tool_confirm_responder: request.tool_confirm_responder,
                },
            ))
            .await
        }
        PreparedTaskBackendExecutionRequest::Delegated(request) => {
            let run_request = match request.prepared_surface.contract {
                TaskBackendContract::HiddenChild => {
                    TaskBackendRunRequest::HiddenChild(PreparedSurfaceTaskBackendRequest {
                        app_handle: request.app_handle,
                        agent_executor: request.agent_executor,
                        session_id: request.session_id,
                        turn_context: &request.prepared_surface.turn_context,
                        execution_context: &request.prepared_surface.execution_context,
                        on_token: request.on_token,
                    })
                }
                TaskBackendContract::EmployeeStep => {
                    TaskBackendRunRequest::EmployeeStep(PreparedSurfaceTaskBackendRequest {
                        app_handle: request.app_handle,
                        agent_executor: request.agent_executor,
                        session_id: request.session_id,
                        turn_context: &request.prepared_surface.turn_context,
                        execution_context: &request.prepared_surface.execution_context,
                        on_token: request.on_token,
                    })
                }
                TaskBackendContract::InteractiveChat => {
                    debug_assert!(
                        false,
                        "interactive chat prepared surfaces should use interactive execution"
                    );
                    return Err(SessionEngineError::Generic(
                        "interactive chat backend requires interactive execution params"
                            .to_string(),
                    ));
                }
            };
            run_task_backend(run_request).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        execute_prepared_task_backend, prepare_task_backend, DelegatedPreparedTaskBackendExecution,
        EmployeeStepTaskBackendPreparationRequest, HiddenChildTaskBackendPreparationRequest,
        PreparedTaskBackendExecutionRequest, TaskBackendContract, TaskBackendPreparationRequest,
        TaskBackendTokenCallback,
    };
    use crate::agent::runtime::kernel::execution_plan::SessionEngineError;
    use crate::agent::runtime::kernel::session_profile::SessionSurfaceKind;
    use crate::agent::runtime::task_state::TaskBackendKind;
    use crate::agent::{AgentExecutor, ToolRegistry};
    use std::sync::Arc;

    #[test]
    fn backend_kind_reports_local_chat_contract() {
        assert_eq!(
            TaskBackendContract::InteractiveChat.backend_kind(),
            TaskBackendKind::InteractiveChatBackend
        );
        assert_eq!(
            TaskBackendContract::InteractiveChat.session_surface(),
            SessionSurfaceKind::LocalChat
        );
    }

    #[test]
    fn backend_kind_reports_delegated_surface_contracts() {
        assert_eq!(
            TaskBackendContract::HiddenChild.backend_kind(),
            TaskBackendKind::HiddenChildBackend
        );
        assert_eq!(
            TaskBackendContract::HiddenChild.session_surface(),
            SessionSurfaceKind::HiddenChildSession
        );
        assert_eq!(
            TaskBackendContract::EmployeeStep.backend_kind(),
            TaskBackendKind::EmployeeStepBackend
        );
        assert_eq!(
            TaskBackendContract::EmployeeStep.session_surface(),
            SessionSurfaceKind::EmployeeStepSession
        );
    }

    #[test]
    fn hidden_child_prepare_projects_hidden_child_surface_contract() {
        let agent_executor = Arc::new(AgentExecutor::new(Arc::new(ToolRegistry::new())));

        let runtime = tokio::runtime::Runtime::new().expect("create tokio runtime");
        let prepared = runtime
            .block_on(prepare_task_backend(
                TaskBackendPreparationRequest::HiddenChild(
                    HiddenChildTaskBackendPreparationRequest {
                        agent_executor: &agent_executor,
                        prompt: "summarize",
                        agent_type: "default",
                        delegate_display_name: "delegate",
                        api_format: "openai",
                        base_url: "http://localhost",
                        api_key: "test-key",
                        model: "gpt-test",
                        allowed_tools: None,
                        max_iterations: 2,
                        work_dir: Some("C:/tmp".to_string()),
                    },
                ),
            ))
            .expect("prepare hidden child backend");

        assert_eq!(prepared.contract, TaskBackendContract::HiddenChild);
        assert_eq!(
            prepared.execution_context.session_profile.surface,
            SessionSurfaceKind::HiddenChildSession
        );
        assert_eq!(prepared.turn_context.user_message, "summarize");
    }

    #[test]
    fn employee_step_prepare_projects_employee_surface_contract() {
        let agent_executor = Arc::new(AgentExecutor::new(Arc::new(ToolRegistry::new())));

        let runtime = tokio::runtime::Runtime::new().expect("create tokio runtime");
        let prepared = runtime
            .block_on(prepare_task_backend(
                TaskBackendPreparationRequest::EmployeeStep(
                    EmployeeStepTaskBackendPreparationRequest {
                        agent_executor: &agent_executor,
                        user_prompt: "review this",
                        employee_step_system_prompt: "act like reviewer",
                        api_format: "openai",
                        base_url: "http://localhost",
                        api_key: "test-key",
                        model: "gpt-test",
                        allowed_tools: None,
                        max_iterations: 2,
                        work_dir: Some("C:/tmp".to_string()),
                    },
                ),
            ))
            .expect("prepare employee backend");

        assert_eq!(prepared.contract, TaskBackendContract::EmployeeStep);
        assert_eq!(
            prepared.execution_context.session_profile.surface,
            SessionSurfaceKind::EmployeeStepSession
        );
        assert_eq!(prepared.turn_context.user_message, "review this");
    }

    #[test]
    fn delegated_execution_rejects_interactive_contract() {
        let prepared_surface = super::PreparedTaskBackendSurface {
            contract: TaskBackendContract::InteractiveChat,
            turn_context: crate::agent::runtime::kernel::execution_plan::TurnContext::default(),
            execution_context:
                crate::agent::runtime::kernel::execution_plan::ExecutionContext::default(),
        };
        let runtime = tokio::runtime::Runtime::new().expect("create tokio runtime");
        let result = runtime.block_on(execute_prepared_task_backend(
            PreparedTaskBackendExecutionRequest::Delegated(DelegatedPreparedTaskBackendExecution {
                prepared_surface: &prepared_surface,
                app_handle: None,
                agent_executor: Arc::new(AgentExecutor::new(Arc::new(ToolRegistry::new()))),
                session_id: "session-1",
                on_token: Arc::new(|_| {}) as TaskBackendTokenCallback,
            }),
        ));

        assert!(
            matches!(result, Err(SessionEngineError::Generic(message)) if message.contains("interactive chat backend requires interactive execution params"))
        );
    }
}
