use crate::agent::runtime::kernel::execution_plan::ExecutionOutcome;
use crate::agent::runtime::runtime_io::{
    append_run_failed_with_pool, append_run_started_with_pool,
};
use crate::agent::runtime::task_backend::{
    attach_active_task_state_to_execution_context, execute_prepared_task_backend_with_context,
    prepare_task_backend, PreparedTaskBackendSurface, TaskBackendExecutionContext,
    TaskBackendPreparationRequest,
};
use crate::agent::runtime::task_execution::TaskExecutionOutcome;
use crate::agent::runtime::task_lifecycle::{self, TaskBeginParentContext};
use crate::agent::runtime::task_record::TaskRecord;
use crate::agent::runtime::task_state::TaskState;
use crate::session_journal::SessionJournalStore;

#[derive(Clone, Copy)]
pub(crate) struct StartedRunContext<'a> {
    pub run_id: &'a str,
    pub user_message_id: &'a str,
}

#[derive(Clone, Copy)]
struct TaskBackendFailureContext<'a> {
    db: &'a sqlx::SqlitePool,
    journal: &'a SessionJournalStore,
    session_id: &'a str,
    run_id: &'a str,
    run_failure_kind: &'a str,
    active_task_record: &'a TaskRecord,
}

pub(crate) struct ActiveTaskBackendRunRequest<'a, F>
where
    F: FnOnce(&mut PreparedTaskBackendSurface),
{
    pub db: &'a sqlx::SqlitePool,
    pub journal: &'a SessionJournalStore,
    pub task_state: TaskState,
    pub parent_context: Option<TaskBeginParentContext<'a>>,
    pub started_run: StartedRunContext<'a>,
    pub preparation_request: TaskBackendPreparationRequest<'a>,
    pub prepare_surface: F,
    pub execution_context: TaskBackendExecutionContext<'a>,
}

async fn handle_task_backend_failure(context: TaskBackendFailureContext<'_>, error: &str) {
    append_run_failed_with_pool(
        context.db,
        context.journal,
        context.session_id,
        context.run_id,
        context.run_failure_kind,
        error,
        None,
    )
    .await;
    let _ = task_lifecycle::mark_task_failed(
        context.db,
        context.journal,
        context.session_id,
        context.active_task_record,
        error,
    )
    .await;
}

async fn prepare_task_backend_for_active_task<'a>(
    request: TaskBackendPreparationRequest<'a>,
    failure_context: TaskBackendFailureContext<'_>,
) -> Result<PreparedTaskBackendSurface, String> {
    match prepare_task_backend(request).await {
        Ok(prepared_surface) => Ok(prepared_surface),
        Err(error) => {
            handle_task_backend_failure(failure_context, &error).await;
            Err(error)
        }
    }
}

async fn execute_prepared_task_backend_for_active_task<'a>(
    prepared_surface: &'a PreparedTaskBackendSurface,
    execution_context: TaskBackendExecutionContext<'a>,
    failure_context: TaskBackendFailureContext<'_>,
) -> Result<ExecutionOutcome, String> {
    match execute_prepared_task_backend_with_context(prepared_surface, execution_context).await {
        Ok(outcome) => Ok(outcome),
        Err(error) => {
            handle_task_backend_failure(failure_context, &error).await;
            Err(error)
        }
    }
}

async fn prepare_and_execute_backend_for_active_task<'a, F>(
    task_state: &TaskState,
    preparation_request: TaskBackendPreparationRequest<'a>,
    prepare_surface: F,
    execution_context: TaskBackendExecutionContext<'a>,
    failure_context: TaskBackendFailureContext<'_>,
) -> Result<ExecutionOutcome, String>
where
    F: FnOnce(&mut PreparedTaskBackendSurface),
{
    let mut prepared_surface =
        prepare_task_backend_for_active_task(preparation_request, failure_context).await?;
    attach_active_task_state_to_execution_context(
        &mut prepared_surface.execution_context,
        task_state,
    );
    prepare_surface(&mut prepared_surface);
    execute_prepared_task_backend_for_active_task(
        &prepared_surface,
        execution_context,
        failure_context,
    )
    .await
}

pub(crate) async fn run_task_backend_with_task_state<'a, F>(
    request: ActiveTaskBackendRunRequest<'a, F>,
) -> Result<TaskExecutionOutcome, String>
where
    F: FnOnce(&mut PreparedTaskBackendSurface),
{
    let ActiveTaskBackendRunRequest {
        db,
        journal,
        task_state,
        parent_context,
        started_run,
        preparation_request,
        prepare_surface,
        execution_context,
    } = request;
    let session_id = task_state.session_id.clone();
    let generic_failure_kind = preparation_request.generic_error_kind();
    let active_task_record =
        task_lifecycle::begin_task_run(db, journal, &task_state, parent_context).await?;

    if let Err(error) = append_run_started_with_pool(
        db,
        journal,
        &session_id,
        started_run.run_id,
        started_run.user_message_id,
    )
    .await
    {
        let _ =
            task_lifecycle::mark_task_failed(db, journal, &session_id, &active_task_record, &error)
                .await;
        return Err(error);
    }

    let execution_outcome = prepare_and_execute_backend_for_active_task(
        &task_state,
        preparation_request,
        prepare_surface,
        execution_context,
        TaskBackendFailureContext {
            db,
            journal,
            session_id: &session_id,
            run_id: started_run.run_id,
            run_failure_kind: generic_failure_kind,
            active_task_record: &active_task_record,
        },
    )
    .await?;

    Ok(TaskExecutionOutcome::new(
        task_state,
        active_task_record,
        execution_outcome,
    ))
}
