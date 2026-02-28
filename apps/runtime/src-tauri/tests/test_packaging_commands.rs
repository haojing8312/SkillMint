use runtime_lib::commands::packaging::{pack_skill, read_skill_dir};

#[tokio::test]
async fn read_skill_dir_requires_skill_md() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let err = read_skill_dir(tmp.path().to_string_lossy().to_string())
        .await
        .expect_err("should fail without SKILL.md");
    assert!(err.contains("SKILL.md"));
}

#[tokio::test]
async fn read_skill_dir_returns_frontmatter_and_files() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let skill_md = tmp.path().join("SKILL.md");
    std::fs::write(
        &skill_md,
        "---\nname: test-skill\ndescription: desc\nversion: 1.0.0\n---\ncontent",
    )
    .expect("write SKILL.md");
    std::fs::write(tmp.path().join("notes.md"), "hello").expect("write notes");

    let info = read_skill_dir(tmp.path().to_string_lossy().to_string())
        .await
        .expect("read skill dir");
    assert!(info.files.iter().any(|f| f == "SKILL.md"));
    assert!(info.files.iter().any(|f| f == "notes.md"));
    assert_eq!(info.front_matter.name.as_deref(), Some("test-skill"));
}

#[tokio::test]
async fn pack_skill_creates_skillpack() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("SKILL.md"),
        "---\nname: test-skill\ndescription: desc\nversion: 1.0.0\n---\nDo work",
    )
    .expect("write SKILL.md");
    std::fs::write(tmp.path().join("extra.txt"), "file").expect("write file");

    let output = tmp.path().join("out.skillpack");
    pack_skill(
        tmp.path().to_string_lossy().to_string(),
        "test-skill".to_string(),
        "desc".to_string(),
        "1.0.0".to_string(),
        "author".to_string(),
        "alice".to_string(),
        "gpt-4o".to_string(),
        output.to_string_lossy().to_string(),
    )
    .await
    .expect("pack succeeds");

    assert!(output.exists());
}
