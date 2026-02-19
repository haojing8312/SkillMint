use sqlx::SqlitePool;
use tauri::State;
use skillpack_rs::{verify_and_unpack, SkillManifest};
use chrono::Utc;

pub struct DbState(pub SqlitePool);

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
        "INSERT OR REPLACE INTO installed_skills (id, manifest, installed_at, username, pack_path) VALUES (?, ?, ?, ?, ?)"
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
