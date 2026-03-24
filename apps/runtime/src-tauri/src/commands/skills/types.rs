use skillpack_rs::SkillManifest;
use sqlx::SqlitePool;

pub struct DbState(pub SqlitePool);

#[derive(serde::Serialize)]
pub struct ImportResult {
    pub manifest: SkillManifest,
    pub missing_mcp: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct LocalSkillPreview {
    pub markdown: String,
    pub save_path: String,
}

#[derive(serde::Serialize, Clone)]
pub struct InstalledSkillSummary {
    pub id: String,
    pub name: String,
}

#[derive(serde::Serialize, Clone)]
pub struct IndustryInstallResult {
    pub pack_id: String,
    pub version: String,
    pub installed_skills: Vec<InstalledSkillSummary>,
    pub missing_mcp: Vec<String>,
}

#[derive(serde::Serialize, Clone)]
pub struct IndustryBundleUpdateCheck {
    pub pack_id: String,
    pub current_version: Option<String>,
    pub candidate_version: String,
    pub has_update: bool,
    pub message: String,
}
