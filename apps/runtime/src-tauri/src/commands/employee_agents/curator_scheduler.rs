use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use tauri::State;

use crate::commands::skills::DbState;

const CURATOR_SCHEDULER_ID: &str = "global";
const DEFAULT_INTERVAL_MINUTES: i64 = 240;
const DEFAULT_MIN_IDLE_MINUTES: i64 = 15;
const SCHEDULER_TICK_SECONDS: u64 = 300;
const SCHEDULER_INITIAL_DELAY_SECONDS: u64 = 30;

#[derive(Clone, Default)]
pub struct EmployeeCuratorSchedulerState(pub Arc<AtomicBool>);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmployeeCuratorSchedulerStatus {
    pub enabled: bool,
    pub running: bool,
    pub interval_minutes: i64,
    pub min_idle_minutes: i64,
    pub active_run_count: i64,
    pub idle: bool,
    pub last_activity_at: String,
    pub last_started_at: String,
    pub last_completed_at: String,
    pub last_error: String,
    pub next_check_at: String,
    pub profile_id: Option<String>,
    pub profile_due: bool,
    pub profile_last_run_at: String,
    pub profile_last_summary: String,
}

#[derive(Debug, Clone)]
struct CuratorSchedulerConfig {
    enabled: bool,
    interval_minutes: i64,
    min_idle_minutes: i64,
    last_started_at: String,
    last_completed_at: String,
    last_error: String,
}

#[derive(Debug, Clone)]
struct CuratorProfileTarget {
    profile_id: String,
    profile_home: String,
}

#[derive(Debug, Clone)]
struct ProfileCuratorRunSnapshot {
    last_run_at: String,
    summary: String,
    due: bool,
}

#[derive(Debug, Clone)]
struct RuntimeIdleSnapshot {
    active_run_count: i64,
    idle: bool,
    last_activity_at: String,
}

pub async fn ensure_curator_scheduler_schema_with_pool(pool: &SqlitePool) -> Result<(), String> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS curator_scheduler_state (
            id TEXT PRIMARY KEY,
            enabled INTEGER NOT NULL DEFAULT 1,
            interval_minutes INTEGER NOT NULL DEFAULT 240,
            min_idle_minutes INTEGER NOT NULL DEFAULT 15,
            last_started_at TEXT NOT NULL DEFAULT '',
            last_completed_at TEXT NOT NULL DEFAULT '',
            last_error TEXT NOT NULL DEFAULT '',
            updated_at TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| format!("创建 curator_scheduler_state 表失败: {e}"))?;

    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT OR IGNORE INTO curator_scheduler_state (
            id, enabled, interval_minutes, min_idle_minutes, updated_at
         ) VALUES (?, 1, ?, ?, ?)",
    )
    .bind(CURATOR_SCHEDULER_ID)
    .bind(DEFAULT_INTERVAL_MINUTES)
    .bind(DEFAULT_MIN_IDLE_MINUTES)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| format!("初始化 curator_scheduler_state 失败: {e}"))?;

    Ok(())
}

async fn table_exists(pool: &SqlitePool, table_name: &str) -> Result<bool, String> {
    let exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM sqlite_master
         WHERE type = 'table' AND name = ?",
    )
    .bind(table_name)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(exists > 0)
}

async fn load_curator_scheduler_config_with_pool(
    pool: &SqlitePool,
) -> Result<CuratorSchedulerConfig, String> {
    ensure_curator_scheduler_schema_with_pool(pool).await?;
    let row = sqlx::query_as::<_, (i64, i64, i64, String, String, String)>(
        "SELECT enabled, interval_minutes, min_idle_minutes,
                COALESCE(last_started_at, ''),
                COALESCE(last_completed_at, ''),
                COALESCE(last_error, '')
         FROM curator_scheduler_state
         WHERE id = ?",
    )
    .bind(CURATOR_SCHEDULER_ID)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("读取 curator scheduler 状态失败: {e}"))?;

    Ok(CuratorSchedulerConfig {
        enabled: row.0 != 0,
        interval_minutes: row.1.max(15),
        min_idle_minutes: row.2.max(0),
        last_started_at: row.3,
        last_completed_at: row.4,
        last_error: row.5,
    })
}

fn parse_timestamp(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|parsed| parsed.with_timezone(&Utc))
}

fn due_since(last_run_at: &str, interval_minutes: i64, now: DateTime<Utc>) -> bool {
    parse_timestamp(last_run_at)
        .map(|last| now.signed_duration_since(last) >= ChronoDuration::minutes(interval_minutes))
        .unwrap_or(true)
}

fn next_check_at(config: &CuratorSchedulerConfig, now: DateTime<Utc>) -> String {
    if !config.enabled {
        return String::new();
    }
    let next = parse_timestamp(&config.last_completed_at)
        .map(|last| last + ChronoDuration::minutes(config.interval_minutes))
        .unwrap_or(now);
    next.to_rfc3339()
}

async fn load_runtime_idle_snapshot_with_pool(
    pool: &SqlitePool,
    min_idle_minutes: i64,
    now: DateTime<Utc>,
) -> Result<RuntimeIdleSnapshot, String> {
    if !table_exists(pool, "session_runs").await? {
        return Ok(RuntimeIdleSnapshot {
            active_run_count: 0,
            idle: true,
            last_activity_at: String::new(),
        });
    }

    let active_run_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM session_runs
         WHERE status IN ('queued', 'running')",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("读取 active session run 数量失败: {e}"))?;

    let last_activity_at: Option<String> =
        sqlx::query_scalar("SELECT MAX(updated_at) FROM session_runs")
            .fetch_one(pool)
            .await
            .map_err(|e| format!("读取 session run 更新时间失败: {e}"))?;
    let last_activity_at = last_activity_at.unwrap_or_default();
    let idle_by_activity = parse_timestamp(&last_activity_at)
        .map(|last| now.signed_duration_since(last) >= ChronoDuration::minutes(min_idle_minutes))
        .unwrap_or(true);

    Ok(RuntimeIdleSnapshot {
        active_run_count,
        idle: active_run_count == 0 && idle_by_activity,
        last_activity_at,
    })
}

async fn list_curator_profile_targets_with_pool(
    pool: &SqlitePool,
    runtime_root: &std::path::Path,
) -> Result<Vec<CuratorProfileTarget>, String> {
    if !table_exists(pool, "agent_profiles").await? {
        return Ok(Vec::new());
    }

    let rows = sqlx::query_as::<_, (String, String)>(
        "SELECT id, COALESCE(profile_home, '')
         FROM agent_profiles
         WHERE TRIM(id) <> ''
         ORDER BY updated_at DESC, id ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("读取 agent profiles 失败: {e}"))?;

    Ok(rows
        .into_iter()
        .filter_map(|(profile_id, profile_home)| {
            let resolved_home = super::resolve_profile_home_for_curator(
                &profile_id,
                profile_home.as_str(),
                Some(runtime_root),
            );
            resolved_home.map(|profile_home| CuratorProfileTarget {
                profile_id,
                profile_home,
            })
        })
        .collect())
}

async fn load_profile_curator_snapshot_with_pool(
    pool: &SqlitePool,
    profile_id: &str,
    interval_minutes: i64,
    now: DateTime<Utc>,
) -> Result<ProfileCuratorRunSnapshot, String> {
    if !table_exists(pool, "curator_runs").await? {
        return Ok(ProfileCuratorRunSnapshot {
            last_run_at: String::new(),
            summary: String::new(),
            due: true,
        });
    }

    let row = sqlx::query_as::<_, (String, String)>(
        "SELECT created_at, summary
         FROM curator_runs
         WHERE profile_id = ?
         ORDER BY created_at DESC, id DESC
         LIMIT 1",
    )
    .bind(profile_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("读取 profile curator 最近运行失败: {e}"))?;

    let (last_run_at, summary) = row.unwrap_or_default();
    Ok(ProfileCuratorRunSnapshot {
        due: due_since(&last_run_at, interval_minutes, now),
        last_run_at,
        summary,
    })
}

async fn mark_scheduler_started_with_pool(pool: &SqlitePool, now: &str) -> Result<(), String> {
    ensure_curator_scheduler_schema_with_pool(pool).await?;
    sqlx::query(
        "UPDATE curator_scheduler_state
         SET last_started_at = ?, last_error = '', updated_at = ?
         WHERE id = ?",
    )
    .bind(now)
    .bind(now)
    .bind(CURATOR_SCHEDULER_ID)
    .execute(pool)
    .await
    .map_err(|e| format!("更新 curator scheduler 开始状态失败: {e}"))?;
    Ok(())
}

async fn mark_scheduler_completed_with_pool(
    pool: &SqlitePool,
    now: &str,
    error: &str,
) -> Result<(), String> {
    ensure_curator_scheduler_schema_with_pool(pool).await?;
    sqlx::query(
        "UPDATE curator_scheduler_state
         SET last_completed_at = ?, last_error = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(now)
    .bind(error)
    .bind(now)
    .bind(CURATOR_SCHEDULER_ID)
    .execute(pool)
    .await
    .map_err(|e| format!("更新 curator scheduler 完成状态失败: {e}"))?;
    Ok(())
}

pub async fn run_due_curator_cycle_with_pool(
    pool: SqlitePool,
    scheduler: EmployeeCuratorSchedulerState,
    runtime_root: PathBuf,
) -> Result<usize, String> {
    if scheduler.0.swap(true, Ordering::SeqCst) {
        return Ok(0);
    }

    let result = async {
        let now = Utc::now();
        let config = load_curator_scheduler_config_with_pool(&pool).await?;
        if !config.enabled {
            return Ok(0);
        }

        let idle =
            load_runtime_idle_snapshot_with_pool(&pool, config.min_idle_minutes, now).await?;
        if !idle.idle {
            return Ok(0);
        }

        let started_at = now.to_rfc3339();
        mark_scheduler_started_with_pool(&pool, &started_at).await?;
        let targets = list_curator_profile_targets_with_pool(&pool, &runtime_root).await?;
        let mut processed = 0usize;
        let mut errors = Vec::new();

        for target in targets {
            let snapshot = load_profile_curator_snapshot_with_pool(
                &pool,
                &target.profile_id,
                config.interval_minutes,
                Utc::now(),
            )
            .await?;
            if !snapshot.due {
                continue;
            }
            let memory_dir = PathBuf::from(&target.profile_home).join("memories");
            match crate::agent::tools::CuratorTool::scan_profile_with_pool(
                pool.clone(),
                target.profile_id.clone(),
                memory_dir,
                true,
            )
            .await
            {
                Ok(_) => processed += 1,
                Err(error) => errors.push(format!("{}: {}", target.profile_id, error)),
            }
        }

        let completed_at = Utc::now().to_rfc3339();
        mark_scheduler_completed_with_pool(&pool, &completed_at, &errors.join("; ")).await?;
        Ok(processed)
    }
    .await;

    scheduler.0.store(false, Ordering::SeqCst);
    result
}

pub fn spawn_curator_scheduler(
    pool: SqlitePool,
    scheduler: EmployeeCuratorSchedulerState,
    runtime_root: PathBuf,
) {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = ensure_curator_scheduler_schema_with_pool(&pool).await {
            eprintln!("[curator] 初始化自动调度器失败: {error}");
            return;
        }

        tokio::time::sleep(Duration::from_secs(SCHEDULER_INITIAL_DELAY_SECONDS)).await;
        loop {
            if let Err(error) = run_due_curator_cycle_with_pool(
                pool.clone(),
                scheduler.clone(),
                runtime_root.clone(),
            )
            .await
            {
                eprintln!("[curator] 自动整理失败: {error}");
            }
            tokio::time::sleep(Duration::from_secs(SCHEDULER_TICK_SECONDS)).await;
        }
    });
}

pub async fn get_curator_scheduler_status_with_pool(
    pool: &SqlitePool,
    scheduler: &EmployeeCuratorSchedulerState,
    employee_id: Option<&str>,
) -> Result<EmployeeCuratorSchedulerStatus, String> {
    let now = Utc::now();
    let config = load_curator_scheduler_config_with_pool(pool).await?;
    let idle = load_runtime_idle_snapshot_with_pool(pool, config.min_idle_minutes, now).await?;
    let profile = match employee_id.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => {
            Some(super::resolve_employee_profile_for_curator_with_pool(pool, value).await?)
        }
        None => None,
    };
    let profile_id = profile.as_ref().map(|(profile_id, _)| profile_id.clone());
    let profile_snapshot = match profile_id.as_deref() {
        Some(value) => Some(
            load_profile_curator_snapshot_with_pool(pool, value, config.interval_minutes, now)
                .await?,
        ),
        None => None,
    };
    let next_check_at = next_check_at(&config, now);

    Ok(EmployeeCuratorSchedulerStatus {
        enabled: config.enabled,
        running: scheduler.0.load(Ordering::SeqCst),
        interval_minutes: config.interval_minutes,
        min_idle_minutes: config.min_idle_minutes,
        active_run_count: idle.active_run_count,
        idle: idle.idle,
        last_activity_at: idle.last_activity_at,
        last_started_at: config.last_started_at,
        last_completed_at: config.last_completed_at,
        last_error: config.last_error,
        next_check_at,
        profile_id,
        profile_due: profile_snapshot
            .as_ref()
            .map(|snapshot| snapshot.due)
            .unwrap_or(false),
        profile_last_run_at: profile_snapshot
            .as_ref()
            .map(|snapshot| snapshot.last_run_at.clone())
            .unwrap_or_default(),
        profile_last_summary: profile_snapshot
            .as_ref()
            .map(|snapshot| snapshot.summary.clone())
            .unwrap_or_default(),
    })
}

#[tauri::command]
pub async fn get_curator_scheduler_status(
    employee_id: Option<String>,
    db: State<'_, DbState>,
    scheduler: State<'_, EmployeeCuratorSchedulerState>,
) -> Result<EmployeeCuratorSchedulerStatus, String> {
    get_curator_scheduler_status_with_pool(&db.0, scheduler.inner(), employee_id.as_deref()).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn memory_pool() -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect sqlite")
    }

    #[tokio::test]
    async fn status_reports_profile_due_without_prior_curator_run() {
        let pool = memory_pool().await;
        ensure_curator_scheduler_schema_with_pool(&pool)
            .await
            .expect("ensure schema");
        sqlx::query(
            "CREATE TABLE agent_profiles (
                id TEXT PRIMARY KEY,
                legacy_employee_row_id TEXT NOT NULL DEFAULT '',
                display_name TEXT NOT NULL DEFAULT '',
                route_aliases_json TEXT NOT NULL DEFAULT '[]',
                profile_home TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("create profiles");
        sqlx::query(
            "INSERT INTO agent_profiles (id, legacy_employee_row_id, profile_home, created_at, updated_at)
             VALUES ('profile-1', 'employee-1', 'D:/profiles/profile-1', '2026-05-10T00:00:00Z', '2026-05-10T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("insert profile");

        let status = get_curator_scheduler_status_with_pool(
            &pool,
            &EmployeeCuratorSchedulerState::default(),
            Some("employee-1"),
        )
        .await
        .expect("status");

        assert!(status.enabled);
        assert!(status.idle);
        assert_eq!(status.profile_id.as_deref(), Some("profile-1"));
        assert!(status.profile_due);
    }

    #[tokio::test]
    async fn status_is_not_idle_when_session_run_is_active() {
        let pool = memory_pool().await;
        ensure_curator_scheduler_schema_with_pool(&pool)
            .await
            .expect("ensure schema");
        sqlx::query(
            "CREATE TABLE session_runs (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'queued',
                updated_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("create session runs");
        sqlx::query(
            "INSERT INTO session_runs (id, session_id, status, updated_at)
             VALUES ('run-1', 'session-1', 'running', '2026-05-10T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("insert run");

        let status = get_curator_scheduler_status_with_pool(
            &pool,
            &EmployeeCuratorSchedulerState::default(),
            None,
        )
        .await
        .expect("status");

        assert_eq!(status.active_run_count, 1);
        assert!(!status.idle);
    }

    #[tokio::test]
    async fn due_cycle_uses_runtime_profile_home_when_db_home_is_empty() {
        let pool = memory_pool().await;
        let temp = tempfile::tempdir().expect("tempdir");
        let profile_dir = temp.path().join("profiles").join("profile-1");
        std::fs::create_dir_all(profile_dir.join("memories")).expect("create profile");
        ensure_curator_scheduler_schema_with_pool(&pool)
            .await
            .expect("ensure schema");
        sqlx::query(
            "CREATE TABLE agent_profiles (
                id TEXT PRIMARY KEY,
                legacy_employee_row_id TEXT NOT NULL DEFAULT '',
                display_name TEXT NOT NULL DEFAULT '',
                route_aliases_json TEXT NOT NULL DEFAULT '[]',
                profile_home TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("create profiles");
        sqlx::query(
            "INSERT INTO agent_profiles (id, legacy_employee_row_id, profile_home, created_at, updated_at)
             VALUES ('profile-1', 'employee-1', '', '2026-05-10T00:00:00Z', '2026-05-10T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("insert profile");

        let processed = run_due_curator_cycle_with_pool(
            pool.clone(),
            EmployeeCuratorSchedulerState::default(),
            temp.path().to_path_buf(),
        )
        .await
        .expect("run cycle");

        assert_eq!(processed, 1);
        assert!(profile_dir.join("curator").join("reports").exists());
    }
}
