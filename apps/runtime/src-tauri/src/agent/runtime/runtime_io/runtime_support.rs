use crate::agent::AgentExecutor;
use chrono::Utc;
use serde::Serialize;
use sha2::{Digest, Sha256};

fn read_memory_candidate(candidate: &std::path::Path) -> Option<(std::path::PathBuf, String)> {
    let memory_file = candidate_memory_file(candidate);
    if memory_file.exists() {
        Some((
            memory_file.clone(),
            std::fs::read_to_string(memory_file).unwrap_or_default(),
        ))
    } else {
        None
    }
}

pub(crate) fn resolve_tool_name_list(
    allowed_tools: &Option<Vec<String>>,
    agent_executor: &AgentExecutor,
) -> Vec<String> {
    match allowed_tools {
        Some(whitelist) => whitelist.clone(),
        None => agent_executor
            .registry()
            .get_tool_definitions()
            .iter()
            .filter_map(|t| t["name"].as_str().map(String::from))
            .collect(),
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn resolve_tool_names(
    allowed_tools: &Option<Vec<String>>,
    agent_executor: &AgentExecutor,
) -> String {
    resolve_tool_name_list(allowed_tools, agent_executor).join(", ")
}
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub struct ProfileMemoryLocator {
    pub profile_memory_dir: Option<std::path::PathBuf>,
    pub project_memory_file: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileMemoryBundle {
    pub content: String,
    pub source: &'static str,
    pub source_path: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub struct ProfileMemoryStatus {
    pub profile_memory_dir: Option<std::path::PathBuf>,
    pub profile_memory_file_path: Option<std::path::PathBuf>,
    pub profile_memory_file_exists: bool,
    pub active_source: &'static str,
    pub active_source_path: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfileSessionManifestInput<'a> {
    pub profile_id: &'a str,
    pub session_id: &'a str,
    pub skill_id: &'a str,
    pub work_dir: Option<&'a str>,
    pub source: &'a str,
}

#[derive(Debug, Serialize)]
struct ProfileSessionManifest<'a> {
    version: u32,
    profile_id: &'a str,
    session_id: &'a str,
    skill_id: &'a str,
    work_dir: &'a str,
    source: &'a str,
    journal_dir: String,
    events_path: String,
    state_path: String,
    transcript_path: String,
    run_summary: ProfileSessionRunSummary,
    tool_summaries: Vec<ProfileSessionToolSummary>,
    compaction_boundaries: Vec<ProfileSessionCompactionBoundary>,
    updated_at: String,
}

#[derive(Debug, Serialize)]
struct ProfileSessionRunSummary {
    status: String,
    latest_run_id: String,
    user_message_id: String,
    buffered_text_preview: String,
    last_error_kind: String,
    last_error_message: String,
}

#[derive(Debug, Serialize)]
struct ProfileSessionToolSummary {
    run_id: String,
    tool_name: String,
    call_id: String,
    status: String,
    is_error: bool,
    input_preview: String,
    output_preview: String,
}

#[derive(Debug, Serialize)]
struct ProfileSessionCompactionBoundary {
    run_id: String,
    transcript_path: String,
    original_tokens: usize,
    compacted_tokens: usize,
    summary: String,
}

fn candidate_memory_file(candidate: &std::path::Path) -> std::path::PathBuf {
    if candidate.is_dir() {
        candidate.join("MEMORY.md")
    } else {
        candidate.to_path_buf()
    }
}

const DEFAULT_PROFILE_MEMORY_BUDGET_CHARS: usize = 12_000;

fn normalize_project_memory_key(work_dir: &std::path::Path) -> Option<String> {
    let normalized = work_dir
        .to_string_lossy()
        .trim()
        .replace('\\', "/")
        .to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }
    let digest = Sha256::digest(normalized.as_bytes());
    Some(format!("{:x}", digest)[..16].to_string())
}

fn trim_memory_to_budget(content: String, budget_chars: usize) -> String {
    if budget_chars == 0 || content.chars().count() <= budget_chars {
        return content;
    }
    let tail = content
        .chars()
        .rev()
        .take(budget_chars)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>();
    format!("[Memory truncated to last {budget_chars} chars]\n{tail}")
}

fn trim_preview(content: &str, budget_chars: usize) -> String {
    if content.chars().count() <= budget_chars {
        return content.to_string();
    }
    content.chars().take(budget_chars).collect()
}

fn read_profile_session_run_summary(state_path: &std::path::Path) -> ProfileSessionRunSummary {
    let default = ProfileSessionRunSummary {
        status: "pending".to_string(),
        latest_run_id: String::new(),
        user_message_id: String::new(),
        buffered_text_preview: String::new(),
        last_error_kind: String::new(),
        last_error_message: String::new(),
    };
    let Ok(raw) = std::fs::read_to_string(state_path) else {
        return default;
    };
    let Ok(state) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return default;
    };
    let Some(runs) = state.get("runs").and_then(serde_json::Value::as_array) else {
        return default;
    };
    let current_run_id = state
        .get("current_run_id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let selected = if current_run_id.trim().is_empty() {
        runs.last()
    } else {
        runs.iter()
            .find(|run| {
                run.get("run_id").and_then(serde_json::Value::as_str) == Some(current_run_id)
            })
            .or_else(|| runs.last())
    };
    let Some(run) = selected else {
        return default;
    };
    ProfileSessionRunSummary {
        status: run
            .get("status")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("pending")
            .to_string(),
        latest_run_id: run
            .get("run_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        user_message_id: run
            .get("user_message_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        buffered_text_preview: trim_preview(
            run.get("buffered_text")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default(),
            500,
        ),
        last_error_kind: run
            .get("last_error_kind")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        last_error_message: run
            .get("last_error_message")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
    }
}

fn read_profile_session_tool_summaries(
    events_path: &std::path::Path,
) -> Vec<ProfileSessionToolSummary> {
    let Ok(raw) = std::fs::read_to_string(events_path) else {
        return Vec::new();
    };
    raw.lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter_map(|record| {
            let event = record.get("event")?;
            if event.get("type").and_then(serde_json::Value::as_str) != Some("tool_completed") {
                return None;
            }
            let is_error = event
                .get("is_error")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            let input_preview = event
                .get("input")
                .map(|input| serde_json::to_string(input).unwrap_or_default())
                .unwrap_or_default();
            Some(ProfileSessionToolSummary {
                run_id: event
                    .get("run_id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                tool_name: event
                    .get("tool_name")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                call_id: event
                    .get("call_id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                status: if is_error { "error" } else { "completed" }.to_string(),
                is_error,
                input_preview: trim_preview(&input_preview, 500),
                output_preview: trim_preview(
                    event
                        .get("output")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default(),
                    500,
                ),
            })
        })
        .take(64)
        .collect()
}

fn read_profile_session_compaction_boundaries(
    state_path: &std::path::Path,
) -> Vec<ProfileSessionCompactionBoundary> {
    let Ok(raw) = std::fs::read_to_string(state_path) else {
        return Vec::new();
    };
    let Ok(state) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return Vec::new();
    };
    state
        .get("runs")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|run| {
            let boundary = run.get("turn_state")?.get("compaction_boundary")?;
            Some(ProfileSessionCompactionBoundary {
                run_id: run
                    .get("run_id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                transcript_path: boundary
                    .get("transcript_path")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                original_tokens: boundary
                    .get("original_tokens")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or_default() as usize,
                compacted_tokens: boundary
                    .get("compacted_tokens")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or_default() as usize,
                summary: trim_preview(
                    boundary
                        .get("summary")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default(),
                    1_000,
                ),
            })
        })
        .take(16)
        .collect()
}

#[cfg_attr(not(test), allow(dead_code))]
fn sanitize_profile_path_component(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut out = String::new();
    let mut prev_sep = false;
    for ch in trimmed.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
            prev_sep = false;
            continue;
        }
        if !prev_sep {
            out.push('_');
            prev_sep = true;
        }
    }
    let normalized = out.trim_matches('_').to_string();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

pub fn write_profile_session_manifest(
    runtime_root: &std::path::Path,
    input: ProfileSessionManifestInput<'_>,
) -> Result<std::path::PathBuf, String> {
    let profile_id = sanitize_profile_path_component(input.profile_id)
        .ok_or_else(|| "profile_id 不能为空".to_string())?;
    let session_id = sanitize_profile_path_component(input.session_id)
        .ok_or_else(|| "session_id 不能为空".to_string())?;
    let manifest_dir = runtime_root
        .join("profiles")
        .join(profile_id)
        .join("sessions")
        .join(session_id);
    std::fs::create_dir_all(&manifest_dir)
        .map_err(|err| format!("创建 profile session manifest 目录失败: {err}"))?;
    let manifest_path = manifest_dir.join("manifest.json");
    let journal_dir = runtime_root.join("sessions").join(input.session_id.trim());
    let events_path = journal_dir.join("events.jsonl");
    let state_path = journal_dir.join("state.json");
    let transcript_path = journal_dir.join("transcript.md");
    let manifest = ProfileSessionManifest {
        version: 1,
        profile_id: input.profile_id.trim(),
        session_id: input.session_id.trim(),
        skill_id: input.skill_id.trim(),
        work_dir: input.work_dir.unwrap_or("").trim(),
        source: input.source.trim(),
        journal_dir: journal_dir.to_string_lossy().to_string(),
        events_path: events_path.to_string_lossy().to_string(),
        state_path: state_path.to_string_lossy().to_string(),
        transcript_path: transcript_path.to_string_lossy().to_string(),
        run_summary: read_profile_session_run_summary(&state_path),
        tool_summaries: read_profile_session_tool_summaries(&events_path),
        compaction_boundaries: read_profile_session_compaction_boundaries(&state_path),
        updated_at: Utc::now().to_rfc3339(),
    };
    let raw = serde_json::to_string_pretty(&manifest)
        .map_err(|err| format!("序列化 profile session manifest 失败: {err}"))?;
    std::fs::write(&manifest_path, raw)
        .map_err(|err| format!("写入 profile session manifest 失败: {err}"))?;
    Ok(manifest_path)
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn build_profile_memory_locator(
    runtime_root: &std::path::Path,
    memory_root: &std::path::Path,
    work_dir: Option<&std::path::Path>,
    skill_id: &str,
    employee_id: &str,
    profile_id: Option<&str>,
    im_role_id: Option<&str>,
) -> ProfileMemoryLocator {
    let _ = (memory_root, skill_id, employee_id, im_role_id);
    let profile_memory_dir =
        profile_id
            .and_then(sanitize_profile_path_component)
            .map(|profile_id| {
                runtime_root
                    .join("profiles")
                    .join(profile_id)
                    .join("memories")
            });
    let project_memory_file = profile_memory_dir.as_ref().and_then(|profile_memory_dir| {
        let work_dir = work_dir?;
        let project_key = normalize_project_memory_key(work_dir)?;
        Some(
            profile_memory_dir
                .join("PROJECTS")
                .join(format!("{project_key}.md")),
            )
    });

    ProfileMemoryLocator {
        profile_memory_dir,
        project_memory_file,
    }
}

pub fn load_profile_memory_bundle(locator: &ProfileMemoryLocator) -> ProfileMemoryBundle {
    load_profile_memory_bundle_with_budget(locator, DEFAULT_PROFILE_MEMORY_BUDGET_CHARS)
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn load_profile_memory_bundle_with_budget(
    locator: &ProfileMemoryLocator,
    budget_chars: usize,
) -> ProfileMemoryBundle {
    if let Some(profile_memory_dir) = &locator.profile_memory_dir {
        if let Some((source_path, content)) = read_memory_candidate(profile_memory_dir) {
            let mut sections = vec![content];
            if let Some(project_memory_file) = &locator.project_memory_file {
                if let Some((_project_path, project_content)) =
                    read_memory_candidate(project_memory_file)
                {
                    if !project_content.trim().is_empty() {
                        sections.push(format!("Project Memory:\n{project_content}"));
                    }
                }
            }
            return ProfileMemoryBundle {
                content: trim_memory_to_budget(sections.join("\n\n"), budget_chars),
                source: "profile",
                source_path: Some(source_path),
            };
        }
    }

    ProfileMemoryBundle {
        content: String::new(),
        source: "none",
        source_path: None,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn collect_profile_memory_status(locator: &ProfileMemoryLocator) -> ProfileMemoryStatus {
    let profile_memory_file_path = locator
        .profile_memory_dir
        .as_deref()
        .map(candidate_memory_file);
    let profile_memory_file_exists = profile_memory_file_path
        .as_deref()
        .is_some_and(std::path::Path::exists);

    let (active_source, active_source_path) = if profile_memory_file_exists {
        ("profile", profile_memory_file_path.clone())
    } else {
        ("none", None)
    };

    ProfileMemoryStatus {
        profile_memory_dir: locator.profile_memory_dir.clone(),
        profile_memory_file_path,
        profile_memory_file_exists,
        active_source,
        active_source_path,
    }
}

pub(crate) fn tool_ctx_from_work_dir(work_dir: &str) -> Option<std::path::PathBuf> {
    if work_dir.trim().is_empty() {
        None
    } else {
        Some(std::path::PathBuf::from(work_dir))
    }
}
