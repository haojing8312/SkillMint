use runtime_lib::agent::runtime::runtime_io::{
    build_profile_memory_locator, ensure_profile_session_index_schema_with_pool,
    index_profile_session_manifest_with_pool, load_profile_memory_bundle,
    load_profile_memory_bundle_with_budget, refresh_profile_session_index_for_session_with_pool,
    search_profile_session_index_with_filters_with_pool, search_profile_session_index_with_pool,
    write_profile_session_manifest, ProfileSessionManifestInput, ProfileSessionSearchFilters,
};
use std::path::Path;

fn rewrite_manifest_updated_at(manifest_path: &Path, updated_at: &str) {
    let mut manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(manifest_path).expect("read manifest"))
            .expect("parse manifest");
    manifest["updated_at"] = serde_json::Value::String(updated_at.to_string());
    std::fs::write(
        manifest_path,
        serde_json::to_string_pretty(&manifest).expect("serialize manifest"),
    )
    .expect("rewrite manifest");
}

fn write_profile_session_manifest_with_updated_at(
    runtime_root: &Path,
    session_id: &str,
    skill_id: &str,
    work_dir: &str,
    source: &str,
    updated_at: &str,
) -> std::path::PathBuf {
    let manifest_path = write_profile_session_manifest(
        runtime_root,
        ProfileSessionManifestInput {
            profile_id: "profile-1",
            session_id,
            skill_id,
            work_dir: Some(work_dir),
            source,
        },
    )
    .expect("write manifest");
    rewrite_manifest_updated_at(&manifest_path, updated_at);
    manifest_path
}

#[test]
fn profile_memory_bundle_includes_workspace_project_memory() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let runtime_root = tmp.path().join("runtime-root");
    let memory_root = runtime_root.join("memory");
    let profile_memory_dir = runtime_root
        .join("profiles")
        .join("profile-1")
        .join("memories");
    std::fs::create_dir_all(&profile_memory_dir).expect("profile memory dir");
    std::fs::write(profile_memory_dir.join("MEMORY.md"), "profile fact")
        .expect("write profile memory");

    let locator = build_profile_memory_locator(
        &runtime_root,
        &memory_root,
        Some(Path::new("E:/workspace/acme")),
        "builtin-general",
        "planner",
        Some("profile-1"),
        None,
    );
    let project_memory_file = locator
        .project_memory_file
        .as_ref()
        .expect("project memory file");
    std::fs::create_dir_all(project_memory_file.parent().expect("project dir"))
        .expect("create project dir");
    std::fs::write(project_memory_file, "project fact").expect("write project memory");

    let bundle = load_profile_memory_bundle(&locator);

    assert_eq!(bundle.source, "profile");
    assert!(bundle.content.contains("profile fact"));
    assert!(bundle.content.contains("Project Memory"));
    assert!(bundle.content.contains("project fact"));
}

#[test]
fn profile_memory_bundle_trims_to_budget() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let runtime_root = tmp.path().join("runtime-root");
    let memory_root = runtime_root.join("memory");
    let profile_memory_dir = runtime_root
        .join("profiles")
        .join("profile-1")
        .join("memories");
    std::fs::create_dir_all(&profile_memory_dir).expect("profile memory dir");
    std::fs::write(
        profile_memory_dir.join("MEMORY.md"),
        format!(
            "ancient-start {}\n{}",
            "old ".repeat(200),
            "fresh memory tail"
        ),
    )
    .expect("write profile memory");

    let locator = build_profile_memory_locator(
        &runtime_root,
        &memory_root,
        None,
        "builtin-general",
        "planner",
        Some("profile-1"),
        None,
    );
    let bundle = load_profile_memory_bundle_with_budget(&locator, 80);

    assert!(bundle.content.contains("fresh memory tail"));
    assert!(!bundle.content.contains("ancient-start"));
    assert!(bundle.content.chars().count() <= 140);
}

#[test]
fn profile_session_manifest_is_written_under_profile_home() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let runtime_root = tmp.path().join("runtime-root");

    let manifest_path = write_profile_session_manifest(
        &runtime_root,
        ProfileSessionManifestInput {
            profile_id: "profile-1",
            session_id: "session-1",
            skill_id: "builtin-general",
            work_dir: Some("E:/workspace/acme"),
            source: "runtime_tool_setup",
        },
    )
    .expect("write manifest");

    assert_eq!(
        manifest_path,
        runtime_root
            .join("profiles")
            .join("profile-1")
            .join("sessions")
            .join("session-1")
            .join("manifest.json")
    );

    let raw = std::fs::read_to_string(&manifest_path).expect("read manifest");
    let manifest: serde_json::Value = serde_json::from_str(&raw).expect("parse manifest");

    assert_eq!(manifest["version"], 1);
    assert_eq!(manifest["profile_id"], "profile-1");
    assert_eq!(manifest["session_id"], "session-1");
    assert_eq!(manifest["skill_id"], "builtin-general");
    assert_eq!(manifest["work_dir"], "E:/workspace/acme");
    assert_eq!(manifest["source"], "runtime_tool_setup");
    let expected_journal_dir = runtime_root.join("sessions").join("session-1");
    let expected_state_path = expected_journal_dir.join("state.json");
    let expected_transcript_path = expected_journal_dir.join("transcript.md");
    assert_eq!(
        manifest["journal_dir"].as_str(),
        Some(expected_journal_dir.to_string_lossy().as_ref())
    );
    assert_eq!(
        manifest["state_path"].as_str(),
        Some(expected_state_path.to_string_lossy().as_ref())
    );
    assert_eq!(
        manifest["transcript_path"].as_str(),
        Some(expected_transcript_path.to_string_lossy().as_ref())
    );
    assert_eq!(manifest["run_summary"]["status"], "pending");
    assert_eq!(manifest["run_summary"]["latest_run_id"], "");
    assert!(manifest["updated_at"]
        .as_str()
        .expect("updated_at")
        .contains('T'));
}

#[test]
fn profile_session_manifest_reads_existing_journal_state_summary() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let runtime_root = tmp.path().join("runtime-root");
    let journal_dir = runtime_root.join("sessions").join("session-2");
    std::fs::create_dir_all(&journal_dir).expect("create journal dir");
    std::fs::write(
        journal_dir.join("state.json"),
        serde_json::json!({
            "session_id": "session-2",
            "current_run_id": "run-2",
            "runs": [
                {
                    "run_id": "run-1",
                    "user_message_id": "msg-1",
                    "status": "failed",
                    "buffered_text": "旧失败结果",
                    "last_error_kind": "tool_error",
                    "last_error_message": "failed"
                },
                {
                    "run_id": "run-2",
                    "user_message_id": "msg-2",
                    "status": "completed",
                    "buffered_text": "最终完成摘要，包含后续可检索内容",
                    "last_error_kind": null,
                    "last_error_message": null
                }
            ]
        })
        .to_string(),
    )
    .expect("write state");

    let manifest_path = write_profile_session_manifest(
        &runtime_root,
        ProfileSessionManifestInput {
            profile_id: "profile-1",
            session_id: "session-2",
            skill_id: "builtin-general",
            work_dir: Some("E:/workspace/acme"),
            source: "runtime_tool_setup",
        },
    )
    .expect("write manifest");

    let raw = std::fs::read_to_string(&manifest_path).expect("read manifest");
    let manifest: serde_json::Value = serde_json::from_str(&raw).expect("parse manifest");

    assert_eq!(manifest["run_summary"]["status"], "completed");
    assert_eq!(manifest["run_summary"]["latest_run_id"], "run-2");
    assert_eq!(manifest["run_summary"]["user_message_id"], "msg-2");
    assert!(manifest["run_summary"]["buffered_text_preview"]
        .as_str()
        .expect("preview")
        .contains("后续可检索内容"));
}

#[test]
fn profile_session_manifest_indexes_tool_call_summaries_from_events() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let runtime_root = tmp.path().join("runtime-root");
    let journal_dir = runtime_root.join("sessions").join("session-tools");
    std::fs::create_dir_all(&journal_dir).expect("create journal dir");
    let started = serde_json::json!({
        "session_id": "session-tools",
        "recorded_at": "2026-05-07T00:00:00Z",
        "event": {
            "type": "tool_started",
            "run_id": "run-tools",
            "tool_name": "web_search",
            "call_id": "call-1",
            "input": { "query": "WorkClaw Hermes memory" }
        }
    });
    let completed = serde_json::json!({
        "session_id": "session-tools",
        "recorded_at": "2026-05-07T00:00:01Z",
        "event": {
            "type": "tool_completed",
            "run_id": "run-tools",
            "tool_name": "web_search",
            "call_id": "call-1",
            "input": { "query": "WorkClaw Hermes memory" },
            "output": "Found Profile Memory and session index references.",
            "is_error": false
        }
    });
    std::fs::write(
        journal_dir.join("events.jsonl"),
        format!("{started}\n{completed}\n"),
    )
    .expect("write events");

    let manifest_path = write_profile_session_manifest(
        &runtime_root,
        ProfileSessionManifestInput {
            profile_id: "profile-1",
            session_id: "session-tools",
            skill_id: "builtin-general",
            work_dir: Some("E:/workspace/acme"),
            source: "runtime_tool_setup",
        },
    )
    .expect("write manifest");

    let raw = std::fs::read_to_string(&manifest_path).expect("read manifest");
    let manifest: serde_json::Value = serde_json::from_str(&raw).expect("parse manifest");
    let summaries = manifest["tool_summaries"]
        .as_array()
        .expect("tool summaries");

    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0]["run_id"], "run-tools");
    assert_eq!(summaries[0]["tool_name"], "web_search");
    assert_eq!(summaries[0]["call_id"], "call-1");
    assert_eq!(summaries[0]["status"], "completed");
    assert_eq!(summaries[0]["is_error"], false);
    assert!(summaries[0]["input_preview"]
        .as_str()
        .expect("input preview")
        .contains("Hermes memory"));
    assert!(summaries[0]["output_preview"]
        .as_str()
        .expect("output preview")
        .contains("session index"));
}

#[test]
fn profile_session_manifest_indexes_compaction_boundaries_from_state() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let runtime_root = tmp.path().join("runtime-root");
    let journal_dir = runtime_root.join("sessions").join("session-compact");
    std::fs::create_dir_all(&journal_dir).expect("create journal dir");
    std::fs::write(
        journal_dir.join("state.json"),
        serde_json::json!({
            "session_id": "session-compact",
            "current_run_id": "run-compact",
            "runs": [
                {
                    "run_id": "run-compact",
                    "user_message_id": "msg-compact",
                    "status": "completed",
                    "buffered_text": "压缩后的最终回答",
                    "last_error_kind": null,
                    "last_error_message": null,
                    "turn_state": {
                        "compaction_boundary": {
                            "transcript_path": "transcripts/session-compact.jsonl",
                            "original_tokens": 4096,
                            "compacted_tokens": 1024,
                            "summary": "保留了需求、关键约束和工具结果。"
                        }
                    }
                }
            ]
        })
        .to_string(),
    )
    .expect("write state");

    let manifest_path = write_profile_session_manifest(
        &runtime_root,
        ProfileSessionManifestInput {
            profile_id: "profile-1",
            session_id: "session-compact",
            skill_id: "builtin-general",
            work_dir: Some("E:/workspace/acme"),
            source: "runtime_tool_setup",
        },
    )
    .expect("write manifest");

    let raw = std::fs::read_to_string(&manifest_path).expect("read manifest");
    let manifest: serde_json::Value = serde_json::from_str(&raw).expect("parse manifest");
    let boundaries = manifest["compaction_boundaries"]
        .as_array()
        .expect("compaction boundaries");

    assert_eq!(boundaries.len(), 1);
    assert_eq!(boundaries[0]["run_id"], "run-compact");
    assert_eq!(
        boundaries[0]["transcript_path"],
        "transcripts/session-compact.jsonl"
    );
    assert_eq!(boundaries[0]["original_tokens"], 4096);
    assert_eq!(boundaries[0]["compacted_tokens"], 1024);
    assert!(boundaries[0]["summary"]
        .as_str()
        .expect("summary")
        .contains("关键约束"));
}

#[tokio::test]
async fn profile_session_index_searches_manifest_tool_and_compaction_text() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let runtime_root = tmp.path().join("runtime-root");
    let journal_dir = runtime_root.join("sessions").join("session-search");
    std::fs::create_dir_all(&journal_dir).expect("create journal dir");
    std::fs::write(
        journal_dir.join("state.json"),
        serde_json::json!({
            "session_id": "session-search",
            "current_run_id": "run-search",
            "runs": [
                {
                    "run_id": "run-search",
                    "user_message_id": "msg-search",
                    "status": "completed",
                    "buffered_text": "完成 Hermes 对齐分析",
                    "last_error_kind": null,
                    "last_error_message": null,
                    "turn_state": {
                        "compaction_boundary": {
                            "transcript_path": "transcripts/session-search.jsonl",
                            "original_tokens": 5000,
                            "compacted_tokens": 1200,
                            "summary": "保留了 Profile Home、关键约束和后续迭代方向。"
                        }
                    }
                }
            ]
        })
        .to_string(),
    )
    .expect("write state");
    std::fs::write(
        journal_dir.join("events.jsonl"),
        format!(
            "{}\n",
            serde_json::json!({
                "session_id": "session-search",
                "recorded_at": "2026-05-07T00:00:01Z",
                "event": {
                    "type": "tool_completed",
                    "run_id": "run-search",
                    "tool_name": "memory",
                    "call_id": "call-memory",
                    "input": { "action": "add", "text": "Hermes memory index" },
                    "output": "Profile session index ready for recall.",
                    "is_error": false
                }
            })
        ),
    )
    .expect("write events");

    let manifest_path = write_profile_session_manifest(
        &runtime_root,
        ProfileSessionManifestInput {
            profile_id: "profile-1",
            session_id: "session-search",
            skill_id: "builtin-general",
            work_dir: Some("E:/workspace/acme"),
            source: "runtime_tool_setup",
        },
    )
    .expect("write manifest");
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("create sqlite memory pool");

    ensure_profile_session_index_schema_with_pool(&pool)
        .await
        .expect("create profile session index schema");
    index_profile_session_manifest_with_pool(&pool, &manifest_path)
        .await
        .expect("index manifest");

    let results = search_profile_session_index_with_pool(&pool, "profile-1", "关键约束", 10)
        .await
        .expect("search profile sessions");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].profile_id, "profile-1");
    assert_eq!(results[0].session_id, "session-search");
    assert_eq!(results[0].skill_id, "builtin-general");
    assert_eq!(results[0].run_status, "completed");
    assert_eq!(results[0].tool_summary_count, 1);
    assert_eq!(results[0].compaction_boundary_count, 1);
    assert!(results[0].snippet.contains("关键约束"));
}

#[tokio::test]
async fn profile_session_index_searches_db_user_and_assistant_messages() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let runtime_root = tmp.path().join("runtime-root");
    let manifest_path = write_profile_session_manifest(
        &runtime_root,
        ProfileSessionManifestInput {
            profile_id: "profile-1",
            session_id: "session-db-messages",
            skill_id: "builtin-general",
            work_dir: Some("E:/workspace/acme"),
            source: "runtime_tool_setup",
        },
    )
    .expect("write manifest");
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("create sqlite memory pool");
    sqlx::query(
        "CREATE TABLE messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            content_json TEXT,
            created_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("create messages");
    sqlx::query(
        "INSERT INTO messages (id, session_id, role, content, content_json, created_at)
         VALUES
         ('msg-user', 'session-db-messages', 'user', '用户问题：如何把历史经验召回？', NULL, '2026-05-07T00:00:00Z'),
         ('msg-assistant', 'session-db-messages', 'assistant', ?, NULL, '2026-05-07T00:00:01Z')",
    )
    .bind(
        serde_json::json!({
            "text": "最终方案：让 memory.search 检索 DB 消息与工具摘要。",
            "reasoning": "hidden"
        })
        .to_string(),
    )
    .execute(&pool)
    .await
    .expect("seed messages");

    ensure_profile_session_index_schema_with_pool(&pool)
        .await
        .expect("create profile session index schema");
    index_profile_session_manifest_with_pool(&pool, &manifest_path)
        .await
        .expect("index manifest and db messages");

    let results = search_profile_session_index_with_pool(&pool, "profile-1", "最终方案", 10)
        .await
        .expect("search profile sessions");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].session_id, "session-db-messages");
    assert!(results[0].snippet.contains("最终方案"));
    assert!(results[0].snippet.contains("历史经验召回"));
}

#[tokio::test]
async fn profile_session_index_writes_profile_transcript_mirror() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let runtime_root = tmp.path().join("runtime-root");
    let journal_dir = runtime_root
        .join("sessions")
        .join("session-transcript-mirror");
    std::fs::create_dir_all(&journal_dir).expect("journal dir");
    std::fs::write(
        journal_dir.join("state.json"),
        serde_json::json!({
            "session_id": "session-transcript-mirror",
            "current_run_id": "run-transcript",
            "runs": [{
                "run_id": "run-transcript",
                "user_message_id": "msg-user",
                "status": "completed",
                "buffered_text": "最终输出 preview",
                "turn_state": {
                    "compaction_boundary": {
                        "transcript_path": "transcripts/session-transcript-mirror.jsonl",
                        "original_tokens": 3000,
                        "compacted_tokens": 900,
                        "summary": "压缩摘要：保留关键事实。"
                    }
                }
            }]
        })
        .to_string(),
    )
    .expect("write state");
    std::fs::write(
        journal_dir.join("events.jsonl"),
        format!(
            "{}\n",
            serde_json::json!({
                "session_id": "session-transcript-mirror",
                "recorded_at": "2026-05-08T00:00:02Z",
                "event": {
                    "type": "tool_completed",
                    "run_id": "run-transcript",
                    "tool_name": "memory",
                    "call_id": "call-transcript",
                    "input": { "action": "search", "query": "经验" },
                    "output": "工具摘要：召回历史经验。",
                    "is_error": false
                }
            })
        ),
    )
    .expect("write events");
    let manifest_path = write_profile_session_manifest(
        &runtime_root,
        ProfileSessionManifestInput {
            profile_id: "profile-1",
            session_id: "session-transcript-mirror",
            skill_id: "builtin-general",
            work_dir: Some("E:/workspace/acme"),
            source: "runtime_tool_setup",
        },
    )
    .expect("write manifest");
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("create sqlite memory pool");
    sqlx::query(
        "CREATE TABLE messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            content_json TEXT,
            created_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("create messages");
    sqlx::query(
        "INSERT INTO messages (id, session_id, role, content, content_json, created_at)
         VALUES
         ('msg-user', 'session-transcript-mirror', 'user', '用户问题：生成 profile transcript mirror', NULL, '2026-05-08T00:00:00Z'),
         ('msg-assistant', 'session-transcript-mirror', 'assistant', '助手回答：transcript mirror 已生成。', NULL, '2026-05-08T00:00:01Z')",
    )
    .execute(&pool)
    .await
    .expect("seed messages");
    sqlx::query(
        "CREATE TABLE session_runs (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            user_message_id TEXT NOT NULL DEFAULT '',
            assistant_message_id TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'queued',
            buffered_text TEXT NOT NULL DEFAULT '',
            error_kind TEXT NOT NULL DEFAULT '',
            error_message TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("create session runs");
    sqlx::query(
        "INSERT INTO session_runs (id, session_id, user_message_id, assistant_message_id, status, buffered_text, error_kind, error_message, created_at, updated_at)
         VALUES ('run-transcript', 'session-transcript-mirror', 'msg-user', 'msg-assistant', 'completed', '', '', '', '2026-05-08T00:00:00Z', '2026-05-08T00:00:01Z')",
    )
    .execute(&pool)
    .await
    .expect("seed run");

    ensure_profile_session_index_schema_with_pool(&pool)
        .await
        .expect("create profile session index schema");
    index_profile_session_manifest_with_pool(&pool, &manifest_path)
        .await
        .expect("index manifest and write transcript mirror");

    let transcript_path = manifest_path.parent().unwrap().join("transcript.md");
    let transcript = std::fs::read_to_string(transcript_path).expect("read transcript mirror");
    assert!(transcript.contains("# Profile Session Transcript"));
    assert!(transcript.contains("session-transcript-mirror"));
    assert!(transcript.contains("run-transcript"));
    assert!(transcript.contains("用户问题：生成 profile transcript mirror"));
    assert!(transcript.contains("助手回答：transcript mirror 已生成。"));
    assert!(transcript.contains("工具摘要：召回历史经验。"));
    assert!(transcript.contains("压缩摘要：保留关键事实。"));
}

#[tokio::test]
async fn profile_session_index_refreshes_from_session_row_after_final_message() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let runtime_root = tmp.path().join("runtime-root");
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("create sqlite memory pool");
    sqlx::query(
        "CREATE TABLE sessions (
            id TEXT PRIMARY KEY,
            skill_id TEXT NOT NULL,
            title TEXT,
            created_at TEXT NOT NULL,
            model_id TEXT NOT NULL,
            work_dir TEXT NOT NULL DEFAULT '',
            profile_id TEXT
        )",
    )
    .execute(&pool)
    .await
    .expect("create sessions");
    sqlx::query(
        "CREATE TABLE messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            content_json TEXT,
            created_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("create messages");
    sqlx::query(
        "INSERT INTO sessions (id, skill_id, title, created_at, model_id, work_dir, profile_id)
         VALUES ('session-refresh', 'builtin-general', 'Refresh', '2026-05-07T00:00:00Z', 'model-1', 'E:/workspace/acme', 'profile-1')",
    )
    .execute(&pool)
    .await
    .expect("seed session");
    sqlx::query(
        "INSERT INTO messages (id, session_id, role, content, content_json, created_at)
         VALUES ('msg-final', 'session-refresh', 'assistant', '最终回答：本轮完成后立即刷新 profile session index。', NULL, '2026-05-07T00:00:01Z')",
    )
    .execute(&pool)
    .await
    .expect("seed final message");

    let manifest_path = refresh_profile_session_index_for_session_with_pool(
        &pool,
        &runtime_root,
        "session-refresh",
        "test",
    )
    .await
    .expect("refresh profile session index")
    .expect("manifest path");

    assert!(manifest_path.exists());

    let results = search_profile_session_index_with_pool(&pool, "profile-1", "立即刷新", 10)
        .await
        .expect("search refreshed profile session");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].session_id, "session-refresh");
    assert!(results[0].snippet.contains("本轮完成后立即刷新"));
}

#[tokio::test]
async fn profile_session_index_reports_matched_run_for_turn_level_hits() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let runtime_root = tmp.path().join("runtime-root");
    let manifest_path = write_profile_session_manifest(
        &runtime_root,
        ProfileSessionManifestInput {
            profile_id: "profile-1",
            session_id: "session-run-docs",
            skill_id: "builtin-general",
            work_dir: Some("E:/workspace/acme"),
            source: "runtime_tool_setup",
        },
    )
    .expect("write manifest");
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("create sqlite memory pool");
    sqlx::query(
        "CREATE TABLE messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            content_json TEXT,
            created_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("create messages");
    sqlx::query(
        "CREATE TABLE session_runs (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            user_message_id TEXT NOT NULL DEFAULT '',
            assistant_message_id TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'queued',
            buffered_text TEXT NOT NULL DEFAULT '',
            error_kind TEXT NOT NULL DEFAULT '',
            error_message TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("create session runs");
    sqlx::query(
        "INSERT INTO messages (id, session_id, role, content, content_json, created_at)
         VALUES
         ('msg-old-user', 'session-run-docs', 'user', '用户问题：旧流程怎么做？', NULL, '2026-05-07T00:00:00Z'),
         ('msg-old-assistant', 'session-run-docs', 'assistant', '旧方案：保持原状。', NULL, '2026-05-07T00:00:01Z'),
         ('msg-target-user', 'session-run-docs', 'user', '用户问题：如何做 turn-level index？', NULL, '2026-05-07T00:01:00Z'),
         ('msg-target-assistant', 'session-run-docs', 'assistant', '目标方案：使用 run-level document 精确召回。', NULL, '2026-05-07T00:01:01Z')",
    )
    .execute(&pool)
    .await
    .expect("seed messages");
    sqlx::query(
        "INSERT INTO session_runs (id, session_id, user_message_id, assistant_message_id, status, buffered_text, error_kind, error_message, created_at, updated_at)
         VALUES
         ('run-old', 'session-run-docs', 'msg-old-user', 'msg-old-assistant', 'completed', '', '', '', '2026-05-07T00:00:00Z', '2026-05-07T00:00:01Z'),
         ('run-target', 'session-run-docs', 'msg-target-user', 'msg-target-assistant', 'completed', '', '', '', '2026-05-07T00:01:00Z', '2026-05-07T00:01:01Z')",
    )
    .execute(&pool)
    .await
    .expect("seed runs");

    ensure_profile_session_index_schema_with_pool(&pool)
        .await
        .expect("create profile session index schema");
    index_profile_session_manifest_with_pool(&pool, &manifest_path)
        .await
        .expect("index manifest and run docs");

    let results =
        search_profile_session_index_with_pool(&pool, "profile-1", "run-level document", 10)
            .await
            .expect("search run-level docs");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].session_id, "session-run-docs");
    assert_eq!(results[0].document_kind, "run");
    assert_eq!(results[0].matched_run_id, "run-target");
    assert!(results[0].snippet.contains("run-level document"));
}

#[tokio::test]
async fn profile_session_index_filters_by_workspace_time_skill_and_source() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let runtime_root = tmp.path().join("runtime-root");
    let target = write_profile_session_manifest_with_updated_at(
        &runtime_root,
        "session-target-filter",
        "skill-target",
        "E:/workspace/acme",
        "runtime_tool_setup",
        "2026-05-08T09:30:00Z",
    );
    let wrong_workspace = write_profile_session_manifest_with_updated_at(
        &runtime_root,
        "session-wrong-workspace",
        "skill-target",
        "E:/workspace/other",
        "runtime_tool_setup",
        "2026-05-08T09:35:00Z",
    );
    let wrong_skill = write_profile_session_manifest_with_updated_at(
        &runtime_root,
        "session-wrong-skill",
        "skill-other",
        "E:/workspace/acme",
        "runtime_tool_setup",
        "2026-05-08T09:40:00Z",
    );
    let wrong_source = write_profile_session_manifest_with_updated_at(
        &runtime_root,
        "session-wrong-source",
        "skill-target",
        "E:/workspace/acme",
        "manual_fixture",
        "2026-05-08T09:45:00Z",
    );
    let outside_time = write_profile_session_manifest_with_updated_at(
        &runtime_root,
        "session-outside-time",
        "skill-target",
        "E:/workspace/acme",
        "runtime_tool_setup",
        "2026-05-07T09:30:00Z",
    );

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("create sqlite memory pool");
    ensure_profile_session_index_schema_with_pool(&pool)
        .await
        .expect("create profile session index schema");
    for manifest_path in [
        target,
        wrong_workspace,
        wrong_skill,
        wrong_source,
        outside_time,
    ] {
        index_profile_session_manifest_with_pool(&pool, &manifest_path)
            .await
            .expect("index manifest");
    }

    let results = search_profile_session_index_with_filters_with_pool(
        &pool,
        "profile-1",
        "skill-target",
        10,
        ProfileSessionSearchFilters {
            work_dir: Some("E:/workspace/acme".to_string()),
            updated_after: Some("2026-05-08T00:00:00Z".to_string()),
            updated_before: Some("2026-05-09T00:00:00Z".to_string()),
            skill_id: Some("skill-target".to_string()),
            source: Some("runtime_tool_setup".to_string()),
        },
    )
    .await
    .expect("search with filters");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].session_id, "session-target-filter");
    assert_eq!(results[0].skill_id, "skill-target");
    assert_eq!(results[0].work_dir, "E:/workspace/acme");
    assert_eq!(results[0].document_kind, "session");
    assert_eq!(results[0].source, "runtime_tool_setup");
}

#[tokio::test]
async fn profile_session_index_applies_filters_to_empty_query() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let runtime_root = tmp.path().join("runtime-root");
    let target = write_profile_session_manifest_with_updated_at(
        &runtime_root,
        "session-empty-filter-target",
        "skill-target",
        "E:/workspace/acme",
        "runtime_tool_setup",
        "2026-05-08T09:30:00Z",
    );
    let other = write_profile_session_manifest_with_updated_at(
        &runtime_root,
        "session-empty-filter-other",
        "skill-target",
        "E:/workspace/other",
        "runtime_tool_setup",
        "2026-05-08T09:35:00Z",
    );
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("create sqlite memory pool");
    ensure_profile_session_index_schema_with_pool(&pool)
        .await
        .expect("create profile session index schema");
    for manifest_path in [target, other] {
        index_profile_session_manifest_with_pool(&pool, &manifest_path)
            .await
            .expect("index manifest");
    }

    let results = search_profile_session_index_with_filters_with_pool(
        &pool,
        "profile-1",
        "",
        10,
        ProfileSessionSearchFilters {
            work_dir: Some("E:/workspace/acme".to_string()),
            ..ProfileSessionSearchFilters::default()
        },
    )
    .await
    .expect("search empty query with filters");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].session_id, "session-empty-filter-target");
    assert_eq!(results[0].document_kind, "session");
}

#[tokio::test]
async fn profile_session_index_applies_filters_to_run_level_hits() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let runtime_root = tmp.path().join("runtime-root");
    let manifest_path = write_profile_session_manifest_with_updated_at(
        &runtime_root,
        "session-run-filter-target",
        "skill-target",
        "E:/workspace/acme",
        "runtime_tool_setup",
        "2026-05-08T09:30:00Z",
    );
    let other_manifest = write_profile_session_manifest_with_updated_at(
        &runtime_root,
        "session-run-filter-other",
        "skill-target",
        "E:/workspace/other",
        "runtime_tool_setup",
        "2026-05-08T09:35:00Z",
    );
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("create sqlite memory pool");
    sqlx::query(
        "CREATE TABLE messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            content_json TEXT,
            created_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("create messages");
    sqlx::query(
        "CREATE TABLE session_runs (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            user_message_id TEXT NOT NULL DEFAULT '',
            assistant_message_id TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'queued',
            buffered_text TEXT NOT NULL DEFAULT '',
            error_kind TEXT NOT NULL DEFAULT '',
            error_message TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("create session runs");
    sqlx::query(
        "INSERT INTO messages (id, session_id, role, content, content_json, created_at)
         VALUES
         ('msg-target-user', 'session-run-filter-target', 'user', 'target workspace turn recall', NULL, '2026-05-08T00:00:00Z'),
         ('msg-target-assistant', 'session-run-filter-target', 'assistant', 'run filter precise hit', NULL, '2026-05-08T00:00:01Z'),
         ('msg-other-user', 'session-run-filter-other', 'user', 'target workspace turn recall', NULL, '2026-05-08T00:00:00Z'),
         ('msg-other-assistant', 'session-run-filter-other', 'assistant', 'run filter wrong workspace', NULL, '2026-05-08T00:00:01Z')",
    )
    .execute(&pool)
    .await
    .expect("seed messages");
    sqlx::query(
        "INSERT INTO session_runs (id, session_id, user_message_id, assistant_message_id, status, buffered_text, error_kind, error_message, created_at, updated_at)
         VALUES
         ('run-target-filter', 'session-run-filter-target', 'msg-target-user', 'msg-target-assistant', 'completed', '', '', '', '2026-05-08T00:00:00Z', '2026-05-08T00:00:01Z'),
         ('run-other-filter', 'session-run-filter-other', 'msg-other-user', 'msg-other-assistant', 'completed', '', '', '', '2026-05-08T00:00:00Z', '2026-05-08T00:00:01Z')",
    )
    .execute(&pool)
    .await
    .expect("seed runs");
    ensure_profile_session_index_schema_with_pool(&pool)
        .await
        .expect("create profile session index schema");
    for manifest_path in [manifest_path, other_manifest] {
        index_profile_session_manifest_with_pool(&pool, &manifest_path)
            .await
            .expect("index manifest");
    }

    let results = search_profile_session_index_with_filters_with_pool(
        &pool,
        "profile-1",
        "precise hit",
        10,
        ProfileSessionSearchFilters {
            work_dir: Some("E:/workspace/acme".to_string()),
            ..ProfileSessionSearchFilters::default()
        },
    )
    .await
    .expect("search filtered run docs");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].session_id, "session-run-filter-target");
    assert_eq!(results[0].document_kind, "run");
    assert_eq!(results[0].matched_run_id, "run-target-filter");
}
