mod helpers;

use runtime_lib::agent::group_orchestrator::{simulate_group_run, GroupRunRequest};
use runtime_lib::commands::im_config::{
    bind_thread_roles_with_pool, get_thread_role_config_with_pool,
};
use runtime_lib::commands::im_gateway::process_im_event;
use runtime_lib::im::feishu_formatter::format_role_message;
use runtime_lib::im::memory::{capture_entry, MemoryEntry};
use runtime_lib::im::types::{ImEvent, ImEventType};

#[test]
fn group_orchestrator_transitions_across_required_phases() {
    let outcome = simulate_group_run(GroupRunRequest {
        group_id: "group-1".to_string(),
        coordinator_employee_id: "project_manager".to_string(),
        member_employee_ids: vec![
            "project_manager".to_string(),
            "dev_team".to_string(),
            "qa_team".to_string(),
        ],
        user_goal: "做一个桌面端拉群协作功能".to_string(),
    });

    let phase_names = outcome
        .states
        .iter()
        .map(|s| s.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        phase_names,
        vec!["planning", "executing", "synthesizing", "done"]
    );

    assert!(!outcome.plan.is_empty(), "plan should not be empty");
    assert!(!outcome.execution.is_empty(), "execution should not be empty");
    assert!(outcome.final_report.contains("计划"));
    assert!(outcome.final_report.contains("执行"));
    assert!(outcome.final_report.contains("汇报"));
}

#[tokio::test]
async fn feishu_thread_multi_role_collaboration_e2e() {
    let (pool, tmp) = helpers::setup_test_db().await;

    bind_thread_roles_with_pool(
        &pool,
        "thread-e2e-1",
        "tenant-a",
        "opportunity_review",
        &[
            "presales".to_string(),
            "pm".to_string(),
            "architect".to_string(),
        ],
    )
    .await
    .expect("bind roles");

    let callback_result = process_im_event(
        &pool,
        ImEvent {
            event_type: ImEventType::MessageCreated,
            thread_id: "thread-e2e-1".to_string(),
            event_id: Some("evt-e2e-1".to_string()),
            message_id: Some("msg-e2e-1".to_string()),
            text: Some("请评估这个商机".to_string()),
            role_id: None,
            tenant_id: Some("tenant-a".to_string()),
        },
    )
    .await
    .expect("accept callback");
    assert!(callback_result.accepted);
    assert!(!callback_result.deduped);

    let cfg = get_thread_role_config_with_pool(&pool, "thread-e2e-1")
        .await
        .expect("load thread config");
    assert_eq!(cfg.roles.len(), 3);

    let memory_root = tmp.path().join("memory");
    let cap = capture_entry(
        &memory_root,
        "thread-e2e-1",
        "presales",
        &MemoryEntry {
            category: "decision".to_string(),
            content: "可承接，建议进入澄清会".to_string(),
            confirmed: true,
            source_msg_id: "msg-e2e-1".to_string(),
            author_role: "presales".to_string(),
            confidence: 0.88,
        },
    )
    .expect("capture memory");
    assert!(cap.long_term_written);

    let outbound = format_role_message(
        "建议承接",
        "历史上有同类交付经验",
        "接口文档仍需补齐",
        "发起技术澄清会议",
    );
    assert!(outbound.contains("结论"));
    assert!(outbound.contains("下一步"));
}
