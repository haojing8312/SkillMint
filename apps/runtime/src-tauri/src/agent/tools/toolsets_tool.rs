use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::agent::tool_manifest::{ToolCategory, ToolManifestEntry, ToolSource};
use crate::agent::types::{Tool, ToolContext};
use crate::agent::ToolRegistry;

const KNOWN_TOOLSETS: &[&str] = &[
    "core", "memory", "skills", "web", "browser", "im", "desktop", "media", "mcp",
];

#[derive(Debug, Clone, Serialize)]
struct ToolsetProjectionEntry {
    name: String,
    display_name: String,
    description: String,
    category: ToolCategory,
    source: ToolSource,
    read_only: bool,
    destructive: bool,
    open_world: bool,
    requires_approval: bool,
    toolsets: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ToolsetProjectionGroup {
    toolset: String,
    tools: Vec<ToolsetProjectionEntry>,
}

pub struct ToolsetsTool {
    registry: Arc<ToolRegistry>,
    profile_policy_pool: Option<SqlitePool>,
    profile_id: Option<String>,
}

impl ToolsetsTool {
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self {
            registry,
            profile_policy_pool: None,
            profile_id: None,
        }
    }

    pub fn with_profile_policy(mut self, pool: SqlitePool, profile_id: impl Into<String>) -> Self {
        self.profile_policy_pool = Some(pool);
        self.profile_id = Some(profile_id.into());
        self
    }

    fn normalize_toolset_list(raw: &Value) -> Result<Vec<String>> {
        let values = match raw {
            Value::Array(items) => items
                .iter()
                .map(|item| {
                    item.as_str()
                        .map(str::to_string)
                        .ok_or_else(|| anyhow!("allowed_toolsets 只能包含字符串"))
                })
                .collect::<Result<Vec<_>>>()?,
            Value::String(value) => value
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect(),
            Value::Null => Vec::new(),
            _ => return Err(anyhow!("allowed_toolsets 必须是数组或逗号分隔字符串")),
        };
        let known = KNOWN_TOOLSETS.iter().copied().collect::<BTreeSet<_>>();
        let mut normalized = BTreeSet::new();
        for value in values {
            let item = value.trim().to_ascii_lowercase();
            if item.is_empty() {
                continue;
            }
            if !known.contains(item.as_str()) {
                return Err(anyhow!("未知 toolset: {item}"));
            }
            normalized.insert(item);
        }
        Ok(normalized.into_iter().collect())
    }

    fn block_on<T, F>(&self, fut: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|err| anyhow!("构建 toolsets runtime 失败: {err}"))?;
        rt.block_on(fut)
    }

    fn profile_scope(&self) -> Result<(SqlitePool, String)> {
        let pool = self
            .profile_policy_pool
            .clone()
            .ok_or_else(|| anyhow!("当前 toolsets 工具未绑定 profile policy 存储"))?;
        let profile_id = self
            .profile_id
            .as_deref()
            .unwrap_or_default()
            .trim()
            .to_string();
        if profile_id.is_empty() {
            return Err(anyhow!("当前 session 未绑定 profile_id"));
        }
        Ok((pool, profile_id))
    }

    async fn ensure_profile_policy_schema(pool: &SqlitePool) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS profile_toolset_policies (
                profile_id TEXT PRIMARY KEY,
                allowed_toolsets_json TEXT NOT NULL DEFAULT '[]',
                updated_at TEXT NOT NULL
            )",
        )
        .execute(pool)
        .await
        .map_err(|err| anyhow!("创建 profile_toolset_policies 表失败: {err}"))?;
        Ok(())
    }

    async fn load_profile_policy(pool: SqlitePool, profile_id: String) -> Result<Value> {
        Self::ensure_profile_policy_schema(&pool).await?;
        let raw = sqlx::query_scalar::<_, String>(
            "SELECT allowed_toolsets_json
             FROM profile_toolset_policies
             WHERE profile_id = ?",
        )
        .bind(&profile_id)
        .fetch_optional(&pool)
        .await
        .map_err(|err| anyhow!("读取 profile toolset policy 失败: {err}"))?
        .unwrap_or_else(|| "[]".to_string());
        let allowed_toolsets = serde_json::from_str::<Vec<String>>(&raw).unwrap_or_default();
        Ok(json!({
            "profile_id": profile_id,
            "allowed_toolsets": allowed_toolsets,
            "enforced": false
        }))
    }

    async fn save_profile_policy(
        pool: SqlitePool,
        profile_id: String,
        allowed_toolsets: Vec<String>,
    ) -> Result<Value> {
        Self::ensure_profile_policy_schema(&pool).await?;
        let raw = serde_json::to_string(&allowed_toolsets)
            .map_err(|err| anyhow!("序列化 profile toolset policy 失败: {err}"))?;
        sqlx::query(
            "INSERT INTO profile_toolset_policies (
                profile_id, allowed_toolsets_json, updated_at
             ) VALUES (?, ?, datetime('now'))
             ON CONFLICT(profile_id) DO UPDATE SET
                allowed_toolsets_json = excluded.allowed_toolsets_json,
                updated_at = excluded.updated_at",
        )
        .bind(&profile_id)
        .bind(raw)
        .execute(&pool)
        .await
        .map_err(|err| anyhow!("保存 profile toolset policy 失败: {err}"))?;
        Ok(json!({
            "profile_id": profile_id,
            "allowed_toolsets": allowed_toolsets,
            "enforced": false
        }))
    }

    fn toolsets_for_entry(entry: &ToolManifestEntry) -> Vec<String> {
        let mut toolsets = BTreeSet::new();
        let name = entry.name.to_ascii_lowercase();

        match entry.category {
            ToolCategory::Memory => {
                toolsets.insert("memory");
            }
            ToolCategory::Search | ToolCategory::Web => {
                toolsets.insert("web");
            }
            ToolCategory::Browser => {
                toolsets.insert("browser");
            }
            ToolCategory::Agent => {
                toolsets.insert("skills");
            }
            ToolCategory::System => {
                toolsets.insert("desktop");
            }
            ToolCategory::File
            | ToolCategory::Shell
            | ToolCategory::Planning
            | ToolCategory::Other => {
                toolsets.insert("core");
            }
            ToolCategory::Integration => {
                toolsets.insert("core");
            }
        }

        match entry.source {
            ToolSource::Mcp => {
                toolsets.insert("mcp");
            }
            ToolSource::Sidecar => {
                toolsets.insert("browser");
            }
            ToolSource::Native | ToolSource::Runtime | ToolSource::Plugin | ToolSource::Alias => {}
        }

        if name == "memory" {
            toolsets.insert("memory");
        }
        if matches!(
            name.as_str(),
            "skills" | "skill" | "task" | "curator" | "employee_manage"
        ) || name.contains("skill")
        {
            toolsets.insert("skills");
        }
        if name.starts_with("browser_") {
            toolsets.insert("browser");
        }
        if name.starts_with("mcp_") {
            toolsets.insert("mcp");
        }
        if name.starts_with("im_") || name.contains("feishu") || name.contains("wecom") {
            toolsets.insert("im");
        }
        if matches!(
            name.as_str(),
            "open_in_folder" | "screenshot" | "document_analyze" | "vision_analyze"
        ) {
            toolsets.insert("desktop");
        }
        if matches!(name.as_str(), "vision_analyze" | "screenshot") {
            toolsets.insert("media");
        }
        if matches!(
            name.as_str(),
            "web_search" | "web_fetch" | "clawhub_search" | "clawhub_recommend"
        ) {
            toolsets.insert("web");
        }

        toolsets.into_iter().map(str::to_string).collect()
    }

    fn projection_entry(entry: ToolManifestEntry) -> ToolsetProjectionEntry {
        ToolsetProjectionEntry {
            toolsets: Self::toolsets_for_entry(&entry),
            name: entry.name,
            display_name: entry.display_name,
            description: entry.description,
            category: entry.category,
            source: entry.source,
            read_only: entry.read_only,
            destructive: entry.destructive,
            open_world: entry.open_world,
            requires_approval: entry.requires_approval,
        }
    }

    fn projection_groups(&self) -> Vec<ToolsetProjectionGroup> {
        let entries = self
            .registry
            .tool_manifest_entries()
            .into_iter()
            .map(Self::projection_entry)
            .collect::<Vec<_>>();
        let mut by_toolset: BTreeMap<String, Vec<ToolsetProjectionEntry>> = BTreeMap::new();
        for name in KNOWN_TOOLSETS {
            by_toolset.entry((*name).to_string()).or_default();
        }
        for entry in entries {
            for toolset in &entry.toolsets {
                by_toolset
                    .entry(toolset.clone())
                    .or_default()
                    .push(entry.clone());
            }
        }
        by_toolset
            .into_iter()
            .map(|(toolset, mut tools)| {
                tools.sort_by(|left, right| left.name.cmp(&right.name));
                ToolsetProjectionGroup { toolset, tools }
            })
            .collect()
    }

    fn list(&self) -> Result<String> {
        let groups = self.projection_groups();
        let profile_policy = self.profile_scope().ok().and_then(|(pool, profile_id)| {
            self.block_on(Self::load_profile_policy(pool, profile_id))
                .ok()
        });
        let summary = groups
            .iter()
            .map(|group| {
                json!({
                    "toolset": group.toolset,
                    "tool_count": group.tools.len()
                })
            })
            .collect::<Vec<_>>();
        serde_json::to_string_pretty(&json!({
            "action": "list",
            "summary": summary,
            "groups": groups,
            "profile_policy": profile_policy
        }))
        .map_err(|err| anyhow!("序列化 toolsets list 失败: {err}"))
    }

    fn view(&self, input: &Value) -> Result<String> {
        let requested = input["toolset"]
            .as_str()
            .ok_or_else(|| anyhow!("toolsets.view 缺少 toolset 参数"))?
            .trim()
            .to_ascii_lowercase();
        if requested.is_empty() {
            return Err(anyhow!("toolset 不能为空"));
        }
        let group = self
            .projection_groups()
            .into_iter()
            .find(|group| group.toolset == requested)
            .unwrap_or(ToolsetProjectionGroup {
                toolset: requested,
                tools: Vec::new(),
            });
        serde_json::to_string_pretty(&json!({
            "action": "view",
            "group": group
        }))
        .map_err(|err| anyhow!("序列化 toolsets view 失败: {err}"))
    }

    fn profile_policy(&self) -> Result<String> {
        let (pool, profile_id) = self.profile_scope()?;
        let profile_policy = self.block_on(Self::load_profile_policy(pool, profile_id))?;
        serde_json::to_string_pretty(&json!({
            "action": "profile_policy",
            "profile_policy": profile_policy
        }))
        .map_err(|err| anyhow!("序列化 profile toolset policy 失败: {err}"))
    }

    fn set_profile_policy(&self, input: &Value) -> Result<String> {
        let allowed_toolsets = Self::normalize_toolset_list(&input["allowed_toolsets"])?;
        let (pool, profile_id) = self.profile_scope()?;
        let profile_policy = self.block_on(Self::save_profile_policy(
            pool,
            profile_id,
            allowed_toolsets,
        ))?;
        serde_json::to_string_pretty(&json!({
            "action": "set_profile_policy",
            "profile_policy": profile_policy
        }))
        .map_err(|err| anyhow!("序列化 profile toolset policy 失败: {err}"))
    }
}

impl Tool for ToolsetsTool {
    fn name(&self) -> &str {
        "toolsets"
    }

    fn description(&self) -> &str {
        "Toolset Gateway 投影工具。按 core/memory/skills/web/browser/im/desktop/media/mcp 查看当前运行时可用工具和风险元数据，并记录 profile 默认 toolset 偏好；不会改变执行权限。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["list", "view", "profile_policy", "set_profile_policy"],
                    "description": "list 查看所有 toolset 投影；view 查看单个 toolset；profile_policy 查看当前 profile 默认 toolsets；set_profile_policy 保存默认 allowed_toolsets"
                },
                "toolset": {
                    "type": "string",
                    "enum": ["core", "memory", "skills", "web", "browser", "im", "desktop", "media", "mcp"],
                    "description": "view 需要的 toolset 名称"
                },
                "allowed_toolsets": {
                    "oneOf": [
                        {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "enum": ["core", "memory", "skills", "web", "browser", "im", "desktop", "media", "mcp"]
                            }
                        },
                        {
                            "type": "string"
                        }
                    ],
                    "description": "set_profile_policy 需要保存的默认 allowed toolsets"
                }
            },
            "required": ["action"]
        })
    }

    fn execute(&self, input: Value, _ctx: &ToolContext) -> Result<String> {
        match input["action"].as_str().unwrap_or("list") {
            "list" => self.list(),
            "view" => self.view(&input),
            "profile_policy" => self.profile_policy(),
            "set_profile_policy" => self.set_profile_policy(&input),
            action => Err(anyhow!("未知 toolsets 操作: {}", action)),
        }
    }
}
