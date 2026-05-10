use anyhow::{Result, anyhow};
use chrono::Utc;
use serde_json::{Value, json};
use sqlx::SqlitePool;

use crate::agent::types::{Tool, ToolContext};

pub struct SkillOsTool {
    pool: SqlitePool,
}

fn normalized_skill_create_content(
    input: &Value,
    name: &str,
    description: &str,
) -> Result<String> {
    let raw = input["content"]
        .as_str()
        .or_else(|| input["instructions"].as_str())
        .ok_or_else(|| anyhow!("skill_create 操作缺少 content 参数"))?
        .trim();
    if raw.is_empty() {
        return Err(anyhow!("skill_create 操作缺少 content 参数"));
    }
    if raw.starts_with("---") || raw.contains("# ") || raw.contains("## ") {
        return Ok(raw.to_string());
    }
    let description = description.trim();
    let description = if description.is_empty() {
        "Agent-created skill"
    } else {
        description
    };
    Ok(format!(
        "---\nname: {name}\ndescription: {description}\n---\n\n# {name}\n\n{raw}\n"
    ))
}

impl SkillOsTool {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    fn block_on<T, F>(&self, fut: F) -> Result<T>
    where
        F: std::future::Future<Output = std::result::Result<T, String>>,
    {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|err| anyhow!("构建 skills tool runtime 失败: {err}"))?;
        rt.block_on(fut).map_err(|err| anyhow!(err))
    }

    fn list_skills(&self) -> Result<String> {
        let items = self.block_on(
            crate::agent::runtime::runtime_io::list_skill_os_index_with_pool(&self.pool),
        )?;
        serde_json::to_string_pretty(&json!({
            "action": "skills_list",
            "items": items
        }))
        .map_err(|err| anyhow!("序列化 Skill OS index 失败: {err}"))
    }

    fn view_skill(&self, input: &Value) -> Result<String> {
        let skill_id = input["skill_id"]
            .as_str()
            .ok_or_else(|| anyhow!("skill_view 操作缺少 skill_id 参数"))?;
        let view = self.block_on(
            crate::agent::runtime::runtime_io::view_skill_os_entry_with_pool(&self.pool, skill_id),
        )?;
        let Some(view) = view else {
            return Err(anyhow!("Skill 不存在: {skill_id}"));
        };
        self.block_on(
            crate::agent::runtime::runtime_io::record_skill_os_usage_with_pool(
                &self.pool, skill_id, "view",
            ),
        )?;
        serde_json::to_string_pretty(&json!({
            "action": "skill_view",
            "skill": view
        }))
        .map_err(|err| anyhow!("序列化 Skill OS view 失败: {err}"))
    }

    fn line_diff(before: &str, after: &str) -> String {
        if before == after {
            return String::new();
        }
        let before_lines = before.lines().collect::<Vec<_>>();
        let after_lines = after.lines().collect::<Vec<_>>();
        let max_len = before_lines.len().max(after_lines.len());
        let mut out = Vec::new();
        for index in 0..max_len {
            match (before_lines.get(index), after_lines.get(index)) {
                (Some(left), Some(right)) if left == right => {}
                (Some(left), Some(right)) => {
                    out.push(format!("-{left}"));
                    out.push(format!("+{right}"));
                }
                (Some(left), None) => out.push(format!("-{left}")),
                (None, Some(right)) => out.push(format!("+{right}")),
                (None, None) => {}
            }
        }
        out.join("\n")
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

    async fn resolve_profile_id_for_session(
        pool: &SqlitePool,
        session_id: Option<&str>,
    ) -> std::result::Result<String, String> {
        let Some(session_id) = session_id.map(str::trim).filter(|value| !value.is_empty()) else {
            return Ok(String::new());
        };
        let columns: Vec<String> =
            sqlx::query_scalar("SELECT name FROM pragma_table_info('sessions')")
                .fetch_all(pool)
                .await
                .unwrap_or_default();
        if !columns.iter().any(|name| name == "profile_id") {
            return Ok(String::new());
        }
        sqlx::query_scalar::<_, String>(
            "SELECT COALESCE(profile_id, '') FROM sessions WHERE id = ?",
        )
        .bind(session_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("读取 session profile_id 失败: {e}"))
        .map(|value| value.unwrap_or_default())
    }

    fn profile_id_for_context(&self, ctx: &ToolContext) -> Result<String> {
        self.block_on(Self::resolve_profile_id_for_session(
            &self.pool,
            ctx.session_id.as_deref(),
        ))
        .map(|profile_id| profile_id.trim().to_string())
    }

    async fn record_growth_event(
        pool: &SqlitePool,
        ctx: &ToolContext,
        event_type: &str,
        skill_id: &str,
        summary: &str,
        evidence: Value,
    ) -> std::result::Result<String, String> {
        Self::ensure_growth_events_schema(pool).await?;
        let id = format!("grw_{}", uuid::Uuid::new_v4().simple());
        let session_id = ctx
            .session_id
            .as_deref()
            .unwrap_or_default()
            .trim()
            .to_string();
        let profile_id =
            Self::resolve_profile_id_for_session(pool, ctx.session_id.as_deref()).await?;
        let created_at = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO growth_events (
                id, profile_id, session_id, event_type, target_type, target_id, summary, evidence_json, created_at
             ) VALUES (?, ?, ?, ?, 'skill', ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(profile_id)
        .bind(session_id)
        .bind(event_type)
        .bind(skill_id)
        .bind(summary)
        .bind(serde_json::to_string(&evidence).unwrap_or_else(|_| "{}".to_string()))
        .bind(created_at)
        .execute(pool)
        .await
        .map_err(|e| format!("写入 growth event 失败: {e}"))?;
        Ok(id)
    }

    fn patch_skill(&self, input: &Value, ctx: &ToolContext) -> Result<String> {
        let skill_id = input["skill_id"]
            .as_str()
            .ok_or_else(|| anyhow!("skill_patch 操作缺少 skill_id 参数"))?;
        let content = input["content"]
            .as_str()
            .ok_or_else(|| anyhow!("skill_patch 操作缺少 content 参数"))?;
        let summary = input["summary"].as_str().unwrap_or_default();
        let before = self
            .block_on(
                crate::agent::runtime::runtime_io::view_skill_os_entry_with_pool(
                    &self.pool, skill_id,
                ),
            )?
            .ok_or_else(|| anyhow!("Skill 不存在: {skill_id}"))?;
        let diff = Self::line_diff(&before.content, content);
        let view = self.block_on(
            crate::agent::runtime::runtime_io::patch_skill_os_entry_with_pool(
                &self.pool, skill_id, content, summary,
            ),
        )?;
        let version_id = self
            .block_on(
                crate::agent::runtime::runtime_io::list_skill_os_versions_with_pool(
                    &self.pool, skill_id, 1,
                ),
            )?
            .into_iter()
            .next()
            .map(|version| version.version_id)
            .unwrap_or_default();
        let growth_event_id = self.block_on(Self::record_growth_event(
            &self.pool,
            ctx,
            "skill_patch",
            skill_id,
            summary,
            json!({
                "version_id": version_id,
                "source_type": view.entry.source.raw_source_type,
                "diff": diff
            }),
        ))?;
        serde_json::to_string_pretty(&json!({
            "action": "skill_patch",
            "skill": view,
            "version_id": version_id,
            "growth_event_id": growth_event_id,
            "diff": diff
        }))
        .map_err(|err| anyhow!("序列化 skill_patch 结果失败: {err}"))
    }

    fn create_skill(&self, input: &Value, ctx: &ToolContext) -> Result<String> {
        let name = input["name"]
            .as_str()
            .ok_or_else(|| anyhow!("skill_create 操作缺少 name 参数"))?;
        let description = input["description"].as_str().unwrap_or_default();
        let content = normalized_skill_create_content(input, name, description)?;
        let summary = input["summary"].as_str().unwrap_or_default();
        let profile_id = self.profile_id_for_context(ctx)?;
        let view = self.block_on(
            crate::agent::runtime::runtime_io::create_agent_skill_os_entry_with_pool(
                &self.pool,
                &profile_id,
                name,
                description,
                &content,
                summary,
            ),
        )?;
        let version_id = self
            .block_on(
                crate::agent::runtime::runtime_io::list_skill_os_versions_with_pool(
                    &self.pool,
                    &view.entry.skill_id,
                    1,
                ),
            )?
            .into_iter()
            .next()
            .map(|version| version.version_id)
            .unwrap_or_default();
        let growth_event_id = self.block_on(Self::record_growth_event(
            &self.pool,
            ctx,
            "skill_create",
            &view.entry.skill_id,
            summary,
            json!({
                "version_id": version_id,
                "source_type": view.entry.source.raw_source_type,
                "created_path": ""
            }),
        ))?;
        serde_json::to_string_pretty(&json!({
            "action": "skill_create",
            "skill": view,
            "version_id": version_id,
            "growth_event_id": growth_event_id
        }))
        .map_err(|err| anyhow!("序列化 skill_create 结果失败: {err}"))
    }

    fn archive_skill(&self, input: &Value, ctx: &ToolContext) -> Result<String> {
        if input["confirm"].as_bool() != Some(true) {
            return Ok("skill_archive 是高风险操作，需要 confirm=true".to_string());
        }
        let skill_id = input["skill_id"]
            .as_str()
            .ok_or_else(|| anyhow!("skill_archive 操作缺少 skill_id 参数"))?;
        let summary = input["summary"].as_str().unwrap_or_default();
        let view = self.block_on(
            crate::agent::runtime::runtime_io::archive_skill_os_entry_with_pool(
                &self.pool, skill_id, summary,
            ),
        )?;
        let version_id = self
            .block_on(
                crate::agent::runtime::runtime_io::list_skill_os_versions_with_pool(
                    &self.pool, skill_id, 1,
                ),
            )?
            .into_iter()
            .next()
            .map(|version| version.version_id)
            .unwrap_or_default();
        let growth_event_id = self.block_on(Self::record_growth_event(
            &self.pool,
            ctx,
            "skill_archive",
            skill_id,
            summary,
            json!({
                "version_id": version_id,
                "source_type": view.entry.source.raw_source_type
            }),
        ))?;
        serde_json::to_string_pretty(&json!({
            "action": "skill_archive",
            "skill": view,
            "version_id": version_id,
            "growth_event_id": growth_event_id
        }))
        .map_err(|err| anyhow!("序列化 skill_archive 结果失败: {err}"))
    }

    fn restore_skill(&self, input: &Value, ctx: &ToolContext) -> Result<String> {
        let skill_id = input["skill_id"]
            .as_str()
            .ok_or_else(|| anyhow!("skill_restore 操作缺少 skill_id 参数"))?;
        let summary = input["summary"].as_str().unwrap_or_default();
        let view = self.block_on(
            crate::agent::runtime::runtime_io::restore_skill_os_entry_with_pool(
                &self.pool, skill_id, summary,
            ),
        )?;
        let version_id = self
            .block_on(
                crate::agent::runtime::runtime_io::list_skill_os_versions_with_pool(
                    &self.pool, skill_id, 1,
                ),
            )?
            .into_iter()
            .next()
            .map(|version| version.version_id)
            .unwrap_or_default();
        let growth_event_id = self.block_on(Self::record_growth_event(
            &self.pool,
            ctx,
            "skill_restore",
            skill_id,
            summary,
            json!({
                "version_id": version_id,
                "source_type": view.entry.source.raw_source_type
            }),
        ))?;
        serde_json::to_string_pretty(&json!({
            "action": "skill_restore",
            "skill": view,
            "version_id": version_id,
            "growth_event_id": growth_event_id
        }))
        .map_err(|err| anyhow!("序列化 skill_restore 结果失败: {err}"))
    }

    fn delete_skill(&self, input: &Value, ctx: &ToolContext) -> Result<String> {
        if input["confirm"].as_bool() != Some(true) {
            return Ok("skill_delete 是高风险操作，需要 confirm=true".to_string());
        }
        let skill_id = input["skill_id"]
            .as_str()
            .ok_or_else(|| anyhow!("skill_delete 操作缺少 skill_id 参数"))?;
        let summary = input["summary"].as_str().unwrap_or_default();
        let (view, version, removed_path, removed_files) = self.block_on(
            crate::agent::runtime::runtime_io::delete_skill_os_entry_with_pool(
                &self.pool, skill_id, summary,
            ),
        )?;
        let growth_event_id = self.block_on(Self::record_growth_event(
            &self.pool,
            ctx,
            "skill_delete",
            skill_id,
            summary,
            json!({
                "version_id": version.version_id,
                "source_type": view.entry.source.raw_source_type,
                "removed_path": removed_path,
                "removed_files": removed_files
            }),
        ))?;
        serde_json::to_string_pretty(&json!({
            "action": "skill_delete",
            "skill": view,
            "version_id": version.version_id,
            "growth_event_id": growth_event_id,
            "removed_path": removed_path,
            "removed_files": removed_files
        }))
        .map_err(|err| anyhow!("序列化 skill_delete 结果失败: {err}"))
    }

    fn list_versions(&self, input: &Value) -> Result<String> {
        let skill_id = input["skill_id"]
            .as_str()
            .ok_or_else(|| anyhow!("skill_versions 操作缺少 skill_id 参数"))?;
        let limit = input["limit"].as_i64().unwrap_or(20);
        let items = self.block_on(
            crate::agent::runtime::runtime_io::list_skill_os_versions_with_pool(
                &self.pool, skill_id, limit,
            ),
        )?;
        serde_json::to_string_pretty(&json!({
            "action": "skill_versions",
            "items": items
        }))
        .map_err(|err| anyhow!("序列化 skill_versions 结果失败: {err}"))
    }

    fn view_version(&self, input: &Value) -> Result<String> {
        let skill_id = input["skill_id"]
            .as_str()
            .ok_or_else(|| anyhow!("skill_view_version 操作缺少 skill_id 参数"))?;
        let version_id = input["version_id"]
            .as_str()
            .ok_or_else(|| anyhow!("skill_view_version 操作缺少 version_id 参数"))?;
        let view = self.block_on(
            crate::agent::runtime::runtime_io::view_skill_os_version_with_pool(
                &self.pool, skill_id, version_id,
            ),
        )?;
        let Some(view) = view else {
            return Err(anyhow!("skill version 不存在: {version_id}"));
        };
        serde_json::to_string_pretty(&json!({
            "action": "skill_view_version",
            "version": view
        }))
        .map_err(|err| anyhow!("序列化 skill_view_version 结果失败: {err}"))
    }

    fn rollback_skill(&self, input: &Value, ctx: &ToolContext) -> Result<String> {
        if input["confirm"].as_bool() != Some(true) {
            return Ok("skill_rollback 是高风险操作，需要 confirm=true".to_string());
        }
        let skill_id = input["skill_id"]
            .as_str()
            .ok_or_else(|| anyhow!("skill_rollback 操作缺少 skill_id 参数"))?;
        let version_id = input["version_id"]
            .as_str()
            .ok_or_else(|| anyhow!("skill_rollback 操作缺少 version_id 参数"))?;
        let summary = input["summary"].as_str().unwrap_or_default();
        let before = self
            .block_on(
                crate::agent::runtime::runtime_io::view_skill_os_entry_with_pool(
                    &self.pool, skill_id,
                ),
            )?
            .ok_or_else(|| anyhow!("Skill 不存在: {skill_id}"))?;
        let view = self.block_on(
            crate::agent::runtime::runtime_io::rollback_skill_os_entry_with_pool(
                &self.pool, skill_id, version_id, summary,
            ),
        )?;
        let diff = Self::line_diff(&before.content, &view.content);
        let growth_event_id = self.block_on(Self::record_growth_event(
            &self.pool,
            ctx,
            "skill_rollback",
            skill_id,
            summary,
            json!({
                "rollback_to_version_id": version_id,
                "source_type": view.entry.source.raw_source_type,
                "diff": diff
            }),
        ))?;
        serde_json::to_string_pretty(&json!({
            "action": "skill_rollback",
            "skill": view,
            "rollback_to_version_id": version_id,
            "growth_event_id": growth_event_id,
            "diff": diff
        }))
        .map_err(|err| anyhow!("序列化 skill_rollback 结果失败: {err}"))
    }

    fn reset_skill(&self, input: &Value, ctx: &ToolContext) -> Result<String> {
        if input["confirm"].as_bool() != Some(true) {
            return Ok("skill_reset 是高风险操作，需要 confirm=true".to_string());
        }
        let skill_id = input["skill_id"]
            .as_str()
            .ok_or_else(|| anyhow!("skill_reset 操作缺少 skill_id 参数"))?;
        let summary = input["summary"].as_str().unwrap_or_default();
        let before = self
            .block_on(
                crate::agent::runtime::runtime_io::view_skill_os_entry_with_pool(
                    &self.pool, skill_id,
                ),
            )?
            .ok_or_else(|| anyhow!("Skill 不存在: {skill_id}"))?;
        let (view, reset_to_version_id) = self.block_on(
            crate::agent::runtime::runtime_io::reset_skill_os_entry_with_pool(
                &self.pool, skill_id, summary,
            ),
        )?;
        let diff = Self::line_diff(&before.content, &view.content);
        let version_id = self
            .block_on(
                crate::agent::runtime::runtime_io::list_skill_os_versions_with_pool(
                    &self.pool, skill_id, 1,
                ),
            )?
            .into_iter()
            .next()
            .map(|version| version.version_id)
            .unwrap_or_default();
        let growth_event_id = self.block_on(Self::record_growth_event(
            &self.pool,
            ctx,
            "skill_reset",
            skill_id,
            summary,
            json!({
                "version_id": version_id,
                "reset_to_version_id": reset_to_version_id,
                "source_type": view.entry.source.raw_source_type,
                "diff": diff
            }),
        ))?;
        serde_json::to_string_pretty(&json!({
            "action": "skill_reset",
            "skill": view,
            "version_id": version_id,
            "reset_to_version_id": reset_to_version_id,
            "growth_event_id": growth_event_id,
            "diff": diff
        }))
        .map_err(|err| anyhow!("序列化 skill_reset 结果失败: {err}"))
    }
}

impl Tool for SkillOsTool {
    fn name(&self) -> &str {
        "skills"
    }

    fn description(&self) -> &str {
        "Skill OS 工具。使用 skills_list 查看摘要，skill_view 按需加载详情；可对 local/preset/agent_created skill 执行 skill_create、skill_patch、skill_archive、skill_restore、skill_delete、skill_versions、skill_view_version、skill_rollback、skill_reset；不会解包或改写 .skillpack。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["skills_list", "skill_view", "skill_create", "skill_patch", "skill_archive", "skill_restore", "skill_delete", "skill_versions", "skill_view_version", "skill_rollback", "skill_reset"],
                    "description": "skills_list 列出技能索引；skill_view 查看单个技能详情；skill_create 创建 agent_created skill；skill_patch 修改可变更技能；skill_archive/skill_restore/skill_delete 管理生命周期；skill_versions/view_version/rollback/reset 管理版本"
                },
                "skill_id": {
                    "type": "string",
                    "description": "skill_view/skill_patch/skill_archive/skill_restore/skill_delete/skill_versions/skill_view_version/skill_rollback/skill_reset 需要的 skill id"
                },
                "name": {
                    "type": "string",
                    "description": "skill_create 的技能名称"
                },
                "description": {
                    "type": "string",
                    "description": "skill_create 的技能描述"
                },
                "content": {
                    "type": "string",
                    "description": "skill_create/skill_patch 写入的完整 SKILL.md 内容"
                },
                "instructions": {
                    "type": "string",
                    "description": "skill_create 的正文说明；当模型还没有生成完整 SKILL.md 时，可用它自动组装为 SKILL.md"
                },
                "version_id": {
                    "type": "string",
                    "description": "skill_view_version/skill_rollback 需要的 skill version id"
                },
                "summary": {
                    "type": "string",
                    "description": "skill_patch/skill_delete/skill_rollback/skill_reset 的变更摘要"
                },
                "confirm": {
                    "type": "boolean",
                    "description": "skill_archive/skill_delete/skill_rollback/skill_reset 需要 confirm=true"
                },
                "limit": {
                    "type": "integer",
                    "description": "skill_versions 返回数量"
                }
            },
            "required": ["action"]
        })
    }

    fn execute(&self, input: Value, _ctx: &ToolContext) -> Result<String> {
        match input["action"].as_str().unwrap_or_default() {
            "skills_list" => self.list_skills(),
            "skill_view" => self.view_skill(&input),
            "skill_create" => self.create_skill(&input, _ctx),
            "skill_patch" => self.patch_skill(&input, _ctx),
            "skill_archive" => self.archive_skill(&input, _ctx),
            "skill_restore" => self.restore_skill(&input, _ctx),
            "skill_delete" => self.delete_skill(&input, _ctx),
            "skill_versions" => self.list_versions(&input),
            "skill_view_version" => self.view_version(&input),
            "skill_rollback" => self.rollback_skill(&input, _ctx),
            "skill_reset" => self.reset_skill(&input, _ctx),
            action => Err(anyhow!("未知 skills 操作: {}", action)),
        }
    }
}
