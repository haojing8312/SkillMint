mod helpers;

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
}
