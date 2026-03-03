mod helpers;

use runtime_lib::commands::packaging::pack_industry_bundle;
use runtime_lib::commands::skills::{
    check_industry_bundle_update_from_pool, install_industry_bundle_to_pool,
};

fn create_skill(dir: &std::path::Path, name: &str, tags: &[&str]) {
    std::fs::create_dir_all(dir).expect("create skill dir");
    let tag_yaml = if tags.is_empty() {
        String::new()
    } else {
        format!(
            "tags:\n{}",
            tags.iter()
                .map(|t| format!("  - {}", t))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };
    std::fs::write(
        dir.join("SKILL.md"),
        format!(
            "---\nname: {}\ndescription: test\nversion: 1.0.0\n{}\n---\ncontent",
            name, tag_yaml
        ),
    )
    .expect("write SKILL.md");
}

#[tokio::test]
async fn install_industry_bundle_imports_all_skills() {
    let (pool, _tmp_db) = helpers::setup_test_db().await;
    let tmp = tempfile::tempdir().expect("create temp dir");

    let skills_root = tmp.path().join("skills");
    std::fs::create_dir_all(&skills_root).expect("create skills root");
    let skill_a = skills_root.join("teacher-helper");
    let skill_b = skills_root.join("auto-grader");
    create_skill(&skill_a, "Teacher Helper", &["教师"]);
    create_skill(&skill_b, "Auto Grader", &["教师", "作业"]);

    let bundle_path = tmp.path().join("edu.industrypack");
    pack_industry_bundle(
        vec![
            skill_a.to_string_lossy().to_string(),
            skill_b.to_string_lossy().to_string(),
        ],
        "教师行业包".to_string(),
        "edu-teacher-suite".to_string(),
        "1.0.0".to_string(),
        "教师".to_string(),
        bundle_path.to_string_lossy().to_string(),
    )
    .await
    .expect("pack bundle");

    let install_root = tmp.path().join("installed");
    let result = install_industry_bundle_to_pool(
        bundle_path.to_string_lossy().to_string(),
        Some(install_root.to_string_lossy().to_string()),
        &pool,
    )
    .await
    .expect("install industry bundle");

    assert_eq!(result.pack_id, "edu-teacher-suite");
    assert_eq!(result.version, "1.0.0");
    assert_eq!(result.installed_skills.len(), 2);
    assert!(result.missing_mcp.is_empty());

    let rows = sqlx::query_as::<_, (String,)>(
        "SELECT manifest FROM installed_skills WHERE source_type = 'local'",
    )
    .fetch_all(&pool)
    .await
    .expect("query installed skills");
    assert_eq!(rows.len(), 2);
    assert!(rows
        .iter()
        .any(|(json,)| json.contains("pack:edu-teacher-suite")));
    assert!(rows
        .iter()
        .any(|(json,)| json.contains("pack-version:1.0.0")));
}

#[tokio::test]
async fn check_industry_bundle_update_compares_with_installed_version() {
    let (pool, _tmp_db) = helpers::setup_test_db().await;
    let tmp = tempfile::tempdir().expect("create temp dir");

    let skill_dir = tmp.path().join("skills/teacher-helper");
    create_skill(&skill_dir, "Teacher Helper", &["教师"]);

    let old_bundle = tmp.path().join("edu-v1.industrypack");
    pack_industry_bundle(
        vec![skill_dir.to_string_lossy().to_string()],
        "教师行业包".to_string(),
        "edu-teacher-suite".to_string(),
        "1.0.0".to_string(),
        "教师".to_string(),
        old_bundle.to_string_lossy().to_string(),
    )
    .await
    .expect("pack old bundle");

    install_industry_bundle_to_pool(old_bundle.to_string_lossy().to_string(), None, &pool)
        .await
        .expect("install old bundle");

    let new_bundle = tmp.path().join("edu-v2.industrypack");
    pack_industry_bundle(
        vec![skill_dir.to_string_lossy().to_string()],
        "教师行业包".to_string(),
        "edu-teacher-suite".to_string(),
        "1.2.0".to_string(),
        "教师".to_string(),
        new_bundle.to_string_lossy().to_string(),
    )
    .await
    .expect("pack new bundle");

    let check =
        check_industry_bundle_update_from_pool(new_bundle.to_string_lossy().to_string(), &pool)
            .await
            .expect("check industry update");

    assert!(check.has_update);
    assert_eq!(check.pack_id, "edu-teacher-suite");
    assert_eq!(check.current_version.as_deref(), Some("1.0.0"));
    assert_eq!(check.candidate_version, "1.2.0");
}
