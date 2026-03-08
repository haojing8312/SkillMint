mod helpers;

use std::path::PathBuf;

#[tokio::test]
async fn builtin_team_template_loads_default_sansheng_liubu() {
    let template = runtime_lib::team_templates::load_builtin_template("sansheng-liubu")
        .expect("template should load");

    assert_eq!(template.template_id, "sansheng-liubu");
    assert!(template.seed_on_first_run);
    assert!(template
        .roles
        .iter()
        .any(|role| role.role_type == "reviewer" && role.employee_key == "menxia"));
    assert!(template
        .employees
        .iter()
        .any(|employee| employee.employee_id == "taizi"));
    assert!(template
        .rules
        .iter()
        .any(|rule| rule.relation_type == "review"));
}

#[tokio::test]
async fn first_run_bootstrap_seeds_default_team_once() {
    let (pool, tmp) = helpers::setup_test_db().await;

    runtime_lib::team_templates::seed_builtin_team_templates_with_root(&pool, tmp.path())
        .await
        .expect("seed builtin templates");
    runtime_lib::team_templates::seed_builtin_team_templates_with_root(&pool, tmp.path())
        .await
        .expect("seed builtin templates twice");

    let (employee_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM agent_employees")
        .fetch_one(&pool)
        .await
        .expect("count seeded employees");
    assert!(employee_count >= 10, "expected seeded employee count >= 10");

    let (skill_binding_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM agent_employee_skills")
            .fetch_one(&pool)
            .await
            .expect("count seeded employee skills");
    assert!(
        skill_binding_count >= 10,
        "expected seeded skill binding count >= 10"
    );

    let (group_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM employee_groups WHERE template_id = 'sansheng-liubu'")
            .fetch_one(&pool)
            .await
            .expect("count seeded groups");
    assert_eq!(group_count, 1);

    let (rule_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM employee_group_rules WHERE group_id IN (
            SELECT id FROM employee_groups WHERE template_id = 'sansheng-liubu'
        )",
    )
    .fetch_one(&pool)
    .await
    .expect("count seeded rules");
    assert!(rule_count > 0, "expected seeded rules");

    let (seed_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM seeded_team_templates WHERE template_id = 'sansheng-liubu'",
    )
    .fetch_one(&pool)
    .await
    .expect("count seed records");
    assert_eq!(seed_count, 1);

    let (taizi_work_dir,): (String,) =
        sqlx::query_as("SELECT default_work_dir FROM agent_employees WHERE employee_id = 'taizi'")
            .fetch_one(&pool)
            .await
            .expect("load taizi work dir");
    let profile_dir = PathBuf::from(taizi_work_dir).join("openclaw").join("taizi");
    assert!(profile_dir.join("AGENTS.md").exists());
    assert!(profile_dir.join("SOUL.md").exists());
    assert!(profile_dir.join("USER.md").exists());
}
