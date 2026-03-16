mod helpers;

use runtime_lib::approval_bus::{ApprovalDecision, ApprovalManager, ApprovalResolveResult};
use runtime_lib::approval_rules::{find_matching_approval_rule_with_pool, list_approval_rules_with_pool};
use runtime_lib::commands::approvals::list_pending_approvals_with_pool;
use runtime_lib::commands::session_runs::{append_session_run_event_with_pool, list_session_runs_with_pool};
use runtime_lib::session_journal::{SessionJournalStore, SessionRunEvent, SessionRunStatus};
use serde_json::json;

#[tokio::test]
async fn approval_records_persist_and_project_waiting_status() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    let journal_dir = tempfile::tempdir().expect("create journal dir");
    let journal = SessionJournalStore::new(journal_dir.path().to_path_buf());

    append_session_run_event_with_pool(
        &pool,
        &journal,
        "sess-approval",
        SessionRunEvent::RunStarted {
            run_id: "run-approval".into(),
            user_message_id: "user-approval".into(),
        },
    )
    .await
    .expect("append run started");

    append_session_run_event_with_pool(
        &pool,
        &journal,
        "sess-approval",
        SessionRunEvent::ApprovalRequested {
            run_id: "run-approval".into(),
            approval_id: "approval-1".into(),
            tool_name: "file_delete".into(),
            call_id: "call-1".into(),
            input: json!({ "path": "E:\\workspace\\danger.txt", "recursive": true }),
            summary: "将递归删除 E:\\workspace\\danger.txt".into(),
            impact: Some("该操作不可逆，删除后无法自动恢复。".into()),
            irreversible: true,
        },
    )
    .await
    .expect("append approval requested");

    let runs = list_session_runs_with_pool(&pool, "sess-approval")
        .await
        .expect("list session runs");
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].id, "run-approval");
    assert_eq!(runs[0].status, "waiting_approval");

    let journal_state = journal.read_state("sess-approval").await.expect("read journal state");
    assert_eq!(journal_state.current_run_id.as_deref(), Some("run-approval"));
    assert_eq!(journal_state.runs[0].status, SessionRunStatus::WaitingApproval);

    let (approval_status, approval_tool, approval_summary): (String, String, String) = sqlx::query_as(
        "SELECT status, tool_name, summary
         FROM approvals
         WHERE id = ?",
    )
    .bind("approval-1")
    .fetch_one(&pool)
    .await
    .expect("load approval row");
    assert_eq!(approval_status, "pending");
    assert_eq!(approval_tool, "file_delete");
    assert_eq!(approval_summary, "将递归删除 E:\\workspace\\danger.txt");

    let (event_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*)
         FROM session_run_events
         WHERE run_id = ? AND event_type = ?",
    )
    .bind("run-approval")
    .bind("approval_requested")
    .fetch_one(&pool)
    .await
    .expect("count approval events");
    assert_eq!(event_count, 1);

    let pending = list_pending_approvals_with_pool(&pool, Some("sess-approval"))
        .await
        .expect("list pending approvals");
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].approval_id, "approval-1");
    assert_eq!(pending[0].tool_name, "file_delete");
}

#[tokio::test]
async fn approval_manager_allows_first_resolver_only() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    let manager = ApprovalManager::default();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO approvals (
            id, session_id, run_id, call_id, tool_name, input_json, summary, impact,
            irreversible, status, decision, notify_targets_json, resume_payload_json,
            resolved_by_surface, resolved_by_user, resolved_at, resumed_at, expires_at,
            created_at, updated_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, NULL, NULL, ?, ?)",
    )
    .bind("approval-cas")
    .bind("sess-approval")
    .bind("run-approval")
    .bind("call-approval")
    .bind("file_delete")
    .bind("{}")
    .bind("删除危险目录")
    .bind("")
    .bind(1_i64)
    .bind("pending")
    .bind("")
    .bind("[]")
    .bind("{}")
    .bind("")
    .bind("")
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .expect("insert pending approval");

    let first = manager
        .resolve_with_pool(
            &pool,
            "approval-cas",
            ApprovalDecision::AllowOnce,
            "desktop",
            "user-desktop",
        )
        .await
        .expect("resolve approval first time");
    assert_eq!(
        first,
        ApprovalResolveResult::Applied {
            approval_id: "approval-cas".into(),
            status: "approved".into(),
            decision: ApprovalDecision::AllowOnce,
        }
    );

    let second = manager
        .resolve_with_pool(
            &pool,
            "approval-cas",
            ApprovalDecision::Deny,
            "feishu",
            "user-feishu",
        )
        .await
        .expect("resolve approval second time");
    assert_eq!(
        second,
        ApprovalResolveResult::AlreadyResolved {
            approval_id: "approval-cas".into(),
            status: "approved".into(),
            decision: Some(ApprovalDecision::AllowOnce),
        }
    );

    let (status, decision, surface, user_id): (String, String, String, String) = sqlx::query_as(
        "SELECT status, decision, resolved_by_surface, resolved_by_user
         FROM approvals
         WHERE id = ?",
    )
    .bind("approval-cas")
    .fetch_one(&pool)
    .await
    .expect("load resolved approval");
    assert_eq!(status, "approved");
    assert_eq!(decision, "allow_once");
    assert_eq!(surface, "desktop");
    assert_eq!(user_id, "user-desktop");
}

#[tokio::test]
async fn allow_always_creates_reusable_rule_and_skips_reapproval() {
    let (pool, _tmp) = helpers::setup_test_db().await;
    let manager = ApprovalManager::default();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO approvals (
            id, session_id, run_id, call_id, tool_name, input_json, summary, impact,
            irreversible, status, decision, notify_targets_json, resume_payload_json,
            resolved_by_surface, resolved_by_user, resolved_at, resumed_at, expires_at,
            created_at, updated_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, NULL, NULL, ?, ?)",
    )
    .bind("approval-rule-file-delete")
    .bind("sess-rule")
    .bind("run-rule")
    .bind("call-rule-file-delete")
    .bind("file_delete")
    .bind(r#"{"path":"E:\\workspace\\danger.txt","recursive":true}"#)
    .bind("删除危险目录")
    .bind("目录会被永久删除")
    .bind(1_i64)
    .bind("pending")
    .bind("")
    .bind("[]")
    .bind("{}")
    .bind("")
    .bind("")
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .expect("insert file_delete approval");

    sqlx::query(
        "INSERT INTO approvals (
            id, session_id, run_id, call_id, tool_name, input_json, summary, impact,
            irreversible, status, decision, notify_targets_json, resume_payload_json,
            resolved_by_surface, resolved_by_user, resolved_at, resumed_at, expires_at,
            created_at, updated_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, NULL, NULL, ?, ?)",
    )
    .bind("approval-rule-bash")
    .bind("sess-rule")
    .bind("run-rule")
    .bind("call-rule-bash")
    .bind("bash")
    .bind(r#"{"command":"Remove-Item -Recurse C:\\temp\\danger"}"#)
    .bind("执行危险 bash 删除命令")
    .bind("命令会递归删除目录")
    .bind(1_i64)
    .bind("pending")
    .bind("")
    .bind("[]")
    .bind("{}")
    .bind("")
    .bind("")
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .expect("insert bash approval");

    manager
        .resolve_with_pool(
            &pool,
            "approval-rule-file-delete",
            ApprovalDecision::AllowAlways,
            "desktop",
            "user-desktop",
        )
        .await
        .expect("resolve file_delete approval as allow_always");
    manager
        .resolve_with_pool(
            &pool,
            "approval-rule-bash",
            ApprovalDecision::AllowAlways,
            "feishu",
            "ou_approver",
        )
        .await
        .expect("resolve bash approval as allow_always");

    let rules = list_approval_rules_with_pool(&pool)
        .await
        .expect("list approval rules");
    assert_eq!(rules.len(), 2);

    let matched_delete = find_matching_approval_rule_with_pool(
        &pool,
        "file_delete",
        &json!({
            "path": "E:\\workspace\\danger.txt",
            "recursive": true
        }),
    )
    .await
    .expect("match file_delete rule");
    assert!(matched_delete.is_some());

    let unmatched_delete = find_matching_approval_rule_with_pool(
        &pool,
        "file_delete",
        &json!({
            "path": "E:\\workspace\\other.txt",
            "recursive": true
        }),
    )
    .await
    .expect("mismatch file_delete rule");
    assert!(unmatched_delete.is_none());

    let matched_bash = find_matching_approval_rule_with_pool(
        &pool,
        "bash",
        &json!({
            "command": "Remove-Item -Recurse C:\\temp\\danger"
        }),
    )
    .await
    .expect("match bash rule");
    assert!(matched_bash.is_some());

    let unmatched_bash = find_matching_approval_rule_with_pool(
        &pool,
        "bash",
        &json!({
            "command": "Remove-Item -Recurse C:\\temp\\other"
        }),
    )
    .await
    .expect("mismatch bash rule");
    assert!(unmatched_bash.is_none());
}
