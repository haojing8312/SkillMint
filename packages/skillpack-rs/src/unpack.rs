use std::io::Read;
use anyhow::{Result, anyhow};

use crate::crypto::{derive_key, decrypt, check_verify_token};
use crate::types::SkillManifest;

#[derive(Debug)]
pub struct UnpackedSkill {
    pub manifest: SkillManifest,
    /// Map of relative path -> decrypted content bytes
    /// e.g. "SKILL.md" -> b"..."
    pub files: std::collections::HashMap<String, Vec<u8>>,
}

pub fn verify_and_unpack(pack_path: &str, username: &str) -> Result<UnpackedSkill> {
    let file = std::fs::File::open(pack_path)?;
    let mut zip = zip::ZipArchive::new(file)?;

    // Read manifest
    let manifest: SkillManifest = {
        let mut entry = zip.by_name("manifest.json")
            .map_err(|_| anyhow!("manifest.json not found in skillpack"))?;
        let mut buf = String::new();
        entry.read_to_string(&mut buf)?;
        serde_json::from_str(&buf)?
    };

    // Derive key and verify username
    let key = derive_key(username, &manifest.id, &manifest.name);
    if !check_verify_token(&manifest.encrypted_verify, &key) {
        return Err(anyhow!("用户名错误，无法解密此 Skill"));
    }

    // Collect encrypted file names first
    let names: Vec<String> = (0..zip.len())
        .filter_map(|i| {
            let entry = zip.by_index(i).ok()?;
            let name = entry.name().to_string();
            if name.starts_with("encrypted/") && name.ends_with(".enc") {
                Some(name)
            } else {
                None
            }
        })
        .collect();

    // Decrypt all files in encrypted/
    let mut files = std::collections::HashMap::new();
    for enc_name in names {
        let mut entry = zip.by_name(&enc_name)?;
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        let plain = decrypt(&buf, &key)?;

        // Strip "encrypted/" prefix and ".enc" suffix to get original path
        let rel = enc_name
            .strip_prefix("encrypted/").unwrap()
            .strip_suffix(".enc").unwrap()
            .to_string();
        files.insert(rel, plain);
    }

    Ok(UnpackedSkill { manifest, files })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pack::pack;
    use crate::types::PackConfig;
    use tempfile::tempdir;
    use std::fs;
    use std::path::Path;

    fn setup_and_pack(dir: &Path, username: &str) -> String {
        let skill_dir = dir.join("skill");
        fs::create_dir(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "---\nname: Test\n---\nYou are a test.").unwrap();
        let output = dir.join("test.skillpack");
        pack(&PackConfig {
            dir_path: skill_dir.to_string_lossy().to_string(),
            name: "Test".to_string(),
            description: "desc".to_string(),
            version: "1.0.0".to_string(),
            author: "author".to_string(),
            username: username.to_string(),
            recommended_model: "claude-3-5-sonnet-20241022".to_string(),
            output_path: output.to_string_lossy().to_string(),
        }).unwrap();
        output.to_string_lossy().to_string()
    }

    #[test]
    fn test_correct_username_unpacks() {
        let dir = tempdir().unwrap();
        let pack_path = setup_and_pack(dir.path(), "alice");
        let result = verify_and_unpack(&pack_path, "alice").unwrap();
        assert_eq!(result.manifest.name, "Test");
        assert!(result.files.contains_key("SKILL.md"));
    }

    #[test]
    fn test_wrong_username_fails() {
        let dir = tempdir().unwrap();
        let pack_path = setup_and_pack(dir.path(), "alice");
        let result = verify_and_unpack(&pack_path, "bob");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("用户名错误"));
    }
}
