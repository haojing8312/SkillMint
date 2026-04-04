use crate::agent::runtime::runtime_io::{
    WorkspaceSkillRouteExecutionMode, WorkspaceSkillRouteProjection,
};
use crate::agent::runtime::skill_routing::intent::{
    RouteConfidence, RouteDecision, RouteFallbackReason,
};
use crate::agent::runtime::skill_routing::recall::SkillRecallCandidate;

const ROUTE_SCORE_FLOOR: u32 = 60;
const ROUTE_SCORE_GAP: u32 = 20;
const ROUTE_SCORE_RATIO_NUMERATOR: u32 = 3;
const ROUTE_SCORE_RATIO_DENOMINATOR: u32 = 2;

pub fn adjudicate_route(candidates: &[SkillRecallCandidate]) -> RouteDecision {
    if candidates.is_empty() {
        return RouteDecision::OpenTask {
            confidence: RouteConfidence::new(0.0).expect("zero confidence is valid"),
            fallback_reason: Some(RouteFallbackReason::NoCandidates),
        };
    }

    let top = &candidates[0];
    let runner_up_score = runner_up_score(candidates);
    if !is_clear_winner(top.score, runner_up_score) {
        return RouteDecision::OpenTask {
            confidence: ambiguous_confidence(top.score),
            fallback_reason: Some(RouteFallbackReason::AmbiguousCandidates),
        };
    }

    route_to_skill(&top.projection, routed_confidence(top.score, runner_up_score))
}

fn route_to_skill(
    projection: &WorkspaceSkillRouteProjection,
    confidence: RouteConfidence,
) -> RouteDecision {
    match projection.execution_mode {
        WorkspaceSkillRouteExecutionMode::DirectDispatch => RouteDecision::DirectDispatchSkill {
            skill_id: projection.skill_id.clone(),
            confidence,
        },
        WorkspaceSkillRouteExecutionMode::Fork => RouteDecision::PromptSkillFork {
            skill_id: projection.skill_id.clone(),
            confidence,
        },
        WorkspaceSkillRouteExecutionMode::Inline => RouteDecision::PromptSkillInline {
            skill_id: projection.skill_id.clone(),
            confidence,
        },
    }
}

fn runner_up_score(candidates: &[SkillRecallCandidate]) -> u32 {
    candidates.get(1).map(|candidate| candidate.score).unwrap_or(0)
}

fn is_clear_winner(top_score: u32, runner_up_score: u32) -> bool {
    if top_score < ROUTE_SCORE_FLOOR {
        return false;
    }

    if runner_up_score == 0 {
        return true;
    }

    let score_gap = top_score.saturating_sub(runner_up_score);
    score_gap >= ROUTE_SCORE_GAP
        && top_score.saturating_mul(ROUTE_SCORE_RATIO_DENOMINATOR)
            >= runner_up_score.saturating_mul(ROUTE_SCORE_RATIO_NUMERATOR)
}

fn ambiguous_confidence(top_score: u32) -> RouteConfidence {
    let raw = if top_score >= ROUTE_SCORE_FLOOR { 0.55 } else { 0.45 };
    RouteConfidence::new(raw).expect("ambiguous confidence is normalized")
}

fn routed_confidence(top_score: u32, runner_up_score: u32) -> RouteConfidence {
    let raw = if runner_up_score == 0 {
        if top_score >= 120 {
            0.95
        } else if top_score >= 90 {
            0.85
        } else {
            0.70
        }
    } else if top_score >= 120 || top_score.saturating_sub(runner_up_score) >= 40 {
        0.95
    } else if top_score >= 90 || top_score.saturating_sub(runner_up_score) >= 30 {
        0.85
    } else {
        0.70
    };

    RouteConfidence::new(raw).expect("route confidence is normalized")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::runtime::skill_routing::recall::SkillRecallCandidate;
    use runtime_skill_core::{
        SkillCommandArgMode, SkillCommandDispatchKind, SkillCommandDispatchSpec,
        SkillInvocationPolicy,
    };

    fn build_projection(
        skill_id: &str,
        execution_mode: WorkspaceSkillRouteExecutionMode,
        command_dispatch: Option<SkillCommandDispatchSpec>,
    ) -> WorkspaceSkillRouteProjection {
        WorkspaceSkillRouteProjection {
            skill_id: skill_id.to_string(),
            display_name: skill_id.to_string(),
            aliases: vec![skill_id.to_string()],
            description: skill_id.to_string(),
            when_to_use: skill_id.to_string(),
            family_key: Some("feishu-pm".to_string()),
            domain_tags: vec!["项管".to_string(), "日报".to_string()],
            allowed_tools: vec!["read_file".to_string()],
            max_iterations: Some(3),
            invocation: SkillInvocationPolicy {
                user_invocable: true,
                disable_model_invocation: command_dispatch.is_some(),
            },
            execution_mode,
            command_dispatch,
        }
    }

    fn build_candidate(
        skill_id: &str,
        score: u32,
        execution_mode: WorkspaceSkillRouteExecutionMode,
        command_dispatch: Option<SkillCommandDispatchSpec>,
    ) -> SkillRecallCandidate {
        SkillRecallCandidate {
            projection: build_projection(skill_id, execution_mode, command_dispatch),
            score,
        }
    }

    #[test]
    fn clear_single_winner_routes_to_skill() {
        let decision = adjudicate_route(&[build_candidate(
            "feishu-pm-daily-sync",
            60,
            WorkspaceSkillRouteExecutionMode::Inline,
            None,
        )]);

        assert_eq!(decision.skill_id(), Some("feishu-pm-daily-sync"));
        assert_eq!(decision.confidence().score(), 0.70);
        assert_eq!(
            decision.intent(),
            crate::agent::runtime::skill_routing::intent::InvocationIntent::PromptSkillInline {
                skill_id: "feishu-pm-daily-sync".to_string(),
            }
        );
    }

    #[test]
    fn weak_single_candidate_below_floor_falls_back_to_open_task() {
        let decision = adjudicate_route(&[build_candidate(
            "feishu-pm-daily-sync",
            59,
            WorkspaceSkillRouteExecutionMode::Inline,
            None,
        )]);

        assert_eq!(decision.skill_id(), None);
        assert_eq!(
            decision.fallback_reason(),
            Some(RouteFallbackReason::AmbiguousCandidates)
        );
        assert_eq!(decision.confidence().score(), 0.45);
    }

    #[test]
    fn close_tie_falls_back_to_open_task_with_ambiguous_candidates() {
        let decision = adjudicate_route(&[
            build_candidate(
                "feishu-pm-daily-sync",
                80,
                WorkspaceSkillRouteExecutionMode::Inline,
                None,
            ),
            build_candidate(
                "feishu-pm-weekly-work-summary",
                70,
                WorkspaceSkillRouteExecutionMode::Inline,
                None,
            ),
        ]);

        assert_eq!(decision.skill_id(), None);
        assert_eq!(
            decision.fallback_reason(),
            Some(RouteFallbackReason::AmbiguousCandidates)
        );
        assert_eq!(
            decision.intent(),
            crate::agent::runtime::skill_routing::intent::InvocationIntent::OpenTask {
                fallback_reason: Some(RouteFallbackReason::AmbiguousCandidates),
            }
        );
        assert_eq!(decision.confidence().score(), 0.55);
    }

    #[test]
    fn no_candidate_falls_back_to_open_task_with_no_candidates() {
        let decision = adjudicate_route(&[]);

        assert_eq!(decision.skill_id(), None);
        assert_eq!(
            decision.fallback_reason(),
            Some(RouteFallbackReason::NoCandidates)
        );
        assert_eq!(
            decision.intent(),
            crate::agent::runtime::skill_routing::intent::InvocationIntent::OpenTask {
                fallback_reason: Some(RouteFallbackReason::NoCandidates),
            }
        );
    }

    #[test]
    fn execution_mode_selects_prompt_or_dispatch_lane() {
        let inline = adjudicate_route(&[build_candidate(
            "feishu-pm-inline",
            95,
            WorkspaceSkillRouteExecutionMode::Inline,
            None,
        )]);
        let fork = adjudicate_route(&[build_candidate(
            "feishu-pm-fork",
            95,
            WorkspaceSkillRouteExecutionMode::Fork,
            None,
        )]);
        let dispatch = adjudicate_route(&[build_candidate(
            "feishu-pm-dispatch",
            120,
            WorkspaceSkillRouteExecutionMode::DirectDispatch,
            Some(SkillCommandDispatchSpec {
                kind: SkillCommandDispatchKind::Tool,
                tool_name: "exec".to_string(),
                arg_mode: SkillCommandArgMode::Raw,
            }),
        )]);

        assert_eq!(inline.skill_id(), Some("feishu-pm-inline"));
        assert_eq!(
            inline.intent(),
            crate::agent::runtime::skill_routing::intent::InvocationIntent::PromptSkillInline {
                skill_id: "feishu-pm-inline".to_string(),
            }
        );
        assert_eq!(inline.confidence().score(), 0.85);

        assert_eq!(fork.skill_id(), Some("feishu-pm-fork"));
        assert_eq!(
            fork.intent(),
            crate::agent::runtime::skill_routing::intent::InvocationIntent::PromptSkillFork {
                skill_id: "feishu-pm-fork".to_string(),
            }
        );
        assert_eq!(fork.confidence().score(), 0.85);

        assert_eq!(dispatch.skill_id(), Some("feishu-pm-dispatch"));
        assert_eq!(
            dispatch.intent(),
            crate::agent::runtime::skill_routing::intent::InvocationIntent::DirectDispatchSkill {
                skill_id: "feishu-pm-dispatch".to_string(),
            }
        );
        assert_eq!(dispatch.confidence().score(), 0.95);
    }
}
