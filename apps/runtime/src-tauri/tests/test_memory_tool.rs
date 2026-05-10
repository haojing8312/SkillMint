use runtime_lib::agent::runtime::runtime_io::{
    ensure_profile_session_index_schema_with_pool, index_profile_session_manifest_with_pool,
    write_profile_session_manifest, ProfileSessionManifestInput,
};
use runtime_lib::agent::tools::MemoryTool;
use runtime_lib::agent::types::{Tool, ToolContext};
use serde_json::json;
use std::fs;

/// 创建临时目录并返回 MemoryTool 实例
fn create_test_memory() -> (MemoryTool, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let tool = MemoryTool::new(dir.path().to_path_buf());
    (tool, dir)
}

#[test]
fn test_memory_write_and_read() {
    let (tool, _dir) = create_test_memory();
    let ctx = ToolContext::default();

    // 写入内容
    let write_result = tool
        .execute(
            json!({
                "action": "write",
                "key": "test",
                "content": "Hello Memory"
            }),
            &ctx,
        )
        .unwrap();
    assert!(write_result.contains("已写入"));

    // 读回并验证内容一致
    let read_result = tool
        .execute(
            json!({
                "action": "read",
                "key": "test"
            }),
            &ctx,
        )
        .unwrap();
    assert_eq!(read_result, "Hello Memory");
}

#[test]
fn test_memory_list() {
    let (tool, _dir) = create_test_memory();
    let ctx = ToolContext::default();

    // 未写入时应返回空提示
    let result = tool.execute(json!({"action": "list"}), &ctx).unwrap();
    assert!(result.contains("内存为空"));

    // 写入两个键后列表应包含两个键名
    tool.execute(json!({"action": "write", "key": "a", "content": "1"}), &ctx)
        .unwrap();
    tool.execute(json!({"action": "write", "key": "b", "content": "2"}), &ctx)
        .unwrap();
    let result = tool.execute(json!({"action": "list"}), &ctx).unwrap();
    assert!(result.contains("a"));
    assert!(result.contains("b"));
}

#[test]
fn test_memory_delete() {
    let (tool, _dir) = create_test_memory();
    let ctx = ToolContext::default();

    // 写入后删除
    tool.execute(
        json!({"action": "write", "key": "del", "content": "x"}),
        &ctx,
    )
    .unwrap();
    let result = tool
        .execute(json!({"action": "delete", "key": "del"}), &ctx)
        .unwrap();
    assert!(result.contains("已删除"));

    // 删除后读取应返回不存在提示
    let read_result = tool
        .execute(json!({"action": "read", "key": "del"}), &ctx)
        .unwrap();
    assert!(read_result.contains("不存在"));
}

#[test]
fn test_memory_read_nonexistent() {
    let (tool, _dir) = create_test_memory();
    let ctx = ToolContext::default();

    // 读取不存在的键应返回友好提示而非 error
    let result = tool
        .execute(json!({"action": "read", "key": "nope"}), &ctx)
        .unwrap();
    assert!(result.contains("不存在"));
}

#[test]
fn test_memory_missing_action() {
    let (tool, _dir) = create_test_memory();
    let ctx = ToolContext::default();

    // 缺少 action 参数应返回错误
    let result = tool.execute(json!({}), &ctx);
    assert!(result.is_err());
}

#[test]
fn test_memory_overwrite() {
    let (tool, _dir) = create_test_memory();
    let ctx = ToolContext::default();

    // 同一个键多次写入，应以最新内容为准
    tool.execute(
        json!({"action": "write", "key": "k", "content": "first"}),
        &ctx,
    )
    .unwrap();
    tool.execute(
        json!({"action": "write", "key": "k", "content": "second"}),
        &ctx,
    )
    .unwrap();
    let result = tool
        .execute(json!({"action": "read", "key": "k"}), &ctx)
        .unwrap();
    assert_eq!(result, "second");
}

#[test]
fn test_memory_delete_nonexistent() {
    let (tool, _dir) = create_test_memory();
    let ctx = ToolContext::default();

    // 删除不存在的键应返回友好提示而非 error
    let result = tool
        .execute(json!({"action": "delete", "key": "ghost"}), &ctx)
        .unwrap();
    assert!(result.contains("不存在"));
}

#[test]
fn test_memory_unknown_action() {
    let (tool, _dir) = create_test_memory();
    let ctx = ToolContext::default();

    // 未知操作应返回错误
    let result = tool.execute(json!({"action": "explode"}), &ctx);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("未知操作"));
}

#[test]
fn test_memory_list_sorted() {
    let (tool, _dir) = create_test_memory();
    let ctx = ToolContext::default();

    // 写入乱序键，列表结果应按字母排序
    tool.execute(json!({"action": "write", "key": "c", "content": "3"}), &ctx)
        .unwrap();
    tool.execute(json!({"action": "write", "key": "a", "content": "1"}), &ctx)
        .unwrap();
    tool.execute(json!({"action": "write", "key": "b", "content": "2"}), &ctx)
        .unwrap();

    let result = tool.execute(json!({"action": "list"}), &ctx).unwrap();
    // 验证 a 在 b 前面，b 在 c 前面
    let pos_a = result.find('a').unwrap();
    let pos_b = result.find('b').unwrap();
    let pos_c = result.find('c').unwrap();
    assert!(pos_a < pos_b);
    assert!(pos_b < pos_c);
}

#[test]
fn test_profile_memory_add_view_history() {
    let (tool, dir) = create_test_memory();
    let ctx = ToolContext::default();

    let result = tool
        .execute(
            json!({
                "action": "add",
                "content": "用户偏好先给结论，再给细节。",
                "source": "session:test"
            }),
            &ctx,
        )
        .unwrap();
    assert!(result.contains("已追加"));

    let memory_path = dir.path().join("MEMORY.md");
    let memory = fs::read_to_string(&memory_path).unwrap();
    assert!(memory.contains("用户偏好先给结论"));

    let viewed = tool.execute(json!({"action": "view"}), &ctx).unwrap();
    assert_eq!(viewed, memory);

    let history = tool.execute(json!({"action": "history"}), &ctx).unwrap();
    assert!(history.contains("\"action\":\"add\""));
    assert!(history.contains("\"source\":\"session:test\""));
}

#[test]
fn test_profile_memory_versions_and_rollback() {
    let dir = tempfile::tempdir().unwrap();
    let tool = MemoryTool::new(dir.path().to_path_buf());
    let ctx = ToolContext::default();

    tool.execute(
        json!({
            "action": "add",
            "content": "first memory",
            "source": "session:test",
            "change_summary": "seed profile memory"
        }),
        &ctx,
    )
    .unwrap();
    tool.execute(
        json!({
            "action": "replace",
            "content": "second memory",
            "source": "session:test",
            "change_summary": "replace profile memory"
        }),
        &ctx,
    )
    .unwrap();

    let versions_raw = tool.execute(json!({"action": "versions"}), &ctx).unwrap();
    let versions: serde_json::Value = serde_json::from_str(&versions_raw).unwrap();
    let items = versions.as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["action"], "add");
    assert_eq!(items[0]["change_summary"], "seed profile memory");
    assert_eq!(items[1]["action"], "replace");
    let first_version = items[0]["version_id"].as_str().unwrap();

    let first_content = tool
        .execute(
            json!({
                "action": "view_version",
                "version_id": first_version
            }),
            &ctx,
        )
        .unwrap();
    assert!(first_content.contains("first memory"));
    assert!(!first_content.contains("second memory"));

    let refused = tool
        .execute(
            json!({
                "action": "rollback",
                "version_id": first_version
            }),
            &ctx,
        )
        .unwrap();
    assert!(refused.contains("confirm=true"));
    assert_eq!(
        fs::read_to_string(dir.path().join("MEMORY.md")).unwrap(),
        "second memory\n"
    );

    tool.execute(
        json!({
            "action": "rollback",
            "version_id": first_version,
            "confirm": true,
            "reason": "restore first version"
        }),
        &ctx,
    )
    .unwrap();
    assert_eq!(
        fs::read_to_string(dir.path().join("MEMORY.md")).unwrap(),
        "first memory\n"
    );

    let history = tool.execute(json!({"action": "history"}), &ctx).unwrap();
    assert!(history.contains("\"action\":\"rollback\""));
    assert!(history.contains(first_version));
}

#[test]
fn test_profile_memory_mutations_write_growth_events() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let pool = runtime
        .block_on(async {
            sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(5)
                .connect("sqlite::memory:")
                .await
        })
        .expect("create sqlite pool");
    let dir = tempfile::tempdir().unwrap();
    let tool = MemoryTool::new(dir.path().to_path_buf())
        .with_profile_session_search(pool.clone(), "profile-memory-growth".to_string());
    let ctx = ToolContext {
        session_id: Some("session-memory-growth".to_string()),
        ..ToolContext::default()
    };

    tool.execute(
        json!({
            "action": "add",
            "content": "first memory",
            "change_summary": "learn first memory"
        }),
        &ctx,
    )
    .unwrap();
    let versions_raw = tool.execute(json!({"action": "versions"}), &ctx).unwrap();
    let versions: serde_json::Value = serde_json::from_str(&versions_raw).unwrap();
    let first_version = versions[0]["version_id"].as_str().unwrap().to_string();

    tool.execute(
        json!({
            "action": "replace",
            "content": "second memory",
            "change_summary": "replace memory"
        }),
        &ctx,
    )
    .unwrap();
    tool.execute(
        json!({
            "action": "remove",
            "confirm": true,
            "change_summary": "remove stale memory"
        }),
        &ctx,
    )
    .unwrap();
    tool.execute(
        json!({
            "action": "rollback",
            "version_id": first_version,
            "confirm": true,
            "reason": "restore learned memory"
        }),
        &ctx,
    )
    .unwrap();

    let events: Vec<(String, String, String, String)> = runtime
        .block_on(async {
            sqlx::query_as(
                "SELECT event_type, target_type, session_id, evidence_json
                 FROM growth_events
                 WHERE profile_id = 'profile-memory-growth'
                 ORDER BY created_at ASC, id ASC",
            )
            .fetch_all(&pool)
            .await
        })
        .expect("query growth events");

    let event_types = events
        .iter()
        .map(|(event_type, _, _, _)| event_type.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        event_types,
        vec![
            "memory_add",
            "memory_replace",
            "memory_remove",
            "memory_rollback"
        ]
    );
    assert!(events.iter().all(|(_, target_type, session_id, evidence)| {
        target_type == "profile_memory"
            && session_id == "session-memory-growth"
            && evidence.contains("version_id")
    }));
}

#[test]
fn test_user_correction_memory_source_writes_dedicated_growth_event() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let pool = runtime
        .block_on(async {
            sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(5)
                .connect("sqlite::memory:")
                .await
        })
        .expect("create sqlite pool");
    let dir = tempfile::tempdir().unwrap();
    let tool = MemoryTool::new(dir.path().to_path_buf())
        .with_profile_session_search(pool.clone(), "profile-correction".to_string());
    let ctx = ToolContext {
        session_id: Some("session-correction".to_string()),
        ..ToolContext::default()
    };

    tool.execute(
        json!({
            "action": "add",
            "content": "用户纠正：日报摘要必须区分已完成和阻塞项。",
            "source": "user-correction",
            "change_summary": "用户纠正日报摘要规则"
        }),
        &ctx,
    )
    .unwrap();

    let row: (String, String, String) = runtime
        .block_on(async {
            sqlx::query_as(
                "SELECT event_type, target_type, summary
                 FROM growth_events
                 WHERE profile_id = 'profile-correction'
                 LIMIT 1",
            )
            .fetch_one(&pool)
            .await
        })
        .expect("query growth event");

    assert_eq!(row.0, "user_correction");
    assert_eq!(row.1, "profile_memory");
    assert_eq!(row.2, "用户纠正日报摘要规则");
}

#[test]
fn test_project_memory_versions_are_isolated_from_profile_versions() {
    let dir = tempfile::tempdir().unwrap();
    let project_memory_file = dir.path().join("PROJECTS").join("workspace-a.md");
    let tool =
        MemoryTool::new(dir.path().to_path_buf()).with_project_memory_path(project_memory_file);
    let ctx = ToolContext::default();

    tool.execute(
        json!({
            "action": "replace",
            "scope": "project",
            "content": "project memory",
            "source": "session:project",
            "change_summary": "seed project memory"
        }),
        &ctx,
    )
    .unwrap();

    let project_versions_raw = tool
        .execute(json!({"action": "versions", "scope": "project"}), &ctx)
        .unwrap();
    let project_versions: serde_json::Value = serde_json::from_str(&project_versions_raw).unwrap();
    assert_eq!(project_versions.as_array().unwrap().len(), 1);
    assert_eq!(project_versions[0]["scope"], "project");
    assert_eq!(project_versions[0]["target_key"], "workspace-a");
    assert!(dir
        .path()
        .join("versions")
        .join("projects")
        .join("workspace-a")
        .exists());

    let profile_versions_raw = tool.execute(json!({"action": "versions"}), &ctx).unwrap();
    let profile_versions: serde_json::Value = serde_json::from_str(&profile_versions_raw).unwrap();
    assert_eq!(profile_versions.as_array().unwrap().len(), 0);
}

#[test]
fn test_profile_memory_remove_creates_tombstone_version() {
    let dir = tempfile::tempdir().unwrap();
    let tool = MemoryTool::new(dir.path().to_path_buf());
    let ctx = ToolContext::default();

    tool.execute(
        json!({
            "action": "replace",
            "content": "temporary memory"
        }),
        &ctx,
    )
    .unwrap();
    tool.execute(
        json!({
            "action": "remove",
            "confirm": true,
            "change_summary": "delete stale memory"
        }),
        &ctx,
    )
    .unwrap();

    let versions_raw = tool.execute(json!({"action": "versions"}), &ctx).unwrap();
    let versions: serde_json::Value = serde_json::from_str(&versions_raw).unwrap();
    let items = versions.as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[1]["action"], "remove");
    assert_eq!(items[1]["deleted"], true);
    assert_eq!(items[1]["change_summary"], "delete stale memory");

    let removed_version = items[1]["version_id"].as_str().unwrap();
    let removed_snapshot = tool
        .execute(
            json!({
                "action": "view_version",
                "version_id": removed_version
            }),
            &ctx,
        )
        .unwrap();
    assert_eq!(removed_snapshot, "");
}

#[test]
fn test_profile_memory_replace_and_confirmed_remove() {
    let (tool, dir) = create_test_memory();
    let ctx = ToolContext::default();

    tool.execute(
        json!({
            "action": "add",
            "content": "旧记忆"
        }),
        &ctx,
    )
    .unwrap();

    let result = tool
        .execute(
            json!({
                "action": "replace",
                "content": "新记忆",
                "source": "user-correction"
            }),
            &ctx,
        )
        .unwrap();
    assert!(result.contains("已替换"));

    let memory = fs::read_to_string(dir.path().join("MEMORY.md")).unwrap();
    assert_eq!(memory, "新记忆\n");

    let unconfirmed = tool.execute(json!({"action": "remove"}), &ctx).unwrap();
    assert!(unconfirmed.contains("confirm=true"));
    assert!(dir.path().join("MEMORY.md").exists());

    let removed = tool
        .execute(json!({"action": "remove", "confirm": true}), &ctx)
        .unwrap();
    assert!(removed.contains("已移除"));
    assert!(!dir.path().join("MEMORY.md").exists());

    let history = tool.execute(json!({"action": "history"}), &ctx).unwrap();
    assert!(history.contains("\"action\":\"replace\""));
    assert!(history.contains("\"action\":\"remove\""));
}

#[test]
fn test_im_memory_uses_separate_memory_dir() {
    let profile_dir = tempfile::tempdir().unwrap();
    let im_dir = tempfile::tempdir().unwrap();
    let tool = MemoryTool::new(profile_dir.path().to_path_buf())
        .with_im_memory_dir(im_dir.path().to_path_buf());
    let ctx = ToolContext::default();

    let result = tool
        .execute(
            json!({
                "action": "capture_im",
                "thread_id": "thread-1",
                "role_id": "role-1",
                "category": "fact",
                "content": "IM 长期事实",
                "confirmed": true,
                "confidence": 0.9,
                "source_msg_id": "msg-1"
            }),
            &ctx,
        )
        .unwrap();
    assert!(result.contains("IM 记忆写入完成"));

    assert!(!profile_dir.path().join("roles").exists());
    assert!(im_dir
        .path()
        .join("roles")
        .join("role-1")
        .join("MEMORY.md")
        .exists());
}

#[test]
fn test_project_memory_scope_uses_project_memory_file() {
    let profile_dir = tempfile::tempdir().unwrap();
    let project_dir = tempfile::tempdir().unwrap();
    let project_memory_file = project_dir.path().join("workspace.md");
    let tool = MemoryTool::new(profile_dir.path().to_path_buf())
        .with_project_memory_path(project_memory_file.clone());
    let ctx = ToolContext::default();

    let result = tool
        .execute(
            json!({
                "action": "add",
                "scope": "project",
                "content": "项目约定：先跑快速验证。"
            }),
            &ctx,
        )
        .unwrap();
    assert!(result.contains("Project Memory"));

    let project_memory = fs::read_to_string(&project_memory_file).unwrap();
    assert!(project_memory.contains("项目约定"));
    assert!(!profile_dir.path().join("MEMORY.md").exists());

    let viewed = tool
        .execute(json!({"action": "view", "scope": "project"}), &ctx)
        .unwrap();
    assert_eq!(viewed, project_memory);
}

#[test]
fn test_memory_search_reads_profile_session_index() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let tmp = tempfile::tempdir().unwrap();
    let runtime_root = tmp.path().join("runtime-root");
    let journal_dir = runtime_root.join("sessions").join("session-memory-search");
    fs::create_dir_all(&journal_dir).unwrap();
    fs::write(
        journal_dir.join("state.json"),
        json!({
            "session_id": "session-memory-search",
            "current_run_id": "run-memory-search",
            "runs": [
                {
                    "run_id": "run-memory-search",
                    "user_message_id": "msg-memory-search",
                    "status": "completed",
                    "buffered_text": "完成历史经验召回",
                    "last_error_kind": null,
                    "last_error_message": null,
                    "turn_state": {
                        "compaction_boundary": {
                            "transcript_path": "transcripts/session-memory-search.jsonl",
                            "original_tokens": 4096,
                            "compacted_tokens": 1024,
                            "summary": "历史经验：Profile session search 可以召回工具调用摘要。"
                        }
                    }
                }
            ]
        })
        .to_string(),
    )
    .unwrap();
    fs::write(
        journal_dir.join("events.jsonl"),
        format!(
            "{}\n",
            json!({
                "session_id": "session-memory-search",
                "recorded_at": "2026-05-07T00:00:01Z",
                "event": {
                    "type": "tool_completed",
                    "run_id": "run-memory-search",
                    "tool_name": "memory",
                    "call_id": "call-search",
                    "input": { "action": "search", "query": "工具调用摘要" },
                    "output": "工具调用摘要可被后续任务召回。",
                    "is_error": false
                }
            })
        ),
    )
    .unwrap();
    let manifest_path = write_profile_session_manifest(
        &runtime_root,
        ProfileSessionManifestInput {
            profile_id: "profile-1",
            session_id: "session-memory-search",
            skill_id: "builtin-general",
            work_dir: Some("E:/workspace/acme"),
            source: "test",
        },
    )
    .unwrap();
    let pool = runtime.block_on(async {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        ensure_profile_session_index_schema_with_pool(&pool)
            .await
            .unwrap();
        index_profile_session_manifest_with_pool(&pool, &manifest_path)
            .await
            .unwrap();
        pool
    });

    let tool = MemoryTool::new(tmp.path().join("profile-memory"))
        .with_profile_session_search(pool, "profile-1".to_string());
    let result = tool
        .execute(
            json!({
                "action": "search",
                "query": "工具调用摘要",
                "limit": 5
            }),
            &ToolContext::default(),
        )
        .unwrap();

    assert!(result.contains("session-memory-search"));
    assert!(result.contains("tool_summary_count"));
    assert!(result.contains("工具调用摘要"));
}

#[test]
fn test_memory_search_applies_profile_session_filters() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let tmp = tempfile::tempdir().unwrap();
    let runtime_root = tmp.path().join("runtime-root");
    let target_manifest = write_profile_session_manifest(
        &runtime_root,
        ProfileSessionManifestInput {
            profile_id: "profile-1",
            session_id: "session-memory-search-filter-target",
            skill_id: "skill-target",
            work_dir: Some("E:/workspace/acme"),
            source: "runtime_tool_setup",
        },
    )
    .unwrap();
    let other_manifest = write_profile_session_manifest(
        &runtime_root,
        ProfileSessionManifestInput {
            profile_id: "profile-1",
            session_id: "session-memory-search-filter-other",
            skill_id: "skill-target",
            work_dir: Some("E:/workspace/other"),
            source: "runtime_tool_setup",
        },
    )
    .unwrap();
    let pool = runtime.block_on(async {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        ensure_profile_session_index_schema_with_pool(&pool)
            .await
            .unwrap();
        for manifest_path in [target_manifest, other_manifest] {
            index_profile_session_manifest_with_pool(&pool, &manifest_path)
                .await
                .unwrap();
        }
        pool
    });

    let tool = MemoryTool::new(tmp.path().join("profile-memory"))
        .with_profile_session_search(pool, "profile-1".to_string());
    let result = tool
        .execute(
            json!({
                "action": "search",
                "query": "skill-target",
                "work_dir": "E:/workspace/acme",
                "skill_id": "skill-target",
                "source": "runtime_tool_setup",
                "limit": 5
            }),
            &ToolContext::default(),
        )
        .unwrap();

    assert!(result.contains("session-memory-search-filter-target"));
    assert!(!result.contains("session-memory-search-filter-other"));
    assert!(result.contains("\"source\": \"runtime_tool_setup\""));
}
