use crate::agent::runtime::attempt_runner::RouteExecutionOutcome;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExecutionLane {
    OpenTask,
    PromptInline,
    PromptFork,
    DirectDispatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExecutionPlan {
    pub lane: ExecutionLane,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ExecutionContext;

#[derive(Debug, Clone)]
pub(crate) enum ExecutionOutcome {
    DirectDispatch(String),
    RouteExecution {
        route_execution: RouteExecutionOutcome,
        reconstructed_history_len: usize,
    },
}

#[cfg(test)]
mod tests {
    use crate::agent::runtime::kernel::execution_plan::ExecutionLane;

    #[test]
    fn execution_plan_supports_all_desktop_runtime_lanes() {
        let lanes = [
            ExecutionLane::OpenTask,
            ExecutionLane::PromptInline,
            ExecutionLane::PromptFork,
            ExecutionLane::DirectDispatch,
        ];

        assert_eq!(lanes.len(), 4);
    }
}
