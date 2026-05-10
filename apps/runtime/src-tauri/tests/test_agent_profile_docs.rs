mod helpers;

use runtime_lib::commands::agent_profile::{
    apply_agent_profile_with_pool, export_agent_profile_with_pool,
    generate_agent_profile_draft_with_pool, get_agent_profile_files_with_pool,
    AgentProfileAnswerInput, AgentProfilePayload,
};
use runtime_lib::commands::employee_agents::{
    upsert_agent_employee_with_pool, UpsertAgentEmployeeInput,
};

#[tokio::test]
async fn apply_agent_profile_writes_canonical_instruction_files() {
    let (pool, tmp) = helpers::setup_test_db().await;
    let work_dir = tmp
        .path()
        .join("employee-workspace")
        .to_string_lossy()
        .to_string();

    let employee_db_id = upsert_agent_employee_with_pool(
        &pool,
        UpsertAgentEmployeeInput {
            id: None,
            employee_id: "project_manager".to_string(),
            name: "项目经理".to_string(),
            role_id: "project_manager".to_string(),
            persona: "".to_string(),
            feishu_open_id: "".to_string(),
            feishu_app_id: "".to_string(),
            feishu_app_secret: "".to_string(),
            primary_skill_id: "builtin-general".to_string(),
            default_work_dir: work_dir.clone(),
            openclaw_agent_id: "project_manager".to_string(),
            routing_priority: 100,
            enabled_scopes: vec!["feishu".to_string()],
            enabled: true,
            is_default: true,
            skill_ids: vec![],
        },
    )
    .await
    .expect("upsert employee");

    let payload = AgentProfilePayload {
        employee_db_id: employee_db_id.clone(),
        answers: vec![
            AgentProfileAnswerInput {
                key: "mission".to_string(),
                question: "该员工的核心使命是什么？".to_string(),
                answer: "推进需求到上线的高质量交付。".to_string(),
            },
            AgentProfileAnswerInput {
                key: "tone".to_string(),
                question: "沟通风格是什么？".to_string(),
                answer: "专业、直接、可执行。".to_string(),
            },
        ],
    };

    let draft = generate_agent_profile_draft_with_pool(&pool, payload.clone())
        .await
        .expect("generate draft");
    assert!(draft.agents_md.contains("# RULES"));
    assert!(draft.soul_md.contains("# PERSONA"));
    assert!(draft.user_md.contains("# USER_CONTEXT"));

    let result = apply_agent_profile_with_pool(&pool, payload)
        .await
        .expect("apply profile");
    assert_eq!(result.files.len(), 3);
    assert!(result.files.iter().all(|file| file.ok));

    let profile_root = std::path::PathBuf::from(&work_dir)
        .join("profiles")
        .join(&employee_db_id);
    let instructions_root = profile_root.join("instructions");
    let rules_path = instructions_root.join("RULES.md");
    let persona_path = instructions_root.join("PERSONA.md");
    let user_context_path = instructions_root.join("USER_CONTEXT.md");

    assert!(rules_path.exists(), "RULES.md should exist");
    assert!(persona_path.exists(), "PERSONA.md should exist");
    assert!(user_context_path.exists(), "USER_CONTEXT.md should exist");
    assert!(
        !std::path::PathBuf::from(&work_dir)
            .join("openclaw")
            .join("project_manager")
            .exists(),
        "new profile runtime must not create an OpenClaw mirror directory"
    );

    let rules_text = std::fs::read_to_string(&rules_path).expect("read RULES.md");
    let persona_text = std::fs::read_to_string(&persona_path).expect("read PERSONA.md");
    let user_context_text =
        std::fs::read_to_string(&user_context_path).expect("read USER_CONTEXT.md");

    assert!(rules_text.contains("项目经理"));
    assert!(rules_text.contains("推进需求到上线的高质量交付"));
    assert!(persona_text.contains("专业、直接、可执行"));
    assert!(user_context_text.contains("# USER_CONTEXT"));

    let (profile_home,): (String,) =
        sqlx::query_as("SELECT profile_home FROM agent_profiles WHERE id = ?")
            .bind(&employee_db_id)
            .fetch_one(&pool)
            .await
            .expect("load recorded profile home");
    assert_eq!(profile_home, profile_root.to_string_lossy());

    let view = get_agent_profile_files_with_pool(&pool, &employee_db_id)
        .await
        .expect("load profile files");
    let instructions = view
        .artifacts
        .iter()
        .find(|artifact| artifact.name == "instructions")
        .expect("instructions artifact");
    assert!(instructions.exists);
    assert_eq!(instructions.file_count, 3);
    assert!(view
        .artifacts
        .iter()
        .any(|artifact| artifact.name == "memories" && artifact.exists));
}

#[tokio::test]
async fn agent_profile_draft_uses_employee_enabled_scopes_in_rules_doc() {
    let (pool, tmp) = helpers::setup_test_db().await;
    let work_dir = tmp
        .path()
        .join("employee-workspace-wecom")
        .to_string_lossy()
        .to_string();

    let employee_db_id = upsert_agent_employee_with_pool(
        &pool,
        UpsertAgentEmployeeInput {
            id: None,
            employee_id: "wecom_operator".to_string(),
            name: "企业微信运营".to_string(),
            role_id: "wecom_operator".to_string(),
            persona: "".to_string(),
            feishu_open_id: "".to_string(),
            feishu_app_id: "".to_string(),
            feishu_app_secret: "".to_string(),
            primary_skill_id: "builtin-general".to_string(),
            default_work_dir: work_dir,
            openclaw_agent_id: "wecom_operator".to_string(),
            routing_priority: 100,
            enabled_scopes: vec!["app".to_string(), "wecom".to_string()],
            enabled: true,
            is_default: false,
            skill_ids: vec![],
        },
    )
    .await
    .expect("upsert employee");

    let draft = generate_agent_profile_draft_with_pool(
        &pool,
        AgentProfilePayload {
            employee_db_id,
            answers: vec![],
        },
    )
    .await
    .expect("generate draft");

    assert!(draft.agents_md.contains("适用范围: app, wecom"));
    assert!(!draft.agents_md.contains("飞书范围: feishu"));
}

#[tokio::test]
async fn export_agent_profile_writes_profile_home_artifact_zip() {
    let (pool, tmp) = helpers::setup_test_db().await;
    let work_dir = tmp
        .path()
        .join("employee-workspace-export")
        .to_string_lossy()
        .to_string();

    let employee_db_id = upsert_agent_employee_with_pool(
        &pool,
        UpsertAgentEmployeeInput {
            id: None,
            employee_id: "profile_exporter".to_string(),
            name: "Profile Exporter".to_string(),
            role_id: "profile_exporter".to_string(),
            persona: "".to_string(),
            feishu_open_id: "".to_string(),
            feishu_app_id: "".to_string(),
            feishu_app_secret: "".to_string(),
            primary_skill_id: "builtin-general".to_string(),
            default_work_dir: work_dir.clone(),
            openclaw_agent_id: "profile_exporter".to_string(),
            routing_priority: 100,
            enabled_scopes: vec!["app".to_string()],
            enabled: true,
            is_default: false,
            skill_ids: vec![],
        },
    )
    .await
    .expect("upsert employee");

    apply_agent_profile_with_pool(
        &pool,
        AgentProfilePayload {
            employee_db_id: employee_db_id.clone(),
            answers: vec![AgentProfileAnswerInput {
                key: "mission".to_string(),
                question: "使命".to_string(),
                answer: "导出完整 profile artifact。".to_string(),
            }],
        },
    )
    .await
    .expect("apply profile");

    let profile_root = std::path::PathBuf::from(&work_dir)
        .join("profiles")
        .join(&employee_db_id);
    std::fs::write(
        profile_root.join("memories").join("MEMORY.md"),
        "- profile export memory\n",
    )
    .expect("write memory");
    std::fs::create_dir_all(profile_root.join("curator").join("reports"))
        .expect("create curator reports");
    std::fs::write(
        profile_root
            .join("curator")
            .join("reports")
            .join("curator-run.json"),
        "{\"ok\":true}",
    )
    .expect("write curator report");

    let export_path = tmp.path().join("exports").join("profile-export.zip");
    let result = export_agent_profile_with_pool(
        &pool,
        &employee_db_id,
        export_path.to_string_lossy().as_ref(),
    )
    .await
    .expect("export profile");

    assert_eq!(result.profile_id, employee_db_id);
    assert!(result.file_count >= 5);
    assert!(export_path.exists());

    let file = std::fs::File::open(export_path).expect("open export zip");
    let mut archive = zip::ZipArchive::new(file).expect("read export zip");
    let mut names = Vec::new();
    for index in 0..archive.len() {
        names.push(
            archive
                .by_index(index)
                .expect("read zip entry")
                .name()
                .to_string(),
        );
    }
    assert!(names.iter().any(|name| name == "PROFILE_EXPORT.json"));
    assert!(names.iter().any(|name| name == "instructions/RULES.md"));
    assert!(names.iter().any(|name| name == "memories/MEMORY.md"));
    assert!(names
        .iter()
        .any(|name| name == "curator/reports/curator-run.json"));
}
