use crate::agent::runtime::runtime_io::{
    WorkspaceSkillRouteExecutionMode, WorkspaceSkillRouteProjection,
};
use crate::agent::runtime::skill_routing::intent::{
    RouteConfidence, RouteDecision, RouteFallbackReason,
};
use crate::agent::runtime::skill_routing::recall::SkillRecallCandidate;

pub fn adjudicate_route(candidates: &[SkillRecallCandidate]) -> RouteDecision {
    if candidates.is_empty() {
        return RouteDecision::OpenTask {
            confidence: RouteConfidence::new(0.0).expect("zero confidence is valid"),
            fallback_reason: Some(RouteFallbackReason::NoCandidates),
        };
    }

    let top = &candidates[0];
    if candidates.len() > 1 {
        let runner_up = &candidates[1];
        if !is_clear_winner(top.score, runner_up.score) {
            return RouteDecision::OpenTask {
                confidence: score_confidence(top.score, runner_up.score),
                fallback_reason: Some(RouteFallbackReason::AmbiguousCandidates),
            };
        }
    }

    route_to_skill(&top.projection, score_confidence(top.score, runner_up_score(candidates)))
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
    let tie_gap = (top_score / 5).max(3);
    top_score > runner_up_score && top_score - runner_up_score >= tie_gap
}

fn score_confidence(top_score: u32, runner_up_score: u32) -> RouteConfidence {
    let total = top_score.saturating_add(runner_up_score);
    let raw = if total == 0 {
        0.0
    } else {
        top_score as f32 / total as f32
    };
    RouteConfidence::new(raw.clamp(0.0, 1.0)).expect("confidence is normalized")
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
            40,
            WorkspaceSkillRouteExecutionMode::DirectDispatch,
            Some(SkillCommandDispatchSpec {
                kind: SkillCommandDispatchKind::Tool,
                tool_name: "exec".to_string(),
                arg_mode: SkillCommandArgMode::Raw,
            }),
        )]);

        assert_eq!(decision.skill_id(), Some("feishu-pm-daily-sync"));
        assert_eq!(decision.confidence().score(), 1.0);
        assert_eq!(
            decision.intent(),
            crate::agent::runtime::skill_routing::intent::InvocationIntent::DirectDispatchSkill {
                skill_id: "feishu-pm-daily-sync".to_string(),
            }
        );
    }

    #[test]
    fn close_tie_falls_back_to_open_task_with_ambiguous_candidates() {
        let decision = adjudicate_route(&[
            build_candidate(
                "feishu-pm-daily-sync",
                20,
                WorkspaceSkillRouteExecutionMode::Inline,
                None,
            ),
            build_candidate(
                "feishu-pm-weekly-work-summary",
                18,
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
            50,
            WorkspaceSkillRouteExecutionMode::Inline,
            None,
        )]);
        let fork = adjudicate_route(&[build_candidate(
            "feishu-pm-fork",
            50,
            WorkspaceSkillRouteExecutionMode::Fork,
            None,
        )]);
        let dispatch = adjudicate_route(&[build_candidate(
            "feishu-pm-dispatch",
            50,
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

        assert_eq!(fork.skill_id(), Some("feishu-pm-fork"));
        assert_eq!(
            fork.intent(),
            crate::agent::runtime::skill_routing::intent::InvocationIntent::PromptSkillFork {
                skill_id: "feishu-pm-fork".to_string(),
            }
        );

        assert_eq!(dispatch.skill_id(), Some("feishu-pm-dispatch"));
        assert_eq!(
            dispatch.intent(),
            crate::agent::runtime::skill_routing::intent::InvocationIntent::DirectDispatchSkill {
                skill_id: "feishu-pm-dispatch".to_string(),
            }
        );
    }
}
