use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("repo root should resolve")
}

fn read_repo_file(relative: &str) -> String {
    fs::read_to_string(repo_root().join(relative))
        .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"))
}

#[test]
fn builtin_skill_creator_emphasizes_trigger_quality_and_evaluation() {
    let markdown = read_repo_file("apps/runtime/src-tauri/builtin-skills/skill-creator/SKILL.md");

    assert!(
        markdown.contains("复用")
            || markdown.contains("已有技能")
            || markdown.contains("是否真的需要新建"),
        "skill creator should guide create-vs-reuse decisions"
    );
    assert!(
        markdown.contains("触发示例") || markdown.contains("正向示例"),
        "skill creator should require trigger examples"
    );
    assert!(
        markdown.contains("不触发") || markdown.contains("反例") || markdown.contains("非触发"),
        "skill creator should require non-trigger examples"
    );
    assert!(
        markdown.contains("评测") || markdown.contains("误触发") || markdown.contains("漏触发"),
        "skill creator should mention lightweight evaluation"
    );
}

#[test]
fn builtin_skill_creator_guide_mentions_advanced_frontmatter_and_prompt_quality() {
    let markdown =
        read_repo_file("apps/runtime/src-tauri/builtin-skills/skill-creator-guide/SKILL.md");

    assert!(
        markdown.contains("allowed_tools")
            || markdown.contains("context")
            || markdown.contains("agent")
            || markdown.contains("mcp-servers"),
        "guide should mention optional advanced frontmatter supported by runtime"
    );
    assert!(
        markdown.contains("误触发")
            || markdown.contains("漏触发")
            || markdown.contains("Prompt Examples")
            || markdown.contains("non-trigger"),
        "guide should mention prompt-quality iteration"
    );
}

#[test]
fn local_skill_template_includes_non_trigger_and_prompt_example_sections() {
    let markdown = read_repo_file(
        "apps/runtime/src-tauri/builtin-skills/skill-creator-guide/templates/LOCAL_SKILL_TEMPLATE.md",
    );

    assert!(
        markdown.contains("## When Not to Use"),
        "local skill template should include a non-trigger section"
    );
    assert!(
        markdown.contains("## Prompt Examples"),
        "local skill template should include prompt examples"
    );
}

#[test]
fn runtime_registry_still_embeds_the_builtin_skill_creator_assets() {
    let rust = read_repo_file("apps/runtime/src-tauri/src/builtin_skills.rs");
    let core = read_repo_file("packages/runtime-skill-core/src/builtin_skills.rs");

    assert!(
        rust.contains("pub use runtime_skill_core"),
        "runtime registry should re-export builtin skill assets from runtime-skill-core"
    );
    assert!(
        core.contains("builtin-skills/skill-creator")
            && core.contains("builtin-skills/skill-creator-guide/templates"),
        "runtime-skill-core should embed the builtin skill markdown and local template assets"
    );
}

#[test]
fn builtin_office_skill_directories_include_real_runtime_assets() {
    let docx_root = repo_root().join("apps/runtime/src-tauri/builtin-skills/docx");
    let xlsx_root = repo_root().join("apps/runtime/src-tauri/builtin-skills/xlsx");
    let pdf_root = repo_root().join("apps/runtime/src-tauri/builtin-skills/pdf");
    let pptx_root = repo_root().join("apps/runtime/src-tauri/builtin-skills/pptx");

    assert!(docx_root.join("scripts").join("setup.ps1").exists());
    assert!(docx_root.join("assets").join("xsd").join("business-rules.xsd").exists());
    assert!(xlsx_root.join("scripts").join("xlsx_unpack.py").exists());
    assert!(xlsx_root.join("templates").join("minimal_xlsx").exists());
    assert!(pdf_root.join("scripts").join("render_cover.js").exists());
    assert!(pdf_root.join("design").join("design.md").exists());
    assert!(pptx_root.join("skills").join("ppt-editing-skill").join("SKILL.md").exists());
    assert!(pptx_root.join("agents").join("content-page-generator.md").exists());
}
