use include_dir::{include_dir, Dir};
use std::collections::HashMap;
use std::sync::LazyLock;

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

static BUILTIN_GENERAL_SKILL_DIR: Dir<'_> = include_dir!(
    "$CARGO_MANIFEST_DIR/../../apps/runtime/src-tauri/builtin-skills/general-assistant"
);
static BUILTIN_SKILL_CREATOR_DIR: Dir<'_> =
    include_dir!("$CARGO_MANIFEST_DIR/../../apps/runtime/src-tauri/builtin-skills/skill-creator");
static BUILTIN_DOCX_SKILL_DIR: Dir<'_> =
    include_dir!("$CARGO_MANIFEST_DIR/../../apps/runtime/src-tauri/builtin-skills/docx");
static BUILTIN_PDF_SKILL_DIR: Dir<'_> =
    include_dir!("$CARGO_MANIFEST_DIR/../../apps/runtime/src-tauri/builtin-skills/pdf");
static BUILTIN_PPTX_SKILL_DIR: Dir<'_> =
    include_dir!("$CARGO_MANIFEST_DIR/../../apps/runtime/src-tauri/builtin-skills/pptx");
static BUILTIN_XLSX_SKILL_DIR: Dir<'_> =
    include_dir!("$CARGO_MANIFEST_DIR/../../apps/runtime/src-tauri/builtin-skills/xlsx");
static BUILTIN_FIND_SKILLS_DIR: Dir<'_> =
    include_dir!("$CARGO_MANIFEST_DIR/../../apps/runtime/src-tauri/builtin-skills/find-skills");
static BUILTIN_EMPLOYEE_CREATOR_SKILL_DIR: Dir<'_> = include_dir!(
    "$CARGO_MANIFEST_DIR/../../apps/runtime/src-tauri/builtin-skills/employee-creator"
);
static LOCAL_SKILL_TEMPLATE_DIR: Dir<'_> = include_dir!(
    "$CARGO_MANIFEST_DIR/../../apps/runtime/src-tauri/builtin-skills/skill-creator-guide/templates"
);

fn skill_markdown_from_dir(dir: &'static Dir<'static>) -> &'static str {
    dir.get_file("SKILL.md")
        .and_then(|file| file.contents_utf8())
        .expect("builtin skill directory must contain SKILL.md")
}

fn local_template_from_dir(dir: &'static Dir<'static>) -> &'static str {
    dir.get_file("LOCAL_SKILL_TEMPLATE.md")
        .and_then(|file| file.contents_utf8())
        .expect("builtin local skill template must exist")
}

pub struct BuiltinSkillEntry {
    pub id: &'static str,
    pub markdown: &'static str,
    pub files: &'static Dir<'static>,
}

static BUILTIN_SKILL_ENTRIES: LazyLock<Vec<BuiltinSkillEntry>> = LazyLock::new(|| {
    vec![
        BuiltinSkillEntry {
            id: BUILTIN_GENERAL_SKILL_ID,
            markdown: skill_markdown_from_dir(&BUILTIN_GENERAL_SKILL_DIR),
            files: &BUILTIN_GENERAL_SKILL_DIR,
        },
        BuiltinSkillEntry {
            id: BUILTIN_SKILL_CREATOR_ID,
            markdown: skill_markdown_from_dir(&BUILTIN_SKILL_CREATOR_DIR),
            files: &BUILTIN_SKILL_CREATOR_DIR,
        },
        BuiltinSkillEntry {
            id: BUILTIN_DOCX_SKILL_ID,
            markdown: skill_markdown_from_dir(&BUILTIN_DOCX_SKILL_DIR),
            files: &BUILTIN_DOCX_SKILL_DIR,
        },
        BuiltinSkillEntry {
            id: BUILTIN_PDF_SKILL_ID,
            markdown: skill_markdown_from_dir(&BUILTIN_PDF_SKILL_DIR),
            files: &BUILTIN_PDF_SKILL_DIR,
        },
        BuiltinSkillEntry {
            id: BUILTIN_PPTX_SKILL_ID,
            markdown: skill_markdown_from_dir(&BUILTIN_PPTX_SKILL_DIR),
            files: &BUILTIN_PPTX_SKILL_DIR,
        },
        BuiltinSkillEntry {
            id: BUILTIN_XLSX_SKILL_ID,
            markdown: skill_markdown_from_dir(&BUILTIN_XLSX_SKILL_DIR),
            files: &BUILTIN_XLSX_SKILL_DIR,
        },
        BuiltinSkillEntry {
            id: BUILTIN_FIND_SKILLS_ID,
            markdown: skill_markdown_from_dir(&BUILTIN_FIND_SKILLS_DIR),
            files: &BUILTIN_FIND_SKILLS_DIR,
        },
        BuiltinSkillEntry {
            id: BUILTIN_EMPLOYEE_CREATOR_ID,
            markdown: skill_markdown_from_dir(&BUILTIN_EMPLOYEE_CREATOR_SKILL_DIR),
            files: &BUILTIN_EMPLOYEE_CREATOR_SKILL_DIR,
        },
    ]
});

fn collect_dir_files(dir: &'static Dir<'static>, out: &mut HashMap<String, Vec<u8>>) {
    for file in dir.files() {
        let key = file.path().to_string_lossy().replace('\\', "/");
        out.insert(key, file.contents().to_vec());
    }

    for child in dir.dirs() {
        collect_dir_files(child, out);
    }
}

pub fn builtin_skill_markdown(skill_id: &str) -> Option<&'static str> {
    builtin_skill_entries()
        .iter()
        .find(|entry| entry.id == skill_id)
        .map(|entry| entry.markdown)
}

pub fn builtin_skill_files(skill_id: &str) -> Option<HashMap<String, Vec<u8>>> {
    let entry = builtin_skill_entries()
        .iter()
        .find(|entry| entry.id == skill_id)?;
    let mut files = HashMap::new();
    collect_dir_files(entry.files, &mut files);
    Some(files)
}

pub fn builtin_skill_entries() -> &'static [BuiltinSkillEntry] {
    BUILTIN_SKILL_ENTRIES.as_slice()
}

pub fn builtin_general_skill_markdown() -> &'static str {
    skill_markdown_from_dir(&BUILTIN_GENERAL_SKILL_DIR)
}

pub fn local_skill_template_markdown() -> &'static str {
    local_template_from_dir(&LOCAL_SKILL_TEMPLATE_DIR)
}

pub fn is_multistep_builtin_skill(skill_id: &str, source_type: &str) -> bool {
    if !matches!(source_type, "builtin" | "vendored") {
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
        apply_builtin_todowrite_governance, builtin_skill_files, builtin_skill_markdown,
        is_multistep_builtin_skill, BUILTIN_DOCX_SKILL_ID, BUILTIN_EMPLOYEE_CREATOR_ID,
        BUILTIN_FIND_SKILLS_ID, BUILTIN_GENERAL_SKILL_ID, BUILTIN_MULTISTEP_TODOWRITE_GOVERNANCE,
        BUILTIN_PDF_SKILL_ID, BUILTIN_PPTX_SKILL_ID, BUILTIN_XLSX_SKILL_ID,
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
            BUILTIN_DOCX_SKILL_ID,
            BUILTIN_EMPLOYEE_CREATOR_ID,
            BUILTIN_FIND_SKILLS_ID,
            BUILTIN_GENERAL_SKILL_ID,
            BUILTIN_PDF_SKILL_ID,
            BUILTIN_PPTX_SKILL_ID,
            "builtin-skill-creator",
            BUILTIN_XLSX_SKILL_ID,
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

    #[test]
    fn builtin_office_skills_expose_full_file_trees() {
        for skill in [
            BUILTIN_DOCX_SKILL_ID,
            BUILTIN_PDF_SKILL_ID,
            BUILTIN_PPTX_SKILL_ID,
            BUILTIN_XLSX_SKILL_ID,
        ] {
            let files = builtin_skill_files(skill).expect("builtin skill files should exist");
            assert!(
                files.contains_key("SKILL.md"),
                "expected {skill} to include SKILL.md"
            );
            assert!(
                files.len() > 1,
                "expected {skill} to expose more than markdown-only builtin assets"
            );
        }
    }
}
