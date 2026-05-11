use crate::commands::employee_agents::{AgentEmployee, list_agent_employees_with_pool};
use crate::commands::runtime_preferences::resolve_default_work_dir_with_pool;
use crate::commands::skills::DbState;
use sqlx::SqlitePool;
use std::io::{Read, Write};
use std::path::PathBuf;
use tauri::State;
use walkdir::WalkDir;
use zip::write::FileOptions;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct AgentProfileAnswerInput {
    pub key: String,
    pub question: String,
    pub answer: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct AgentProfilePayload {
    pub employee_db_id: String,
    pub answers: Vec<AgentProfileAnswerInput>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct AgentProfileDraft {
    pub employee_id: String,
    pub employee_name: String,
    pub agents_md: String,
    pub soul_md: String,
    pub user_md: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct AgentProfileFileResult {
    pub path: String,
    pub ok: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ApplyAgentProfileResult {
    pub files: Vec<AgentProfileFileResult>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct AgentProfileFileView {
    pub name: String,
    pub path: String,
    pub exists: bool,
    pub content: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct AgentProfileFilesView {
    pub employee_id: String,
    pub employee_name: String,
    pub profile_dir: String,
    pub artifacts: Vec<AgentProfileArtifactStatus>,
    pub files: Vec<AgentProfileFileView>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct AgentProfileArtifactStatus {
    pub name: String,
    pub path: String,
    pub exists: bool,
    pub file_count: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct AgentProfileExportResult {
    pub employee_id: String,
    pub employee_name: String,
    pub profile_id: String,
    pub profile_dir: String,
    pub export_path: String,
    pub file_count: usize,
    pub total_bytes: u64,
}

fn normalized_answer(answers: &[AgentProfileAnswerInput], key: &str) -> String {
    answers
        .iter()
        .find(|item| item.key.trim().eq_ignore_ascii_case(key))
        .map(|item| item.answer.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_default()
}

fn render_markdown(
    employee: &AgentEmployee,
    answers: &[AgentProfileAnswerInput],
) -> AgentProfileDraft {
    let mission = normalized_answer(answers, "mission");
    let responsibilities = normalized_answer(answers, "responsibilities");
    let collaboration = normalized_answer(answers, "collaboration");
    let tone = normalized_answer(answers, "tone");
    let boundaries = normalized_answer(answers, "boundaries");
    let user_profile = normalized_answer(answers, "user_profile");

    let enabled_scope_text = {
        let scopes = employee
            .enabled_scopes
            .iter()
            .map(|scope| scope.trim().to_lowercase())
            .filter(|scope| !scope.is_empty())
            .collect::<Vec<_>>();
        if scopes.is_empty() {
            "app".to_string()
        } else {
            scopes.join(", ")
        }
    };

    let agents_md = format!(
        "# RULES\n\n## Agent\n- 名称: {name}\n- 员工编号: {employee_id}\n- 适用范围: {enabled_scopes}\n\n## Mission\n{mission}\n\n## Responsibilities\n{responsibilities}\n\n## Collaboration\n{collaboration}\n",
        name = employee.name,
        employee_id = employee.employee_id,
        enabled_scopes = enabled_scope_text,
        mission = if mission.is_empty() {
            "请补充该员工的核心使命。"
        } else {
            mission.as_str()
        },
        responsibilities = if responsibilities.is_empty() {
            "请补充该员工的关键职责。"
        } else {
            responsibilities.as_str()
        },
        collaboration = if collaboration.is_empty() {
            "请补充该员工的协作方式与升级路径。"
        } else {
            collaboration.as_str()
        },
    );

    let soul_md = format!(
        "# PERSONA\n\n## Tone\n{tone}\n\n## Boundaries\n{boundaries}\n\n## Operating Principles\n1. 先澄清上下文，再执行。\n2. 输出可执行步骤与验收标准。\n3. 遇到风险先预警，再给替代方案。\n",
        tone = if tone.is_empty() {
            "专业、简洁、可执行。"
        } else {
            tone.as_str()
        },
        boundaries = if boundaries.is_empty() {
            "不编造事实；权限不明时先确认；高风险操作必须二次确认。"
        } else {
            boundaries.as_str()
        },
    );

    let user_md = format!(
        "# USER_CONTEXT\n\n## User Profile\n{user_profile}\n\n## Communication Preferences\n- 先结论，后细节\n- 默认给出下一步执行建议\n- 对关键决策提供利弊权衡\n",
        user_profile = if user_profile.is_empty() {
            "面向业务与产品协作场景，关注交付结果与效率。"
        } else {
            user_profile.as_str()
        },
    );

    AgentProfileDraft {
        employee_id: employee.employee_id.clone(),
        employee_name: employee.name.clone(),
        agents_md,
        soul_md,
        user_md,
    }
}

async fn find_employee_with_pool(
    pool: &SqlitePool,
    employee_db_id: &str,
) -> Result<AgentEmployee, String> {
    let rows = list_agent_employees_with_pool(pool).await?;
    rows.into_iter()
        .find(|item| item.id == employee_db_id)
        .ok_or_else(|| "employee not found".to_string())
}

async fn resolve_profile_id_with_pool(
    pool: &SqlitePool,
    employee: &AgentEmployee,
) -> Result<String, String> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT id
         FROM agent_profiles
         WHERE legacy_employee_row_id = ? OR id = ?
         ORDER BY CASE WHEN legacy_employee_row_id = ? THEN 0 ELSE 1 END
         LIMIT 1",
    )
    .bind(&employee.id)
    .bind(&employee.id)
    .bind(&employee.id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|row| row.0).unwrap_or_else(|| employee.id.clone()))
}

fn resolve_profile_dir(employee: &AgentEmployee, profile_id: &str, fallback_base: &str) -> PathBuf {
    let base = if employee.default_work_dir.trim().is_empty() {
        fallback_base.to_string()
    } else {
        employee.default_work_dir.trim().to_string()
    };
    PathBuf::from(base).join("profiles").join(profile_id.trim())
}

fn profile_home_artifact_dirs(profile_dir: &std::path::Path) -> [PathBuf; 6] {
    [
        profile_dir.join("instructions"),
        profile_dir.join("memories"),
        profile_dir.join("skills"),
        profile_dir.join("sessions"),
        profile_dir.join("growth"),
        profile_dir.join("curator"),
    ]
}

fn ensure_profile_home_dirs(profile_dir: &std::path::Path) -> Result<(), String> {
    for dir in profile_home_artifact_dirs(profile_dir) {
        std::fs::create_dir_all(&dir).map_err(|e| format!("failed to create profile dir: {e}"))?;
    }
    Ok(())
}

async fn recorded_profile_home_with_pool(
    pool: &SqlitePool,
    employee: &AgentEmployee,
    profile_id: &str,
) -> Result<Option<String>, String> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT COALESCE(profile_home, '')
         FROM agent_profiles
         WHERE legacy_employee_row_id = ? OR id = ?
         ORDER BY CASE WHEN legacy_employee_row_id = ? THEN 0 ELSE 1 END
         LIMIT 1",
    )
    .bind(&employee.id)
    .bind(profile_id)
    .bind(&employee.id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row
        .map(|row| row.0.trim().to_string())
        .filter(|home| !home.is_empty()))
}

async fn record_profile_home_with_pool(
    pool: &SqlitePool,
    employee: &AgentEmployee,
    profile_id: &str,
    profile_dir: &std::path::Path,
) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    let profile_home = profile_dir.to_string_lossy().to_string();
    sqlx::query(
        "INSERT INTO agent_profiles (
            id,
            legacy_employee_row_id,
            display_name,
            route_aliases_json,
            profile_home,
            created_at,
            updated_at
        )
        VALUES (?, ?, ?, '[]', ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            legacy_employee_row_id = excluded.legacy_employee_row_id,
            display_name = excluded.display_name,
            profile_home = excluded.profile_home,
            updated_at = excluded.updated_at",
    )
    .bind(profile_id)
    .bind(&employee.id)
    .bind(&employee.name)
    .bind(profile_home)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn ensure_employee_profile_home_with_pool(
    pool: &SqlitePool,
    employee: &AgentEmployee,
) -> Result<(String, PathBuf), String> {
    let profile_id = resolve_profile_id_with_pool(pool, employee).await?;
    let fallback_base = if employee.default_work_dir.trim().is_empty() {
        resolve_default_work_dir_with_pool(pool).await?
    } else {
        String::new()
    };
    let profile_dir = match recorded_profile_home_with_pool(pool, employee, &profile_id).await? {
        Some(home) => PathBuf::from(home),
        None => resolve_profile_dir(employee, &profile_id, &fallback_base),
    };
    ensure_profile_home_dirs(&profile_dir)?;
    record_profile_home_with_pool(pool, employee, &profile_id, &profile_dir).await?;
    Ok((profile_id, profile_dir))
}

fn write_profile_files(
    profile_dir: &std::path::Path,
    draft: AgentProfileDraft,
) -> Result<ApplyAgentProfileResult, String> {
    let instructions_dir = profile_dir.join("instructions");
    ensure_profile_home_dirs(profile_dir)?;

    let mut files = Vec::with_capacity(3);
    let write_targets = [
        ("RULES.md", draft.agents_md),
        ("PERSONA.md", draft.soul_md),
        ("USER_CONTEXT.md", draft.user_md),
    ];

    for (name, content) in write_targets {
        let file_path = instructions_dir.join(name);
        let path_text = file_path.to_string_lossy().to_string();
        match std::fs::write(&file_path, content.as_bytes()) {
            Ok(_) => files.push(AgentProfileFileResult {
                path: path_text,
                ok: true,
                error: None,
            }),
            Err(e) => files.push(AgentProfileFileResult {
                path: path_text,
                ok: false,
                error: Some(e.to_string()),
            }),
        }
    }

    Ok(ApplyAgentProfileResult { files })
}

pub async fn generate_agent_profile_draft_with_pool(
    pool: &SqlitePool,
    payload: AgentProfilePayload,
) -> Result<AgentProfileDraft, String> {
    let employee = find_employee_with_pool(pool, payload.employee_db_id.trim()).await?;
    Ok(render_markdown(&employee, &payload.answers))
}

pub async fn apply_agent_profile_draft_with_pool(
    pool: &SqlitePool,
    employee_db_id: &str,
    draft: AgentProfileDraft,
) -> Result<ApplyAgentProfileResult, String> {
    let employee = find_employee_with_pool(pool, employee_db_id.trim()).await?;
    let (_profile_id, profile_dir) =
        ensure_employee_profile_home_with_pool(pool, &employee).await?;
    write_profile_files(&profile_dir, draft)
}

pub async fn apply_agent_profile_with_pool(
    pool: &SqlitePool,
    payload: AgentProfilePayload,
) -> Result<ApplyAgentProfileResult, String> {
    let employee = find_employee_with_pool(pool, payload.employee_db_id.trim()).await?;
    let draft = render_markdown(&employee, &payload.answers);
    apply_agent_profile_draft_with_pool(pool, &employee.id, draft).await
}

pub async fn get_agent_profile_files_with_pool(
    pool: &SqlitePool,
    employee_db_id: &str,
) -> Result<AgentProfileFilesView, String> {
    let employee = find_employee_with_pool(pool, employee_db_id.trim()).await?;
    let (_profile_id, profile_dir) =
        ensure_employee_profile_home_with_pool(pool, &employee).await?;
    let instructions_dir = profile_dir.join("instructions");
    let profile_dir_text = profile_dir.to_string_lossy().to_string();
    let mut files = Vec::with_capacity(3);

    for name in ["RULES.md", "PERSONA.md", "USER_CONTEXT.md"] {
        let file_path = instructions_dir.join(name);
        let path_text = file_path.to_string_lossy().to_string();
        match std::fs::read_to_string(&file_path) {
            Ok(content) => files.push(AgentProfileFileView {
                name: name.to_string(),
                path: path_text,
                exists: true,
                content,
                error: None,
            }),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                files.push(AgentProfileFileView {
                    name: name.to_string(),
                    path: path_text,
                    exists: false,
                    content: String::new(),
                    error: None,
                })
            }
            Err(err) => files.push(AgentProfileFileView {
                name: name.to_string(),
                path: path_text,
                exists: false,
                content: String::new(),
                error: Some(err.to_string()),
            }),
        }
    }

    Ok(AgentProfileFilesView {
        employee_id: employee.employee_id,
        employee_name: employee.name,
        profile_dir: profile_dir_text,
        artifacts: profile_artifact_statuses(&profile_dir),
        files,
    })
}

fn zip_entry_name(path: &std::path::Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn count_regular_files(path: &std::path::Path) -> usize {
    if !path.exists() {
        return 0;
    }
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .count()
}

fn profile_artifact_statuses(profile_dir: &std::path::Path) -> Vec<AgentProfileArtifactStatus> {
    [
        "instructions",
        "memories",
        "sessions",
        "skills",
        "growth",
        "curator",
    ]
    .into_iter()
    .map(|name| {
        let path = profile_dir.join(name);
        AgentProfileArtifactStatus {
            name: name.to_string(),
            path: path.to_string_lossy().to_string(),
            exists: path.exists(),
            file_count: count_regular_files(&path),
        }
    })
    .collect()
}

pub async fn export_agent_profile_with_pool(
    pool: &SqlitePool,
    employee_db_id: &str,
    output_path: &str,
) -> Result<AgentProfileExportResult, String> {
    let employee = find_employee_with_pool(pool, employee_db_id.trim()).await?;
    let (profile_id, profile_dir) = ensure_employee_profile_home_with_pool(pool, &employee).await?;
    if !profile_dir.exists() {
        return Err(format!(
            "profile home 不存在，无法导出: {}",
            profile_dir.to_string_lossy()
        ));
    }
    if !profile_dir.is_dir() {
        return Err(format!(
            "profile home 不是目录，无法导出: {}",
            profile_dir.to_string_lossy()
        ));
    }

    let output_path = PathBuf::from(output_path.trim());
    if output_path.as_os_str().is_empty() {
        return Err("output_path 不能为空".to_string());
    }
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建 profile export 目录失败: {e}"))?;
        }
    }

    let canonical_profile_dir = profile_dir
        .canonicalize()
        .map_err(|e| format!("解析 profile home 失败: {e}"))?;
    let canonical_output_path = output_path.canonicalize().ok();
    let file = std::fs::File::create(&output_path)
        .map_err(|e| format!("创建 profile export 文件失败: {e}"))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let mut file_count = 0usize;
    let mut total_bytes = 0u64;

    for entry in WalkDir::new(&canonical_profile_dir)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path == canonical_profile_dir {
            continue;
        }
        if canonical_output_path.as_deref() == Some(path) {
            continue;
        }
        let relative = path
            .strip_prefix(&canonical_profile_dir)
            .map_err(|e| format!("生成 profile export 相对路径失败: {e}"))?;
        let name = zip_entry_name(relative);
        if name.is_empty() {
            continue;
        }
        if entry.file_type().is_dir() {
            zip.add_directory(format!("{name}/"), options)
                .map_err(|e| format!("写入 profile export 目录失败: {e}"))?;
            continue;
        }
        if !entry.file_type().is_file() {
            continue;
        }

        zip.start_file(name, options)
            .map_err(|e| format!("写入 profile export 文件头失败: {e}"))?;
        let mut input =
            std::fs::File::open(path).map_err(|e| format!("读取 profile 文件失败: {e}"))?;
        let mut buffer = Vec::new();
        input
            .read_to_end(&mut buffer)
            .map_err(|e| format!("读取 profile 文件内容失败: {e}"))?;
        total_bytes += buffer.len() as u64;
        file_count += 1;
        zip.write_all(&buffer)
            .map_err(|e| format!("写入 profile export 内容失败: {e}"))?;
    }

    let readme = format!(
        "# WorkClaw Profile Export\n\n\
员工：{}\n\
员工编号：{}\n\
Profile ID：{}\n\
导出时间：{}\n\n\
这个压缩包是该智能体员工的 Profile Home 备份，包含指令、记忆、会话索引、技能、成长记录和 Curator 报告。\n\n\
- instructions/: RULES.md、PERSONA.md、USER_CONTEXT.md 等员工指令文件。\n\
- memories/: Profile Memory OS 的长期记忆。\n\
- sessions/: 会话索引与会话侧上下文。\n\
- skills/: 员工自己的技能库与版本记录。\n\
- growth/: 员工成长记录。\n\
- curator/: Curator 扫描和整理报告。\n\n\
PROFILE_EXPORT.json 是机器可读清单。\n",
        employee.name,
        employee.employee_id,
        profile_id,
        chrono::Utc::now().to_rfc3339()
    );
    zip.start_file("README.md", options)
        .map_err(|e| format!("写入 profile export README 失败: {e}"))?;
    zip.write_all(readme.as_bytes())
        .map_err(|e| format!("写入 profile export README 内容失败: {e}"))?;

    let manifest = serde_json::json!({
        "format": "workclaw-profile-export",
        "version": 1,
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "employee_id": employee.employee_id,
        "employee_db_id": employee.id,
        "employee_name": employee.name,
        "profile_id": profile_id,
        "profile_dir": profile_dir.to_string_lossy().to_string(),
        "file_count": file_count,
        "total_bytes": total_bytes,
        "includes": [
            "instructions",
            "memories",
            "sessions",
            "skills",
            "growth",
            "curator"
        ]
    });
    zip.start_file("PROFILE_EXPORT.json", options)
        .map_err(|e| format!("写入 profile export manifest 失败: {e}"))?;
    zip.write_all(manifest.to_string().as_bytes())
        .map_err(|e| format!("写入 profile export manifest 内容失败: {e}"))?;
    zip.finish()
        .map_err(|e| format!("完成 profile export 失败: {e}"))?;

    Ok(AgentProfileExportResult {
        employee_id: employee.employee_id,
        employee_name: employee.name,
        profile_id,
        profile_dir: profile_dir.to_string_lossy().to_string(),
        export_path: output_path.to_string_lossy().to_string(),
        file_count,
        total_bytes,
    })
}

#[tauri::command]
pub async fn generate_agent_profile_draft(
    payload: AgentProfilePayload,
    db: State<'_, DbState>,
) -> Result<AgentProfileDraft, String> {
    generate_agent_profile_draft_with_pool(&db.0, payload).await
}

#[tauri::command]
pub async fn apply_agent_profile(
    payload: AgentProfilePayload,
    db: State<'_, DbState>,
) -> Result<ApplyAgentProfileResult, String> {
    apply_agent_profile_with_pool(&db.0, payload).await
}

#[tauri::command]
pub async fn get_agent_profile_files(
    employee_db_id: String,
    db: State<'_, DbState>,
) -> Result<AgentProfileFilesView, String> {
    get_agent_profile_files_with_pool(&db.0, employee_db_id.as_str()).await
}

#[tauri::command]
pub async fn export_agent_profile(
    employee_db_id: String,
    output_path: String,
    db: State<'_, DbState>,
) -> Result<AgentProfileExportResult, String> {
    export_agent_profile_with_pool(&db.0, employee_db_id.as_str(), output_path.as_str()).await
}
