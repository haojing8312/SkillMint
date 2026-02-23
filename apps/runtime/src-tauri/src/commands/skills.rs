use sqlx::SqlitePool;
use tauri::State;
use skillpack_rs::{verify_and_unpack, SkillManifest};
use chrono::Utc;

pub struct DbState(pub SqlitePool);

/// 本地 Skill 导入结果，包含 manifest 和缺失的 MCP 服务器列表
#[derive(serde::Serialize)]
pub struct ImportResult {
    pub manifest: skillpack_rs::SkillManifest,
    pub missing_mcp: Vec<String>,
}

#[tauri::command]
pub async fn install_skill(
    pack_path: String,
    username: String,
    db: State<'_, DbState>,
) -> Result<SkillManifest, String> {
    let unpacked = verify_and_unpack(&pack_path, &username)
        .map_err(|e| e.to_string())?;

    let manifest_json = serde_json::to_string(&unpacked.manifest)
        .map_err(|e| e.to_string())?;

    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT OR REPLACE INTO installed_skills (id, manifest, installed_at, username, pack_path, source_type) VALUES (?, ?, ?, ?, ?, 'encrypted')"
    )
    .bind(&unpacked.manifest.id)
    .bind(&manifest_json)
    .bind(&now)
    .bind(&username)
    .bind(&pack_path)
    .execute(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    Ok(unpacked.manifest)
}

/// 导入本地 Skill 目录（读取 SKILL.md 解析 frontmatter）
#[tauri::command]
pub async fn import_local_skill(
    dir_path: String,
    db: State<'_, DbState>,
) -> Result<ImportResult, String> {
    // 读取 SKILL.md
    let skill_md_path = std::path::Path::new(&dir_path).join("SKILL.md");
    let content = std::fs::read_to_string(&skill_md_path)
        .map_err(|e| format!("无法读取 SKILL.md: {}", e))?;

    // 解析 frontmatter
    let config = crate::agent::skill_config::SkillConfig::parse(&content);

    // 构造 manifest
    let name = config.name.clone().unwrap_or_else(|| {
        // 使用目录名作为 fallback
        std::path::Path::new(&dir_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unnamed-skill".to_string())
    });
    let skill_id = format!("local-{}", name);

    let manifest = SkillManifest {
        id: skill_id.clone(),
        name: name.clone(),
        description: config.description.unwrap_or_default(),
        version: "local".to_string(),
        author: String::new(),
        recommended_model: config.model.unwrap_or_default(),
        tags: Vec::new(),
        created_at: Utc::now(),
        username_hint: None,
        encrypted_verify: String::new(),
    };

    let manifest_json = serde_json::to_string(&manifest)
        .map_err(|e| e.to_string())?;

    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT OR REPLACE INTO installed_skills (id, manifest, installed_at, username, pack_path, source_type) VALUES (?, ?, ?, ?, ?, 'local')"
    )
    .bind(&skill_id)
    .bind(&manifest_json)
    .bind(&now)
    .bind("")  // 本地 Skill 无需 username
    .bind(&dir_path)
    .execute(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    // 检查 MCP 依赖：哪些声明的 MCP 服务器尚未在数据库中配置
    let mut missing_mcp = Vec::new();
    for dep in &config.mcp_servers {
        let exists: Option<(String,)> = sqlx::query_as(
            "SELECT id FROM mcp_servers WHERE name = ?"
        )
        .bind(&dep.name)
        .fetch_optional(&db.0)
        .await
        .map_err(|e| e.to_string())?;

        if exists.is_none() {
            missing_mcp.push(dep.name.clone());
        }
    }

    Ok(ImportResult { manifest, missing_mcp })
}

/// 刷新本地 Skill（重新读取 SKILL.md 更新 manifest）
#[tauri::command]
pub async fn refresh_local_skill(
    skill_id: String,
    db: State<'_, DbState>,
) -> Result<SkillManifest, String> {
    // 从 DB 获取 pack_path（即目录路径）
    let (pack_path, source_type): (String, String) = sqlx::query_as(
        "SELECT pack_path, COALESCE(source_type, 'encrypted') FROM installed_skills WHERE id = ?"
    )
    .bind(&skill_id)
    .fetch_one(&db.0)
    .await
    .map_err(|e| format!("Skill 不存在 (skill_id={}): {}", skill_id, e))?;

    if source_type != "local" {
        return Err(format!("Skill {} 不是本地 Skill，无法刷新", skill_id));
    }

    // 重新读取 SKILL.md
    let skill_md_path = std::path::Path::new(&pack_path).join("SKILL.md");
    let content = std::fs::read_to_string(&skill_md_path)
        .map_err(|e| format!("无法读取 SKILL.md: {}", e))?;

    let config = crate::agent::skill_config::SkillConfig::parse(&content);

    let name = config.name.clone().unwrap_or_else(|| {
        std::path::Path::new(&pack_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unnamed-skill".to_string())
    });

    let manifest = SkillManifest {
        id: skill_id.clone(),
        name,
        description: config.description.unwrap_or_default(),
        version: "local".to_string(),
        author: String::new(),
        recommended_model: config.model.unwrap_or_default(),
        tags: Vec::new(),
        created_at: Utc::now(),
        username_hint: None,
        encrypted_verify: String::new(),
    };

    let manifest_json = serde_json::to_string(&manifest)
        .map_err(|e| e.to_string())?;

    // 更新 DB 中的 manifest
    sqlx::query("UPDATE installed_skills SET manifest = ? WHERE id = ?")
        .bind(&manifest_json)
        .bind(&skill_id)
        .execute(&db.0)
        .await
        .map_err(|e| e.to_string())?;

    Ok(manifest)
}

#[tauri::command]
pub async fn list_skills(db: State<'_, DbState>) -> Result<Vec<SkillManifest>, String> {
    let rows = sqlx::query_as::<_, (String,)>(
        "SELECT manifest FROM installed_skills ORDER BY installed_at DESC"
    )
    .fetch_all(&db.0)
    .await
    .map_err(|e| e.to_string())?;

    rows.iter()
        .map(|(json,)| serde_json::from_str::<SkillManifest>(json).map_err(|e| e.to_string()))
        .collect()
}

#[tauri::command]
pub async fn delete_skill(skill_id: String, db: State<'_, DbState>) -> Result<(), String> {
    sqlx::query("DELETE FROM installed_skills WHERE id = ?")
        .bind(&skill_id)
        .execute(&db.0)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
