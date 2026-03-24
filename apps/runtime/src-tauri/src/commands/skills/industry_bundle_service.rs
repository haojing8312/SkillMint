use super::helpers::{compare_semver, extract_tag_value};
use super::local_skill_service::import_local_skill_to_pool;
use super::types::{IndustryBundleUpdateCheck, IndustryInstallResult, InstalledSkillSummary};
use skillpack_rs::SkillManifest;
use sqlx::SqlitePool;
use std::collections::HashSet;
use std::path::Path;

pub async fn install_industry_bundle_to_pool(
    bundle_path: String,
    install_root: Option<String>,
    pool: &SqlitePool,
) -> Result<IndustryInstallResult, String> {
    let unpacked =
        crate::commands::packaging::unpack_industry_bundle_to_root(&bundle_path, install_root)?;

    let mut installed_skills = Vec::new();
    let mut missing_mcp_set = HashSet::new();
    for skill_dir in &unpacked.skill_dirs {
        let local_name = Path::new(skill_dir)
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_string();
        let slug = local_name
            .split_once("--")
            .map(|(_, right)| right.to_string())
            .unwrap_or(local_name.clone());
        let skill_meta = unpacked
            .manifest
            .skills
            .iter()
            .find(|item| item.slug == slug);

        let mut extra_tags = vec![
            format!("pack:{}", unpacked.manifest.pack_id),
            format!("pack-version:{}", unpacked.manifest.version),
        ];
        if !unpacked.manifest.industry_tag.trim().is_empty() {
            extra_tags.push(format!("industry:{}", unpacked.manifest.industry_tag));
        }
        if let Some(meta) = skill_meta {
            extra_tags.extend(meta.tags.clone());
        }

        let import = import_local_skill_to_pool(skill_dir.clone(), pool, &extra_tags).await?; 
        installed_skills.push(InstalledSkillSummary {
            id: import.manifest.id,
            name: import.manifest.name,
        });
        for mcp in import.missing_mcp {
            missing_mcp_set.insert(mcp);
        }
    }

    let mut missing_mcp = missing_mcp_set.into_iter().collect::<Vec<_>>();
    missing_mcp.sort();
    Ok(IndustryInstallResult {
        pack_id: unpacked.manifest.pack_id,
        version: unpacked.manifest.version,
        installed_skills,
        missing_mcp,
    })
}

pub async fn install_industry_bundle(
    bundle_path: String,
    install_root: Option<String>,
    pool: &SqlitePool,
) -> Result<IndustryInstallResult, String> {
    install_industry_bundle_to_pool(bundle_path, install_root, pool).await
}

pub async fn check_industry_bundle_update_from_pool(
    bundle_path: String,
    pool: &SqlitePool,
) -> Result<IndustryBundleUpdateCheck, String> {
    let manifest =
        crate::commands::packaging::read_industry_bundle_manifest_from_path(&bundle_path)?;
    let rows = sqlx::query_as::<_, (String,)>(
        "SELECT manifest FROM installed_skills WHERE COALESCE(source_type, 'local') = 'local'",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut current_version: Option<String> = None;
    for (json,) in rows {
        let Ok(skill_manifest) = serde_json::from_str::<SkillManifest>(&json) else {
            continue;
        };
        let Some(pack_id) = extract_tag_value(&skill_manifest.tags, "pack:") else {
            continue;
        };
        if pack_id != manifest.pack_id {
            continue;
        }
        let Some(version) = extract_tag_value(&skill_manifest.tags, "pack-version:") else {
            continue;
        };
        current_version = match current_version {
            None => Some(version),
            Some(existing) => {
                if compare_semver(&version, &existing) == std::cmp::Ordering::Greater {
                    Some(version)
                } else {
                    Some(existing)
                }
            }
        };
    }

    let has_update = match current_version.as_ref() {
        Some(current) => compare_semver(&manifest.version, current) == std::cmp::Ordering::Greater,
        None => true,
    };
    let message = match current_version.as_ref() {
        Some(current) if has_update => format!("发现新版本：{} -> {}", current, manifest.version),
        Some(current) => format!("已是最新版本（当前 {}）", current),
        None => format!("尚未安装，可导入版本 {}", manifest.version),
    };

    Ok(IndustryBundleUpdateCheck {
        pack_id: manifest.pack_id,
        current_version,
        candidate_version: manifest.version,
        has_update,
        message,
    })
}

pub async fn check_industry_bundle_update(
    bundle_path: String,
    pool: &SqlitePool,
) -> Result<IndustryBundleUpdateCheck, String> {
    check_industry_bundle_update_from_pool(bundle_path, pool).await
}
