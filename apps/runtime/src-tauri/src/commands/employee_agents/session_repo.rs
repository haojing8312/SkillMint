use sqlx::{Row, SqlitePool};

pub(crate) struct ThreadSessionRecord {
    pub session_id: String,
    pub session_exists: bool,
}

pub(crate) struct SessionSeedInput<'a> {
    pub id: &'a str,
    pub skill_id: &'a str,
    pub title: &'a str,
    pub created_at: &'a str,
    pub model_id: &'a str,
    pub work_dir: &'a str,
    pub employee_id: &'a str,
}

pub(crate) struct ThreadSessionLinkInput<'a> {
    pub thread_id: &'a str,
    pub employee_db_id: &'a str,
    pub session_id: &'a str,
    pub route_session_key: &'a str,
    pub created_at: &'a str,
    pub updated_at: &'a str,
}

pub(crate) struct InboundEventLinkInput<'a> {
    pub id: &'a str,
    pub thread_id: &'a str,
    pub session_id: &'a str,
    pub employee_db_id: &'a str,
    pub im_event_id: &'a str,
    pub im_message_id: &'a str,
    pub created_at: &'a str,
}

pub(crate) async fn find_latest_thread_session_id(
    pool: &SqlitePool,
    thread_id: &str,
) -> Result<Option<String>, String> {
    let row = sqlx::query(
        "SELECT ts.session_id
         FROM im_thread_sessions ts
         INNER JOIN sessions s ON s.id = ts.session_id
         WHERE ts.thread_id = ?
         ORDER BY ts.updated_at DESC
         LIMIT 1",
    )
    .bind(thread_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|record| record.try_get(0).expect("latest thread session id")))
}

pub(crate) async fn find_thread_session_record(
    pool: &SqlitePool,
    thread_id: &str,
    employee_db_id: &str,
) -> Result<Option<ThreadSessionRecord>, String> {
    let row = sqlx::query(
        "SELECT ts.session_id,
                CASE WHEN s.id IS NULL THEN 0 ELSE 1 END AS session_exists
         FROM im_thread_sessions ts
         LEFT JOIN sessions s ON s.id = ts.session_id
         WHERE ts.thread_id = ? AND ts.employee_id = ?
         LIMIT 1",
    )
    .bind(thread_id)
    .bind(employee_db_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|record| ThreadSessionRecord {
        session_id: record.try_get(0).expect("thread session record session_id"),
        session_exists: record
            .try_get::<i64, _>(1)
            .expect("thread session record session_exists")
            != 0,
    }))
}

pub(crate) async fn upsert_thread_session_link(
    pool: &SqlitePool,
    input: &ThreadSessionLinkInput<'_>,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO im_thread_sessions (thread_id, employee_id, session_id, route_session_key, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(thread_id, employee_id) DO UPDATE SET
            session_id = excluded.session_id,
            route_session_key = excluded.route_session_key,
            updated_at = excluded.updated_at",
    )
    .bind(input.thread_id)
    .bind(input.employee_db_id)
    .bind(input.session_id)
    .bind(input.route_session_key)
    .bind(input.created_at)
    .bind(input.updated_at)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn find_recent_route_session_id(
    pool: &SqlitePool,
    employee_db_id: &str,
    route_session_key: &str,
) -> Result<Option<String>, String> {
    let row = sqlx::query(
        "SELECT ts.session_id
         FROM im_thread_sessions ts
         INNER JOIN sessions s ON s.id = ts.session_id
         WHERE ts.employee_id = ? AND ts.route_session_key = ?
         ORDER BY ts.updated_at DESC
         LIMIT 1",
    )
    .bind(employee_db_id)
    .bind(route_session_key)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|record| record.try_get(0).expect("recent route session id")))
}

pub(crate) async fn insert_session_seed(
    pool: &SqlitePool,
    input: &SessionSeedInput<'_>,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO sessions (id, skill_id, title, created_at, model_id, permission_mode, work_dir, employee_id)
         VALUES (?, ?, ?, ?, ?, 'standard', ?, ?)",
    )
    .bind(input.id)
    .bind(input.skill_id)
    .bind(input.title)
    .bind(input.created_at)
    .bind(input.model_id)
    .bind(input.work_dir)
    .bind(input.employee_id)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn update_session_employee_id(
    pool: &SqlitePool,
    session_id: &str,
    employee_id: &str,
) -> Result<(), String> {
    sqlx::query("UPDATE sessions SET employee_id = ? WHERE id = ?")
        .bind(employee_id)
        .bind(session_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn insert_inbound_event_link(
    pool: &SqlitePool,
    input: &InboundEventLinkInput<'_>,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO im_message_links (id, thread_id, session_id, employee_id, direction, im_event_id, im_message_id, app_message_id, created_at)
         VALUES (?, ?, ?, ?, 'inbound', ?, ?, '', ?)",
    )
    .bind(input.id)
    .bind(input.thread_id)
    .bind(input.session_id)
    .bind(input.employee_db_id)
    .bind(input.im_event_id)
    .bind(input.im_message_id)
    .bind(input.created_at)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}
