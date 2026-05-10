use anyhow::{Result, anyhow};
use chrono::Utc;
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::agent::types::{Tool, ToolContext};

#[derive(Debug, Clone, Serialize)]
struct CuratorFinding {
    kind: String,
    severity: String,
    target_type: String,
    target_id: String,
    summary: String,
    evidence: Value,
    suggested_action: String,
    reversible: bool,
}

#[derive(Debug, Clone)]
struct SkillCurationScore {
    stale_score: i64,
    improvement_score: i64,
    reasons: Vec<String>,
    low_value: bool,
    actively_used: bool,
}

impl SkillCurationScore {
    fn for_skill(
        content_chars: usize,
        low_value_manifest: bool,
        use_count: i64,
        patch_count: i64,
    ) -> Self {
        let mut stale_score = 0;
        let mut improvement_score = 0;
        let mut reasons = Vec::new();
        let low_value = content_chars < 24 || low_value_manifest;

        if content_chars < 24 {
            stale_score += 45;
            improvement_score += 45;
            reasons.push("content_too_short".to_string());
        }
        if low_value_manifest {
            stale_score += 35;
            improvement_score += 35;
            reasons.push("low_value_manifest".to_string());
        }
        if use_count > 0 {
            stale_score -= 70;
            improvement_score += 30;
            reasons.push("has_runtime_use".to_string());
        }
        if patch_count > 0 {
            stale_score -= 35;
            improvement_score += 20;
            reasons.push("has_patch_history".to_string());
        }

        Self {
            stale_score: stale_score.max(0),
            improvement_score: improvement_score.max(0),
            reasons,
            low_value,
            actively_used: use_count > 0 || patch_count > 0,
        }
    }

    fn should_mark_stale(&self) -> bool {
        self.low_value && !self.actively_used && self.stale_score >= 60
    }

    fn should_suggest_improvement(&self) -> bool {
        self.low_value && self.actively_used && self.improvement_score >= 45
    }
}

pub struct CuratorTool {
    pool: SqlitePool,
    profile_id: String,
    memory_dir: PathBuf,
}

impl CuratorTool {
    pub fn new(pool: SqlitePool, profile_id: String, memory_dir: PathBuf) -> Self {
        Self {
            pool,
            profile_id: profile_id.trim().to_string(),
            memory_dir,
        }
    }

    fn block_on<T, F>(&self, fut: F) -> Result<T>
    where
        F: std::future::Future<Output = std::result::Result<T, String>>,
    {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|err| anyhow!("构建 curator tool runtime 失败: {err}"))?;
        rt.block_on(fut).map_err(|err| anyhow!(err))
    }

    async fn ensure_curator_schema(pool: &SqlitePool) -> std::result::Result<(), String> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS curator_runs (
                id TEXT PRIMARY KEY,
                profile_id TEXT NOT NULL DEFAULT '',
                scope TEXT NOT NULL DEFAULT 'profile',
                summary TEXT NOT NULL DEFAULT '',
                report_json TEXT NOT NULL DEFAULT '{}',
                report_path TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL
            )",
        )
        .execute(pool)
        .await
        .map_err(|e| format!("创建 curator_runs 表失败: {e}"))?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_curator_runs_profile_created
             ON curator_runs(profile_id, created_at DESC)",
        )
        .execute(pool)
        .await
        .map_err(|e| format!("创建 curator_runs 索引失败: {e}"))?;

        Ok(())
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

        Ok(())
    }

    fn profile_home(&self) -> Option<PathBuf> {
        let parent = self.memory_dir.parent()?;
        if parent.file_name().is_some_and(|name| name == "memories") {
            return parent.parent().map(Path::to_path_buf);
        }
        Some(parent.to_path_buf())
    }

    fn report_path(&self, run_id: &str) -> Option<PathBuf> {
        let profile_home = self.profile_home()?;
        Some(
            profile_home
                .join("curator")
                .join("reports")
                .join(format!("{run_id}.json")),
        )
    }

    fn scan_memory(&self) -> Vec<CuratorFinding> {
        let memory_path = self.memory_dir.join("MEMORY.md");
        let content = std::fs::read_to_string(&memory_path).unwrap_or_default();
        let mut findings = Vec::new();
        let mut seen: HashMap<String, (usize, String)> = HashMap::new();
        let mut duplicate_lines = Vec::new();
        let mut reusable_lines = Vec::new();
        let mut low_value_lines = Vec::new();

        for (index, line) in content.lines().enumerate() {
            let trimmed = line.trim().trim_start_matches(['-', '*', ' ']).trim();
            if trimmed.is_empty() {
                continue;
            }
            let normalized = trimmed.to_lowercase();
            if let Some((first_line, original)) = seen.get(&normalized) {
                duplicate_lines.push(json!({
                    "first_line": first_line,
                    "duplicate_line": index + 1,
                    "content": original
                }));
            } else {
                seen.insert(normalized, (index + 1, trimmed.to_string()));
            }

            if trimmed.chars().count() <= 6 {
                low_value_lines.push(json!({
                    "line": index + 1,
                    "content": trimmed
                }));
            }

            let lower = trimmed.to_lowercase();
            if trimmed.chars().count() >= 20
                && (trimmed.contains("流程")
                    || trimmed.contains("步骤")
                    || lower.contains("workflow")
                    || lower.contains("playbook"))
            {
                reusable_lines.push(json!({
                    "line": index + 1,
                    "content": trimmed
                }));
            }
        }

        if !duplicate_lines.is_empty() {
            findings.push(CuratorFinding {
                kind: "duplicate_memory".to_string(),
                severity: "medium".to_string(),
                target_type: "memory".to_string(),
                target_id: "MEMORY.md".to_string(),
                summary: "Profile Memory 中存在重复条目".to_string(),
                evidence: json!({ "duplicates": duplicate_lines }),
                suggested_action: "用 memory.replace 合并重复条目，保留最清晰的一条".to_string(),
                reversible: true,
            });
        }

        if !reusable_lines.is_empty() {
            findings.push(CuratorFinding {
                kind: "reusable_skill_candidate".to_string(),
                severity: "low".to_string(),
                target_type: "memory".to_string(),
                target_id: "MEMORY.md".to_string(),
                summary: "记忆中出现可沉淀为技能的流程型经验".to_string(),
                evidence: json!({ "candidates": reusable_lines }),
                suggested_action: "用 skills.skill_create 将成熟流程沉淀为 agent_created skill"
                    .to_string(),
                reversible: true,
            });
        }

        if !low_value_lines.is_empty() {
            findings.push(CuratorFinding {
                kind: "low_value_debris".to_string(),
                severity: "low".to_string(),
                target_type: "memory".to_string(),
                target_id: "MEMORY.md".to_string(),
                summary: "Profile Memory 中存在过短或信息量不足的条目".to_string(),
                evidence: json!({ "lines": low_value_lines }),
                suggested_action: "用 memory.replace 整理低信息密度条目".to_string(),
                reversible: true,
            });
        }

        findings
    }

    async fn scan_skills_with_pool(
        pool: &SqlitePool,
        mutate: bool,
    ) -> std::result::Result<Vec<CuratorFinding>, String> {
        let has_table: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'installed_skills'",
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        if has_table == 0 {
            return Ok(Vec::new());
        }

        crate::agent::runtime::runtime_io::ensure_skill_os_lifecycle_schema_with_pool(pool).await?;
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                String,
                i64,
                i64,
                i64,
                String,
                i64,
            ),
        >(
            "SELECT s.id, s.manifest, s.pack_path, COALESCE(s.source_type, 'encrypted'),
                    COALESCE(l.state, 'active'),
                    COALESCE(l.view_count, 0),
                    COALESCE(l.use_count, 0),
                    COALESCE(l.patch_count, 0),
                    COALESCE(l.last_used_at, ''),
                    COALESCE(l.pinned, 0)
             FROM installed_skills s
             LEFT JOIN skill_lifecycle l ON l.skill_id = s.id
             ORDER BY s.installed_at DESC",
        )
        .fetch_all(pool)
        .await
        .map_err(|e| format!("读取 installed_skills 失败: {e}"))?;

        let mut findings = Vec::new();
        for (
            skill_id,
            manifest_json,
            pack_path,
            source_type,
            lifecycle_state,
            view_count,
            use_count,
            patch_count,
            last_used_at,
            pinned,
        ) in rows
        {
            let source_type = source_type.trim().to_lowercase();
            if matches!(
                source_type.as_str(),
                "" | "encrypted" | "skillpack" | "preset" | "vendored" | "builtin"
            ) {
                continue;
            }
            let manifest =
                serde_json::from_str::<Value>(&manifest_json).unwrap_or_else(|_| json!({}));
            let name = manifest["name"].as_str().unwrap_or_default();
            let description = manifest["description"].as_str().unwrap_or_default();
            let content = std::fs::read_to_string(PathBuf::from(&pack_path).join("SKILL.md"))
                .unwrap_or_default();
            let low_value_manifest = name.to_lowercase().contains("draft")
                || description.trim().eq_ignore_ascii_case("todo")
                || description.trim().len() < 8;
            let content_chars = content.trim().chars().count();
            let score = SkillCurationScore::for_skill(
                content_chars,
                low_value_manifest,
                use_count,
                patch_count,
            );
            if score.low_value {
                if pinned != 0 {
                    findings.push(CuratorFinding {
                        kind: "pinned_skill_protected".to_string(),
                        severity: "low".to_string(),
                        target_type: "skill".to_string(),
                        target_id: skill_id,
                        summary: "Pinned skill 命中整理规则但已跳过".to_string(),
                        evidence: json!({
                            "source_type": source_type,
                            "state": lifecycle_state,
                            "view_count": view_count,
                            "use_count": use_count,
                            "patch_count": patch_count,
                            "last_used_at": last_used_at,
                            "pinned": true,
                            "curator_score": {
                                "stale_score": score.stale_score,
                                "improvement_score": score.improvement_score,
                                "reasons": score.reasons
                            }
                        }),
                        suggested_action: "保持 pinned；如需整理，先取消 pin".to_string(),
                        reversible: true,
                    });
                    continue;
                }
                if score.should_suggest_improvement() {
                    findings.push(CuratorFinding {
                        kind: "skill_improvement_candidate".to_string(),
                        severity: "low".to_string(),
                        target_type: "skill".to_string(),
                        target_id: skill_id,
                        summary: "可变技能正在被使用，但说明仍像草稿，建议补全而不是标记 stale"
                            .to_string(),
                        evidence: json!({
                            "source_type": source_type,
                            "state": lifecycle_state,
                            "view_count": view_count,
                            "use_count": use_count,
                            "patch_count": patch_count,
                            "last_used_at": last_used_at,
                            "pinned": false,
                            "name": name,
                            "description": description,
                            "content_chars": content_chars,
                            "curator_score": {
                                "stale_score": score.stale_score,
                                "improvement_score": score.improvement_score,
                                "reasons": score.reasons
                            }
                        }),
                        suggested_action:
                            "用 skills.skill_patch 补全触发条件、步骤、边界和工具需求".to_string(),
                        reversible: true,
                    });
                    continue;
                }
                if !score.should_mark_stale() {
                    continue;
                }
                let marked_stale = if mutate && lifecycle_state == "active" {
                    crate::agent::runtime::runtime_io::mark_skill_os_stale_with_pool(
                        pool, &skill_id,
                    )
                    .await?
                } else {
                    false
                };
                findings.push(CuratorFinding {
                    kind: if marked_stale {
                        "stale_skill".to_string()
                    } else {
                        "stale_skill_candidate".to_string()
                    },
                    severity: "low".to_string(),
                    target_type: "skill".to_string(),
                    target_id: skill_id,
                    summary: if marked_stale {
                        "可变技能内容过短或仍像草稿，已标记为 stale".to_string()
                    } else {
                        "可变技能内容过短或仍像草稿".to_string()
                    },
                    evidence: json!({
                        "source_type": source_type,
                        "state": lifecycle_state,
                        "state_changed": marked_stale,
                        "view_count": view_count,
                        "use_count": use_count,
                        "patch_count": patch_count,
                        "last_used_at": last_used_at,
                        "pinned": false,
                        "name": name,
                        "description": description,
                        "content_chars": content_chars,
                        "curator_score": {
                            "stale_score": score.stale_score,
                            "improvement_score": score.improvement_score,
                            "reasons": score.reasons
                        }
                    }),
                    suggested_action: if marked_stale {
                        "后续 curator run 可将长期 stale 且未 pinned 的技能归档".to_string()
                    } else {
                        "用 curator.run 标记 stale，或用 skills.skill_patch 补全说明".to_string()
                    },
                    reversible: true,
                });
            }
        }

        Ok(findings)
    }

    pub async fn scan_profile_with_pool(
        pool: SqlitePool,
        profile_id: String,
        memory_dir: PathBuf,
        mutate: bool,
    ) -> std::result::Result<Value, String> {
        let tool = Self::new(pool, profile_id, memory_dir);
        let mut findings = tool.scan_memory();
        findings.extend(Self::scan_skills_with_pool(&tool.pool, mutate).await?);
        let run_id = format!("cur_{}", uuid::Uuid::new_v4().simple());
        let created_at = Utc::now().to_rfc3339();
        let summary = if findings.is_empty() {
            "未发现需要整理的记忆或技能".to_string()
        } else {
            format!("发现 {} 个可整理项", findings.len())
        };
        let report = json!({
            "run_id": run_id,
            "profile_id": tool.profile_id,
            "scope": "profile",
            "mode": if mutate { "run" } else { "scan" },
            "summary": summary,
            "created_at": created_at,
            "findings": findings
        });
        let report_path = tool.report_path(&run_id);
        if let Some(path) = &report_path {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|err| format!("创建 curator report 目录失败: {err}"))?;
            }
            std::fs::write(
                path,
                serde_json::to_string_pretty(&report)
                    .map_err(|err| format!("序列化 curator report 失败: {err}"))?,
            )
            .map_err(|err| format!("写入 curator report 失败: {err}"))?;
        }
        let report_path_string = report_path
            .as_ref()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default();

        Self::persist_run_with_pool(
            &tool.pool,
            &run_id,
            &tool.profile_id,
            &summary,
            &report,
            &report_path_string,
            &created_at,
            "curator_scan",
        )
        .await?;

        Ok(json!({
            "action": if mutate { "run" } else { "scan" },
            "run_id": run_id,
            "profile_id": tool.profile_id,
            "summary": summary,
            "report_path": report_path_string,
            "findings": report["findings"]
        }))
    }

    fn scan_with_mode(&self, mutate: bool) -> Result<String> {
        let result = self.block_on(Self::scan_profile_with_pool(
            self.pool.clone(),
            self.profile_id.clone(),
            self.memory_dir.clone(),
            mutate,
        ))?;
        serde_json::to_string_pretty(&result)
            .map_err(|err| anyhow!("序列化 curator scan 结果失败: {err}"))
    }

    fn scan(&self) -> Result<String> {
        self.scan_with_mode(false)
    }

    fn run(&self) -> Result<String> {
        self.scan_with_mode(true)
    }

    fn restore(&self, input: &Value) -> Result<String> {
        let skill_id = input["skill_id"]
            .as_str()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow!("curator.restore 缺少 skill_id 参数"))?;
        let restored = self.block_on(
            crate::agent::runtime::runtime_io::restore_stale_skill_os_with_pool(
                &self.pool, skill_id,
            ),
        )?;
        let run_id = format!("cur_{}", uuid::Uuid::new_v4().simple());
        let created_at = Utc::now().to_rfc3339();
        let summary = if restored {
            format!("已将 stale skill 恢复为 active: {skill_id}")
        } else {
            format!("未执行恢复：Skill 当前不是 stale: {skill_id}")
        };
        let report = json!({
            "run_id": run_id,
            "profile_id": self.profile_id,
            "scope": "profile",
            "mode": "restore",
            "summary": summary,
            "created_at": created_at,
            "findings": [{
                "kind": "curator_restore",
                "severity": "low",
                "target_type": "skill",
                "target_id": skill_id,
                "summary": summary,
                "evidence": {
                    "state_changed": restored,
                    "restored_to": if restored { "active" } else { "" }
                },
                "suggested_action": if restored {
                    "继续观察该技能的 use_count、patch_count 和后续任务表现"
                } else {
                    "无需恢复；如需恢复 archived skill，请使用 skills.skill_restore"
                },
                "reversible": true
            }]
        });
        let report_path = self.report_path(&run_id);
        if let Some(path) = &report_path {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, serde_json::to_string_pretty(&report)?)?;
        }
        let report_path_string = report_path
            .as_ref()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default();

        self.block_on(Self::persist_run_with_pool(
            &self.pool,
            &run_id,
            &self.profile_id,
            &summary,
            &report,
            &report_path_string,
            &created_at,
            "curator_restore",
        ))?;

        serde_json::to_string_pretty(&json!({
            "action": "restore",
            "run_id": run_id,
            "profile_id": self.profile_id,
            "skill_id": skill_id,
            "restored": restored,
            "summary": summary,
            "report_path": report_path_string,
            "findings": report["findings"]
        }))
        .map_err(|err| anyhow!("序列化 curator restore 结果失败: {err}"))
    }

    async fn persist_run_with_pool(
        pool: &SqlitePool,
        run_id: &str,
        profile_id: &str,
        summary: &str,
        report: &Value,
        report_path: &str,
        created_at: &str,
        event_type: &str,
    ) -> std::result::Result<(), String> {
        Self::ensure_curator_schema(pool).await?;
        Self::ensure_growth_events_schema(pool).await?;
        let report_json = serde_json::to_string(report)
            .map_err(|e| format!("序列化 curator report 失败: {e}"))?;
        sqlx::query(
            "INSERT INTO curator_runs (id, profile_id, scope, summary, report_json, report_path, created_at)
             VALUES (?, ?, 'profile', ?, ?, ?, ?)",
        )
        .bind(run_id)
        .bind(profile_id)
        .bind(summary)
        .bind(&report_json)
        .bind(report_path)
        .bind(created_at)
        .execute(pool)
        .await
        .map_err(|e| format!("写入 curator_runs 失败: {e}"))?;

        sqlx::query(
            "INSERT INTO growth_events (
                id, profile_id, session_id, event_type, target_type, target_id, summary, evidence_json, created_at
             ) VALUES (?, ?, '', ?, 'curator', ?, ?, ?, ?)",
        )
        .bind(format!("grw_{}", uuid::Uuid::new_v4().simple()))
        .bind(profile_id)
        .bind(event_type)
        .bind(run_id)
        .bind(summary)
        .bind(report_json)
        .bind(created_at)
        .execute(pool)
        .await
        .map_err(|e| format!("写入 curator growth event 失败: {e}"))?;

        Ok(())
    }

    fn history(&self, input: &Value) -> Result<String> {
        let limit = input["limit"].as_i64().unwrap_or(10).clamp(1, 50);
        let items = self.block_on(Self::history_with_pool(&self.pool, &self.profile_id, limit))?;
        serde_json::to_string_pretty(&json!({
            "action": "history",
            "source": "curator_runs",
            "profile_id": self.profile_id,
            "items": items
        }))
        .map_err(|err| anyhow!("序列化 curator history 失败: {err}"))
    }

    fn project_history_item(
        id: String,
        summary: String,
        report_json: String,
        report_path: String,
        created_at: String,
    ) -> Value {
        let report = serde_json::from_str::<Value>(&report_json).unwrap_or_else(|_| json!({}));
        let mode = report["mode"].as_str().unwrap_or("scan").to_string();
        let findings = report["findings"].as_array().cloned().unwrap_or_default();
        let mut changed_targets = Vec::new();
        let mut restore_candidates = Vec::new();

        for finding in findings {
            let target_type = finding["target_type"].as_str().unwrap_or_default();
            let target_id = finding["target_id"].as_str().unwrap_or_default();
            let kind = finding["kind"].as_str().unwrap_or_default();
            let evidence = &finding["evidence"];
            let state_changed = evidence["state_changed"].as_bool().unwrap_or(false);
            let restored_to = evidence["restored_to"].as_str().unwrap_or_default();
            let reversible = finding["reversible"].as_bool().unwrap_or(false);

            if state_changed || !restored_to.is_empty() {
                changed_targets.push(json!({
                    "kind": kind,
                    "target_type": target_type,
                    "target_id": target_id,
                    "state_changed": state_changed,
                    "restored_to": restored_to,
                    "suggested_action": finding["suggested_action"].as_str().unwrap_or_default(),
                    "reversible": reversible
                }));
            }

            if kind == "stale_skill" && target_type == "skill" && state_changed && reversible {
                restore_candidates.push(json!({
                    "target_type": "skill",
                    "target_id": target_id,
                    "tool": "curator",
                    "action": "restore",
                    "input": {
                        "action": "restore",
                        "skill_id": target_id
                    }
                }));
            }
        }

        json!({
            "id": id,
            "mode": mode,
            "summary": summary,
            "changed_targets": changed_targets,
            "restore_candidates": restore_candidates,
            "has_state_changes": !changed_targets.is_empty(),
            "report": report,
            "report_path": report_path,
            "created_at": created_at
        })
    }

    async fn history_with_pool(
        pool: &SqlitePool,
        profile_id: &str,
        limit: i64,
    ) -> std::result::Result<Vec<Value>, String> {
        Self::ensure_curator_schema(pool).await?;
        let rows = sqlx::query_as::<_, (String, String, String, String, String)>(
            "SELECT id, summary, report_json, report_path, created_at
             FROM curator_runs
             WHERE profile_id = ?
             ORDER BY created_at DESC, id DESC
             LIMIT ?",
        )
        .bind(profile_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("读取 curator history 失败: {e}"))?;
        Ok(rows
            .into_iter()
            .map(|(id, summary, report_json, report_path, created_at)| {
                Self::project_history_item(id, summary, report_json, report_path, created_at)
            })
            .collect())
    }
}

impl Tool for CuratorTool {
    fn name(&self) -> &str {
        "curator"
    }

    fn description(&self) -> &str {
        "Curator 工具。扫描当前 profile 的记忆和技能，找出重复记忆、可沉淀技能、低价值碎片；run 可执行 Hermes-aligned stale 标记；restore 可将 stale skill 恢复为 active；所有动作写入 curator_runs/growth_events。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["scan", "run", "restore", "history"],
                    "description": "scan 生成 dry report；run 执行 Hermes-aligned stale 标记；restore 恢复 stale skill；history 查看最近报告"
                },
                "skill_id": {
                    "type": "string",
                    "description": "restore 需要恢复的 stale skill id"
                },
                "limit": {
                    "type": "integer",
                    "description": "history 返回数量"
                }
            },
            "required": ["action"]
        })
    }

    fn execute(&self, input: Value, _ctx: &ToolContext) -> Result<String> {
        match input["action"].as_str().unwrap_or("scan") {
            "scan" => self.scan(),
            "run" => self.run(),
            "restore" => self.restore(&input),
            "history" => self.history(&input),
            action => Err(anyhow!("未知 curator 操作: {}", action)),
        }
    }
}
