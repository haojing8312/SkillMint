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
    family_tokens: HashSet<String>,
    skill_id_compact: String,
    display_name_compact: String,
    alias_compacts: Vec<String>,
    description_compact: String,
    when_to_use_compact: String,
    family_compact: Option<String>,
}

impl ProjectionProfile {
    fn from_projection(projection: &WorkspaceSkillRouteProjection) -> Self {
        let aliases = projection.aliases.clone();
        let family = derive_skill_family(&projection.skill_id);
        let family_compact = family.as_ref().map(|value| normalize_compact(value));

        Self {
            skill_id_tokens: tokenize(&projection.skill_id).into_iter().collect(),
            display_name_tokens: tokenize(&projection.display_name).into_iter().collect(),
            alias_tokens: aliases
                .iter()
                .flat_map(|alias| tokenize(alias))
                .collect::<HashSet<_>>(),
            description_tokens: tokenize(&projection.description).into_iter().collect(),
            when_to_use_tokens: tokenize(&projection.when_to_use).into_iter().collect(),
            family_tokens: family
                .as_ref()
                .map(|value| tokenize(value).into_iter().collect())
                .unwrap_or_default(),
            skill_id_compact: normalize_compact(&projection.skill_id),
            display_name_compact: normalize_compact(&projection.display_name),
            alias_compacts: aliases.iter().map(|alias| normalize_compact(alias)).collect(),
            description_compact: normalize_compact(&projection.description),
            when_to_use_compact: normalize_compact(&projection.when_to_use),
            family_compact,
        }
    }
}

fn score_projection(query: &QueryProfile, projection: &WorkspaceSkillRouteProjection) -> u32 {
    let profile = ProjectionProfile::from_projection(projection);
    let mut score = 0u32;

    score += score_exact_compact_match(query, &profile);
    score += score_token_overlap(query, &profile);

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

    if let Some(family_compact) = &profile.family_compact {
        if family_compact.contains(&query.compact) || query.compact.contains(family_compact) {
            score += 24;
        }
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
        if profile.family_tokens.contains(token) {
            score += 30;
        }
        score
    })
}

fn derive_skill_family(skill_id: &str) -> Option<String> {
    let segments = skill_id
        .split('-')
        .filter(|segment| !segment.trim().is_empty())
        .collect::<Vec<_>>();

    match segments.as_slice() {
        [] => None,
        [single] => Some((*single).to_string()),
        [first, second, ..] => Some(format!("{first}-{second}")),
    }
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
    fn recall_prefers_family_match_for_domain_words() {
        let index = build_index(vec![
            build_entry(
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
            ),
            build_entry(
                "feishu-bitable-analyst",
                "Bitable Analyst",
                "Inspect table health and relationship structure",
                "## When to Use\n- Use when a table needs review.\n",
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

        let candidates = recall_skill_candidates(&index, "bitable");
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].projection.skill_id, "feishu-bitable-analyst");
        assert!(candidates[0].score > 0);
    }

    #[test]
    fn recall_matches_aliases() {
        let index = build_index(vec![
            build_entry(
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
            ),
            build_entry(
                "feishu-pm-weekly-work-summary",
                "项管周工作汇总",
                "Summarize PM work",
                "## When to Use\n- Use when you need to summarize PM updates for a week.\n",
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
                "feishu-pm-alpha",
                "PM Alpha",
                "Track PM alpha work",
                "## When to Use\n- Use when the request is pm related.\n",
                None,
                Some(vec!["read_file"]),
                None,
                SkillInvocationPolicy {
                    user_invocable: true,
                    disable_model_invocation: false,
                },
                Some("alpha"),
                None,
            ),
            build_entry(
                "feishu-pm-beta",
                "PM Beta",
                "Track PM beta work",
                "## When to Use\n- Use when the request is pm related.\n",
                None,
                Some(vec!["read_file"]),
                None,
                SkillInvocationPolicy {
                    user_invocable: true,
                    disable_model_invocation: false,
                },
                Some("beta"),
                None,
            ),
            build_entry(
                "feishu-bitable-analyst",
                "Bitable Analyst",
                "Inspect table health and relationship structure",
                "## When to Use\n- Use when a table needs review.\n",
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

        let candidates = recall_skill_candidates(&index, "pm");
        let skill_ids = candidates
            .iter()
            .map(|candidate| candidate.projection.skill_id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(skill_ids, vec!["feishu-pm-alpha", "feishu-pm-beta"]);
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
        assert!(recall_skill_candidates(&index, "no-such-skill-domain").is_empty());
    }
}
