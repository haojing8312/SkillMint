use anyhow::{Result, anyhow};
use chrono::Utc;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::agent::types::{Tool, ToolContext};

/// 持久内存工具 - 跨会话的知识存储
///
/// Profile Memory 的主文件是 MEMORY.md，支持 view/add/replace/remove/history。
/// 旧版 key-value Markdown 记忆仍支持 read/write/list/delete。
///
/// # 示例
///
/// ```rust
/// use std::path::PathBuf;
/// use runtime_lib::agent::tools::MemoryTool;
/// use runtime_lib::agent::types::{Tool, ToolContext};
/// use serde_json::json;
///
/// let tool = MemoryTool::new(PathBuf::from("/tmp/memory"));
/// let ctx = ToolContext::default();
/// let result = tool.execute(json!({
///     "action": "write",
///     "key": "greeting",
///     "content": "你好，世界！"
/// }), &ctx).unwrap();
/// assert!(result.contains("已写入"));
/// ```
pub struct MemoryTool {
    memory_dir: PathBuf,
    im_memory_dir: PathBuf,
    project_memory_path: Option<PathBuf>,
    profile_session_search: Option<ProfileSessionSearchConfig>,
}

#[derive(Clone)]
struct ProfileSessionSearchConfig {
    pool: SqlitePool,
    profile_id: String,
}

struct MemoryVersionRecord {
    version_id: String,
    metadata: Value,
}

impl MemoryTool {
    /// 创建新的 MemoryTool 实例
    ///
    /// # 参数
    /// - `memory_dir`: Profile Memory 目录，通常为 `{runtime_root}/profiles/{profile_id}/memories`
    pub fn new(memory_dir: PathBuf) -> Self {
        Self {
            im_memory_dir: memory_dir.clone(),
            memory_dir,
            project_memory_path: None,
            profile_session_search: None,
        }
    }

    /// 设置 IM 记忆目录。普通 Profile Memory 与 IM 线程记忆可以分开迁移。
    pub fn with_im_memory_dir(mut self, im_memory_dir: PathBuf) -> Self {
        self.im_memory_dir = im_memory_dir;
        self
    }

    /// 设置当前工作区的 Project Memory 文件路径。
    pub fn with_project_memory_path(mut self, project_memory_path: PathBuf) -> Self {
        self.project_memory_path = Some(project_memory_path);
        self
    }

    /// 设置 Profile Session Search。配置后 `memory.search` 可召回该 profile 的历史经验。
    pub fn with_profile_session_search(mut self, pool: SqlitePool, profile_id: String) -> Self {
        if !profile_id.trim().is_empty() {
            self.profile_session_search = Some(ProfileSessionSearchConfig {
                pool,
                profile_id: profile_id.trim().to_string(),
            });
        }
        self
    }

    fn block_on<T, F>(&self, fut: F) -> Result<T>
    where
        F: std::future::Future<Output = std::result::Result<T, String>>,
    {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|err| anyhow!("构建 memory tool runtime 失败: {err}"))?;
        rt.block_on(fut).map_err(|err| anyhow!(err))
    }

    fn profile_memory_path(&self) -> PathBuf {
        self.memory_dir.join("MEMORY.md")
    }

    fn target_memory_path(&self, input: &Value) -> Result<(PathBuf, &'static str)> {
        match input["scope"].as_str().unwrap_or("profile") {
            "profile" => Ok((self.profile_memory_path(), "Profile Memory")),
            "project" => {
                let path = self.project_memory_path.clone().ok_or_else(|| {
                    anyhow!("project scope 需要当前会话绑定可用的 Project Memory 路径")
                })?;
                Ok((path, "Project Memory"))
            }
            scope => Err(anyhow!("未知 memory scope: {}", scope)),
        }
    }

    fn target_key_for_path(&self, path: &Path, scope: &str) -> String {
        if scope == "project" {
            return path
                .file_stem()
                .map(|value| value.to_string_lossy().to_string())
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "project".to_string());
        }
        "MEMORY".to_string()
    }

    fn history_path(&self, input: &Value) -> PathBuf {
        if input["scope"].as_str() == Some("project") {
            return self.memory_dir.join("project-history.jsonl");
        }
        self.memory_dir.join("history.jsonl")
    }

    fn versions_dir_for(&self, input: &Value, path: &Path) -> PathBuf {
        if input["scope"].as_str() == Some("project") {
            let target_key = self.target_key_for_path(path, "project");
            return self
                .memory_dir
                .join("versions")
                .join("projects")
                .join(target_key);
        }
        self.memory_dir.join("versions").join("profile")
    }

    fn sanitize_version_id(version_id: &str) -> Result<String> {
        let trimmed = version_id.trim();
        if trimmed.is_empty() {
            return Err(anyhow!("version_id 不能为空"));
        }
        if !trimmed
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
        {
            return Err(anyhow!("version_id 包含非法字符"));
        }
        Ok(trimmed.to_string())
    }

    fn make_version_id() -> String {
        let timestamp = Utc::now().format("%Y%m%dT%H%M%S%6fZ");
        let nonce = Uuid::new_v4().simple().to_string();
        format!("v{}_{}", timestamp, &nonce[..8])
    }

    fn memory_content_preview(content: &str) -> String {
        content
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .unwrap_or_else(|| content.trim())
            .chars()
            .take(120)
            .collect::<String>()
    }

    fn memory_change_preview(input: &Value, next_content: Option<&str>) -> String {
        input["content"]
            .as_str()
            .map(Self::memory_content_preview)
            .filter(|preview| !preview.trim().is_empty())
            .or_else(|| {
                next_content
                    .map(Self::memory_content_preview)
                    .filter(|preview| !preview.trim().is_empty())
            })
            .unwrap_or_default()
    }

    fn memory_diff_summary(input: &Value, action: &str, content_preview: &str) -> String {
        input["diff_summary"]
            .as_str()
            .or_else(|| input["change_summary"].as_str())
            .or_else(|| input["reason"].as_str())
            .map(str::trim)
            .filter(|summary| !summary.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| match action {
                "add" if !content_preview.trim().is_empty() => {
                    format!("追加记忆：{content_preview}")
                }
                "replace" if !content_preview.trim().is_empty() => {
                    format!("替换记忆：{content_preview}")
                }
                "remove" => "删除 Profile Memory".to_string(),
                "rollback" if !content_preview.trim().is_empty() => {
                    format!("回滚记忆：{content_preview}")
                }
                _ => format!("Profile Memory {action}"),
            })
    }

    fn content_sha256(content: &str) -> String {
        format!("{:x}", Sha256::digest(content.as_bytes()))
    }

    fn write_version_snapshot(
        &self,
        input: &Value,
        action: &str,
        path: &Path,
        next_content: Option<&str>,
        rollback_of: Option<&str>,
    ) -> Result<MemoryVersionRecord> {
        let scope = input["scope"].as_str().unwrap_or("profile");
        let target_key = self.target_key_for_path(path, scope);
        let versions_dir = self.versions_dir_for(input, path);
        fs::create_dir_all(&versions_dir)?;
        let version_id = Self::make_version_id();
        let snapshot_path = versions_dir.join(format!("{version_id}.md"));
        let metadata_path = versions_dir.join(format!("{version_id}.json"));
        let content = next_content.unwrap_or("");
        fs::write(&snapshot_path, content)?;
        let snapshot_rel = snapshot_path
            .strip_prefix(&self.memory_dir)
            .unwrap_or(snapshot_path.as_path())
            .to_string_lossy()
            .replace('\\', "/");
        let metadata_rel = metadata_path
            .strip_prefix(&self.memory_dir)
            .unwrap_or(metadata_path.as_path())
            .to_string_lossy()
            .replace('\\', "/");
        let content_preview = Self::memory_change_preview(input, next_content);
        let diff_summary = Self::memory_diff_summary(input, action, &content_preview);
        let entry = json!({
            "event_id": format!("evt_{version_id}"),
            "timestamp": Utc::now().to_rfc3339(),
            "action": action,
            "scope": scope,
            "target_key": target_key,
            "version_id": version_id,
            "snapshot_path": snapshot_rel,
            "metadata_path": metadata_rel,
            "content_sha256": Self::content_sha256(content),
            "content_preview": content_preview,
            "diff_summary": diff_summary,
            "deleted": next_content.is_none(),
            "source": input["source"].as_str().unwrap_or("agent"),
            "source_session_id": input["source_session_id"].as_str().unwrap_or(""),
            "source_run_id": input["source_run_id"].as_str().unwrap_or(""),
            "source_tool_call_id": input["source_tool_call_id"].as_str().unwrap_or(""),
            "change_summary": input["change_summary"].as_str().unwrap_or(""),
            "rollback_of": rollback_of,
            "reason": input["reason"].as_str().unwrap_or("")
        });
        fs::write(&metadata_path, serde_json::to_string_pretty(&entry)?)?;
        Ok(MemoryVersionRecord {
            version_id,
            metadata: entry,
        })
    }

    fn append_history_record(&self, input: &Value, metadata: &Value) -> Result<()> {
        fs::create_dir_all(&self.memory_dir)?;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.history_path(input))?;
        writeln!(file, "{}", serde_json::to_string(metadata)?)?;
        Ok(())
    }

    fn persist_memory_version(
        &self,
        input: &Value,
        action: &str,
        path: &Path,
        next_content: Option<&str>,
        rollback_of: Option<&str>,
    ) -> Result<MemoryVersionRecord> {
        let record = self.write_version_snapshot(input, action, path, next_content, rollback_of)?;
        self.append_history_record(input, &record.metadata)?;
        Ok(record)
    }

    async fn ensure_growth_events_schema(pool: &SqlitePool) -> std::result::Result<(), String> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS growth_events (
                id TEXT PRIMARY KEY,
                profile_id TEXT NOT NULL DEFAULT '',
                session_id TEXT NOT NULL DEFAULT '',
                event_type TEXT NOT NULL,
                target_type TEXT NOT NULL,
                target_id TEXT NOT NULL,
                summary TEXT NOT NULL DEFAULT '',
                evidence_json TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL
            )",
        )
        .execute(pool)
        .await
        .map_err(|e| format!("创建 growth_events 表失败: {e}"))?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_growth_events_profile_created
             ON growth_events(profile_id, created_at DESC)",
        )
        .execute(pool)
        .await
        .map_err(|e| format!("创建 growth_events profile 索引失败: {e}"))?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_growth_events_target
             ON growth_events(target_type, target_id, created_at DESC)",
        )
        .execute(pool)
        .await
        .map_err(|e| format!("创建 growth_events target 索引失败: {e}"))?;

        Ok(())
    }

    fn record_memory_growth_event(
        &self,
        input: &Value,
        ctx: &ToolContext,
        action: &str,
        path: &Path,
        record: &MemoryVersionRecord,
    ) -> Result<()> {
        let Some(config) = self.profile_session_search.clone() else {
            return Ok(());
        };
        let scope = input["scope"].as_str().unwrap_or("profile").to_string();
        let target_type = if scope == "project" {
            "project_memory"
        } else {
            "profile_memory"
        }
        .to_string();
        let target_id = self.target_key_for_path(path, &scope);
        let session_id = input["source_session_id"]
            .as_str()
            .filter(|value| !value.trim().is_empty())
            .map(str::to_string)
            .or_else(|| ctx.session_id.clone())
            .unwrap_or_default();
        let summary = input["change_summary"]
            .as_str()
            .or_else(|| input["reason"].as_str())
            .unwrap_or(action)
            .to_string();
        let content_preview = record
            .metadata
            .get("content_preview")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let diff_summary = record
            .metadata
            .get("diff_summary")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let evidence = json!({
            "version_id": record.version_id,
            "content_preview": content_preview,
            "diff_summary": diff_summary,
            "memory_version": record.metadata,
            "scope": scope,
            "path": path.to_string_lossy().replace('\\', "/")
        });
        let evidence_json = serde_json::to_string(&evidence)
            .map_err(|err| anyhow!("序列化 memory growth evidence 失败: {err}"))?;
        let event_id = format!("gr_mem_{}", record.version_id);
        let event_type = Self::memory_growth_event_type(input, action);
        let created_at = Utc::now().to_rfc3339();
        self.block_on(async move {
            Self::ensure_growth_events_schema(&config.pool).await?;
            sqlx::query(
                "INSERT OR REPLACE INTO growth_events (
                    id, profile_id, session_id, event_type, target_type, target_id,
                    summary, evidence_json, created_at
                 ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(event_id)
            .bind(config.profile_id)
            .bind(session_id)
            .bind(event_type)
            .bind(target_type)
            .bind(target_id)
            .bind(summary)
            .bind(evidence_json)
            .bind(created_at)
            .execute(&config.pool)
            .await
            .map_err(|e| format!("写入 memory growth event 失败: {e}"))?;
            Ok(())
        })
    }

    fn memory_growth_event_type(input: &Value, action: &str) -> String {
        let source = input["source"]
            .as_str()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        if matches!(
            source.as_str(),
            "user-correction" | "user_correction" | "correction"
        ) {
            return "user_correction".to_string();
        }
        format!("memory_{action}")
    }

    fn list_versions(&self, input: &Value) -> Result<String> {
        let (path, _) = self.target_memory_path(input)?;
        let versions_dir = self.versions_dir_for(input, &path);
        if !versions_dir.exists() {
            return Ok("[]".to_string());
        }
        let limit = input["limit"].as_u64().unwrap_or(u64::MAX) as usize;
        let mut items = fs::read_dir(versions_dir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .filter_map(|path| fs::read_to_string(path).ok())
            .filter_map(|raw| serde_json::from_str::<Value>(&raw).ok())
            .collect::<Vec<_>>();
        items.sort_by(|a, b| {
            a["timestamp"]
                .as_str()
                .unwrap_or_default()
                .cmp(b["timestamp"].as_str().unwrap_or_default())
        });
        if items.len() > limit {
            items = items.into_iter().rev().take(limit).collect::<Vec<_>>();
            items.reverse();
        }
        serde_json::to_string_pretty(&items)
            .map_err(|err| anyhow!("序列化 memory versions 失败: {err}"))
    }

    fn version_snapshot_path(&self, input: &Value, version_id: &str) -> Result<PathBuf> {
        let version_id = Self::sanitize_version_id(version_id)?;
        let (path, _) = self.target_memory_path(input)?;
        Ok(self
            .versions_dir_for(input, &path)
            .join(format!("{version_id}.md")))
    }

    fn view_version(&self, input: &Value) -> Result<String> {
        let version_id = input["version_id"]
            .as_str()
            .ok_or_else(|| anyhow!("view_version 操作缺少 version_id 参数"))?;
        let snapshot_path = self.version_snapshot_path(input, version_id)?;
        if !snapshot_path.exists() {
            return Err(anyhow!("memory version 不存在: {version_id}"));
        }
        Ok(fs::read_to_string(snapshot_path)?)
    }

    fn rollback_version(&self, input: &Value, ctx: &ToolContext) -> Result<String> {
        if input["confirm"].as_bool() != Some(true) {
            return Ok("rollback 是高风险操作，需要 confirm=true".to_string());
        }
        let version_id = input["version_id"]
            .as_str()
            .ok_or_else(|| anyhow!("rollback 操作缺少 version_id 参数"))?;
        let version_id = Self::sanitize_version_id(version_id)?;
        let (path, label) = self.target_memory_path(input)?;
        let snapshot_path = self.version_snapshot_path(input, &version_id)?;
        if !snapshot_path.exists() {
            return Err(anyhow!("memory version 不存在: {version_id}"));
        }
        let content = fs::read_to_string(snapshot_path)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, &content)?;
        let record = self.persist_memory_version(
            input,
            "rollback",
            &path,
            Some(&content),
            Some(&version_id),
        )?;
        self.record_memory_growth_event(input, ctx, "rollback", &path, &record)?;
        Ok(format!(
            "已回滚 {label} 到 {version_id}，新版本 {}",
            record.version_id
        ))
    }

    fn search_profile_sessions(&self, input: &Value) -> Result<String> {
        let query = input["query"]
            .as_str()
            .or_else(|| input["content"].as_str())
            .ok_or_else(|| anyhow!("search 操作缺少 query 参数"))?;
        if query.trim().is_empty() {
            return Err(anyhow!("search query 不能为空"));
        }
        let limit = input["limit"].as_i64().unwrap_or(5).clamp(1, 20);
        let config = self
            .profile_session_search
            .clone()
            .ok_or_else(|| anyhow!("当前 memory tool 未配置 Profile Session Search"))?;
        let rows = self.block_on(
            crate::agent::runtime::runtime_io::search_profile_session_index_with_filters_with_pool(
                &config.pool,
                &config.profile_id,
                query,
                limit,
                crate::agent::runtime::runtime_io::ProfileSessionSearchFilters {
                    work_dir: input["work_dir"]
                        .as_str()
                        .or_else(|| input["workspace"].as_str())
                        .map(str::to_string),
                    updated_after: input["updated_after"].as_str().map(str::to_string),
                    updated_before: input["updated_before"].as_str().map(str::to_string),
                    skill_id: input["skill_id"].as_str().map(str::to_string),
                    source: input["source"].as_str().map(str::to_string),
                },
            ),
        )?;
        if rows.is_empty() {
            return Ok("Profile Session Search 未找到相关历史经验".to_string());
        }
        let payload = rows
            .into_iter()
            .map(|row| {
                json!({
                    "profile_id": row.profile_id,
                    "session_id": row.session_id,
                    "skill_id": row.skill_id,
                    "work_dir": row.work_dir,
                    "source": row.source,
                    "run_status": row.run_status,
                    "latest_run_id": row.latest_run_id,
                    "document_kind": row.document_kind,
                    "matched_run_id": row.matched_run_id,
                    "tool_summary_count": row.tool_summary_count,
                    "compaction_boundary_count": row.compaction_boundary_count,
                    "updated_at": row.updated_at,
                    "snippet": row.snippet,
                })
            })
            .collect::<Vec<_>>();
        serde_json::to_string_pretty(&payload)
            .map_err(|err| anyhow!("序列化 Profile Session Search 结果失败: {err}"))
    }
}

impl Tool for MemoryTool {
    fn name(&self) -> &str {
        "memory"
    }

    fn description(&self) -> &str {
        "跨会话的 Profile Memory。优先使用 view/add/replace/remove/history/search 管理和召回 Profile Memory；兼容 read/write/list/delete 旧键值记忆。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["view", "add", "replace", "remove", "history", "versions", "view_version", "rollback", "search", "read", "write", "list", "delete", "capture_im", "recall_im"],
                    "description": "操作类型"
                },
                "key": {
                    "type": "string",
                    "description": "内存键名（文件名，不含扩展名）"
                },
                "content": {
                    "type": "string",
                    "description": "写入内容（add/replace/write/capture_im 操作需要）；search 未提供 query 时可作为查询文本"
                },
                "query": {
                    "type": "string",
                    "description": "Profile Session Search 查询文本（search 操作需要）"
                },
                "limit": {
                    "type": "integer",
                    "description": "search/versions 返回数量；search 默认 5，范围 1-20"
                },
                "version_id": {
                    "type": "string",
                    "description": "view_version/rollback 需要的 memory version id"
                },
                "change_summary": {
                    "type": "string",
                    "description": "add/replace/remove/rollback 的变更摘要，会写入版本历史"
                },
                "reason": {
                    "type": "string",
                    "description": "rollback 原因，会写入版本历史"
                },
                "work_dir": {
                    "type": "string",
                    "description": "search 可选过滤：只召回指定 workspace/work_dir 的历史 session"
                },
                "workspace": {
                    "type": "string",
                    "description": "search 可选过滤：work_dir 的别名"
                },
                "updated_after": {
                    "type": "string",
                    "description": "search 可选过滤：只召回 updated_at 大于等于该 RFC3339 时间的历史 session"
                },
                "updated_before": {
                    "type": "string",
                    "description": "search 可选过滤：只召回 updated_at 小于等于该 RFC3339 时间的历史 session"
                },
                "skill_id": {
                    "type": "string",
                    "description": "search 可选过滤：只召回指定 skill_id 的历史 session"
                },
                "scope": {
                    "type": "string",
                    "enum": ["profile", "project"],
                    "description": "Profile Memory 或当前工作区 Project Memory，默认 profile"
                },
                "source": {
                    "type": "string",
                    "description": "记忆来源，例如 session:<id>、user-correction、tool:<name>"
                },
                "source_session_id": {
                    "type": "string",
                    "description": "记忆变更来源 session id"
                },
                "source_run_id": {
                    "type": "string",
                    "description": "记忆变更来源 run id"
                },
                "source_tool_call_id": {
                    "type": "string",
                    "description": "记忆变更来源 tool call id"
                },
                "confirm": {
                    "type": "boolean",
                    "description": "高风险操作确认标记。remove 需要 confirm=true"
                },
                "thread_id": {
                    "type": "string",
                    "description": "IM 线程 ID（capture_im / recall_im）"
                },
                "role_id": {
                    "type": "string",
                    "description": "角色 ID（capture_im / recall_im）"
                },
                "category": {
                    "type": "string",
                    "description": "记忆分类（fact/decision/risk/rule）"
                },
                "confirmed": {
                    "type": "boolean",
                    "description": "是否已确认"
                },
                "source_msg_id": {
                    "type": "string",
                    "description": "来源消息 ID"
                },
                "confidence": {
                    "type": "number",
                    "description": "置信度 0-1"
                }
            },
            "required": ["action"]
        })
    }

    fn execute(&self, input: Value, ctx: &ToolContext) -> Result<String> {
        let action = input["action"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 action 参数"))?;

        match action {
            "view" => {
                let (path, label) = self.target_memory_path(&input)?;
                if !path.exists() {
                    return Ok(format!("{label} 为空"));
                }
                Ok(fs::read_to_string(path)?)
            }
            "add" => {
                let content = input["content"]
                    .as_str()
                    .ok_or_else(|| anyhow!("add 操作缺少 content 参数"))?;
                let (path, label) = self.target_memory_path(&input)?;
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let previous = fs::read_to_string(&path).unwrap_or_default();
                let mut next = previous.clone();
                if !next.trim().is_empty() && !next.ends_with("\n\n") {
                    if next.ends_with('\n') {
                        next.push('\n');
                    } else {
                        next.push_str("\n\n");
                    }
                }
                next.push_str(content.trim_end());
                next.push('\n');
                fs::write(&path, &next)?;
                let record =
                    self.persist_memory_version(&input, "add", &path, Some(&next), None)?;
                self.record_memory_growth_event(&input, ctx, "add", &path, &record)?;
                Ok(format!("已追加 {label}"))
            }
            "replace" => {
                let content = input["content"]
                    .as_str()
                    .ok_or_else(|| anyhow!("replace 操作缺少 content 参数"))?;
                let (path, label) = self.target_memory_path(&input)?;
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut next = content.trim_end().to_string();
                next.push('\n');
                fs::write(&path, &next)?;
                let record =
                    self.persist_memory_version(&input, "replace", &path, Some(&next), None)?;
                self.record_memory_growth_event(&input, ctx, "replace", &path, &record)?;
                Ok(format!("已替换 {label}"))
            }
            "remove" => {
                if input["confirm"].as_bool() != Some(true) {
                    return Ok("remove 是高风险操作，需要 confirm=true".to_string());
                }
                let (path, label) = self.target_memory_path(&input)?;
                if path.exists() {
                    fs::remove_file(&path)?;
                }
                let record = self.persist_memory_version(&input, "remove", &path, None, None)?;
                self.record_memory_growth_event(&input, ctx, "remove", &path, &record)?;
                Ok(format!("已移除 {label}"))
            }
            "history" => {
                let path = self.history_path(&input);
                if !path.exists() {
                    return Ok("Profile Memory 暂无历史".to_string());
                }
                Ok(fs::read_to_string(path)?)
            }
            "versions" => self.list_versions(&input),
            "view_version" => self.view_version(&input),
            "rollback" => self.rollback_version(&input, ctx),
            "search" => self.search_profile_sessions(&input),
            "read" => {
                let key = input["key"]
                    .as_str()
                    .ok_or_else(|| anyhow!("read 操作缺少 key 参数"))?;
                let path = self.memory_dir.join(format!("{}.md", key));
                if !path.exists() {
                    return Ok(format!("内存键 '{}' 不存在", key));
                }
                let content = fs::read_to_string(&path)?;
                Ok(content)
            }
            "write" => {
                let key = input["key"]
                    .as_str()
                    .ok_or_else(|| anyhow!("write 操作缺少 key 参数"))?;
                let content = input["content"]
                    .as_str()
                    .ok_or_else(|| anyhow!("write 操作缺少 content 参数"))?;
                // 确保目录存在
                fs::create_dir_all(&self.memory_dir)?;
                let path = self.memory_dir.join(format!("{}.md", key));
                fs::write(&path, content)?;
                Ok(format!("已写入内存键 '{}'", key))
            }
            "list" => {
                if !self.memory_dir.exists() {
                    return Ok("内存为空".to_string());
                }
                let mut entries: Vec<String> = fs::read_dir(&self.memory_dir)?
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
                    .filter_map(|e| {
                        e.path()
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                    })
                    .collect();
                if entries.is_empty() {
                    Ok("内存为空".to_string())
                } else {
                    // 排序保证输出稳定
                    entries.sort();
                    Ok(format!("内存键列表:\n{}", entries.join("\n")))
                }
            }
            "delete" => {
                let key = input["key"]
                    .as_str()
                    .ok_or_else(|| anyhow!("delete 操作缺少 key 参数"))?;
                let path = self.memory_dir.join(format!("{}.md", key));
                if !path.exists() {
                    return Ok(format!("内存键 '{}' 不存在", key));
                }
                fs::remove_file(&path)?;
                Ok(format!("已删除内存键 '{}'", key))
            }
            "capture_im" => {
                let thread_id = input["thread_id"]
                    .as_str()
                    .ok_or_else(|| anyhow!("capture_im 操作缺少 thread_id 参数"))?;
                let role_id = input["role_id"]
                    .as_str()
                    .ok_or_else(|| anyhow!("capture_im 操作缺少 role_id 参数"))?;
                let category = input["category"].as_str().unwrap_or("fact").to_string();
                let content = input["content"]
                    .as_str()
                    .ok_or_else(|| anyhow!("capture_im 操作缺少 content 参数"))?
                    .to_string();
                let confirmed = input["confirmed"].as_bool().unwrap_or(false);
                let source_msg_id = input["source_msg_id"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string();
                let confidence = input["confidence"].as_f64().unwrap_or(0.6) as f32;

                let entry = crate::im::memory::MemoryEntry {
                    category,
                    content,
                    confirmed,
                    source_msg_id,
                    author_role: role_id.to_string(),
                    confidence,
                };
                let result = crate::im::memory::capture_entry(
                    &self.im_memory_dir,
                    thread_id,
                    role_id,
                    &entry,
                )?;
                Ok(format!(
                    "IM 记忆写入完成: session_written={}, long_term_written={}",
                    result.session_written, result.long_term_written
                ))
            }
            "recall_im" => {
                let thread_id = input["thread_id"]
                    .as_str()
                    .ok_or_else(|| anyhow!("recall_im 操作缺少 thread_id 参数"))?;
                let role_id = input["role_id"]
                    .as_str()
                    .ok_or_else(|| anyhow!("recall_im 操作缺少 role_id 参数"))?;
                let recalled =
                    crate::im::memory::recall_context(&self.im_memory_dir, thread_id, role_id)?;
                if recalled.trim().is_empty() {
                    Ok("无可召回 IM 记忆".to_string())
                } else {
                    Ok(recalled)
                }
            }
            _ => Err(anyhow!("未知操作: {}", action)),
        }
    }
}
