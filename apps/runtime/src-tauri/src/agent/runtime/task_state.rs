use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum TaskKind {
    PrimaryUserTask,
    DelegatedSkillTask,
    SubAgentTask,
    EmployeeStepTask,
    RecoveryTask,
}

impl TaskKind {
    pub(crate) fn journal_key(self) -> &'static str {
        match self {
            TaskKind::PrimaryUserTask => "primary_user_task",
            TaskKind::DelegatedSkillTask => "delegated_skill_task",
            TaskKind::SubAgentTask => "sub_agent_task",
            TaskKind::EmployeeStepTask => "employee_step_task",
            TaskKind::RecoveryTask => "recovery_task",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum TaskSurfaceKind {
    LocalChatSurface,
    HiddenChildSurface,
    EmployeeStepSurface,
}

impl TaskSurfaceKind {
    pub(crate) fn journal_key(self) -> &'static str {
        match self {
            TaskSurfaceKind::LocalChatSurface => "local_chat_surface",
            TaskSurfaceKind::HiddenChildSurface => "hidden_child_surface",
            TaskSurfaceKind::EmployeeStepSurface => "employee_step_surface",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TaskIdentity {
    pub task_id: String,
    pub parent_task_id: Option<String>,
    pub root_task_id: String,
}

impl TaskIdentity {
    pub(crate) fn new(
        task_id: impl Into<String>,
        parent_task_id: Option<impl Into<String>>,
        root_task_id: Option<impl Into<String>>,
    ) -> Self {
        let task_id = task_id.into();
        let root_task_id = root_task_id
            .map(Into::into)
            .unwrap_or_else(|| task_id.clone());

        Self {
            task_id,
            parent_task_id: parent_task_id.map(Into::into),
            root_task_id,
        }
    }

    pub(crate) fn new_root() -> Self {
        let task_id = Uuid::new_v4().to_string();
        Self::new(task_id, Option::<String>::None, Option::<String>::None)
    }

    pub(crate) fn new_child(parent: &TaskIdentity) -> Self {
        let task_id = Uuid::new_v4().to_string();
        Self::new(
            task_id,
            Some(parent.task_id.clone()),
            Some(parent.root_task_id.clone()),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TaskState {
    pub task_identity: TaskIdentity,
    pub task_kind: TaskKind,
    pub surface_kind: TaskSurfaceKind,
    pub session_id: String,
    pub user_message_id: String,
    pub run_id: String,
}

impl TaskState {
    fn new_for_task_kind(
        task_kind: TaskKind,
        surface_kind: TaskSurfaceKind,
        session_id: impl Into<String>,
        user_message_id: impl Into<String>,
        run_id: impl Into<String>,
        parent_task_identity: Option<&TaskIdentity>,
    ) -> Self {
        Self {
            task_identity: parent_task_identity
                .map(TaskIdentity::new_child)
                .unwrap_or_else(TaskIdentity::new_root),
            task_kind,
            surface_kind,
            session_id: session_id.into(),
            user_message_id: user_message_id.into(),
            run_id: run_id.into(),
        }
    }

    pub(crate) fn new_primary_local_chat(
        session_id: impl Into<String>,
        user_message_id: impl Into<String>,
        run_id: impl Into<String>,
    ) -> Self {
        Self::new_for_task_kind(
            TaskKind::PrimaryUserTask,
            TaskSurfaceKind::LocalChatSurface,
            session_id,
            user_message_id,
            run_id,
            None,
        )
    }

    pub(crate) fn new_sub_agent(
        session_id: impl Into<String>,
        user_message_id: impl Into<String>,
        run_id: impl Into<String>,
        parent_task_identity: Option<&TaskIdentity>,
    ) -> Self {
        Self::new_for_task_kind(
            TaskKind::SubAgentTask,
            TaskSurfaceKind::HiddenChildSurface,
            session_id,
            user_message_id,
            run_id,
            parent_task_identity,
        )
    }

    pub(crate) fn new_employee_step(
        session_id: impl Into<String>,
        user_message_id: impl Into<String>,
        run_id: impl Into<String>,
        parent_task_identity: Option<&TaskIdentity>,
    ) -> Self {
        Self::new_for_task_kind(
            TaskKind::EmployeeStepTask,
            TaskSurfaceKind::EmployeeStepSurface,
            session_id,
            user_message_id,
            run_id,
            parent_task_identity,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{TaskIdentity, TaskKind, TaskState, TaskSurfaceKind};

    #[test]
    fn primary_local_chat_task_uses_primary_user_contract() {
        let task_state = TaskState::new_primary_local_chat("session-1", "user-1", "run-1");

        assert_eq!(task_state.task_kind, TaskKind::PrimaryUserTask);
        assert_eq!(task_state.surface_kind, TaskSurfaceKind::LocalChatSurface);
        assert_eq!(task_state.session_id, "session-1");
        assert_eq!(task_state.user_message_id, "user-1");
        assert_eq!(task_state.run_id, "run-1");
    }

    #[test]
    fn primary_local_chat_task_defaults_root_identity_to_itself() {
        let task_state = TaskState::new_primary_local_chat("session-1", "user-1", "run-1");

        assert!(
            !task_state.task_identity.task_id.trim().is_empty(),
            "task_id should be generated"
        );
        assert_eq!(task_state.task_identity.parent_task_id, None);
        assert_eq!(
            task_state.task_identity.root_task_id,
            task_state.task_identity.task_id
        );
    }

    #[test]
    fn task_identity_supports_explicit_parent_and_root_ids() {
        let identity = TaskIdentity::new("task-child", Some("task-parent"), Some("task-root"));

        assert_eq!(identity.task_id, "task-child");
        assert_eq!(identity.parent_task_id.as_deref(), Some("task-parent"));
        assert_eq!(identity.root_task_id, "task-root");
    }

    #[test]
    fn child_task_identity_inherits_parent_root_and_parent_pointer() {
        let parent = TaskIdentity::new("task-parent", Option::<String>::None, Some("task-root"));

        let identity = TaskIdentity::new_child(&parent);

        assert_eq!(identity.parent_task_id.as_deref(), Some("task-parent"));
        assert_eq!(identity.root_task_id, "task-root");
        assert_ne!(identity.task_id, "task-parent");
    }

    #[test]
    fn employee_step_task_can_inherit_existing_task_lineage() {
        let parent = TaskIdentity::new("task-parent", Option::<String>::None, Some("task-root"));

        let task_state =
            TaskState::new_employee_step("session-1", "user-1", "run-1", Some(&parent));

        assert_eq!(task_state.task_kind, TaskKind::EmployeeStepTask);
        assert_eq!(
            task_state.surface_kind,
            TaskSurfaceKind::EmployeeStepSurface
        );
        assert_eq!(
            task_state.task_identity.parent_task_id.as_deref(),
            Some("task-parent")
        );
        assert_eq!(task_state.task_identity.root_task_id, "task-root");
    }
}
