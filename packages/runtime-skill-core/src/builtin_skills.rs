pub const BUILTIN_GENERAL_SKILL_ID: &str = "builtin-general";
pub const BUILTIN_SKILL_CREATOR_ID: &str = "builtin-skill-creator";
pub const BUILTIN_DOCX_SKILL_ID: &str = "builtin-docx";
pub const BUILTIN_PDF_SKILL_ID: &str = "builtin-pdf";
pub const BUILTIN_PPTX_SKILL_ID: &str = "builtin-pptx";
pub const BUILTIN_XLSX_SKILL_ID: &str = "builtin-xlsx";
pub const BUILTIN_FIND_SKILLS_ID: &str = "builtin-find-skills";
pub const BUILTIN_EMPLOYEE_CREATOR_ID: &str = "builtin-employee-creator";
pub const BUILTIN_MULTISTEP_TODOWRITE_GOVERNANCE: &str = r#"
## 内置 Skill 任务计划治理

如果你是“多步骤执行”的内置 Skill，必须遵守以下规则：

1. 在开始连续执行前，先调用 `todo_write` 建立任务计划。
2. “收集信息”“盘点资源”“生成草案”“等待确认”“执行变更”“返回结果”都必须是正式计划步骤。
3. 在用户确认前，不得把会产生副作用的执行步骤标记为 `in_progress`。
4. 每次阶段推进时，都要更新 `todo_write` 状态，确保任务清单反映当前阶段。
5. 不要跳过计划直接进入执行，也不要把等待用户回复的阶段放在计划之外。
"#;

const BUILTIN_GENERAL_SKILL_MD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../apps/runtime/src-tauri/builtin-skills/general-assistant/SKILL.md"
));
const BUILTIN_SKILL_CREATOR_MD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../apps/runtime/src-tauri/builtin-skills/skill-creator/SKILL.md"
));
const BUILTIN_DOCX_SKILL_MD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../apps/runtime/src-tauri/builtin-skills/docx/SKILL.md"
));
const BUILTIN_PDF_SKILL_MD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../apps/runtime/src-tauri/builtin-skills/pdf/SKILL.md"
));
const BUILTIN_PPTX_SKILL_MD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../apps/runtime/src-tauri/builtin-skills/pptx/SKILL.md"
));
const BUILTIN_XLSX_SKILL_MD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../apps/runtime/src-tauri/builtin-skills/xlsx/SKILL.md"
));
const BUILTIN_FIND_SKILLS_MD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../apps/runtime/src-tauri/builtin-skills/find-skills/SKILL.md"
));
const BUILTIN_EMPLOYEE_CREATOR_SKILL_MD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../apps/runtime/src-tauri/builtin-skills/employee-creator/SKILL.md"
));
const LOCAL_SKILL_TEMPLATE_MD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../apps/runtime/src-tauri/builtin-skills/skill-creator-guide/templates/LOCAL_SKILL_TEMPLATE.md"
));

pub struct BuiltinSkillEntry {
    pub id: &'static str,
    pub markdown: &'static str,
}

const BUILTIN_SKILL_ENTRIES: [BuiltinSkillEntry; 8] = [
    BuiltinSkillEntry {
        id: BUILTIN_GENERAL_SKILL_ID,
        markdown: BUILTIN_GENERAL_SKILL_MD,
    },
    BuiltinSkillEntry {
        id: BUILTIN_SKILL_CREATOR_ID,
        markdown: BUILTIN_SKILL_CREATOR_MD,
    },
    BuiltinSkillEntry {
        id: BUILTIN_DOCX_SKILL_ID,
        markdown: BUILTIN_DOCX_SKILL_MD,
    },
    BuiltinSkillEntry {
        id: BUILTIN_PDF_SKILL_ID,
        markdown: BUILTIN_PDF_SKILL_MD,
    },
    BuiltinSkillEntry {
        id: BUILTIN_PPTX_SKILL_ID,
        markdown: BUILTIN_PPTX_SKILL_MD,
    },
    BuiltinSkillEntry {
        id: BUILTIN_XLSX_SKILL_ID,
        markdown: BUILTIN_XLSX_SKILL_MD,
    },
    BuiltinSkillEntry {
        id: BUILTIN_FIND_SKILLS_ID,
        markdown: BUILTIN_FIND_SKILLS_MD,
    },
    BuiltinSkillEntry {
        id: BUILTIN_EMPLOYEE_CREATOR_ID,
        markdown: BUILTIN_EMPLOYEE_CREATOR_SKILL_MD,
    },
];

pub fn builtin_skill_markdown(skill_id: &str) -> Option<&'static str> {
    builtin_skill_entries()
        .iter()
        .find(|entry| entry.id == skill_id)
        .map(|entry| entry.markdown)
}

pub fn builtin_skill_entries() -> &'static [BuiltinSkillEntry] {
    &BUILTIN_SKILL_ENTRIES
}

pub fn builtin_general_skill_markdown() -> &'static str {
    BUILTIN_GENERAL_SKILL_MD
}

pub fn local_skill_template_markdown() -> &'static str {
    LOCAL_SKILL_TEMPLATE_MD
}

pub fn is_multistep_builtin_skill(skill_id: &str, source_type: &str) -> bool {
    if source_type != "builtin" {
        return false;
    }

    matches!(
        skill_id,
        BUILTIN_GENERAL_SKILL_ID
            | BUILTIN_SKILL_CREATOR_ID
            | BUILTIN_DOCX_SKILL_ID
            | BUILTIN_PDF_SKILL_ID
            | BUILTIN_PPTX_SKILL_ID
            | BUILTIN_XLSX_SKILL_ID
            | BUILTIN_FIND_SKILLS_ID
            | BUILTIN_EMPLOYEE_CREATOR_ID
    )
}

pub fn apply_builtin_todowrite_governance(
    skill_id: &str,
    source_type: &str,
    raw_prompt: &str,
) -> String {
    if !is_multistep_builtin_skill(skill_id, source_type) {
        return raw_prompt.to_string();
    }

    if raw_prompt.contains("内置 Skill 任务计划治理") {
        return raw_prompt.to_string();
    }

    format!(
        "{}\n\n---\n\n{}",
        raw_prompt.trim_end(),
        BUILTIN_MULTISTEP_TODOWRITE_GOVERNANCE.trim()
    )
}

#[cfg(test)]
mod tests {
    use super::{
        apply_builtin_todowrite_governance, builtin_skill_markdown, is_multistep_builtin_skill,
        BUILTIN_EMPLOYEE_CREATOR_ID, BUILTIN_FIND_SKILLS_ID, BUILTIN_GENERAL_SKILL_ID,
        BUILTIN_MULTISTEP_TODOWRITE_GOVERNANCE,
    };

    #[test]
    fn classifies_multistep_builtin_skills() {
        assert!(is_multistep_builtin_skill(
            BUILTIN_EMPLOYEE_CREATOR_ID,
            "builtin"
        ));
        assert!(is_multistep_builtin_skill(
            BUILTIN_FIND_SKILLS_ID,
            "builtin"
        ));
        assert!(is_multistep_builtin_skill(
            BUILTIN_GENERAL_SKILL_ID,
            "builtin"
        ));
        assert!(!is_multistep_builtin_skill(
            "builtin-skill-creator-guide",
            "builtin"
        ));
        assert!(!is_multistep_builtin_skill("local-skill", "local"));
    }

    #[test]
    fn injects_governance_for_multistep_builtin_skill_prompts() {
        let prompt = "---\nname: 测试\ndescription: test\n---\n\n# Skill\n内容";
        let governed =
            apply_builtin_todowrite_governance(BUILTIN_EMPLOYEE_CREATOR_ID, "builtin", prompt);

        assert!(governed.contains("内置 Skill 任务计划治理"));
        assert!(governed.contains("先调用 `todo_write` 建立任务计划"));
        assert!(governed.contains(BUILTIN_MULTISTEP_TODOWRITE_GOVERNANCE.trim()));
    }

    #[test]
    fn skips_governance_for_non_multistep_prompts() {
        let prompt = "---\nname: 测试\ndescription: test\n---\n\n# Skill\n内容";
        let governed =
            apply_builtin_todowrite_governance("builtin-skill-creator-guide", "builtin", prompt);

        assert_eq!(governed, prompt);
    }

    #[test]
    fn builtin_skill_markdown_can_be_governed_for_multistep_skills() {
        let skills = [
            "builtin-docx",
            BUILTIN_EMPLOYEE_CREATOR_ID,
            BUILTIN_FIND_SKILLS_ID,
            BUILTIN_GENERAL_SKILL_ID,
            "builtin-pdf",
            "builtin-pptx",
            "builtin-skill-creator",
            "builtin-xlsx",
        ];

        for skill in skills {
            let content = builtin_skill_markdown(skill).unwrap_or("");
            let governed = apply_builtin_todowrite_governance(skill, "builtin", content);
            assert!(
                governed.contains("todo_write"),
                "expected {skill} governance output to mention todo_write"
            );
            assert!(
                governed.contains("更新 `todo_write` 状态"),
                "expected {skill} governance output to require todo_write status updates"
            );
        }
    }
}
