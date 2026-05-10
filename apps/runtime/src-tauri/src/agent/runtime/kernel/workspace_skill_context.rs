use crate::agent::runtime::runtime_io::{
    build_workspace_skill_command_specs, sync_workspace_skills_to_directory,
    WorkspaceSkillCommandSpec, WorkspaceSkillRuntimeEntry,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct WorkspaceSkillContext {
    pub workspace_skills_prompt: Option<String>,
    pub skill_command_specs: Vec<WorkspaceSkillCommandSpec>,
}

pub(crate) fn build_workspace_skill_context(
    work_dir: Option<&std::path::Path>,
    entries: &[WorkspaceSkillRuntimeEntry],
    suppress_workspace_skills_prompt: bool,
) -> Result<WorkspaceSkillContext, String> {
    let Some(work_dir) = work_dir else {
        return Ok(WorkspaceSkillContext::default());
    };

    let skill_command_specs = build_workspace_skill_command_specs(entries);
    let workspace_skills_prompt = if suppress_workspace_skills_prompt {
        sync_workspace_skills_to_directory(work_dir, entries)?;
        None
    } else {
        build_workspace_skill_summary_prompt(entries)
    };

    Ok(WorkspaceSkillContext {
        workspace_skills_prompt,
        skill_command_specs,
    })
}

fn build_workspace_skill_summary_prompt(entries: &[WorkspaceSkillRuntimeEntry]) -> Option<String> {
    let visible_entries = entries
        .iter()
        .filter(|entry| !entry.invocation.disable_model_invocation)
        .collect::<Vec<_>>();
    if visible_entries.is_empty() {
        return None;
    }

    let mut blocks = Vec::with_capacity(visible_entries.len() + 4);
    blocks.push("<available_skills>".to_string());
    blocks.push(
        "Use the `skills` tool with action `skill_view` and the matching `skill_id` to inspect a skill before relying on its detailed instructions or assets."
            .to_string(),
    );
    for entry in visible_entries {
        blocks.push(format!(
            "<skill>\n<skill_id>{}</skill_id>\n<name>{}</name>\n<description>{}</description>\n<source_type>{}</source_type>\n</skill>",
            entry.skill_id.trim(),
            entry.name.trim(),
            entry.description.trim(),
            entry.source_type.trim()
        ));
    }
    blocks.push("</available_skills>".to_string());
    Some(blocks.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::build_workspace_skill_context;
    use crate::agent::runtime::runtime_io::{WorkspaceSkillContent, WorkspaceSkillRuntimeEntry};
    use runtime_skill_core::{SkillConfig, SkillInvocationPolicy};
    use tempfile::tempdir;

    fn build_entry() -> WorkspaceSkillRuntimeEntry {
        WorkspaceSkillRuntimeEntry {
            skill_id: "pm-summary".to_string(),
            name: "PM Summary".to_string(),
            description: "Summarize PM updates".to_string(),
            source_type: "builtin".to_string(),
            projected_dir_name: "pm-summary".to_string(),
            config: SkillConfig::default(),
            invocation: SkillInvocationPolicy {
                user_invocable: true,
                disable_model_invocation: false,
            },
            metadata: None,
            command_dispatch: None,
            content: WorkspaceSkillContent::FileTree(
                [("SKILL.md".to_string(), b"# PM Summary".to_vec())]
                    .into_iter()
                    .collect(),
            ),
        }
    }

    #[test]
    fn workspace_skill_context_keeps_prompt_and_command_specs_together() {
        let tmp = tempdir().expect("tempdir");
        let work_dir = tmp.path().join("workspace");

        let context =
            build_workspace_skill_context(Some(work_dir.as_path()), &[build_entry()], false)
                .expect("workspace skill context");

        assert!(context
            .workspace_skills_prompt
            .as_deref()
            .expect("workspace skills prompt")
            .contains("<available_skills>"));
        assert_eq!(context.skill_command_specs.len(), 1);
        assert_eq!(context.skill_command_specs[0].name, "pm_summary");
    }

    #[test]
    fn workspace_skill_context_uses_summary_prompt_without_projecting_all_skills() {
        let tmp = tempdir().expect("tempdir");
        let work_dir = tmp.path().join("workspace");

        let context =
            build_workspace_skill_context(Some(work_dir.as_path()), &[build_entry()], false)
                .expect("workspace skill context");

        let prompt = context
            .workspace_skills_prompt
            .as_deref()
            .expect("workspace skills prompt");
        assert!(prompt.contains("<available_skills>"));
        assert!(prompt.contains("<skill_id>pm-summary</skill_id>"));
        assert!(prompt.contains("skills"));
        assert!(prompt.contains("skill_view"));
        assert!(!work_dir
            .join("skills")
            .join("pm-summary")
            .join("SKILL.md")
            .exists());
    }

    #[test]
    fn workspace_skill_context_still_syncs_skills_when_prompt_is_suppressed() {
        let tmp = tempdir().expect("tempdir");
        let work_dir = tmp.path().join("workspace");

        let context =
            build_workspace_skill_context(Some(work_dir.as_path()), &[build_entry()], true)
                .expect("workspace skill context");

        assert!(context.workspace_skills_prompt.is_none());
        assert_eq!(context.skill_command_specs.len(), 1);
        assert!(work_dir
            .join("skills")
            .join("pm-summary")
            .join("SKILL.md")
            .exists());
    }
}
