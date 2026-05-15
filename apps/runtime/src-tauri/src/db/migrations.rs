use anyhow::Result;
use sqlx::SqlitePool;

async fn im_thread_sessions_exists(pool: &SqlitePool) -> Result<bool> {
    let table_names: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'im_thread_sessions'",
    )
    .fetch_all(pool)
    .await?;

    Ok(!table_names.is_empty())
}

async fn agent_employees_exists(pool: &SqlitePool) -> Result<bool> {
    let table_names: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'agent_employees'",
    )
    .fetch_all(pool)
    .await?;

    Ok(!table_names.is_empty())
}

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

async fn backfill_agent_profiles_from_employees(pool: &SqlitePool) -> Result<()> {
    if !agent_employees_exists(pool).await? {
        return Ok(());
    }

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
            '[]',
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

pub(super) async fn apply_legacy_migrations(pool: &SqlitePool) -> Result<()> {
    let has_im_thread_sessions = im_thread_sessions_exists(pool).await?;

    let _ = sqlx::query(
        "ALTER TABLE im_routing_bindings ADD COLUMN connector_meta_json TEXT NOT NULL DEFAULT '{}'",
    )
    .execute(pool)
    .await;

    let _ = sqlx::query("ALTER TABLE model_configs ADD COLUMN api_key TEXT NOT NULL DEFAULT ''")
        .execute(pool)
        .await;
    let _ = sqlx::query(
        "ALTER TABLE model_configs ADD COLUMN supports_vision INTEGER NOT NULL DEFAULT 0",
    )
    .execute(pool)
    .await;

    let _ = sqlx::query(
        "ALTER TABLE sessions ADD COLUMN permission_mode TEXT NOT NULL DEFAULT 'accept_edits'",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "ALTER TABLE installed_skills ADD COLUMN source_type TEXT NOT NULL DEFAULT 'encrypted'",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query("ALTER TABLE sessions ADD COLUMN work_dir TEXT NOT NULL DEFAULT ''")
        .execute(pool)
        .await;
    let _ = sqlx::query("ALTER TABLE sessions ADD COLUMN employee_id TEXT NOT NULL DEFAULT ''")
        .execute(pool)
        .await;
    let _ =
        sqlx::query("ALTER TABLE sessions ADD COLUMN session_mode TEXT NOT NULL DEFAULT 'general'")
            .execute(pool)
            .await;
    let _ = sqlx::query("ALTER TABLE sessions ADD COLUMN team_id TEXT NOT NULL DEFAULT ''")
        .execute(pool)
        .await;
    let _ = sqlx::query("ALTER TABLE messages ADD COLUMN content_json TEXT")
        .execute(pool)
        .await;
    let _ = sqlx::query(
        "ALTER TABLE session_runs ADD COLUMN assistant_message_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(pool)
    .await;

    let _ = sqlx::query(
        "ALTER TABLE agent_employees ADD COLUMN feishu_app_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "ALTER TABLE agent_employees ADD COLUMN feishu_app_secret TEXT NOT NULL DEFAULT ''",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "ALTER TABLE agent_employees ADD COLUMN openclaw_agent_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "ALTER TABLE agent_employees ADD COLUMN routing_priority INTEGER NOT NULL DEFAULT 100",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "ALTER TABLE agent_employees ADD COLUMN enabled_scopes_json TEXT NOT NULL DEFAULT '[]'",
    )
    .execute(pool)
    .await;
    let _ =
        sqlx::query("ALTER TABLE agent_employees ADD COLUMN employee_id TEXT NOT NULL DEFAULT ''")
            .execute(pool)
            .await;
    let _ = sqlx::query(
        "UPDATE agent_employees SET employee_id = role_id WHERE TRIM(employee_id) = ''",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_agent_employees_employee_id_unique ON agent_employees(employee_id)",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_agent_employees_role_id_unique ON agent_employees(role_id)",
    )
    .execute(pool)
    .await;
    ensure_agent_profiles_table(pool).await?;
    ensure_sessions_profile_id_column(pool).await?;
    crate::agent::runtime::runtime_io::ensure_profile_session_index_schema_with_pool(pool)
        .await
        .map_err(anyhow::Error::msg)?;
    crate::agent::runtime::runtime_io::ensure_skill_os_versions_schema_with_pool(pool)
        .await
        .map_err(anyhow::Error::msg)?;
    crate::agent::runtime::runtime_io::ensure_skill_os_lifecycle_schema_with_pool(pool)
        .await
        .map_err(anyhow::Error::msg)?;
    crate::commands::employee_agents::curator_scheduler::ensure_curator_scheduler_schema_with_pool(
        pool,
    )
    .await
    .map_err(anyhow::Error::msg)?;
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
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_growth_events_profile_created
         ON growth_events(profile_id, created_at DESC)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_growth_events_target
         ON growth_events(target_type, target_id, created_at DESC)",
    )
    .execute(pool)
    .await?;
    backfill_agent_profiles_from_employees(pool).await?;

    let _ = sqlx::query(
        "ALTER TABLE im_thread_sessions ADD COLUMN route_session_key TEXT NOT NULL DEFAULT ''",
    )
    .execute(pool)
    .await;
    ensure_agent_conversation_binding_tables(pool).await?;
    if has_im_thread_sessions {
        ensure_im_thread_sessions_conversation_columns(pool).await?;
    }
    ensure_im_conversation_sessions_table(pool).await?;
    backfill_authority_binding_tables(pool).await?;
    if has_im_thread_sessions {
        let _ = sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_im_thread_sessions_route_key ON im_thread_sessions(route_session_key)",
        )
        .execute(pool)
        .await;
        let _ = sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_im_thread_sessions_conversation_id ON im_thread_sessions(conversation_id)",
        )
        .execute(pool)
        .await;
        let _ = sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_im_thread_sessions_channel_account_conversation ON im_thread_sessions(channel, account_id, conversation_id)",
        )
        .execute(pool)
        .await;
    }
    let _ = sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_employee_groups_coordinator ON employee_groups(coordinator_employee_id)",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_employee_group_rules_group_id ON employee_group_rules(group_id)",
    )
    .execute(pool)
    .await;
    let _ =
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_group_runs_group_id ON group_runs(group_id)")
            .execute(pool)
            .await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_group_runs_state ON group_runs(state)")
        .execute(pool)
        .await;
    let _ = sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_group_run_events_run_id ON group_run_events(run_id)",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_group_run_steps_run_id ON group_run_steps(run_id)",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_group_run_steps_round_no ON group_run_steps(round_no)",
    )
    .execute(pool)
    .await;

    let _ =
        sqlx::query("ALTER TABLE employee_groups ADD COLUMN template_id TEXT NOT NULL DEFAULT ''")
            .execute(pool)
            .await;
    let _ = sqlx::query(
        "ALTER TABLE employee_groups ADD COLUMN entry_employee_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "ALTER TABLE employee_groups ADD COLUMN review_mode TEXT NOT NULL DEFAULT 'none'",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "ALTER TABLE employee_groups ADD COLUMN execution_mode TEXT NOT NULL DEFAULT 'sequential'",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "ALTER TABLE employee_groups ADD COLUMN visibility_mode TEXT NOT NULL DEFAULT 'internal'",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "ALTER TABLE employee_groups ADD COLUMN is_bootstrap_seeded INTEGER NOT NULL DEFAULT 0",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "ALTER TABLE employee_groups ADD COLUMN config_json TEXT NOT NULL DEFAULT '{}'",
    )
    .execute(pool)
    .await;

    let _ =
        sqlx::query("ALTER TABLE group_runs ADD COLUMN current_phase TEXT NOT NULL DEFAULT 'plan'")
            .execute(pool)
            .await;
    let _ =
        sqlx::query("ALTER TABLE group_runs ADD COLUMN entry_session_id TEXT NOT NULL DEFAULT ''")
            .execute(pool)
            .await;
    let _ =
        sqlx::query("ALTER TABLE group_runs ADD COLUMN main_employee_id TEXT NOT NULL DEFAULT ''")
            .execute(pool)
            .await;
    let _ =
        sqlx::query("ALTER TABLE group_runs ADD COLUMN review_round INTEGER NOT NULL DEFAULT 0")
            .execute(pool)
            .await;
    let _ = sqlx::query("ALTER TABLE group_runs ADD COLUMN status_reason TEXT NOT NULL DEFAULT ''")
        .execute(pool)
        .await;
    let _ =
        sqlx::query("ALTER TABLE group_runs ADD COLUMN template_version TEXT NOT NULL DEFAULT ''")
            .execute(pool)
            .await;
    let _ = sqlx::query(
        "ALTER TABLE group_runs ADD COLUMN waiting_for_employee_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "ALTER TABLE group_runs ADD COLUMN waiting_for_user INTEGER NOT NULL DEFAULT 0",
    )
    .execute(pool)
    .await;

    let _ = sqlx::query(
        "ALTER TABLE group_run_steps ADD COLUMN parent_step_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query("ALTER TABLE group_run_steps ADD COLUMN phase TEXT NOT NULL DEFAULT ''")
        .execute(pool)
        .await;
    let _ = sqlx::query(
        "ALTER TABLE group_run_steps ADD COLUMN step_kind TEXT NOT NULL DEFAULT 'execute'",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "ALTER TABLE group_run_steps ADD COLUMN requires_review INTEGER NOT NULL DEFAULT 0",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "ALTER TABLE group_run_steps ADD COLUMN review_status TEXT NOT NULL DEFAULT 'not_required'",
    )
    .execute(pool)
    .await;
    let _ =
        sqlx::query("ALTER TABLE group_run_steps ADD COLUMN attempt_no INTEGER NOT NULL DEFAULT 0")
            .execute(pool)
            .await;
    let _ =
        sqlx::query("ALTER TABLE group_run_steps ADD COLUMN session_id TEXT NOT NULL DEFAULT ''")
            .execute(pool)
            .await;
    let _ = sqlx::query(
        "ALTER TABLE group_run_steps ADD COLUMN dispatch_source_employee_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query("ALTER TABLE group_run_steps ADD COLUMN assignee_profile_id TEXT")
        .execute(pool)
        .await;
    let _ = sqlx::query("ALTER TABLE group_run_steps ADD COLUMN dispatch_source_profile_id TEXT")
        .execute(pool)
        .await;
    let _ = sqlx::query(
        "ALTER TABLE group_run_steps ADD COLUMN input_summary TEXT NOT NULL DEFAULT ''",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "ALTER TABLE group_run_steps ADD COLUMN output_summary TEXT NOT NULL DEFAULT ''",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "ALTER TABLE group_run_steps ADD COLUMN visibility TEXT NOT NULL DEFAULT 'internal'",
    )
    .execute(pool)
    .await;

    Ok(())
}

pub(super) async fn ensure_agent_conversation_binding_tables(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS agent_conversation_bindings (
            conversation_id TEXT NOT NULL,
            channel TEXT NOT NULL,
            account_id TEXT NOT NULL DEFAULT '',
            agent_id TEXT NOT NULL,
            session_key TEXT NOT NULL,
            session_id TEXT NOT NULL DEFAULT '',
            base_conversation_id TEXT NOT NULL DEFAULT '',
            parent_conversation_candidates_json TEXT NOT NULL DEFAULT '[]',
            scope TEXT NOT NULL DEFAULT '',
            peer_kind TEXT NOT NULL DEFAULT '',
            peer_id TEXT NOT NULL DEFAULT '',
            topic_id TEXT NOT NULL DEFAULT '',
            sender_id TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            PRIMARY KEY (conversation_id, agent_id)
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_agent_conversation_bindings_session_key ON agent_conversation_bindings(session_key)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_agent_conversation_bindings_channel_account ON agent_conversation_bindings(channel, account_id)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS channel_delivery_routes (
            session_key TEXT NOT NULL PRIMARY KEY,
            channel TEXT NOT NULL,
            account_id TEXT NOT NULL DEFAULT '',
            conversation_id TEXT NOT NULL,
            reply_target TEXT NOT NULL DEFAULT '',
            updated_at TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_channel_delivery_routes_conversation ON channel_delivery_routes(conversation_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_channel_delivery_routes_channel_account ON channel_delivery_routes(channel, account_id)",
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub(super) async fn ensure_im_conversation_sessions_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS im_conversation_sessions (
            conversation_id TEXT NOT NULL,
            employee_id TEXT NOT NULL,
            thread_id TEXT NOT NULL DEFAULT '',
            session_id TEXT NOT NULL,
            route_session_key TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            channel TEXT NOT NULL DEFAULT '',
            account_id TEXT NOT NULL DEFAULT '',
            base_conversation_id TEXT NOT NULL DEFAULT '',
            parent_conversation_candidates_json TEXT NOT NULL DEFAULT '[]',
            scope TEXT NOT NULL DEFAULT '',
            peer_kind TEXT NOT NULL DEFAULT '',
            peer_id TEXT NOT NULL DEFAULT '',
            topic_id TEXT NOT NULL DEFAULT '',
            sender_id TEXT NOT NULL DEFAULT '',
            PRIMARY KEY (conversation_id, employee_id)
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_im_conversation_sessions_session_id ON im_conversation_sessions(session_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_im_conversation_sessions_thread_id ON im_conversation_sessions(thread_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_im_conversation_sessions_channel_account ON im_conversation_sessions(channel, account_id)",
    )
    .execute(pool)
    .await?;

    if im_thread_sessions_exists(pool).await? {
        sqlx::query(
            "INSERT INTO im_conversation_sessions (
            conversation_id,
            employee_id,
            thread_id,
            session_id,
            route_session_key,
            created_at,
            updated_at,
            channel,
            account_id,
            base_conversation_id,
            parent_conversation_candidates_json,
            scope,
            peer_kind,
            peer_id,
            topic_id,
            sender_id
        )
        SELECT
            COALESCE(NULLIF(TRIM(conversation_id), ''), thread_id),
            employee_id,
            thread_id,
            session_id,
            route_session_key,
            created_at,
            updated_at,
            COALESCE(channel, ''),
            COALESCE(account_id, ''),
            COALESCE(NULLIF(TRIM(base_conversation_id), ''), COALESCE(NULLIF(TRIM(conversation_id), ''), thread_id)),
            COALESCE(parent_conversation_candidates_json, '[]'),
            COALESCE(scope, ''),
            COALESCE(peer_kind, ''),
            COALESCE(NULLIF(TRIM(peer_id), ''), thread_id),
            COALESCE(topic_id, ''),
            COALESCE(sender_id, '')
        FROM im_thread_sessions
        WHERE TRIM(thread_id) <> ''
        ON CONFLICT(conversation_id, employee_id) DO UPDATE SET
            thread_id = excluded.thread_id,
            session_id = excluded.session_id,
            route_session_key = excluded.route_session_key,
            updated_at = excluded.updated_at,
            channel = excluded.channel,
            account_id = excluded.account_id,
            base_conversation_id = excluded.base_conversation_id,
            parent_conversation_candidates_json = excluded.parent_conversation_candidates_json,
            scope = excluded.scope,
            peer_kind = excluded.peer_kind,
            peer_id = excluded.peer_id,
            topic_id = excluded.topic_id,
            sender_id = excluded.sender_id",
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub(super) async fn backfill_authority_binding_tables(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "INSERT INTO agent_conversation_bindings (
            conversation_id,
            channel,
            account_id,
            agent_id,
            session_key,
            session_id,
            base_conversation_id,
            parent_conversation_candidates_json,
            scope,
            peer_kind,
            peer_id,
            topic_id,
            sender_id,
            created_at,
            updated_at
        )
        SELECT
            COALESCE(NULLIF(TRIM(conversation_id), ''), thread_id),
            COALESCE(channel, ''),
            COALESCE(account_id, ''),
            employee_id,
            COALESCE(NULLIF(TRIM(route_session_key), ''), session_id),
            session_id,
            COALESCE(NULLIF(TRIM(base_conversation_id), ''), COALESCE(NULLIF(TRIM(conversation_id), ''), thread_id)),
            COALESCE(parent_conversation_candidates_json, '[]'),
            COALESCE(scope, ''),
            COALESCE(peer_kind, ''),
            COALESCE(NULLIF(TRIM(peer_id), ''), thread_id),
            COALESCE(topic_id, ''),
            COALESCE(sender_id, ''),
            created_at,
            updated_at
        FROM im_conversation_sessions
        WHERE TRIM(employee_id) <> ''
          AND TRIM(session_id) <> ''
        ON CONFLICT(conversation_id, agent_id) DO UPDATE SET
            channel = excluded.channel,
            account_id = excluded.account_id,
            session_key = excluded.session_key,
            session_id = excluded.session_id,
            base_conversation_id = excluded.base_conversation_id,
            parent_conversation_candidates_json = excluded.parent_conversation_candidates_json,
            scope = excluded.scope,
            peer_kind = excluded.peer_kind,
            peer_id = excluded.peer_id,
            topic_id = excluded.topic_id,
            sender_id = excluded.sender_id,
            updated_at = excluded.updated_at",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO channel_delivery_routes (
            session_key,
            channel,
            account_id,
            conversation_id,
            reply_target,
            updated_at
        )
        SELECT
            COALESCE(NULLIF(TRIM(route_session_key), ''), session_id),
            COALESCE(channel, ''),
            COALESCE(account_id, ''),
            COALESCE(NULLIF(TRIM(conversation_id), ''), thread_id),
            COALESCE(NULLIF(TRIM(thread_id), ''), COALESCE(NULLIF(TRIM(peer_id), ''), COALESCE(NULLIF(TRIM(conversation_id), ''), ''))),
            updated_at
        FROM im_conversation_sessions
        WHERE TRIM(session_id) <> ''
        ON CONFLICT(session_key) DO UPDATE SET
            channel = excluded.channel,
            account_id = excluded.account_id,
            conversation_id = excluded.conversation_id,
            reply_target = excluded.reply_target,
            updated_at = excluded.updated_at",
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub(super) async fn ensure_im_thread_sessions_conversation_columns(
    pool: &SqlitePool,
) -> Result<()> {
    ensure_im_thread_sessions_channel_column(pool).await?;

    let _ = sqlx::query(
        "ALTER TABLE im_thread_sessions ADD COLUMN account_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "ALTER TABLE im_thread_sessions ADD COLUMN conversation_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "ALTER TABLE im_thread_sessions ADD COLUMN base_conversation_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "ALTER TABLE im_thread_sessions ADD COLUMN parent_conversation_candidates_json TEXT NOT NULL DEFAULT '[]'",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query("ALTER TABLE im_thread_sessions ADD COLUMN scope TEXT NOT NULL DEFAULT ''")
        .execute(pool)
        .await;
    let _ =
        sqlx::query("ALTER TABLE im_thread_sessions ADD COLUMN peer_kind TEXT NOT NULL DEFAULT ''")
            .execute(pool)
            .await;
    let _ =
        sqlx::query("ALTER TABLE im_thread_sessions ADD COLUMN peer_id TEXT NOT NULL DEFAULT ''")
            .execute(pool)
            .await;
    let _ =
        sqlx::query("ALTER TABLE im_thread_sessions ADD COLUMN topic_id TEXT NOT NULL DEFAULT ''")
            .execute(pool)
            .await;
    let _ =
        sqlx::query("ALTER TABLE im_thread_sessions ADD COLUMN sender_id TEXT NOT NULL DEFAULT ''")
            .execute(pool)
            .await;

    Ok(())
}

pub(super) async fn ensure_im_thread_sessions_channel_column(pool: &SqlitePool) -> Result<()> {
    let columns: Vec<String> =
        sqlx::query_scalar("SELECT name FROM pragma_table_info('im_thread_sessions')")
            .fetch_all(pool)
            .await?;
    if columns.iter().any(|name| name == "channel") {
        return Ok(());
    }

    let _ =
        sqlx::query("ALTER TABLE im_thread_sessions ADD COLUMN channel TEXT NOT NULL DEFAULT ''")
            .execute(pool)
            .await;

    Ok(())
}

#[cfg(test)]
pub(super) async fn apply_legacy_migrations_for_test(pool: &SqlitePool) -> Result<()> {
    apply_legacy_migrations(pool).await
}

#[cfg(test)]
mod tests {
    use super::apply_legacy_migrations_for_test;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn legacy_migrations_create_openclaw_binding_tables_without_legacy_im_tables() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("create sqlite memory pool");

        apply_legacy_migrations_for_test(&pool)
            .await
            .expect("apply legacy migrations");

        let tables: Vec<String> = sqlx::query_scalar(
            "SELECT name FROM sqlite_master
             WHERE type = 'table'
             AND name IN ('agent_conversation_bindings', 'channel_delivery_routes')",
        )
        .fetch_all(&pool)
        .await
        .expect("query openclaw binding tables");

        assert_eq!(tables.len(), 2, "expected openclaw binding tables");
    }

    #[tokio::test]
    async fn legacy_thread_only_db_backfills_authority_binding_tables() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("create sqlite memory pool");

        sqlx::query(
            "CREATE TABLE im_thread_sessions (
                thread_id TEXT NOT NULL,
                employee_id TEXT NOT NULL,
                session_id TEXT NOT NULL,
                route_session_key TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (thread_id, employee_id)
            )",
        )
        .execute(&pool)
        .await
        .expect("create legacy im_thread_sessions table");

        sqlx::query(
            "INSERT INTO im_thread_sessions (
                thread_id,
                employee_id,
                session_id,
                route_session_key,
                created_at,
                updated_at
             )
             VALUES (
                'legacy-thread',
                'emp-legacy',
                'session-legacy',
                '',
                '2026-04-22T00:00:00Z',
                '2026-04-22T00:00:01Z'
             )",
        )
        .execute(&pool)
        .await
        .expect("seed legacy im_thread_sessions row");

        apply_legacy_migrations_for_test(&pool)
            .await
            .expect("apply legacy migrations");

        let counts: (i64, i64, i64) = sqlx::query_as(
            "SELECT
                (SELECT COUNT(*) FROM im_conversation_sessions),
                (SELECT COUNT(*) FROM agent_conversation_bindings),
                (SELECT COUNT(*) FROM channel_delivery_routes)",
        )
        .fetch_one(&pool)
        .await
        .expect("query migrated authority counts");

        assert_eq!(counts.0, 1, "expected conversation session backfill");
        assert_eq!(counts.1, 1, "expected agent conversation binding backfill");
        assert_eq!(counts.2, 1, "expected channel delivery route backfill");

        let binding: (String, String, String) = sqlx::query_as(
            "SELECT conversation_id, session_key, session_id
             FROM agent_conversation_bindings
             WHERE agent_id = 'emp-legacy'",
        )
        .fetch_one(&pool)
        .await
        .expect("query authority binding");
        assert_eq!(binding.0, "legacy-thread");
        assert_eq!(binding.1, "session-legacy");
        assert_eq!(binding.2, "session-legacy");

        let route: (String, String) = sqlx::query_as(
            "SELECT session_key, reply_target
             FROM channel_delivery_routes
             WHERE session_key = 'session-legacy'",
        )
        .fetch_one(&pool)
        .await
        .expect("query authority route");
        assert_eq!(route.0, "session-legacy");
        assert_eq!(route.1, "legacy-thread");
    }

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

    #[tokio::test]
    async fn legacy_migrations_create_profile_session_index_tables() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("create sqlite memory pool");

        apply_legacy_migrations_for_test(&pool)
            .await
            .expect("apply legacy migrations");

        let tables: Vec<String> = sqlx::query_scalar(
            "SELECT name FROM sqlite_master
             WHERE type = 'table'
             AND name IN ('profile_session_index', 'profile_session_fts')
             ORDER BY name",
        )
        .fetch_all(&pool)
        .await
        .expect("query profile session index tables");

        assert_eq!(
            tables,
            vec![
                "profile_session_fts".to_string(),
                "profile_session_index".to_string()
            ]
        );
    }
}
