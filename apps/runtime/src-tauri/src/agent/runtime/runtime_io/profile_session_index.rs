use serde::{Deserialize, Serialize};
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProfileSessionSearchResult {
    pub profile_id: String,
    pub session_id: String,
    pub skill_id: String,
    pub work_dir: String,
    pub source: String,
    pub run_status: String,
    pub latest_run_id: String,
    pub document_kind: String,
    pub matched_run_id: String,
    pub tool_summary_count: i64,
    pub compaction_boundary_count: i64,
    pub manifest_path: String,
    pub updated_at: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProfileSessionSearchFilters {
    pub work_dir: Option<String>,
    pub updated_after: Option<String>,
    pub updated_before: Option<String>,
    pub skill_id: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Default)]
struct SessionMessageIndexText {
    combined_text: String,
}

#[derive(Debug, Default)]
struct SessionTranscriptMessage {
    role: String,
    content: String,
    created_at: String,
    run_id: String,
}

#[derive(Debug, Default)]
struct RunLevelIndexDocument {
    run_id: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct ManifestRunSummary {
    #[serde(default)]
    status: String,
    #[serde(default)]
    latest_run_id: String,
    #[serde(default)]
    user_message_id: String,
    #[serde(default)]
    buffered_text_preview: String,
    #[serde(default)]
    last_error_kind: String,
    #[serde(default)]
    last_error_message: String,
}

#[derive(Debug, Deserialize)]
struct ManifestToolSummary {
    #[serde(default)]
    run_id: String,
    #[serde(default)]
    tool_name: String,
    #[serde(default)]
    call_id: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    input_preview: String,
    #[serde(default)]
    output_preview: String,
}

#[derive(Debug, Deserialize)]
struct ManifestCompactionBoundary {
    #[serde(default)]
    run_id: String,
    #[serde(default)]
    transcript_path: String,
    #[serde(default)]
    original_tokens: usize,
    #[serde(default)]
    compacted_tokens: usize,
    #[serde(default)]
    summary: String,
}

#[derive(Debug, Deserialize)]
struct ProfileSessionManifestRecord {
    profile_id: String,
    session_id: String,
    #[serde(default)]
    skill_id: String,
    #[serde(default)]
    work_dir: String,
    #[serde(default)]
    source: String,
    #[serde(default)]
    run_summary: Option<ManifestRunSummary>,
    #[serde(default)]
    tool_summaries: Vec<ManifestToolSummary>,
    #[serde(default)]
    compaction_boundaries: Vec<ManifestCompactionBoundary>,
    #[serde(default)]
    updated_at: String,
}

pub async fn ensure_profile_session_index_schema_with_pool(
    pool: &SqlitePool,
) -> Result<(), String> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS profile_session_index (
            profile_id TEXT NOT NULL,
            session_id TEXT NOT NULL,
            skill_id TEXT NOT NULL DEFAULT '',
            work_dir TEXT NOT NULL DEFAULT '',
            source TEXT NOT NULL DEFAULT '',
            run_status TEXT NOT NULL DEFAULT '',
            latest_run_id TEXT NOT NULL DEFAULT '',
            tool_summary_count INTEGER NOT NULL DEFAULT 0,
            compaction_boundary_count INTEGER NOT NULL DEFAULT 0,
            manifest_path TEXT NOT NULL DEFAULT '',
            search_text TEXT NOT NULL DEFAULT '',
            updated_at TEXT NOT NULL DEFAULT '',
            PRIMARY KEY (profile_id, session_id)
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| format!("创建 profile_session_index 失败: {e}"))?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_profile_session_index_updated
         ON profile_session_index(profile_id, updated_at DESC)",
    )
    .execute(pool)
    .await
    .map_err(|e| format!("创建 profile_session_index 更新时间索引失败: {e}"))?;

    let fts_columns: Vec<String> =
        sqlx::query_scalar("SELECT name FROM pragma_table_info('profile_session_fts')")
            .fetch_all(pool)
            .await
            .unwrap_or_default();
    let required_fts_columns = ["document_kind", "run_id", "message_text"];
    if !fts_columns.is_empty()
        && required_fts_columns
            .iter()
            .any(|column| !fts_columns.iter().any(|name| name == column))
    {
        sqlx::query("DROP TABLE IF EXISTS profile_session_fts")
            .execute(pool)
            .await
            .map_err(|e| format!("重建 profile_session_fts 失败: {e}"))?;
    }

    sqlx::query(
        "CREATE VIRTUAL TABLE IF NOT EXISTS profile_session_fts USING fts5(
            profile_id UNINDEXED,
            session_id UNINDEXED,
            document_kind UNINDEXED,
            run_id UNINDEXED,
            skill_id,
            work_dir,
            run_text,
            tool_text,
            compaction_text,
            message_text,
            tokenize = 'unicode61'
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| format!("创建 profile_session_fts 失败: {e}"))?;

    Ok(())
}

fn join_non_empty(parts: impl IntoIterator<Item = String>) -> String {
    parts
        .into_iter()
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn manifest_run_text(run: Option<&ManifestRunSummary>) -> String {
    let Some(run) = run else {
        return String::new();
    };
    join_non_empty([
        run.status.clone(),
        run.latest_run_id.clone(),
        run.user_message_id.clone(),
        run.buffered_text_preview.clone(),
        run.last_error_kind.clone(),
        run.last_error_message.clone(),
    ])
}

fn manifest_tool_text(tools: &[ManifestToolSummary]) -> String {
    join_non_empty(tools.iter().map(|tool| {
        join_non_empty([
            tool.run_id.clone(),
            tool.tool_name.clone(),
            tool.call_id.clone(),
            tool.status.clone(),
            tool.input_preview.clone(),
            tool.output_preview.clone(),
        ])
    }))
}

fn manifest_compaction_text(boundaries: &[ManifestCompactionBoundary]) -> String {
    join_non_empty(boundaries.iter().map(|boundary| {
        join_non_empty([
            boundary.run_id.clone(),
            boundary.transcript_path.clone(),
            boundary.original_tokens.to_string(),
            boundary.compacted_tokens.to_string(),
            boundary.summary.clone(),
        ])
    }))
}

fn group_tool_text_by_run(tools: &[ManifestToolSummary]) -> HashMap<String, String> {
    let mut grouped: HashMap<String, Vec<String>> = HashMap::new();
    for tool in tools {
        let run_id = tool.run_id.trim();
        if run_id.is_empty() {
            continue;
        }
        grouped
            .entry(run_id.to_string())
            .or_default()
            .push(join_non_empty([
                tool.tool_name.clone(),
                tool.call_id.clone(),
                tool.status.clone(),
                tool.input_preview.clone(),
                tool.output_preview.clone(),
            ]));
    }
    grouped
        .into_iter()
        .map(|(run_id, parts)| (run_id, join_non_empty(parts)))
        .collect()
}

fn group_compaction_text_by_run(
    boundaries: &[ManifestCompactionBoundary],
) -> HashMap<String, String> {
    let mut grouped: HashMap<String, Vec<String>> = HashMap::new();
    for boundary in boundaries {
        let run_id = boundary.run_id.trim();
        if run_id.is_empty() {
            continue;
        }
        grouped
            .entry(run_id.to_string())
            .or_default()
            .push(join_non_empty([
                boundary.transcript_path.clone(),
                boundary.original_tokens.to_string(),
                boundary.compacted_tokens.to_string(),
                boundary.summary.clone(),
            ]));
    }
    grouped
        .into_iter()
        .map(|(run_id, parts)| (run_id, join_non_empty(parts)))
        .collect()
}

async fn sqlite_table_exists(pool: &SqlitePool, table_name: &str) -> Result<bool, String> {
    let rows: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?")
            .bind(table_name)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("读取 sqlite schema 失败: {e}"))?;
    Ok(!rows.is_empty())
}

async fn sqlite_table_has_column(
    pool: &SqlitePool,
    table_name: &str,
    column_name: &str,
) -> Result<bool, String> {
    let query = format!("SELECT name FROM pragma_table_info('{table_name}')");
    let rows: Vec<String> = sqlx::query_scalar(&query)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("读取 {table_name} schema 失败: {e}"))?;
    Ok(rows.iter().any(|name| name == column_name))
}

fn collect_json_text_fragments(value: &serde_json::Value, out: &mut Vec<String>) {
    match value {
        serde_json::Value::String(text) => {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                out.push(trimmed.to_string());
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_json_text_fragments(item, out);
            }
        }
        serde_json::Value::Object(map) => {
            for key in ["text", "content", "summary", "output"] {
                if let Some(value) = map.get(key) {
                    collect_json_text_fragments(value, out);
                }
            }
        }
        _ => {}
    }
}

fn render_message_index_text(role: &str, content: &str, content_json: Option<&str>) -> String {
    let mut fragments = Vec::new();
    if let Some(value) =
        content_json.and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
    {
        collect_json_text_fragments(&value, &mut fragments);
    }
    if role == "assistant" {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(content) {
            collect_json_text_fragments(&value, &mut fragments);
        }
    }
    if fragments.is_empty() {
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            fragments.push(trimmed.to_string());
        }
    }
    join_non_empty(fragments)
}

async fn read_session_message_index_text(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<SessionMessageIndexText, String> {
    if !sqlite_table_exists(pool, "messages").await? {
        return Ok(SessionMessageIndexText::default());
    }
    let has_content_json = sqlite_table_has_column(pool, "messages", "content_json").await?;
    let mut user_parts = Vec::new();
    let mut assistant_parts = Vec::new();

    if has_content_json {
        let rows = sqlx::query_as::<_, (String, String, Option<String>)>(
            "SELECT role, content, content_json
             FROM messages
             WHERE session_id = ?
             ORDER BY created_at ASC",
        )
        .bind(session_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("读取 session messages 失败: {e}"))?;
        for (role, content, content_json) in rows {
            let rendered = render_message_index_text(&role, &content, content_json.as_deref());
            if rendered.trim().is_empty() {
                continue;
            }
            if role == "assistant" {
                assistant_parts.push(rendered);
            } else {
                user_parts.push(rendered);
            }
        }
    } else {
        let rows = sqlx::query_as::<_, (String, String)>(
            "SELECT role, content
             FROM messages
             WHERE session_id = ?
             ORDER BY created_at ASC",
        )
        .bind(session_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("读取 legacy session messages 失败: {e}"))?;
        for (role, content) in rows {
            let rendered = render_message_index_text(&role, &content, None);
            if rendered.trim().is_empty() {
                continue;
            }
            if role == "assistant" {
                assistant_parts.push(rendered);
            } else {
                user_parts.push(rendered);
            }
        }
    }

    let user_text = join_non_empty(user_parts);
    let assistant_text = join_non_empty(assistant_parts);
    let combined_text = join_non_empty([user_text.clone(), assistant_text.clone()]);
    Ok(SessionMessageIndexText { combined_text })
}

async fn read_session_transcript_messages(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Vec<SessionTranscriptMessage>, String> {
    if !sqlite_table_exists(pool, "messages").await? {
        return Ok(Vec::new());
    }
    let has_content_json = sqlite_table_has_column(pool, "messages", "content_json").await?;
    let has_session_runs = sqlite_table_exists(pool, "session_runs").await?
        && sqlite_table_has_column(pool, "session_runs", "user_message_id").await?
        && sqlite_table_has_column(pool, "session_runs", "assistant_message_id").await?;

    if has_session_runs && has_content_json {
        let rows = sqlx::query_as::<_, (String, String, Option<String>, String, String)>(
            "SELECT
                m.role,
                m.content,
                m.content_json,
                m.created_at,
                COALESCE(NULLIF(sra.id, ''), NULLIF(sru.id, ''), '') AS run_id
             FROM messages m
             LEFT JOIN session_runs sra ON sra.assistant_message_id = m.id
             LEFT JOIN session_runs sru ON sru.user_message_id = m.id
             WHERE m.session_id = ?
             ORDER BY m.created_at ASC",
        )
        .bind(session_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("读取 transcript mirror messages 失败: {e}"))?;
        return Ok(rows
            .into_iter()
            .map(
                |(role, content, content_json, created_at, run_id)| SessionTranscriptMessage {
                    content: render_message_index_text(&role, &content, content_json.as_deref()),
                    role,
                    created_at,
                    run_id,
                },
            )
            .collect());
    }

    if has_content_json {
        let rows = sqlx::query_as::<_, (String, String, Option<String>, String)>(
            "SELECT role, content, content_json, created_at
             FROM messages
             WHERE session_id = ?
             ORDER BY created_at ASC",
        )
        .bind(session_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("读取 transcript mirror messages 失败: {e}"))?;
        return Ok(rows
            .into_iter()
            .map(
                |(role, content, content_json, created_at)| SessionTranscriptMessage {
                    content: render_message_index_text(&role, &content, content_json.as_deref()),
                    role,
                    created_at,
                    run_id: String::new(),
                },
            )
            .collect());
    }

    let rows = sqlx::query_as::<_, (String, String, String)>(
        "SELECT role, content, created_at
         FROM messages
         WHERE session_id = ?
         ORDER BY created_at ASC",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("读取 legacy transcript mirror messages 失败: {e}"))?;
    Ok(rows
        .into_iter()
        .map(|(role, content, created_at)| SessionTranscriptMessage {
            content: render_message_index_text(&role, &content, None),
            role,
            created_at,
            run_id: String::new(),
        })
        .collect())
}

fn render_profile_session_transcript_mirror(
    manifest: &ProfileSessionManifestRecord,
    messages: &[SessionTranscriptMessage],
) -> String {
    let mut out = String::new();
    out.push_str("# Profile Session Transcript\n\n");
    out.push_str(&format!("- profile_id: {}\n", manifest.profile_id.trim()));
    out.push_str(&format!("- session_id: {}\n", manifest.session_id.trim()));
    out.push_str(&format!("- skill_id: {}\n", manifest.skill_id.trim()));
    out.push_str(&format!("- work_dir: {}\n", manifest.work_dir.trim()));
    out.push_str(&format!("- source: {}\n", manifest.source.trim()));
    out.push_str(&format!("- updated_at: {}\n", manifest.updated_at.trim()));
    if let Some(run) = manifest.run_summary.as_ref() {
        out.push_str(&format!("- latest_run_id: {}\n", run.latest_run_id.trim()));
        out.push_str(&format!("- run_status: {}\n", run.status.trim()));
    }
    out.push('\n');

    out.push_str("## Messages\n\n");
    if messages.is_empty() {
        out.push_str("_No DB messages available._\n\n");
    }
    for message in messages {
        let role = if message.role == "assistant" {
            "assistant"
        } else {
            "user"
        };
        out.push_str(&format!("### {} ({})\n\n", role, message.created_at.trim()));
        if !message.run_id.trim().is_empty() {
            out.push_str(&format!("- run_id: {}\n\n", message.run_id.trim()));
        }
        out.push_str(message.content.trim());
        out.push_str("\n\n");
    }

    out.push_str("## Tool Summaries\n\n");
    if manifest.tool_summaries.is_empty() {
        out.push_str("_No tool summaries._\n\n");
    }
    for tool in &manifest.tool_summaries {
        out.push_str(&format!(
            "- run_id: {} | tool: {} | call_id: {} | status: {}\n",
            tool.run_id.trim(),
            tool.tool_name.trim(),
            tool.call_id.trim(),
            tool.status.trim()
        ));
        if !tool.input_preview.trim().is_empty() {
            out.push_str(&format!("  - input: {}\n", tool.input_preview.trim()));
        }
        if !tool.output_preview.trim().is_empty() {
            out.push_str(&format!("  - output: {}\n", tool.output_preview.trim()));
        }
    }
    out.push('\n');

    out.push_str("## Compaction Boundaries\n\n");
    if manifest.compaction_boundaries.is_empty() {
        out.push_str("_No compaction boundaries._\n");
    }
    for boundary in &manifest.compaction_boundaries {
        out.push_str(&format!(
            "- run_id: {} | tokens: {} -> {} | transcript: {}\n",
            boundary.run_id.trim(),
            boundary.original_tokens,
            boundary.compacted_tokens,
            boundary.transcript_path.trim()
        ));
        if !boundary.summary.trim().is_empty() {
            out.push_str(&format!("  - summary: {}\n", boundary.summary.trim()));
        }
    }

    out
}

async fn write_profile_session_transcript_mirror(
    pool: &SqlitePool,
    manifest_path: &Path,
    manifest: &ProfileSessionManifestRecord,
) -> Result<(), String> {
    let Some(parent) = manifest_path.parent() else {
        return Ok(());
    };
    let messages = read_session_transcript_messages(pool, manifest.session_id.trim()).await?;
    let transcript = render_profile_session_transcript_mirror(manifest, &messages);
    std::fs::write(parent.join("transcript.md"), transcript)
        .map_err(|e| format!("写入 profile session transcript mirror 失败: {e}"))?;
    Ok(())
}

async fn read_run_level_index_documents(
    pool: &SqlitePool,
    session_id: &str,
    tool_summaries: &[ManifestToolSummary],
    compaction_boundaries: &[ManifestCompactionBoundary],
) -> Result<Vec<RunLevelIndexDocument>, String> {
    if !sqlite_table_exists(pool, "session_runs").await? {
        return Ok(Vec::new());
    }
    for column in [
        "user_message_id",
        "assistant_message_id",
        "buffered_text",
        "error_kind",
        "error_message",
        "created_at",
    ] {
        if !sqlite_table_has_column(pool, "session_runs", column).await? {
            return Ok(Vec::new());
        }
    }

    let tool_text_by_run = group_tool_text_by_run(tool_summaries);
    let compaction_text_by_run = group_compaction_text_by_run(compaction_boundaries);
    let has_messages = sqlite_table_exists(pool, "messages").await?;
    let has_content_json = if has_messages {
        sqlite_table_has_column(pool, "messages", "content_json").await?
    } else {
        false
    };

    if has_messages && has_content_json {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                Option<String>,
                String,
                String,
                Option<String>,
            ),
        >(
            "SELECT
                sr.id,
                sr.status,
                sr.buffered_text,
                sr.error_kind,
                sr.error_message,
                COALESCE(um.role, ''),
                COALESCE(um.content, ''),
                um.content_json,
                COALESCE(am.role, ''),
                COALESCE(am.content, ''),
                am.content_json
             FROM session_runs sr
             LEFT JOIN messages um ON um.id = sr.user_message_id
             LEFT JOIN messages am ON am.id = sr.assistant_message_id
             WHERE sr.session_id = ?
             ORDER BY sr.created_at ASC",
        )
        .bind(session_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("读取 run-level session index 失败: {e}"))?;

        return Ok(rows
            .into_iter()
            .map(
                |(
                    run_id,
                    run_status,
                    buffered_text,
                    error_kind,
                    error_message,
                    user_role,
                    user_content,
                    user_content_json,
                    assistant_role,
                    assistant_content,
                    assistant_content_json,
                )| {
                    let user_text = render_message_index_text(
                        &user_role,
                        &user_content,
                        user_content_json.as_deref(),
                    );
                    let assistant_text = render_message_index_text(
                        &assistant_role,
                        &assistant_content,
                        assistant_content_json.as_deref(),
                    );
                    let text = join_non_empty([
                        run_id.clone(),
                        run_status.clone(),
                        buffered_text,
                        error_kind,
                        error_message,
                        user_text,
                        assistant_text,
                        tool_text_by_run.get(&run_id).cloned().unwrap_or_default(),
                        compaction_text_by_run
                            .get(&run_id)
                            .cloned()
                            .unwrap_or_default(),
                    ]);
                    RunLevelIndexDocument { run_id, text }
                },
            )
            .filter(|doc| !doc.text.trim().is_empty())
            .collect());
    }

    if has_messages {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                String,
            ),
        >(
            "SELECT
                sr.id,
                sr.status,
                sr.buffered_text,
                sr.error_kind,
                sr.error_message,
                COALESCE(um.role, ''),
                COALESCE(um.content, ''),
                COALESCE(am.role, ''),
                COALESCE(am.content, '')
             FROM session_runs sr
             LEFT JOIN messages um ON um.id = sr.user_message_id
             LEFT JOIN messages am ON am.id = sr.assistant_message_id
             WHERE sr.session_id = ?
             ORDER BY sr.created_at ASC",
        )
        .bind(session_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("读取 legacy run-level session index 失败: {e}"))?;

        return Ok(rows
            .into_iter()
            .map(
                |(
                    run_id,
                    run_status,
                    buffered_text,
                    error_kind,
                    error_message,
                    user_role,
                    user_content,
                    assistant_role,
                    assistant_content,
                )| {
                    let user_text = render_message_index_text(&user_role, &user_content, None);
                    let assistant_text =
                        render_message_index_text(&assistant_role, &assistant_content, None);
                    let text = join_non_empty([
                        run_id.clone(),
                        run_status.clone(),
                        buffered_text,
                        error_kind,
                        error_message,
                        user_text,
                        assistant_text,
                        tool_text_by_run.get(&run_id).cloned().unwrap_or_default(),
                        compaction_text_by_run
                            .get(&run_id)
                            .cloned()
                            .unwrap_or_default(),
                    ]);
                    RunLevelIndexDocument { run_id, text }
                },
            )
            .filter(|doc| !doc.text.trim().is_empty())
            .collect());
    }

    let rows = sqlx::query_as::<_, (String, String, String, String, String)>(
        "SELECT id, status, buffered_text, error_kind, error_message
         FROM session_runs
         WHERE session_id = ?
         ORDER BY created_at ASC",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("读取无消息 run-level session index 失败: {e}"))?;

    Ok(rows
        .into_iter()
        .map(
            |(run_id, run_status, buffered_text, error_kind, error_message)| {
                let text = join_non_empty([
                    run_id.clone(),
                    run_status.clone(),
                    buffered_text,
                    error_kind,
                    error_message,
                    tool_text_by_run.get(&run_id).cloned().unwrap_or_default(),
                    compaction_text_by_run
                        .get(&run_id)
                        .cloned()
                        .unwrap_or_default(),
                ]);
                RunLevelIndexDocument { run_id, text }
            },
        )
        .filter(|doc| !doc.text.trim().is_empty())
        .collect())
}

fn escape_fts_query(query: &str) -> String {
    query
        .split_whitespace()
        .map(|term| format!("\"{}\"", term.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" ")
}

type SearchRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    i64,
    i64,
    String,
    String,
    String,
);

fn filter_value(value: &Option<String>) -> Option<&str> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn push_profile_session_filters<'a>(
    builder: &mut QueryBuilder<'a, Sqlite>,
    filters: &'a ProfileSessionSearchFilters,
    table_alias: &'static str,
) {
    if let Some(work_dir) = filter_value(&filters.work_dir) {
        builder
            .push(" AND ")
            .push(table_alias)
            .push(".work_dir = ")
            .push_bind(work_dir);
    }
    if let Some(updated_after) = filter_value(&filters.updated_after) {
        builder
            .push(" AND ")
            .push(table_alias)
            .push(".updated_at >= ")
            .push_bind(updated_after);
    }
    if let Some(updated_before) = filter_value(&filters.updated_before) {
        builder
            .push(" AND ")
            .push(table_alias)
            .push(".updated_at <= ")
            .push_bind(updated_before);
    }
    if let Some(skill_id) = filter_value(&filters.skill_id) {
        builder
            .push(" AND ")
            .push(table_alias)
            .push(".skill_id = ")
            .push_bind(skill_id);
    }
    if let Some(source) = filter_value(&filters.source) {
        builder
            .push(" AND ")
            .push(table_alias)
            .push(".source = ")
            .push_bind(source);
    }
}

fn search_rows_to_results(rows: Vec<SearchRow>) -> Vec<ProfileSessionSearchResult> {
    rows.into_iter()
        .map(
            |(
                profile_id,
                session_id,
                skill_id,
                work_dir,
                source,
                run_status,
                latest_run_id,
                document_kind,
                matched_run_id,
                tool_summary_count,
                compaction_boundary_count,
                manifest_path,
                updated_at,
                snippet,
            )| ProfileSessionSearchResult {
                profile_id,
                session_id,
                skill_id,
                work_dir,
                source,
                run_status,
                latest_run_id,
                document_kind,
                matched_run_id,
                tool_summary_count,
                compaction_boundary_count,
                manifest_path,
                updated_at,
                snippet,
            },
        )
        .collect()
}

pub async fn index_profile_session_manifest_with_pool(
    pool: &SqlitePool,
    manifest_path: &Path,
) -> Result<(), String> {
    ensure_profile_session_index_schema_with_pool(pool).await?;

    let raw = std::fs::read_to_string(manifest_path)
        .map_err(|e| format!("读取 profile session manifest 失败: {e}"))?;
    let manifest: ProfileSessionManifestRecord = serde_json::from_str(&raw)
        .map_err(|e| format!("解析 profile session manifest 失败: {e}"))?;
    let profile_id = manifest.profile_id.trim();
    let session_id = manifest.session_id.trim();
    if profile_id.is_empty() || session_id.is_empty() {
        return Err("profile session manifest 缺少 profile_id 或 session_id".to_string());
    }

    let run_text = manifest_run_text(manifest.run_summary.as_ref());
    let tool_text = manifest_tool_text(&manifest.tool_summaries);
    let compaction_text = manifest_compaction_text(&manifest.compaction_boundaries);
    let message_text = read_session_message_index_text(pool, session_id).await?;
    let run_documents = read_run_level_index_documents(
        pool,
        session_id,
        &manifest.tool_summaries,
        &manifest.compaction_boundaries,
    )
    .await?;
    write_profile_session_transcript_mirror(pool, manifest_path, &manifest).await?;
    let search_text = join_non_empty([
        manifest.skill_id.clone(),
        manifest.work_dir.clone(),
        run_text.clone(),
        tool_text.clone(),
        compaction_text.clone(),
        message_text.combined_text.clone(),
    ]);
    let run_status = manifest
        .run_summary
        .as_ref()
        .map(|run| run.status.trim().to_string())
        .unwrap_or_default();
    let latest_run_id = manifest
        .run_summary
        .as_ref()
        .map(|run| run.latest_run_id.trim().to_string())
        .unwrap_or_default();
    let updated_at = if manifest.updated_at.trim().is_empty() {
        chrono::Utc::now().to_rfc3339()
    } else {
        manifest.updated_at
    };

    sqlx::query(
        "INSERT INTO profile_session_index (
            profile_id,
            session_id,
            skill_id,
            work_dir,
            source,
            run_status,
            latest_run_id,
            tool_summary_count,
            compaction_boundary_count,
            manifest_path,
            search_text,
            updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(profile_id, session_id) DO UPDATE SET
            skill_id = excluded.skill_id,
            work_dir = excluded.work_dir,
            source = excluded.source,
            run_status = excluded.run_status,
            latest_run_id = excluded.latest_run_id,
            tool_summary_count = excluded.tool_summary_count,
            compaction_boundary_count = excluded.compaction_boundary_count,
            manifest_path = excluded.manifest_path,
            search_text = excluded.search_text,
            updated_at = excluded.updated_at",
    )
    .bind(profile_id)
    .bind(session_id)
    .bind(manifest.skill_id.trim())
    .bind(manifest.work_dir.trim())
    .bind(manifest.source.trim())
    .bind(&run_status)
    .bind(&latest_run_id)
    .bind(manifest.tool_summaries.len() as i64)
    .bind(manifest.compaction_boundaries.len() as i64)
    .bind(manifest_path.to_string_lossy().to_string())
    .bind(&search_text)
    .bind(&updated_at)
    .execute(pool)
    .await
    .map_err(|e| format!("写入 profile_session_index 失败: {e}"))?;

    sqlx::query("DELETE FROM profile_session_fts WHERE profile_id = ? AND session_id = ?")
        .bind(profile_id)
        .bind(session_id)
        .execute(pool)
        .await
        .map_err(|e| format!("清理 profile_session_fts 旧记录失败: {e}"))?;
    sqlx::query(
        "INSERT INTO profile_session_fts (
            profile_id,
            session_id,
            document_kind,
            run_id,
            skill_id,
            work_dir,
            run_text,
            tool_text,
            compaction_text,
            message_text
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(profile_id)
    .bind(session_id)
    .bind("session")
    .bind(&latest_run_id)
    .bind(manifest.skill_id.trim())
    .bind(manifest.work_dir.trim())
    .bind(&run_text)
    .bind(&tool_text)
    .bind(&compaction_text)
    .bind("")
    .execute(pool)
    .await
    .map_err(|e| format!("写入 profile_session_fts 失败: {e}"))?;

    for run_document in run_documents {
        sqlx::query(
            "INSERT INTO profile_session_fts (
                profile_id,
                session_id,
                document_kind,
                run_id,
                skill_id,
                work_dir,
                run_text,
                tool_text,
                compaction_text,
                message_text
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(profile_id)
        .bind(session_id)
        .bind("run")
        .bind(&run_document.run_id)
        .bind(manifest.skill_id.trim())
        .bind(manifest.work_dir.trim())
        .bind(&run_document.text)
        .bind("")
        .bind("")
        .bind(&run_document.text)
        .execute(pool)
        .await
        .map_err(|e| format!("写入 run-level profile_session_fts 失败: {e}"))?;
    }

    Ok(())
}

pub async fn refresh_profile_session_index_for_session_with_pool(
    pool: &SqlitePool,
    runtime_root: &Path,
    session_id: &str,
    source: &str,
) -> Result<Option<PathBuf>, String> {
    let session_id = session_id.trim();
    if session_id.is_empty() || !sqlite_table_exists(pool, "sessions").await? {
        return Ok(None);
    }
    if !sqlite_table_has_column(pool, "sessions", "profile_id").await? {
        return Ok(None);
    }
    let has_work_dir = sqlite_table_has_column(pool, "sessions", "work_dir").await?;
    let work_dir_select = if has_work_dir {
        "COALESCE(work_dir, '')"
    } else {
        "''"
    };
    let query = format!(
        "SELECT COALESCE(profile_id, ''), COALESCE(skill_id, ''), {work_dir_select}
         FROM sessions
         WHERE id = ?"
    );
    let row = sqlx::query_as::<_, (String, String, String)>(&query)
        .bind(session_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("读取 session profile index 输入失败: {e}"))?;
    let Some((profile_id, skill_id, work_dir)) = row else {
        return Ok(None);
    };
    if profile_id.trim().is_empty() {
        return Ok(None);
    }
    let manifest_path = super::runtime_support::write_profile_session_manifest(
        runtime_root,
        super::runtime_support::ProfileSessionManifestInput {
            profile_id: profile_id.trim(),
            session_id,
            skill_id: skill_id.trim(),
            work_dir: Some(work_dir.trim()).filter(|value| !value.is_empty()),
            source,
        },
    )?;
    index_profile_session_manifest_with_pool(pool, &manifest_path).await?;
    Ok(Some(manifest_path))
}

pub async fn search_profile_session_index_with_pool(
    pool: &SqlitePool,
    profile_id: &str,
    query: &str,
    limit: i64,
) -> Result<Vec<ProfileSessionSearchResult>, String> {
    search_profile_session_index_with_filters_with_pool(
        pool,
        profile_id,
        query,
        limit,
        ProfileSessionSearchFilters::default(),
    )
    .await
}

pub async fn search_profile_session_index_with_filters_with_pool(
    pool: &SqlitePool,
    profile_id: &str,
    query: &str,
    limit: i64,
    filters: ProfileSessionSearchFilters,
) -> Result<Vec<ProfileSessionSearchResult>, String> {
    ensure_profile_session_index_schema_with_pool(pool).await?;
    let profile_id = profile_id.trim();
    if profile_id.is_empty() {
        return Ok(Vec::new());
    }
    let limit = limit.clamp(1, 50);
    let raw_query = query.trim();
    let query = escape_fts_query(raw_query);

    if query.is_empty() {
        let mut builder = QueryBuilder::new(
            "SELECT
                profile_id,
                session_id,
                skill_id,
                work_dir,
                source,
                run_status,
                latest_run_id,
                'session' AS document_kind,
                latest_run_id AS matched_run_id,
                tool_summary_count,
                compaction_boundary_count,
                manifest_path,
                updated_at,
                search_text AS snippet
             FROM profile_session_index
             WHERE profile_id = ",
        );
        builder.push_bind(profile_id);
        push_profile_session_filters(&mut builder, &filters, "profile_session_index");
        builder.push(" ORDER BY updated_at DESC LIMIT ");
        builder.push_bind(limit);
        let rows = builder
            .build_query_as::<SearchRow>()
            .fetch_all(pool)
            .await
            .map_err(|e| format!("查询 profile_session_index 失败: {e}"))?;
        return Ok(search_rows_to_results(rows));
    }

    let mut builder = QueryBuilder::new(
        "SELECT
            i.profile_id,
            i.session_id,
            i.skill_id,
            i.work_dir,
            i.source,
            i.run_status,
            i.latest_run_id,
            profile_session_fts.document_kind,
            profile_session_fts.run_id AS matched_run_id,
            i.tool_summary_count,
            i.compaction_boundary_count,
            i.manifest_path,
            i.updated_at,
            CASE
                WHEN profile_session_fts.document_kind = 'run'
                THEN profile_session_fts.message_text
                ELSE snippet(profile_session_fts, -1, '[', ']', '...', 24)
            END AS snippet
         FROM profile_session_fts
         JOIN profile_session_index i
           ON i.profile_id = profile_session_fts.profile_id
          AND i.session_id = profile_session_fts.session_id
         WHERE profile_session_fts MATCH ",
    );
    builder.push_bind(&query);
    builder.push(" AND profile_session_fts.profile_id = ");
    builder.push_bind(profile_id);
    push_profile_session_filters(&mut builder, &filters, "i");
    builder.push(" ORDER BY bm25(profile_session_fts), i.updated_at DESC LIMIT ");
    builder.push_bind(limit);
    let mut rows = builder
        .build_query_as::<SearchRow>()
        .fetch_all(pool)
        .await
        .map_err(|e| format!("搜索 profile_session_fts 失败: {e}"))?;

    if rows.is_empty() {
        let like_pattern = format!("%{raw_query}%");
        let mut builder = QueryBuilder::new(
            "SELECT
                profile_id,
                session_id,
                skill_id,
                work_dir,
                source,
                run_status,
                latest_run_id,
                'session' AS document_kind,
                latest_run_id AS matched_run_id,
                tool_summary_count,
                compaction_boundary_count,
                manifest_path,
                updated_at,
                search_text AS snippet
             FROM profile_session_index
             WHERE profile_id = ",
        );
        builder.push_bind(profile_id);
        builder.push(" AND search_text LIKE ");
        builder.push_bind(like_pattern);
        push_profile_session_filters(&mut builder, &filters, "profile_session_index");
        builder.push(" ORDER BY updated_at DESC LIMIT ");
        builder.push_bind(limit);
        rows = builder
            .build_query_as::<SearchRow>()
            .fetch_all(pool)
            .await
            .map_err(|e| format!("回退查询 profile_session_index 失败: {e}"))?;
    }

    Ok(search_rows_to_results(rows))
}
