use skillpack_rs::SkillManifest;
use sqlx::SqlitePool;

pub struct DbState(pub SqlitePool);

#[derive(serde::Serialize)]
pub struct ImportResult {
    pub manifest: SkillManifest,
    pub missing_mcp: Vec<String>,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct LocalImportInstalledItem {
    pub dir_path: String,
    pub manifest: SkillManifest,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct LocalImportFailedItem {
    pub dir_path: String,
    pub name_hint: String,
    pub error: String,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct LocalImportBatchResult {
    pub installed: Vec<LocalImportInstalledItem>,
    pub failed: Vec<LocalImportFailedItem>,
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
pub struct InstalledSkillListItem {
    #[serde(flatten)]
    pub manifest: SkillManifest,
    pub source_type: String,
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

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SkillRuntimeDependencyCheck {
    pub key: String,
    pub label: String,
    pub satisfied: bool,
    pub detail: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SkillRuntimeEnvironmentStatus {
    pub skill_id: String,
    pub skill_name: String,
    pub source_type: String,
    pub ready: bool,
    pub primary_env: Option<String>,
    pub missing_bins: Vec<String>,
    pub missing_any_bins: Vec<String>,
    pub missing_env: Vec<String>,
    pub missing_config: Vec<String>,
    pub warnings: Vec<String>,
    pub checks: Vec<SkillRuntimeDependencyCheck>,
}
