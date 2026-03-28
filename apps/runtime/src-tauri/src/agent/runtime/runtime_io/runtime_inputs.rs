use super::extract_assistant_text_content;

async fn maybe_self_heal_builtin_skill_source_with_pool(
    pool: &sqlx::SqlitePool,
    skill_id: &str,
    username: &str,
    pack_path: &str,
    source_type: &str,
) -> Result<(String, String, String), String> {
    if source_type != "builtin" {
        return Ok((
            username.to_string(),
            pack_path.to_string(),
            source_type.to_string(),
        ));
    }

    let pack_root = std::path::Path::new(pack_path);
    if pack_path.trim().is_empty() || !pack_root.exists() {
        return Ok((
            username.to_string(),
            pack_path.to_string(),
            source_type.to_string(),
        ));
    }

    sqlx::query(
        "UPDATE installed_skills
         SET username = '', source_type = 'vendored'
         WHERE id = ? AND COALESCE(source_type, 'encrypted') = 'builtin'",
    )
    .bind(skill_id)
    .execute(pool)
    .await
    .map_err(|e| format!("自愈 legacy builtin skill 失败 (skill_id={skill_id}): {e}"))?;

    Ok(("".to_string(), pack_path.to_string(), "vendored".to_string()))
}

pub(crate) async fn load_session_runtime_inputs_with_pool(
    pool: &sqlx::SqlitePool,
    session_id: &str,
) -> Result<(String, String, String, String, String), String> {
    sqlx::query_as::<_, (String, String, String, String, String)>(
        "SELECT skill_id, model_id, permission_mode, COALESCE(work_dir, ''), COALESCE(employee_id, '') FROM sessions WHERE id = ?",
    )
    .bind(session_id)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("会话不存在 (session_id={session_id}): {e}"))
}

pub(crate) async fn load_installed_skill_source_with_pool(
    pool: &sqlx::SqlitePool,
    skill_id: &str,
) -> Result<(String, String, String, String), String> {
    let (manifest, username, pack_path, source_type) = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT manifest, username, pack_path, COALESCE(source_type, 'encrypted') FROM installed_skills WHERE id = ?",
    )
    .bind(skill_id)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Skill 不存在 (skill_id={skill_id}): {e}"))?;

    let (username, pack_path, source_type) = maybe_self_heal_builtin_skill_source_with_pool(
        pool,
        skill_id,
        &username,
        &pack_path,
        &source_type,
    )
    .await?;

    Ok((manifest, username, pack_path, source_type))
}

pub(crate) async fn load_session_history_with_pool(
    pool: &sqlx::SqlitePool,
    session_id: &str,
) -> Result<Vec<(String, String, Option<String>)>, String> {
    let rows = sqlx::query_as::<_, (String, String, Option<String>)>(
        "SELECT role, content, content_json FROM messages WHERE session_id = ? ORDER BY created_at ASC",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|(role, content, content_json)| {
            if role == "assistant" {
                (role, extract_assistant_text_content(&content), None)
            } else {
                (role, content, content_json)
            }
        })
        .collect())
}

pub(crate) async fn load_default_search_provider_config_with_pool(
    pool: &sqlx::SqlitePool,
) -> Result<Option<(String, String, String, String)>, String> {
    sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT api_format, base_url, api_key, model_name FROM model_configs WHERE api_format LIKE 'search_%' AND is_default = 1 LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::load_installed_skill_source_with_pool;
    use chrono::Utc;
    use skillpack_rs::SkillManifest;
    use sqlx::sqlite::SqlitePoolOptions;
    use tempfile::tempdir;

    async fn setup_memory_pool() -> sqlx::SqlitePool {
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
                last_used_at TEXT,
                username TEXT NOT NULL,
                pack_path TEXT NOT NULL DEFAULT '',
                source_type TEXT NOT NULL DEFAULT 'encrypted'
            )",
        )
        .execute(&pool)
        .await
        .expect("create installed_skills table");

        pool
    }

    #[tokio::test]
    async fn load_installed_skill_source_self_heals_builtin_rows_with_existing_pack_path() {
        let pool = setup_memory_pool().await;
        let vendor_root = tempdir().expect("create vendor root");
        let skill_dir = vendor_root.path().join("builtin-general");
        std::fs::create_dir_all(&skill_dir).expect("create skill dir");
        std::fs::write(skill_dir.join("SKILL.md"), "# Builtin").expect("write skill markdown");

        let manifest = SkillManifest {
            id: "builtin-general".to_string(),
            name: "通用助手".to_string(),
            description: "Generic assistant".to_string(),
            version: "builtin".to_string(),
            author: "WorkClaw".to_string(),
            recommended_model: "gpt-4o".to_string(),
            tags: vec![],
            created_at: Utc::now(),
            username_hint: None,
            encrypted_verify: String::new(),
        };

        sqlx::query(
            "INSERT INTO installed_skills (id, manifest, installed_at, username, pack_path, source_type)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind("builtin-general")
        .bind(serde_json::to_string(&manifest).unwrap())
        .bind(Utc::now().to_rfc3339())
        .bind("legacy-user")
        .bind(skill_dir.to_string_lossy().to_string())
        .bind("builtin")
        .execute(&pool)
        .await
        .expect("insert legacy builtin row");

        let (_, username, pack_path, source_type) =
            load_installed_skill_source_with_pool(&pool, "builtin-general")
                .await
                .expect("load builtin skill source");

        assert_eq!(username, "");
        assert_eq!(pack_path, skill_dir.to_string_lossy());
        assert_eq!(source_type, "vendored");

        let (stored_source_type, stored_username): (String, String) = sqlx::query_as(
            "SELECT source_type, username FROM installed_skills WHERE id = 'builtin-general'",
        )
        .fetch_one(&pool)
        .await
        .expect("query self-healed row");
        assert_eq!(stored_source_type, "vendored");
        assert_eq!(stored_username, "");
    }

    #[tokio::test]
    async fn load_installed_skill_source_keeps_legacy_builtin_rows_without_pack_path() {
        let pool = setup_memory_pool().await;
        let manifest = SkillManifest {
            id: "builtin-general".to_string(),
            name: "通用助手".to_string(),
            description: "Generic assistant".to_string(),
            version: "builtin".to_string(),
            author: "WorkClaw".to_string(),
            recommended_model: "gpt-4o".to_string(),
            tags: vec![],
            created_at: Utc::now(),
            username_hint: None,
            encrypted_verify: String::new(),
        };

        sqlx::query(
            "INSERT INTO installed_skills (id, manifest, installed_at, username, pack_path, source_type)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind("builtin-general")
        .bind(serde_json::to_string(&manifest).unwrap())
        .bind(Utc::now().to_rfc3339())
        .bind("legacy-user")
        .bind("")
        .bind("builtin")
        .execute(&pool)
        .await
        .expect("insert legacy builtin row");

        let (_, username, pack_path, source_type) =
            load_installed_skill_source_with_pool(&pool, "builtin-general")
                .await
                .expect("load builtin skill source");

        assert_eq!(username, "legacy-user");
        assert_eq!(pack_path, "");
        assert_eq!(source_type, "builtin");
    }
}
