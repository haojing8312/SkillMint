use crate::agent::runtime::attempt_runner::RouteExecutionOutcome;
use crate::agent::runtime::runtime_io::{
    WorkspaceSkillCommandSpec, WorkspaceSkillContent, WorkspaceSkillRuntimeEntry,
};
use crate::agent::runtime::skill_routing::intent::RouteFallbackReason;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RoutedSkillToolSetup {
    pub skill_id: String,
    pub skill_system_prompt: String,
    pub skill_allowed_tools: Option<Vec<String>>,
    pub max_iterations: Option<usize>,
    pub source_type: String,
    pub pack_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RouteRunPlan {
    OpenTask {
        fallback_reason: Option<RouteFallbackReason>,
    },
    PromptSkillInline {
        skill_id: String,
        setup: RoutedSkillToolSetup,
    },
    PromptSkillFork {
        skill_id: String,
        setup: RoutedSkillToolSetup,
    },
    DirectDispatchSkill {
        skill_id: String,
        setup: RoutedSkillToolSetup,
        command_spec: WorkspaceSkillCommandSpec,
        raw_args: String,
    },
}

#[derive(Debug)]
pub(crate) enum RouteRunOutcome {
    OpenTask,
    DirectDispatch(String),
    Prompt {
        route_execution: RouteExecutionOutcome,
        reconstructed_history_len: usize,
    },
}

pub(crate) fn build_routed_skill_tool_setup(
    entry: &WorkspaceSkillRuntimeEntry,
) -> RoutedSkillToolSetup {
    RoutedSkillToolSetup {
        skill_id: entry.skill_id.clone(),
        skill_system_prompt: entry.config.system_prompt.clone(),
        skill_allowed_tools: entry.config.allowed_tools.clone(),
        max_iterations: entry.config.max_iterations,
        source_type: entry.source_type.clone(),
        pack_path: match &entry.content {
            WorkspaceSkillContent::LocalDir(path) => path.to_string_lossy().to_string(),
            WorkspaceSkillContent::FileTree(_) => String::new(),
        },
    }
}
