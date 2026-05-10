# Profile Runtime Foundation Slice 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish WorkClaw's profile runtime identity foundation without changing memory injection, prompt output, approval behavior, or `.skillpack` format.

**Architecture:** Add `agent_profiles` as the new runtime identity root, keep `agent_employees` as UI/migration data, add nullable `sessions.profile_id`, and introduce alias resolution that maps legacy employee codes/row ids/route aliases to profiles. In parallel, add a source policy layer for installed skills so encrypted `.skillpack` skills are immutable to future self-improving mutation paths. After the 2026-05-07 Hermes pivot, this plan is migration-compatible only; new profile runtime work must not preserve OpenClaw-shaped directories as a target.

**Tech Stack:** Rust, Tauri, sqlx, SQLite, in-memory SQLite tests, existing WorkClaw runtime modules.

---

## Strategy Summary

- Change surface: SQLite schema/migrations, chat session creation/listing, profile alias resolution, skill source policy, skill deletion guard, tests.
- Affected modules: `apps/runtime/src-tauri/src/db/*`, `apps/runtime/src-tauri/src/commands/chat_session_io/*`, new `apps/runtime/src-tauri/src/profile_runtime/*`, `apps/runtime/src-tauri/src/commands/skills.rs`, skill runtime IO helpers.
- Main risk: breaking old databases or changing user-visible behavior while introducing `profile_id`.
- Recommended smallest safe path: add nullable fields and read-only resolvers first; keep legacy columns and prompt behavior unchanged.
- Required verification: `pnpm test:rust-fast`; focused Rust tests for DB migration, session fallback, profile alias resolution, and `.skillpack` immutability.
- Release impact: runtime DB schema changes are user-visible migration-sensitive; no packaging or installer metadata changes.

## Non-Goals

This slice must not do these things:

- Move or rewrite memory files.
- Inject `MEMORY.md`, profile instructions, or project memory from profile home.
- Rewrite team/group JSON.
- Implement `skill_manage`, Curator, Growth Loop, or Memory OS.
- Change tool approval behavior.
- Change `.skillpack` format.
- Change prompt output intentionally.

## Planned File Structure

Create:

- `apps/runtime/src-tauri/src/profile_runtime/mod.rs`  
  Module entrypoint for profile runtime helpers.

- `apps/runtime/src-tauri/src/profile_runtime/types.rs`  
  Small structs for `AgentProfileRecord`, `ProfileAliasCandidate`, and resolver outputs.

- `apps/runtime/src-tauri/src/profile_runtime/repo.rs`  
  SQL helpers for profile seed/backfill and alias lookup.

- `apps/runtime/src-tauri/src/profile_runtime/alias_resolver.rs`  
  Pure-ish profile alias resolution logic on top of repo rows.

- `apps/runtime/src-tauri/src/agent/runtime/runtime_io/skill_source_policy.rs`  
  Source policy enum and mutation rules for installed skill source types.

Modify:

- `apps/runtime/src-tauri/src/lib.rs`  
  Add `pub(crate) mod profile_runtime;`.

- `apps/runtime/src-tauri/src/db/schema.rs`  
  Add `agent_profiles`, `sessions.profile_id`, and supporting indexes for current schema.

- `apps/runtime/src-tauri/src/db/migrations.rs`  
  Add legacy migration steps for `agent_profiles`, `sessions.profile_id`, and safe backfill.

- `apps/runtime/src-tauri/src/commands/chat_session_io/session_store.rs`  
  Populate `sessions.profile_id` during new session creation where an employee alias can resolve to a profile, while preserving `employee_id`.

- `apps/runtime/src-tauri/src/commands/skills.rs`  
  Add backend deletion guard using source policy.

- `apps/runtime/src-tauri/src/agent/runtime/runtime_io/workspace_skills.rs`  
  Use source policy for directory-backed/encrypted classification, without changing projection behavior.

- `apps/runtime/src-tauri/src/agent/runtime/runtime_io/runtime_inputs.rs`  
  Use source policy normalization around installed skill source reads, preserving legacy builtin self-heal.

Test:

- Existing unit tests in the modified files.
- New unit tests colocated in `db/schema.rs`, `db/migrations.rs`, `profile_runtime/*`, `commands/skills.rs`, and `skill_source_policy.rs`.

---

## Task 1: Add Current Schema Shape For Profiles

**Files:**

- Modify: `apps/runtime/src-tauri/src/db/schema.rs`

- [ ] **Step 1: Add failing schema test for current installs**

Add this test under the existing `#[cfg(test)] mod tests` in `schema.rs`:

```rust
#[tokio::test]
async fn current_schema_creates_profile_runtime_tables_and_columns() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("create sqlite memory pool");

    apply_current_schema(&pool)
        .await
        .expect("apply current schema");

    let profile_tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master
         WHERE type = 'table'
         AND name = 'agent_profiles'",
    )
    .fetch_all(&pool)
    .await
    .expect("query profile tables");
    assert_eq!(profile_tables, vec!["agent_profiles".to_string()]);

    let session_columns: Vec<String> =
        sqlx::query_scalar("SELECT name FROM pragma_table_info('sessions')")
            .fetch_all(&pool)
            .await
            .expect("query session columns");
    assert!(
        session_columns.iter().any(|name| name == "profile_id"),
        "sessions should include nullable profile_id"
    );

    let indexes: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master
         WHERE type = 'index'
         AND name IN ('idx_agent_profiles_employee_row_id', 'idx_sessions_profile_id')",
    )
    .fetch_all(&pool)
    .await
    .expect("query profile indexes");
    assert_eq!(indexes.len(), 2, "expected profile runtime indexes");
}
```

- [ ] **Step 2: Run the focused schema test and confirm it fails**

Run:

```powershell
pnpm test:rust-fast -- current_schema_creates_profile_runtime_tables_and_columns
```

Expected: FAIL because `agent_profiles` and `sessions.profile_id` do not exist yet.

- [ ] **Step 3: Add `agent_profiles` to current schema**

In `apply_current_schema`, add this table after `sessions` or directly before it:

```rust
sqlx::query(
    "CREATE TABLE IF NOT EXISTS agent_profiles (
        id TEXT PRIMARY KEY,
        legacy_employee_row_id TEXT NOT NULL DEFAULT '',
        display_name TEXT NOT NULL DEFAULT '',
        route_aliases_json TEXT NOT NULL DEFAULT '[]',
        profile_home TEXT NOT NULL DEFAULT '',
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    )",
)
.execute(pool)
.await?;

sqlx::query(
    "CREATE INDEX IF NOT EXISTS idx_agent_profiles_employee_row_id
     ON agent_profiles(legacy_employee_row_id)",
)
.execute(pool)
.await?;
```

- [ ] **Step 4: Add `profile_id` to current `sessions` schema**

Update the `CREATE TABLE IF NOT EXISTS sessions` statement to include:

```sql
profile_id TEXT
```

Place it after `employee_id TEXT NOT NULL DEFAULT ''`.

Then add this index after sessions creation:

```rust
sqlx::query(
    "CREATE INDEX IF NOT EXISTS idx_sessions_profile_id
     ON sessions(profile_id)",
)
.execute(pool)
.await?;
```

- [ ] **Step 5: Re-run the focused schema test**

Run:

```powershell
pnpm test:rust-fast -- current_schema_creates_profile_runtime_tables_and_columns
```

Expected: PASS.

## Task 2: Add Legacy Migration And Backfill For Profiles

**Files:**

- Modify: `apps/runtime/src-tauri/src/db/migrations.rs`

- [ ] **Step 1: Add failing legacy migration test**

Add this test under `#[cfg(test)] mod tests` in `migrations.rs`:

```rust
#[tokio::test]
async fn legacy_employee_and_session_db_backfills_profiles_without_breaking_sessions() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("create sqlite memory pool");

    sqlx::query(
        "CREATE TABLE agent_employees (
            id TEXT PRIMARY KEY,
            role_id TEXT NOT NULL,
            name TEXT NOT NULL,
            primary_skill_id TEXT NOT NULL DEFAULT '',
            default_work_dir TEXT NOT NULL DEFAULT ''
        )",
    )
    .execute(&pool)
    .await
    .expect("create legacy agent_employees");

    sqlx::query(
        "INSERT INTO agent_employees (id, role_id, name, primary_skill_id, default_work_dir)
         VALUES ('employee-row-1', 'planner', 'Planner', 'builtin-general', 'D:/work')",
    )
    .execute(&pool)
    .await
    .expect("seed legacy employee");

    sqlx::query(
        "CREATE TABLE sessions (
            id TEXT PRIMARY KEY,
            skill_id TEXT NOT NULL,
            title TEXT,
            created_at TEXT NOT NULL,
            model_id TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("create legacy sessions");

    sqlx::query(
        "INSERT INTO sessions (id, skill_id, title, created_at, model_id)
         VALUES ('session-1', 'builtin-general', 'Legacy', '2026-05-06T00:00:00Z', 'model-1')",
    )
    .execute(&pool)
    .await
    .expect("seed legacy session");

    apply_legacy_migrations_for_test(&pool)
        .await
        .expect("apply legacy migrations");

    let profile: (String, String, String) = sqlx::query_as(
        "SELECT id, legacy_employee_row_id, display_name
         FROM agent_profiles
         WHERE legacy_employee_row_id = 'employee-row-1'",
    )
    .fetch_one(&pool)
    .await
    .expect("query profile");

    assert_eq!(profile.0, "employee-row-1");
    assert_eq!(profile.1, "employee-row-1");
    assert_eq!(profile.2, "Planner");

    let session_columns: Vec<String> =
        sqlx::query_scalar("SELECT name FROM pragma_table_info('sessions')")
            .fetch_all(&pool)
            .await
            .expect("query session columns");
    assert!(session_columns.iter().any(|name| name == "profile_id"));
}
```

- [ ] **Step 2: Run the focused migration test and confirm it fails**

Run:

```powershell
pnpm test:rust-fast -- legacy_employee_and_session_db_backfills_profiles_without_breaking_sessions
```

Expected: FAIL because migration does not create/backfill `agent_profiles`.

- [ ] **Step 3: Add migration helpers**

Add helper functions above `apply_legacy_migrations`:

```rust
async fn ensure_agent_profiles_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS agent_profiles (
            id TEXT PRIMARY KEY,
            legacy_employee_row_id TEXT NOT NULL DEFAULT '',
            display_name TEXT NOT NULL DEFAULT '',
            route_aliases_json TEXT NOT NULL DEFAULT '[]',
            profile_home TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_agent_profiles_employee_row_id
         ON agent_profiles(legacy_employee_row_id)",
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn ensure_sessions_profile_id_column(pool: &SqlitePool) -> Result<()> {
    let _ = sqlx::query("ALTER TABLE sessions ADD COLUMN profile_id TEXT")
        .execute(pool)
        .await;
    let _ = sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_sessions_profile_id
         ON sessions(profile_id)",
    )
    .execute(pool)
    .await;
    Ok(())
}
```

- [ ] **Step 4: Add profile backfill**

Add:

```rust
async fn backfill_agent_profiles_from_employees(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "INSERT OR IGNORE INTO agent_profiles (
            id,
            legacy_employee_row_id,
            display_name,
            route_aliases_json,
            profile_home,
            created_at,
            updated_at
        )
        SELECT
            id,
            id,
            COALESCE(NULLIF(TRIM(name), ''), COALESCE(NULLIF(TRIM(employee_id), ''), COALESCE(NULLIF(TRIM(role_id), ''), id))),
            json_array(
                COALESCE(employee_id, ''),
                COALESCE(role_id, ''),
                COALESCE(openclaw_agent_id, ''),
                id
            ),
            '',
            datetime('now'),
            datetime('now')
        FROM agent_employees
        WHERE TRIM(id) <> ''",
    )
    .execute(pool)
    .await?;
    Ok(())
}
```

If SQLite JSON1 is unavailable in the local build, replace `json_array(...)` with a conservative string:

```sql
'[]'
```

and let the alias resolver compute aliases from `agent_employees`.

- [ ] **Step 5: Call helpers from `apply_legacy_migrations`**

After the `agent_employees` compatibility columns are added and backfilled, call:

```rust
ensure_agent_profiles_table(pool).await?;
ensure_sessions_profile_id_column(pool).await?;
backfill_agent_profiles_from_employees(pool).await?;
```

- [ ] **Step 6: Re-run migration tests**

Run:

```powershell
pnpm test:rust-fast -- legacy_employee_and_session_db_backfills_profiles_without_breaking_sessions
```

Expected: PASS.

## Task 3: Add Profile Runtime Module And Alias Resolver

**Files:**

- Create: `apps/runtime/src-tauri/src/profile_runtime/mod.rs`
- Create: `apps/runtime/src-tauri/src/profile_runtime/types.rs`
- Create: `apps/runtime/src-tauri/src/profile_runtime/repo.rs`
- Create: `apps/runtime/src-tauri/src/profile_runtime/alias_resolver.rs`
- Modify: `apps/runtime/src-tauri/src/lib.rs`

- [ ] **Step 1: Add module declaration**

In `lib.rs`, add:

```rust
pub(crate) mod profile_runtime;
```

near the other runtime modules.

- [ ] **Step 2: Add `mod.rs`**

Create `profile_runtime/mod.rs`:

```rust
pub(crate) mod alias_resolver;
pub(crate) mod repo;
pub(crate) mod types;

pub(crate) use alias_resolver::resolve_profile_for_alias_with_pool;
pub(crate) use repo::load_profile_alias_candidates_with_pool;
pub(crate) use types::{ProfileAliasCandidate, ProfileAliasResolution};
```

- [ ] **Step 3: Add types**

Create `profile_runtime/types.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProfileAliasCandidate {
    pub profile_id: String,
    pub legacy_employee_row_id: String,
    pub employee_id: String,
    pub role_id: String,
    pub openclaw_agent_id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProfileAliasResolution {
    pub profile_id: String,
    pub matched_alias: String,
    pub display_name: String,
}
```

- [ ] **Step 4: Add pure resolver test**

Create `profile_runtime/alias_resolver.rs` with this test first:

```rust
use super::types::{ProfileAliasCandidate, ProfileAliasResolution};

fn normalize_alias(raw: &str) -> String {
    raw.trim().to_lowercase()
}

pub(crate) fn resolve_profile_for_alias(
    candidates: &[ProfileAliasCandidate],
    alias: &str,
) -> Option<ProfileAliasResolution> {
    let normalized = normalize_alias(alias);
    if normalized.is_empty() {
        return None;
    }

    for candidate in candidates {
        let aliases = [
            candidate.profile_id.as_str(),
            candidate.legacy_employee_row_id.as_str(),
            candidate.employee_id.as_str(),
            candidate.role_id.as_str(),
            candidate.openclaw_agent_id.as_str(),
        ];
        if aliases
            .iter()
            .map(|value| normalize_alias(value))
            .any(|value| value == normalized)
        {
            return Some(ProfileAliasResolution {
                profile_id: candidate.profile_id.clone(),
                matched_alias: alias.trim().to_string(),
                display_name: candidate.display_name.clone(),
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{resolve_profile_for_alias, ProfileAliasCandidate};

    fn candidate() -> ProfileAliasCandidate {
        ProfileAliasCandidate {
            profile_id: "profile-1".to_string(),
            legacy_employee_row_id: "employee-row-1".to_string(),
            employee_id: "planner".to_string(),
            role_id: "planner-role".to_string(),
            openclaw_agent_id: "oc-planner".to_string(),
            display_name: "Planner".to_string(),
        }
    }

    #[test]
    fn resolves_profile_from_employee_code_role_openclaw_or_row_id() {
        for alias in ["planner", "planner-role", "oc-planner", "employee-row-1", "profile-1"] {
            let resolved = resolve_profile_for_alias(&[candidate()], alias)
                .expect("alias should resolve");
            assert_eq!(resolved.profile_id, "profile-1");
            assert_eq!(resolved.display_name, "Planner");
        }
    }

    #[test]
    fn ignores_empty_or_unknown_aliases() {
        assert!(resolve_profile_for_alias(&[candidate()], "").is_none());
        assert!(resolve_profile_for_alias(&[candidate()], "missing").is_none());
    }
}
```

- [ ] **Step 5: Add repo lookup**

Create `profile_runtime/repo.rs`:

```rust
use super::types::ProfileAliasCandidate;

pub(crate) async fn load_profile_alias_candidates_with_pool(
    pool: &sqlx::SqlitePool,
) -> Result<Vec<ProfileAliasCandidate>, String> {
    let rows = sqlx::query_as::<_, (String, String, String, String, String, String)>(
        "SELECT
            COALESCE(p.id, e.id),
            COALESCE(p.legacy_employee_row_id, e.id),
            COALESCE(e.employee_id, ''),
            COALESCE(e.role_id, ''),
            COALESCE(e.openclaw_agent_id, ''),
            COALESCE(NULLIF(TRIM(p.display_name), ''), COALESCE(e.name, ''))
         FROM agent_employees e
         LEFT JOIN agent_profiles p
           ON p.legacy_employee_row_id = e.id OR p.id = e.id",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(
            |(
                profile_id,
                legacy_employee_row_id,
                employee_id,
                role_id,
                openclaw_agent_id,
                display_name,
            )| ProfileAliasCandidate {
                profile_id,
                legacy_employee_row_id,
                employee_id,
                role_id,
                openclaw_agent_id,
                display_name,
            },
        )
        .collect())
}
```

- [ ] **Step 6: Add async resolver wrapper**

Append to `alias_resolver.rs`:

```rust
pub(crate) async fn resolve_profile_for_alias_with_pool(
    pool: &sqlx::SqlitePool,
    alias: &str,
) -> Result<Option<ProfileAliasResolution>, String> {
    let candidates = crate::profile_runtime::repo::load_profile_alias_candidates_with_pool(pool).await?;
    Ok(resolve_profile_for_alias(&candidates, alias))
}
```

- [ ] **Step 7: Run resolver tests**

Run:

```powershell
pnpm test:rust-fast -- resolves_profile_from_employee_code_role_openclaw_or_row_id
```

Expected: PASS.

## Task 4: Populate `sessions.profile_id` On New Sessions Without Breaking Legacy Reads

**Files:**

- Modify: `apps/runtime/src-tauri/src/commands/chat_session_io/session_store.rs`

- [ ] **Step 1: Add test for session creation with profile alias**

Add a test in `session_store.rs` if the file already has a test module. If it does not, add one at the bottom:

```rust
#[cfg(test)]
mod profile_tests {
    use super::create_session_with_pool;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_pool() -> sqlx::SqlitePool {
        let pool = SqlitePoolOptions::new()
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
                permission_mode TEXT NOT NULL DEFAULT 'accept_edits',
                work_dir TEXT NOT NULL DEFAULT '',
                employee_id TEXT NOT NULL DEFAULT '',
                profile_id TEXT,
                session_mode TEXT NOT NULL DEFAULT 'general',
                team_id TEXT NOT NULL DEFAULT ''
            )",
        )
        .execute(&pool)
        .await
        .expect("create sessions");

        sqlx::query("CREATE TABLE app_settings (key TEXT PRIMARY KEY, value TEXT NOT NULL)")
            .execute(&pool)
            .await
            .expect("create app_settings");

        sqlx::query(
            "CREATE TABLE agent_employees (
                id TEXT PRIMARY KEY,
                employee_id TEXT NOT NULL DEFAULT '',
                role_id TEXT NOT NULL DEFAULT '',
                openclaw_agent_id TEXT NOT NULL DEFAULT '',
                name TEXT NOT NULL DEFAULT ''
            )",
        )
        .execute(&pool)
        .await
        .expect("create agent_employees");

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
        .expect("create agent_profiles");

        sqlx::query(
            "INSERT INTO agent_employees (id, employee_id, role_id, openclaw_agent_id, name)
             VALUES ('employee-row-1', 'planner', 'planner-role', 'oc-planner', 'Planner')",
        )
        .execute(&pool)
        .await
        .expect("seed employee");

        sqlx::query(
            "INSERT INTO agent_profiles (id, legacy_employee_row_id, display_name, created_at, updated_at)
             VALUES ('profile-1', 'employee-row-1', 'Planner', '2026-05-06T00:00:00Z', '2026-05-06T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("seed profile");

        pool
    }

    #[tokio::test]
    async fn create_session_stores_profile_id_when_employee_alias_resolves() {
        let pool = setup_pool().await;

        let session_id = create_session_with_pool(
            &pool,
            "builtin-general".to_string(),
            "model-1".to_string(),
            Some("D:/work".to_string()),
            Some("planner".to_string()),
            Some("Task".to_string()),
            None,
            Some("employee_direct".to_string()),
            None,
        )
        .await
        .expect("create session");

        let row: (String, String) =
            sqlx::query_as("SELECT employee_id, COALESCE(profile_id, '') FROM sessions WHERE id = ?")
                .bind(session_id)
                .fetch_one(&pool)
                .await
                .expect("query session");

        assert_eq!(row.0, "planner");
        assert_eq!(row.1, "profile-1");
    }
}
```

- [ ] **Step 2: Run the test and confirm it fails**

Run:

```powershell
pnpm test:rust-fast -- create_session_stores_profile_id_when_employee_alias_resolves
```

Expected: FAIL because `create_session_with_pool` does not write `profile_id`.

- [ ] **Step 3: Resolve profile before insert**

In `create_session_with_pool`, after `resolved_work_dir`, add:

```rust
let resolved_profile_id = if prepared.normalized_employee_id.trim().is_empty() {
    None
} else {
    crate::profile_runtime::resolve_profile_for_alias_with_pool(
        pool,
        &prepared.normalized_employee_id,
    )
    .await?
    .map(|resolved| resolved.profile_id)
};
```

- [ ] **Step 4: Add `profile_id` to insert**

Change the insert SQL to:

```rust
sqlx::query(
    "INSERT INTO sessions (id, skill_id, title, created_at, model_id, permission_mode, work_dir, employee_id, profile_id, session_mode, team_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
)
```

and add:

```rust
.bind(&resolved_profile_id)
```

between `employee_id` and `session_mode`.

- [ ] **Step 5: Re-run the session creation test**

Run:

```powershell
pnpm test:rust-fast -- create_session_stores_profile_id_when_employee_alias_resolves
```

Expected: PASS.

## Task 5: Add Skill Source Policy Enum

**Files:**

- Create: `apps/runtime/src-tauri/src/agent/runtime/runtime_io/skill_source_policy.rs`
- Modify: `apps/runtime/src-tauri/src/agent/runtime/runtime_io/mod.rs` or the nearest module file that exports runtime IO children
- Modify: `apps/runtime/src-tauri/src/agent/runtime/runtime_io/workspace_skills.rs`

- [ ] **Step 1: Create source policy module with tests**

Create `skill_source_policy.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SkillSourceKind {
    Skillpack,
    Local,
    Preset,
    LegacyBuiltin,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SkillSourcePolicy {
    pub kind: SkillSourceKind,
    pub canonical_label: &'static str,
    pub directory_backed: bool,
    pub immutable_content: bool,
    pub can_delete_installed_row: bool,
}

pub(crate) fn resolve_skill_source_policy(source_type: &str) -> SkillSourcePolicy {
    match source_type.trim().to_lowercase().as_str() {
        "" | "encrypted" | "skillpack" => SkillSourcePolicy {
            kind: SkillSourceKind::Skillpack,
            canonical_label: "skillpack",
            directory_backed: false,
            immutable_content: true,
            can_delete_installed_row: true,
        },
        "local" => SkillSourcePolicy {
            kind: SkillSourceKind::Local,
            canonical_label: "local",
            directory_backed: true,
            immutable_content: false,
            can_delete_installed_row: true,
        },
        "vendored" | "preset" => SkillSourcePolicy {
            kind: SkillSourceKind::Preset,
            canonical_label: "preset",
            directory_backed: true,
            immutable_content: false,
            can_delete_installed_row: true,
        },
        "builtin" => SkillSourcePolicy {
            kind: SkillSourceKind::LegacyBuiltin,
            canonical_label: "builtin",
            directory_backed: false,
            immutable_content: false,
            can_delete_installed_row: true,
        },
        _ => SkillSourcePolicy {
            kind: SkillSourceKind::Unknown,
            canonical_label: "unknown",
            directory_backed: false,
            immutable_content: true,
            can_delete_installed_row: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_skill_source_policy, SkillSourceKind};

    #[test]
    fn skillpack_sources_are_immutable_content() {
        for source in ["", "encrypted", "skillpack"] {
            let policy = resolve_skill_source_policy(source);
            assert_eq!(policy.kind, SkillSourceKind::Skillpack);
            assert!(policy.immutable_content);
            assert!(!policy.directory_backed);
            assert!(policy.can_delete_installed_row);
        }
    }

    #[test]
    fn preset_aliases_are_directory_backed_and_mutable_with_history() {
        for source in ["vendored", "preset"] {
            let policy = resolve_skill_source_policy(source);
            assert_eq!(policy.kind, SkillSourceKind::Preset);
            assert!(policy.directory_backed);
            assert!(!policy.immutable_content);
        }
    }

    #[test]
    fn unknown_sources_are_not_silently_treated_as_skillpacks_for_mutation() {
        let policy = resolve_skill_source_policy("future-source");
        assert_eq!(policy.kind, SkillSourceKind::Unknown);
        assert!(policy.immutable_content);
        assert!(!policy.can_delete_installed_row);
    }
}
```

- [ ] **Step 2: Export the module**

In the `runtime_io` module file, add:

```rust
pub(crate) mod skill_source_policy;
```

If `runtime_io` is a directory module with a `mod.rs`, add it there. If it is declared in the parent runtime module, add it beside the existing child module declarations.

- [ ] **Step 3: Run source policy tests**

Run:

```powershell
pnpm test:rust-fast -- skillpack_sources_are_immutable_content
```

Expected: PASS.

- [ ] **Step 4: Use policy in workspace skill root resolution**

In `workspace_skills.rs`, import:

```rust
use super::skill_source_policy::{resolve_skill_source_policy, SkillSourceKind};
```

Update `resolve_directory_backed_skill_root` so `local` and `preset` use policy:

```rust
pub(crate) fn resolve_directory_backed_skill_root(
    source_type: &str,
    pack_path: &str,
) -> Option<std::path::PathBuf> {
    let policy = resolve_skill_source_policy(source_type);
    match policy.kind {
        SkillSourceKind::Local | SkillSourceKind::Preset => {
            let path = std::path::PathBuf::from(pack_path);
            path.exists().then_some(path)
        }
        SkillSourceKind::LegacyBuiltin => {
            let path = std::path::PathBuf::from(pack_path);
            if pack_path.trim().is_empty() || !path.exists() {
                None
            } else {
                Some(path)
            }
        }
        SkillSourceKind::Skillpack | SkillSourceKind::Unknown => None,
    }
}
```

- [ ] **Step 5: Run workspace skill tests**

Run:

```powershell
pnpm test:rust-fast -- workspace_skill
```

Expected: existing workspace skill tests pass.

## Task 6: Add Backend Guard For Skill Deletion

**Files:**

- Modify: `apps/runtime/src-tauri/src/commands/skills.rs`

- [ ] **Step 1: Add source guard helper**

Above `delete_skill`, add:

```rust
async fn ensure_skill_can_be_deleted(
    pool: &sqlx::SqlitePool,
    skill_id: &str,
) -> Result<(), String> {
    let source_type = sqlx::query_scalar::<_, String>(
        "SELECT COALESCE(source_type, 'encrypted') FROM installed_skills WHERE id = ?",
    )
    .bind(skill_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("Skill 不存在 (skill_id={skill_id})"))?;

    let policy =
        crate::agent::runtime::runtime_io::skill_source_policy::resolve_skill_source_policy(
            &source_type,
        );
    if !policy.can_delete_installed_row {
        return Err(format!(
            "Skill source '{}' is not deletable by this runtime",
            source_type
        ));
    }
    Ok(())
}
```

- [ ] **Step 2: Add test for unknown source guard**

Add a `#[cfg(test)]` test module at the bottom of `skills.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::ensure_skill_can_be_deleted;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn delete_guard_blocks_unknown_source_type() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("create sqlite memory pool");

        sqlx::query(
            "CREATE TABLE installed_skills (
                id TEXT PRIMARY KEY,
                manifest TEXT NOT NULL,
                installed_at TEXT NOT NULL,
                username TEXT NOT NULL,
                pack_path TEXT NOT NULL DEFAULT '',
                source_type TEXT NOT NULL DEFAULT 'encrypted'
            )",
        )
        .execute(&pool)
        .await
        .expect("create installed_skills");

        sqlx::query(
            "INSERT INTO installed_skills (id, manifest, installed_at, username, pack_path, source_type)
             VALUES ('future-skill', '{}', '2026-05-06T00:00:00Z', '', '', 'future-source')",
        )
        .execute(&pool)
        .await
        .expect("seed skill");

        let err = ensure_skill_can_be_deleted(&pool, "future-skill")
            .await
            .expect_err("unknown source should be blocked");
        assert!(err.contains("not deletable"));
    }

    #[tokio::test]
    async fn delete_guard_allows_skillpack_installed_row_without_mutating_pack_content() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("create sqlite memory pool");

        sqlx::query(
            "CREATE TABLE installed_skills (
                id TEXT PRIMARY KEY,
                manifest TEXT NOT NULL,
                installed_at TEXT NOT NULL,
                username TEXT NOT NULL,
                pack_path TEXT NOT NULL DEFAULT '',
                source_type TEXT NOT NULL DEFAULT 'encrypted'
            )",
        )
        .execute(&pool)
        .await
        .expect("create installed_skills");

        sqlx::query(
            "INSERT INTO installed_skills (id, manifest, installed_at, username, pack_path, source_type)
             VALUES ('encrypted-skill', '{}', '2026-05-06T00:00:00Z', 'alice', 'D:/packs/a.skillpack', 'encrypted')",
        )
        .execute(&pool)
        .await
        .expect("seed skill");

        ensure_skill_can_be_deleted(&pool, "encrypted-skill")
            .await
            .expect("skillpack installed row deletion is explicit user uninstall");
    }
}
```

- [ ] **Step 3: Call guard from `delete_skill`**

Update `delete_skill`:

```rust
#[tauri::command]
pub async fn delete_skill(skill_id: String, db: State<'_, DbState>) -> Result<(), String> {
    ensure_skill_can_be_deleted(&db.0, &skill_id).await?;
    sqlx::query("DELETE FROM installed_skills WHERE id = ?")
        .bind(&skill_id)
        .execute(&db.0)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

- [ ] **Step 4: Run deletion guard tests**

Run:

```powershell
pnpm test:rust-fast -- delete_guard
```

Expected: PASS.

## Task 7: Final Verification And Roadmap Update

**Files:**

- Modify: `docs/plans/2026-05-06-self-improving-profile-runtime-roadmap.md`
- Optionally modify: `docs/plans/2026-05-06-profile-runtime-phase-0-synthesis.md`

- [x] **Step 1: Run Rust fast path**

Run:

```powershell
pnpm test:rust-fast
```

Expected: PASS.

- [x] **Step 2: Run builtin skill checks if source policy touched shared skill behavior**

Run:

```powershell
pnpm test:builtin-skills
```

Expected: PASS.

- [x] **Step 3: Update roadmap Phase 0 checkboxes**

Only if the tests above pass and code-level `.skillpack` source policy guard exists, update Phase 0:

```markdown
- `[x]` 产出 legacy migration matrix，说明哪些字段保留、哪些字段废弃、哪些字段仅迁移期使用。OpenClaw-shaped runtime directories are no longer a new compatibility target after the Hermes pivot.
- `[x]` 定义 `profile_id` 与现有 `employee_id` 的迁移映射规则。
- `[~]` 定义旧记忆目录到 profile home 的迁移规则。
- `[x]` 明确 `.skillpack` 不可变边界：禁止 curator、skill_manage、自动 patch 改写其内容。
```

Keep the overall Phase 0 status as `[~]` unless the implementation also adds all required legacy regression tests and verifies them.

- [x] **Step 4: Record verification evidence**

Append a short note under Phase 0 in the roadmap:

```markdown
Verification evidence:

- `pnpm test:rust-fast`: PASS, run from repo root after Profile Runtime Foundation Slice 1.
- `pnpm test:builtin-skills`: PASS, run from repo root after skill source policy changes.
- `cargo check --manifest-path apps/runtime/src-tauri/Cargo.toml --lib`: PASS.
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib --no-run`: PASS; Tauri lib test binary compiles.
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --lib resolve_workspace_skill_runtime_entry_rejects_unknown_source_type -- --nocapture`: BLOCKED at test binary launch with `STATUS_ENTRYPOINT_NOT_FOUND` in this Windows environment.
- `cargo test --manifest-path apps/runtime/src-tauri/Cargo.toml --test test_chat_commands -- --nocapture`: PASS; Windows-friendly integration coverage for profile runtime schema in test DB and session `profile_id` alias persistence.
```

If `pnpm test:builtin-skills` is not run because no builtin skill asset or shared skill crate changed, write: `Not run; no builtin skill asset or shared skill crate behavior changed in this slice.`

- [x] **Step 5: Stop before Memory OS**

Do not begin Memory OS implementation in this plan. The next plan should be written only after this foundation slice is merged or explicitly accepted.

## Execution Notes

Recommended parallelization:

- Task 1 and Task 2 should be done together by one worker because they touch DB schema and migrations.
- Task 3 and Task 4 should be done together by one worker because session creation depends on alias resolution.
- Task 5 and Task 6 can be done by a separate worker because source policy is mostly independent.
- Task 7 should be done by the coordinator after integrating all workers.

Do not run two workers against `schema.rs` / `migrations.rs` at the same time.

## Self-Review

Spec coverage:

- `agent_profiles`: Task 1 and Task 2.
- nullable `sessions.profile_id`: Task 1, Task 2, Task 4.
- alias resolver: Task 3 and Task 4.
- legacy regression tests: Task 2 and Task 4.
- skill source policy and `.skillpack` guard: Task 5 and Task 6.
- no memory migration / prompt changes / approval changes: Non-goals and task exclusions.

Placeholder scan:

- No `TBD`, `TODO`, or open-ended implementation steps are intentionally left in the plan.

Type consistency:

- `ProfileAliasCandidate`, `ProfileAliasResolution`, `resolve_profile_for_alias_with_pool`, and `resolve_skill_source_policy` are defined before downstream use.
