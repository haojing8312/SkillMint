use crate::agent::runtime::kernel::execution_plan::ExecutionOutcome;
use crate::agent::runtime::task_record::TaskRecord;
use crate::agent::runtime::task_state::TaskState;

#[derive(Debug, Clone)]
pub(crate) struct TaskExecutionOutcome {
    pub task_state: TaskState,
    pub active_task_record: TaskRecord,
    pub execution_outcome: ExecutionOutcome,
}

impl TaskExecutionOutcome {
    pub(crate) fn new(
        task_state: TaskState,
        active_task_record: TaskRecord,
        execution_outcome: ExecutionOutcome,
    ) -> Self {
        Self {
            task_state,
            active_task_record,
            execution_outcome,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TaskExecutionOutcome;
    use crate::agent::runtime::kernel::execution_plan::ExecutionOutcome;
    use crate::agent::runtime::kernel::turn_state::TurnStateSnapshot;
    use crate::agent::runtime::task_record::TaskRecord;
    use crate::agent::runtime::task_state::TaskState;

    #[test]
    fn task_execution_outcome_keeps_task_state_and_execution_outcome_together() {
        let task_state = TaskState::new_primary_local_chat("session-1", "user-1", "run-1");
        let task_record = TaskRecord::new_pending(
            task_state.task_identity.clone(),
            task_state.task_kind,
            task_state.surface_kind,
            task_state.backend_kind,
            task_state.session_id.clone(),
            task_state.user_message_id.clone(),
            task_state.run_id.clone(),
            "2026-04-10T10:00:00Z",
        );

        let wrapped = TaskExecutionOutcome::new(
            task_state.clone(),
            task_record.clone(),
            ExecutionOutcome::DirectDispatch {
                output: "done".to_string(),
                turn_state: TurnStateSnapshot::default(),
            },
        );

        assert_eq!(wrapped.task_state, task_state);
        assert_eq!(wrapped.active_task_record, task_record);
        assert!(matches!(
            wrapped.execution_outcome,
            ExecutionOutcome::DirectDispatch { .. }
        ));
    }
}
