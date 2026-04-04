use crate::agent::runtime::runtime_io::WorkspaceSkillRouteProjection;
use crate::agent::runtime::skill_routing::index::SkillRouteIndex;
use std::collections::HashSet;

const MAX_RECALL_CANDIDATES: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillRecallCandidate {
    pub projection: WorkspaceSkillRouteProjection,
    pub score: u32,
}

pub fn recall_skill_candidates(
    route_index: &SkillRouteIndex,
    query: &str,
) -> Vec<SkillRecallCandidate> {
    let query_profile = QueryProfile::from_query(query);
    if query_profile.is_empty() {
        return Vec::new();
    }

    let mut scored = route_index
        .entries()
        .enumerate()
        .filter_map(|(ordinal, projection)| {
            let score = score_projection(&query_profile, projection);
            (score > 0).then_some(ScoredCandidate {
                ordinal,
                score,
                projection: projection.clone(),
            })
        })
        .collect::<Vec<_>>();

    scored.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.ordinal.cmp(&right.ordinal))
            .then_with(|| left.projection.skill_id.cmp(&right.projection.skill_id))
    });

    scored
        .into_iter()
        .take(MAX_RECALL_CANDIDATES)
        .map(|candidate| SkillRecallCandidate {
            projection: candidate.projection,
            score: candidate.score,
        })
        .collect()
}

#[derive(Debug, Clone)]
struct ScoredCandidate {
    ordinal: usize,
    score: u32,
    projection: WorkspaceSkillRouteProjection,
}

#[derive(Debug, Clone)]
struct QueryProfile {
    compact: String,
    tokens: HashSet<String>,
}

impl QueryProfile {
    fn from_query(query: &str) -> Self {
        let compact = normalize_compact(query);
        let tokens = tokenize(query).into_iter().collect();
        Self { compact, tokens }
    }

    fn is_empty(&self) -> bool {
        self.compact.is_empty() || self.tokens.is_empty()
    }
}

#[derive(Debug, Clone)]
struct ProjectionProfile {
    skill_id_tokens: HashSet<String>,
    display_name_tokens: HashSet<String>,
    alias_tokens: HashSet<String>,
    description_tokens: HashSet<String>,
    when_to_use_tokens: HashSet<String>,
    skill_id_compact: String,
    display_name_compact: String,
    alias_compacts: Vec<String>,
    description_compact: String,
    when_to_use_compact: String,
    domain_tag_compacts: Vec<String>,
}

impl ProjectionProfile {
    fn from_projection(projection: &WorkspaceSkillRouteProjection) -> Self {
        let aliases = projection.aliases.clone();

        Self {
            skill_id_tokens: tokenize(&projection.skill_id).into_iter().collect(),
            display_name_tokens: tokenize(&projection.display_name).into_iter().collect(),
            alias_tokens: aliases
                .iter()
                .flat_map(|alias| tokenize(alias))
                .collect::<HashSet<_>>(),
            description_tokens: tokenize(&projection.description).into_iter().collect(),
            when_to_use_tokens: tokenize(&projection.when_to_use).into_iter().collect(),
            skill_id_compact: normalize_compact(&projection.skill_id),
            display_name_compact: normalize_compact(&projection.display_name),
            alias_compacts: aliases.iter().map(|alias| normalize_compact(alias)).collect(),
            description_compact: normalize_compact(&projection.description),
            when_to_use_compact: normalize_compact(&projection.when_to_use),
            domain_tag_compacts: projection
                .domain_tags
                .iter()
                .map(|tag| normalize_compact(tag))
                .filter(|tag| !tag.is_empty())
                .collect(),
        }
    }
}

fn score_projection(query: &QueryProfile, projection: &WorkspaceSkillRouteProjection) -> u32 {
    let profile = ProjectionProfile::from_projection(projection);
    let mut score = 0u32;

    score += score_exact_compact_match(query, &profile);
    score += score_token_overlap(query, &profile);
    score += score_domain_tags(query, &profile);

    score
}

fn score_exact_compact_match(query: &QueryProfile, profile: &ProjectionProfile) -> u32 {
    let mut score = 0;

    if profile.skill_id_compact == query.compact
        || profile.display_name_compact == query.compact
        || profile.alias_compacts.iter().any(|alias| alias == &query.compact)
    {
        score += 120;
    }

    if profile.skill_id_compact.contains(&query.compact) {
        score += 30;
    }

    if profile.display_name_compact.contains(&query.compact) {
        score += 18;
    }

    if profile.alias_compacts.iter().any(|alias| alias.contains(&query.compact)) {
        score += 40;
    }

    if profile.description_compact.contains(&query.compact) {
        score += 12;
    }

    if profile.when_to_use_compact.contains(&query.compact) {
        score += 15;
    }

    score
}

fn score_token_overlap(query: &QueryProfile, profile: &ProjectionProfile) -> u32 {
    query.tokens.iter().fold(0, |mut score, token| {
        if profile.skill_id_tokens.contains(token) {
            score += 18;
        }
        if profile.display_name_tokens.contains(token) {
            score += 8;
        }
        if profile.alias_tokens.contains(token) {
            score += 24;
        }
        if profile.description_tokens.contains(token) {
            score += 6;
        }
        if profile.when_to_use_tokens.contains(token) {
            score += 10;
        }
        score
    })
}

fn score_domain_tags(query: &QueryProfile, profile: &ProjectionProfile) -> u32 {
    profile
        .domain_tag_compacts
        .iter()
        .fold(0, |mut score, tag| {
            if query.compact == *tag {
                score += 38;
            } else if query.compact.contains(tag) || tag.contains(&query.compact) {
                score += 22;
            }
            score
        })
}

fn normalize_compact(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

fn tokenize(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in value.chars() {
        if ch.is_alphanumeric() {
            current.extend(ch.to_lowercase());
        } else if !current.is_empty() {
            tokens.push(std::mem::take(&mut current));
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::runtime::runtime_io::{
        WorkspaceSkillContent, WorkspaceSkillRuntimeEntry,
    };
    use runtime_skill_core::{
        OpenClawSkillMetadata, SkillCommandArgMode, SkillCommandDispatchKind,
        SkillCommandDispatchSpec, SkillConfig, SkillInvocationPolicy,
    };

    fn build_entry(
        skill_id: &str,
        name: &str,
        description: &str,
        system_prompt: &str,
        context: Option<&str>,
        allowed_tools: Option<Vec<&str>>,
        max_iterations: Option<usize>,
        invocation: SkillInvocationPolicy,
        metadata_skill_key: Option<&str>,
        command_dispatch: Option<SkillCommandDispatchSpec>,
    ) -> WorkspaceSkillRuntimeEntry {
        let command_dispatch_for_config = command_dispatch.clone();
        let allowed_tools_for_config = allowed_tools
            .clone()
            .map(|values| values.into_iter().map(|value| value.to_string()).collect());
        WorkspaceSkillRuntimeEntry {
            skill_id: skill_id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            source_type: "local".to_string(),
            projected_dir_name: skill_id.to_string(),
            config: SkillConfig {
                name: Some(name.to_string()),
                description: Some(description.to_string()),
                allowed_tools: allowed_tools_for_config,
                model: None,
                max_iterations,
                argument_hint: None,
                disable_model_invocation: invocation.disable_model_invocation,
                user_invocable: invocation.user_invocable,
                invocation: invocation.clone(),
                metadata: metadata_skill_key.map(|skill_key| OpenClawSkillMetadata {
                    skill_key: Some(skill_key.to_string()),
                    ..Default::default()
                }),
                command_dispatch: command_dispatch_for_config,
                context: context.map(|value| value.to_string()),
                agent: None,
                mcp_servers: vec![],
                system_prompt: system_prompt.to_string(),
            },
            invocation,
            metadata: metadata_skill_key.map(|skill_key| OpenClawSkillMetadata {
                skill_key: Some(skill_key.to_string()),
                ..Default::default()
            }),
            command_dispatch,
            content: WorkspaceSkillContent::FileTree(std::collections::HashMap::new()),
        }
    }

    fn build_index(entries: Vec<WorkspaceSkillRuntimeEntry>) -> SkillRouteIndex {
        SkillRouteIndex::build(&entries)
    }

    #[test]
    fn recall_prefers_family_domain_tags_for_chinese_vocabulary() {
        let index = build_index(vec![
            build_entry(
                "feishu-pm-daily-sync",
                "PM Daily Sync",
                "Coordinate routine reporting",
                "## When to Use\n- Keep the routine reporting flow aligned.\n",
                None,
                Some(vec!["read_file"]),
                Some(4),
                SkillInvocationPolicy {
                    user_invocable: true,
                    disable_model_invocation: false,
                },
                Some("daily-sync"),
                None,
            ),
            build_entry(
                "feishu-bitable-analyst",
                "Bitable Analyst",
                "Inspect table health and relationships",
                "## When to Use\n- Review table layout and fields.\n",
                None,
                Some(vec!["read_file"]),
                None,
                SkillInvocationPolicy {
                    user_invocable: true,
                    disable_model_invocation: false,
                },
                None,
                None,
            ),
        ]);

        let candidates = recall_skill_candidates(&index, "帮我整理项管日报并同步");
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].projection.skill_id, "feishu-pm-daily-sync");
        assert!(candidates[0].score > 0);
    }

    #[test]
    fn recall_matches_aliases() {
        let index = build_index(vec![
            build_entry(
                "feishu-pm-task-dispatch",
                "PM Task Dispatch",
                "Create or dispatch PM follow-up tasks",
                "## When to Use\n- Dispatch a correction task for a leader.\n",
                Some("fork"),
                Some(vec!["exec", "read_file"]),
                Some(11),
                SkillInvocationPolicy {
                    user_invocable: true,
                    disable_model_invocation: true,
                },
                Some("task-dispatch"),
                Some(SkillCommandDispatchSpec {
                    kind: SkillCommandDispatchKind::Tool,
                    tool_name: "exec".to_string(),
                    arg_mode: SkillCommandArgMode::Raw,
                }),
            ),
            build_entry(
                "feishu-pm-weekly-work-summary",
                "PM Weekly Summary",
                "Organize weekly reporting",
                "## When to Use\n- Keep reporting aligned.\n",
                None,
                Some(vec!["read_file"]),
                Some(3),
                SkillInvocationPolicy {
                    user_invocable: true,
                    disable_model_invocation: false,
                },
                Some("weekly-summary"),
                None,
            ),
        ]);

        let candidates = recall_skill_candidates(&index, "task-dispatch");
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].projection.skill_id, "feishu-pm-task-dispatch");
        assert!(candidates[0].score > 0);
    }

    #[test]
    fn recall_orders_multiple_candidates_deterministically() {
        let index = build_index(vec![
            build_entry(
                "feishu-pm-weekly-work-summary",
                "PM Weekly Summary",
                "项管日报汇总与任务追踪",
                "## When to Use\n- 处理项管日报汇总并整理任务。\n",
                None,
                Some(vec!["read_file"]),
                None,
                SkillInvocationPolicy {
                    user_invocable: true,
                    disable_model_invocation: false,
                },
                Some("weekly-summary"),
                None,
            ),
            build_entry(
                "feishu-pm-daily-sync",
                "PM Daily Sync",
                "项管日报同步",
                "## When to Use\n- 同步项管日报到看板。\n",
                None,
                Some(vec!["read_file"]),
                None,
                SkillInvocationPolicy {
                    user_invocable: true,
                    disable_model_invocation: false,
                },
                Some("daily-sync"),
                None,
            ),
            build_entry(
                "feishu-bitable-analyst",
                "Bitable Analyst",
                "Inspect table health and relationships",
                "## When to Use\n- Review table layout and fields.\n",
                None,
                Some(vec!["read_file"]),
                None,
                SkillInvocationPolicy {
                    user_invocable: true,
                    disable_model_invocation: false,
                },
                None,
                None,
            ),
        ]);

        let candidates = recall_skill_candidates(&index, "日报汇总");
        let skill_ids = candidates
            .iter()
            .map(|candidate| candidate.projection.skill_id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            skill_ids,
            vec!["feishu-pm-weekly-work-summary", "feishu-pm-daily-sync"]
        );
        assert!(candidates[0].score > candidates[1].score);
    }

    #[test]
    fn recall_returns_empty_for_blank_and_unmatched_queries() {
        let index = build_index(vec![build_entry(
            "feishu-pm-task-dispatch",
            "PM Task Dispatch",
            "Create or dispatch PM follow-up tasks",
            "## When to Use\n- Use when a leader wants to create a correction task.\n",
            Some("fork"),
            Some(vec!["exec", "read_file"]),
            Some(11),
            SkillInvocationPolicy {
                user_invocable: true,
                disable_model_invocation: true,
            },
            Some("task-dispatch"),
            Some(SkillCommandDispatchSpec {
                kind: SkillCommandDispatchKind::Tool,
                tool_name: "exec".to_string(),
                arg_mode: SkillCommandArgMode::Raw,
            }),
        )]);

        assert!(recall_skill_candidates(&index, "   ").is_empty());
        assert!(recall_skill_candidates(&index, "完全无关的查询").is_empty());
    }
}
