use crate::agent::runtime::attempt_runner::RouteExecutionOutcome;
use crate::agent::runtime::kernel::capability_snapshot::CapabilitySnapshot;
use crate::agent::run_guard::RunStopReason;
use crate::agent::runtime::skill_routing::runner::RouteRunPlan;

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
    pub route_plan: Option<RouteRunPlan>,
}

impl ExecutionPlan {
    pub(crate) fn from_route_plan(route_plan: RouteRunPlan) -> Self {
        let lane = Self::lane_for_route_plan(&route_plan);
        Self {
            lane,
            route_plan: Some(route_plan),
        }
    }

    pub(crate) fn lane_for_route_plan(route_plan: &RouteRunPlan) -> ExecutionLane {
        match route_plan {
            RouteRunPlan::OpenTask { .. } => ExecutionLane::OpenTask,
            RouteRunPlan::PromptSkillInline { .. } => ExecutionLane::PromptInline,
            RouteRunPlan::PromptSkillFork { .. } => ExecutionLane::PromptFork,
            RouteRunPlan::DirectDispatchSkill { .. } => ExecutionLane::DirectDispatch,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ExecutionContext {
    pub capability_snapshot: Option<CapabilitySnapshot>,
}

#[derive(Debug, Clone)]
pub(crate) enum ExecutionOutcome {
    DirectDispatch(String),
    RouteExecution {
        route_execution: RouteExecutionOutcome,
        reconstructed_history_len: usize,
    },
    SkillCommandFailed(String),
    SkillCommandStopped {
        stop_reason: RunStopReason,
        error: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SessionEngineError {
    Generic(String),
}

#[cfg(test)]
mod tests {
    use super::{ExecutionLane, ExecutionPlan};
    use crate::agent::runtime::skill_routing::intent::RouteFallbackReason;
    use crate::agent::runtime::skill_routing::runner::RouteRunPlan;

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

    #[test]
    fn execution_plan_captures_lane_and_route_plan() {
        let route_plan = RouteRunPlan::OpenTask {
            fallback_reason: Some(RouteFallbackReason::NoCandidates),
        };

        let execution_plan = ExecutionPlan::from_route_plan(route_plan.clone());

        assert_eq!(execution_plan.lane, ExecutionLane::OpenTask);
        assert!(matches!(
            execution_plan.route_plan,
            Some(RouteRunPlan::OpenTask {
                fallback_reason: Some(RouteFallbackReason::NoCandidates)
            })
        ));
    }
}
