#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SkillSourceKind {
    Skillpack,
    Local,
    Preset,
    AgentCreated,
    LegacyBuiltin,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SkillSourcePolicy {
    pub kind: SkillSourceKind,
    pub canonical_label: &'static str,
    pub directory_backed: bool,
    pub immutable_content: bool,
    pub can_delete_installed_row: bool,
}

pub(crate) fn resolve_skill_source_policy(source_type: &str) -> SkillSourcePolicy {
    match source_type.trim().to_lowercase().as_str() {
        "" | "encrypted" | "skillpack" => SkillSourcePolicy {
            kind: SkillSourceKind::Skillpack,
            canonical_label: "skillpack",
            directory_backed: false,
            immutable_content: true,
            can_delete_installed_row: true,
        },
        "local" => SkillSourcePolicy {
            kind: SkillSourceKind::Local,
            canonical_label: "local",
            directory_backed: true,
            immutable_content: false,
            can_delete_installed_row: true,
        },
        "vendored" | "preset" => SkillSourcePolicy {
            kind: SkillSourceKind::Preset,
            canonical_label: "preset",
            directory_backed: true,
            immutable_content: false,
            can_delete_installed_row: true,
        },
        "agent_created" | "agent-created" => SkillSourcePolicy {
            kind: SkillSourceKind::AgentCreated,
            canonical_label: "agent_created",
            directory_backed: true,
            immutable_content: false,
            can_delete_installed_row: true,
        },
        "builtin" => SkillSourcePolicy {
            kind: SkillSourceKind::LegacyBuiltin,
            canonical_label: "builtin",
            directory_backed: false,
            immutable_content: false,
            can_delete_installed_row: true,
        },
        _ => SkillSourcePolicy {
            kind: SkillSourceKind::Unknown,
            canonical_label: "unknown",
            directory_backed: false,
            immutable_content: true,
            can_delete_installed_row: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_skill_source_policy, SkillSourceKind};

    #[test]
    fn skillpack_sources_are_immutable_content() {
        for source in ["", "encrypted", "skillpack"] {
            let policy = resolve_skill_source_policy(source);
            assert_eq!(policy.kind, SkillSourceKind::Skillpack);
            assert!(policy.immutable_content);
            assert!(!policy.directory_backed);
            assert!(policy.can_delete_installed_row);
        }
    }

    #[test]
    fn preset_aliases_are_directory_backed_and_mutable_with_history() {
        for source in ["vendored", "preset"] {
            let policy = resolve_skill_source_policy(source);
            assert_eq!(policy.kind, SkillSourceKind::Preset);
            assert!(policy.directory_backed);
            assert!(!policy.immutable_content);
        }
    }

    #[test]
    fn agent_created_skills_are_directory_backed_and_mutable() {
        for source in ["agent_created", "agent-created"] {
            let policy = resolve_skill_source_policy(source);
            assert_eq!(policy.kind, SkillSourceKind::AgentCreated);
            assert!(policy.directory_backed);
            assert!(!policy.immutable_content);
            assert!(policy.can_delete_installed_row);
        }
    }

    #[test]
    fn unknown_sources_are_not_silently_treated_as_skillpacks_for_mutation() {
        let policy = resolve_skill_source_policy("future-source");
        assert_eq!(policy.kind, SkillSourceKind::Unknown);
        assert!(policy.immutable_content);
        assert!(!policy.can_delete_installed_row);
    }
}
