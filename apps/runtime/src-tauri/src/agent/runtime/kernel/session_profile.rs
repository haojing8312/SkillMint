#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum SessionSurfaceKind {
    #[default]
    LocalChat,
    HiddenChildSession,
    EmployeeStepSession,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct SessionExecutionProfile {
    pub surface: SessionSurfaceKind,
}

impl SessionExecutionProfile {
    pub(crate) fn for_surface(surface: SessionSurfaceKind) -> Self {
        Self { surface }
    }

    pub(crate) fn local_chat() -> Self {
        Self::for_surface(SessionSurfaceKind::LocalChat)
    }

    pub(crate) fn hidden_child_session() -> Self {
        Self::for_surface(SessionSurfaceKind::HiddenChildSession)
    }

    pub(crate) fn employee_step_session() -> Self {
        Self::for_surface(SessionSurfaceKind::EmployeeStepSession)
    }
}
