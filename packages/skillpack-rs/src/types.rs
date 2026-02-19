use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub recommended_model: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub username_hint: Option<String>,
    pub encrypted_verify: String,
}

#[derive(Debug, Clone)]
pub struct PackConfig {
    pub dir_path: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub username: String,
    pub recommended_model: String,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontMatter {
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub model: Option<String>,
}
