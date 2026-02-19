use skillpack_rs::{pack, PackConfig, FrontMatter};
use skillpack_rs::pack::parse_front_matter;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillDirInfo {
    pub files: Vec<String>,
    pub front_matter: FrontMatter,
}

#[tauri::command]
pub async fn read_skill_dir(dir_path: String) -> Result<SkillDirInfo, String> {
    let skill_dir = Path::new(&dir_path);
    let skill_md_path = skill_dir.join("SKILL.md");

    if !skill_md_path.exists() {
        return Err("SKILL.md not found in selected directory".to_string());
    }

    let skill_md_content = fs::read_to_string(&skill_md_path)
        .map_err(|e| e.to_string())?;
    let front_matter = parse_front_matter(&skill_md_content);

    let files: Vec<String> = WalkDir::new(skill_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| {
            e.path()
                .strip_prefix(skill_dir)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();

    Ok(SkillDirInfo { files, front_matter })
}

#[tauri::command]
pub async fn pack_skill(
    dir_path: String,
    name: String,
    description: String,
    version: String,
    author: String,
    username: String,
    recommended_model: String,
    output_path: String,
) -> Result<(), String> {
    let config = PackConfig {
        dir_path,
        name,
        description,
        version,
        author,
        username,
        recommended_model,
        output_path,
    };
    pack(&config).map_err(|e| e.to_string())
}
