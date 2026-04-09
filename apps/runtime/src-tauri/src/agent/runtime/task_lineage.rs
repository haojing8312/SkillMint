use crate::session_journal::{SessionRunTaskIdentitySnapshot, SessionRunTurnStateSnapshot};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionRunTaskGraphNode {
    pub task_id: String,
    pub parent_task_id: Option<String>,
    pub root_task_id: String,
    pub task_kind: String,
    pub surface_kind: String,
    pub task_path: String,
}

pub fn effective_task_identity<'a>(
    task_identity: Option<&'a SessionRunTaskIdentitySnapshot>,
    turn_state: Option<&'a SessionRunTurnStateSnapshot>,
) -> Option<&'a SessionRunTaskIdentitySnapshot> {
    task_identity.or_else(|| turn_state.and_then(|turn_state| turn_state.task_identity.as_ref()))
}

pub fn build_task_path(task_identity: &SessionRunTaskIdentitySnapshot) -> Option<String> {
    let mut segments = Vec::new();
    for candidate in [
        Some(task_identity.root_task_id.as_str()),
        task_identity.parent_task_id.as_deref(),
        Some(task_identity.task_id.as_str()),
    ] {
        let Some(candidate) = candidate.map(str::trim).filter(|value| !value.is_empty()) else {
            continue;
        };
        if !segments
            .iter()
            .any(|existing: &String| existing == candidate)
        {
            segments.push(candidate.to_string());
        }
    }

    if segments.is_empty() {
        None
    } else {
        Some(segments.join(" -> "))
    }
}

pub fn project_task_graph_nodes<'a>(
    task_identities: impl IntoIterator<Item = &'a SessionRunTaskIdentitySnapshot>,
) -> Vec<SessionRunTaskGraphNode> {
    let mut nodes = Vec::new();
    for task_identity in task_identities {
        let task_id = task_identity.task_id.trim();
        if task_id.is_empty() {
            continue;
        }
        if nodes
            .iter()
            .any(|node: &SessionRunTaskGraphNode| node.task_id == task_id)
        {
            continue;
        }
        nodes.push(SessionRunTaskGraphNode {
            task_id: task_id.to_string(),
            parent_task_id: task_identity
                .parent_task_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned),
            root_task_id: task_identity.root_task_id.trim().to_string(),
            task_kind: task_identity.task_kind.trim().to_string(),
            surface_kind: task_identity.surface_kind.trim().to_string(),
            task_path: build_task_path(task_identity).unwrap_or_else(|| task_id.to_string()),
        });
    }
    nodes
}

#[cfg(test)]
mod tests {
    use super::{
        build_task_path, effective_task_identity, project_task_graph_nodes, SessionRunTaskGraphNode,
    };
    use crate::session_journal::{SessionRunTaskIdentitySnapshot, SessionRunTurnStateSnapshot};

    #[test]
    fn effective_task_identity_falls_back_to_turn_state_snapshot() {
        let turn_state = SessionRunTurnStateSnapshot {
            task_identity: Some(SessionRunTaskIdentitySnapshot {
                task_id: "task-child".to_string(),
                parent_task_id: Some("task-parent".to_string()),
                root_task_id: "task-root".to_string(),
                task_kind: "sub_agent_task".to_string(),
                surface_kind: "hidden_child_surface".to_string(),
            }),
            session_surface: None,
            execution_lane: None,
            selected_runner: None,
            selected_skill: None,
            fallback_reason: None,
            allowed_tools: Vec::new(),
            invoked_skills: Vec::new(),
            partial_assistant_text: String::new(),
            tool_failure_streak: 0,
            reconstructed_history_len: None,
            compaction_boundary: None,
        };

        let effective = effective_task_identity(None, Some(&turn_state)).expect("task identity");
        assert_eq!(effective.task_id, "task-child");
    }

    #[test]
    fn project_task_graph_nodes_deduplicates_task_ids_and_keeps_task_path() {
        let task_identity = SessionRunTaskIdentitySnapshot {
            task_id: "task-child".to_string(),
            parent_task_id: Some("task-parent".to_string()),
            root_task_id: "task-root".to_string(),
            task_kind: "sub_agent_task".to_string(),
            surface_kind: "hidden_child_surface".to_string(),
        };

        let nodes = project_task_graph_nodes([&task_identity, &task_identity]);

        assert_eq!(
            nodes,
            vec![SessionRunTaskGraphNode {
                task_id: "task-child".to_string(),
                parent_task_id: Some("task-parent".to_string()),
                root_task_id: "task-root".to_string(),
                task_kind: "sub_agent_task".to_string(),
                surface_kind: "hidden_child_surface".to_string(),
                task_path: build_task_path(&task_identity).expect("task path should be projected"),
            }]
        );
    }
}
