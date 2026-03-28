use super::types::{SkillRuntimeDependencyCheck, SkillRuntimeEnvironmentStatus};
use crate::agent::runtime::runtime_io::{
    load_installed_skill_source_with_pool, resolve_directory_backed_skill_root,
    resolve_workspace_skill_runtime_entry,
};
use sqlx::SqlitePool;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

pub async fn get_skill_runtime_environment_status_with_pool(
    pool: &SqlitePool,
    skill_id: &str,
) -> Result<SkillRuntimeEnvironmentStatus, String> {
    let (manifest_json, username, pack_path, source_type) =
        load_installed_skill_source_with_pool(pool, skill_id).await?;
    let runtime_entry = resolve_workspace_skill_runtime_entry(
        skill_id,
        &manifest_json,
        &username,
        &pack_path,
        &source_type,
    )?;

    let metadata = runtime_entry.metadata.clone().unwrap_or_default();
    let requires = metadata.requires.clone().unwrap_or_default();

    let mut checks = Vec::new();
    let mut missing_bins = Vec::new();
    let mut missing_any_bins = Vec::new();
    let mut missing_env = Vec::new();
    let missing_config = requires.config.clone();
    let mut warnings = Vec::new();

    for bin in &requires.bins {
        let satisfied = resolve_command_path(bin).is_some();
        if !satisfied {
            missing_bins.push(bin.clone());
        }
        checks.push(SkillRuntimeDependencyCheck {
            key: format!("bin:{bin}"),
            label: format!("Executable `{bin}`"),
            satisfied,
            detail: if satisfied {
                format!("`{bin}` is available on PATH.")
            } else {
                format!("`{bin}` was not found on PATH.")
            },
        });
    }

    if !requires.any_bins.is_empty() {
        let resolved_any = requires
            .any_bins
            .iter()
            .find_map(|bin| resolve_command_path(bin).map(|path| (bin.clone(), path)));
        if resolved_any.is_none() {
            missing_any_bins = requires.any_bins.clone();
        }
        checks.push(SkillRuntimeDependencyCheck {
            key: "any-bin-group".to_string(),
            label: "At least one compatible executable".to_string(),
            satisfied: resolved_any.is_some(),
            detail: match resolved_any {
                Some((bin, path)) => {
                    format!("Using `{bin}` at {}.", path.to_string_lossy())
                }
                None => format!(
                    "None of these executables were found on PATH: {}.",
                    requires.any_bins.join(", ")
                ),
            },
        });
    }

    for env_name in &requires.env {
        let satisfied = std::env::var(env_name)
            .ok()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);
        if !satisfied {
            missing_env.push(env_name.clone());
        }
        checks.push(SkillRuntimeDependencyCheck {
            key: format!("env:{env_name}"),
            label: format!("Environment variable `{env_name}`"),
            satisfied,
            detail: if satisfied {
                format!("`{env_name}` is set.")
            } else {
                format!("`{env_name}` is not set.")
            },
        });
    }

    let skill_root = resolve_directory_backed_skill_root(&source_type, &pack_path).or_else(|| {
        if source_type == "builtin" {
            Some(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("builtin-skills")
                    .join(skill_id.strip_prefix("builtin-").unwrap_or(skill_id)),
            )
        } else {
            None
        }
    });
    checks.extend(build_office_builtin_checks(
        skill_id,
        skill_root.as_deref(),
        &mut warnings,
    ));

    let ready = missing_bins.is_empty()
        && missing_any_bins.is_empty()
        && missing_env.is_empty()
        && missing_config.is_empty()
        && checks.iter().all(|check| check.satisfied);

    Ok(SkillRuntimeEnvironmentStatus {
        skill_id: skill_id.to_string(),
        skill_name: runtime_entry.name,
        source_type,
        ready,
        primary_env: metadata.primary_env,
        missing_bins,
        missing_any_bins,
        missing_env,
        missing_config,
        warnings,
        checks,
    })
}

fn build_office_builtin_checks(
    skill_id: &str,
    skill_root: Option<&Path>,
    warnings: &mut Vec<String>,
) -> Vec<SkillRuntimeDependencyCheck> {
    match skill_id {
        "builtin-docx" => build_docx_checks(),
        "builtin-xlsx" => build_xlsx_checks(skill_root),
        "builtin-pdf" => build_pdf_checks(skill_root),
        "builtin-pptx" => build_pptx_checks(skill_root, warnings),
        _ => Vec::new(),
    }
}

fn build_docx_checks() -> Vec<SkillRuntimeDependencyCheck> {
    let dotnet_path = resolve_command_path("dotnet");
    let mut checks = vec![binary_check("dotnet", "dotnet")];
    let sdk_probe = if dotnet_path.is_some() {
        command_succeeds("dotnet", &["--list-sdks"])
            .filter(|output| !output.trim().is_empty())
            .is_some()
    } else {
        false
    };
    checks.push(SkillRuntimeDependencyCheck {
        key: "dotnet-sdk".to_string(),
        label: ".NET SDK".to_string(),
        satisfied: sdk_probe,
        detail: if sdk_probe {
            ".NET SDKs were detected via `dotnet --list-sdks`.".to_string()
        } else {
            "No .NET SDK was detected. MiniMax DOCX needs the SDK, not just the runtime."
                .to_string()
        },
    });
    checks
}

fn build_xlsx_checks(skill_root: Option<&Path>) -> Vec<SkillRuntimeDependencyCheck> {
    let python_bin = preferred_python_bin();
    let mut checks = vec![any_binary_check(
        "python-family",
        "Python",
        &["python", "python3", "py"],
    )];
    if let Some(python) = python_bin.as_deref() {
        checks.push(python_module_check(
            python,
            "xlsx-python-stdlib",
            "Python XML/zip support",
            &["zipfile", "xml.etree.ElementTree"],
        ));
        if script_exists(skill_root, &["scripts", "xlsx_reader.py"]) {
            checks.push(python_module_check(
                python,
                "xlsx-pandas",
                "Python module `pandas`",
                &["pandas"],
            ));
        }
    }
    checks
}

fn build_pdf_checks(skill_root: Option<&Path>) -> Vec<SkillRuntimeDependencyCheck> {
    let python_bin = preferred_python_bin();
    let mut checks = vec![
        any_binary_check("python-family", "Python", &["python", "python3", "py"]),
        binary_check("node", "node"),
        binary_check("npm", "npm"),
        binary_check("npx", "npx"),
    ];
    if let Some(python) = python_bin.as_deref() {
        checks.push(python_module_check(
            python,
            "pdf-python-modules",
            "Python modules `reportlab`, `pypdf`, `matplotlib`",
            &["reportlab", "pypdf", "matplotlib"],
        ));
    }
    checks.push(node_require_check(
        "playwright",
        "Node module `playwright`",
        "playwright",
    ));
    if script_exists(skill_root, &["scripts", "render_cover.js"]) {
        checks.push(node_script_check(
            "pdf-render-cover-help",
            "PDF cover renderer script",
            &["scripts/render_cover.js", "--help"],
            skill_root,
        ));
    }
    checks
}

fn build_pptx_checks(
    skill_root: Option<&Path>,
    warnings: &mut Vec<String>,
) -> Vec<SkillRuntimeDependencyCheck> {
    let python_bin = preferred_python_bin();
    let mut checks = vec![
        any_binary_check("python-family", "Python", &["python", "python3", "py"]),
        binary_check("node", "node"),
    ];
    if let Some(python) = python_bin.as_deref() {
        checks.push(python_module_check(
            python,
            "markitdown",
            "Python module `markitdown`",
            &["markitdown"],
        ));
    }
    checks.push(node_require_check(
        "pptxgenjs",
        "Node module `pptxgenjs`",
        "pptxgenjs",
    ));
    let has_local_edit_scripts = skill_root
        .map(|root| script_exists(Some(root), &["skills", "ppt-editing-skill", "unpack.py"]))
        .unwrap_or(false);
    if !has_local_edit_scripts {
        warnings.push(
            "PPTX preinstalled skill currently vendors prompt assets but not the full XML helper scripts mentioned by MiniMax; template-edit flows still rely on user-provided tooling."
                .to_string(),
        );
    }
    checks
}

fn script_exists(skill_root: Option<&Path>, parts: &[&str]) -> bool {
    skill_root
        .map(|root| {
            parts
                .iter()
                .fold(root.to_path_buf(), |acc, part| acc.join(part))
                .exists()
        })
        .unwrap_or(false)
}

fn preferred_python_bin() -> Option<String> {
    ["python", "python3", "py"]
        .iter()
        .find_map(|candidate| resolve_command_path(candidate).map(|_| (*candidate).to_string()))
}

fn binary_check(key: &str, bin: &str) -> SkillRuntimeDependencyCheck {
    let path = resolve_command_path(bin);
    SkillRuntimeDependencyCheck {
        key: key.to_string(),
        label: format!("Executable `{bin}`"),
        satisfied: path.is_some(),
        detail: match path {
            Some(path) => format!("Found at {}.", path.to_string_lossy()),
            None => format!("`{bin}` was not found on PATH."),
        },
    }
}

fn any_binary_check(key: &str, label: &str, bins: &[&str]) -> SkillRuntimeDependencyCheck {
    let resolved = bins
        .iter()
        .find_map(|bin| resolve_command_path(bin).map(|path| (*bin, path)));
    SkillRuntimeDependencyCheck {
        key: key.to_string(),
        label: label.to_string(),
        satisfied: resolved.is_some(),
        detail: match resolved {
            Some((bin, path)) => format!("Using `{bin}` at {}.", path.to_string_lossy()),
            None => format!("None of these executables were found: {}.", bins.join(", ")),
        },
    }
}

fn python_module_check(
    python_bin: &str,
    key: &str,
    label: &str,
    modules: &[&str],
) -> SkillRuntimeDependencyCheck {
    let mut import_list = String::new();
    for module in modules {
        if !import_list.is_empty() {
            import_list.push_str(", ");
        }
        import_list.push_str(module);
    }
    let script = format!("import {import_list}; print('ok')");
    let satisfied = command_succeeds(python_bin, &["-c", &script]).is_some();
    SkillRuntimeDependencyCheck {
        key: key.to_string(),
        label: label.to_string(),
        satisfied,
        detail: if satisfied {
            format!("Verified with `{python_bin} -c`.")
        } else {
            format!(
                "One or more Python modules are missing: {}.",
                modules.join(", ")
            )
        },
    }
}

fn node_require_check(key: &str, label: &str, module_name: &str) -> SkillRuntimeDependencyCheck {
    let satisfied =
        command_succeeds("node", &["-e", &format!("require('{module_name}')")]).is_some();
    SkillRuntimeDependencyCheck {
        key: key.to_string(),
        label: label.to_string(),
        satisfied,
        detail: if satisfied {
            format!("Node can resolve `{module_name}`.")
        } else {
            format!("Node could not resolve `{module_name}`.")
        },
    }
}

fn node_script_check(
    key: &str,
    label: &str,
    args: &[&str],
    skill_root: Option<&Path>,
) -> SkillRuntimeDependencyCheck {
    let Some(root) = skill_root else {
        return SkillRuntimeDependencyCheck {
            key: key.to_string(),
            label: label.to_string(),
            satisfied: false,
            detail: "Skill root is unavailable, so the script could not be checked.".to_string(),
        };
    };
    let mut command = Command::new("node");
    command.args(args).current_dir(root);
    let output = command.output().ok();
    let satisfied = output
        .as_ref()
        .map(|out| {
            out.status.success()
                || !String::from_utf8_lossy(&out.stderr).contains("MODULE_NOT_FOUND")
        })
        .unwrap_or(false);
    SkillRuntimeDependencyCheck {
        key: key.to_string(),
        label: label.to_string(),
        satisfied,
        detail: if satisfied {
            format!(
                "`node {}` executed without a missing-module failure.",
                args.join(" ")
            )
        } else {
            format!("`node {}` could not start cleanly.", args.join(" "))
        },
    }
}

fn command_succeeds(bin: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(bin).args(args).output().ok()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stdout.is_empty() {
            Some(String::from_utf8_lossy(&output.stderr).trim().to_string())
        } else {
            Some(stdout)
        }
    } else {
        None
    }
}

fn resolve_command_path(command: &str) -> Option<PathBuf> {
    let candidate = Path::new(command);
    if candidate.components().count() > 1 {
        return candidate.exists().then(|| candidate.to_path_buf());
    }

    let path_var = std::env::var_os("PATH")?;
    let mut exts = BTreeSet::new();
    if cfg!(windows) {
        if let Some(pathext) = std::env::var_os("PATHEXT") {
            for ext in pathext.to_string_lossy().split(';') {
                let trimmed = ext.trim();
                if !trimmed.is_empty() {
                    exts.insert(trimmed.trim_start_matches('.').to_ascii_lowercase());
                }
            }
        }
        exts.insert(String::new());
    } else {
        exts.insert(String::new());
    }

    for dir in std::env::split_paths(&path_var) {
        let base = dir.join(command);
        if base.is_file() {
            return Some(base);
        }
        if cfg!(windows) {
            for ext in &exts {
                if ext.is_empty() {
                    continue;
                }
                let candidate = dir.join(format!("{command}.{}", ext));
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use skillpack_rs::SkillManifest;
    use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

    async fn setup_memory_pool() -> SqlitePool {
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

    async fn insert_skill(
        pool: &SqlitePool,
        id: &str,
        name: &str,
        pack_path: &str,
        source_type: &str,
    ) {
        let manifest = SkillManifest {
            id: id.to_string(),
            name: name.to_string(),
            description: format!("{name} description"),
            version: "builtin".to_string(),
            author: "WorkClaw".to_string(),
            recommended_model: String::new(),
            tags: Vec::new(),
            created_at: Utc::now(),
            username_hint: None,
            encrypted_verify: String::new(),
        };
        let manifest_json = serde_json::to_string(&manifest).expect("serialize manifest");
        sqlx::query(
            "INSERT INTO installed_skills (id, manifest, installed_at, username, pack_path, source_type)
             VALUES (?, ?, ?, '', ?, ?)",
        )
        .bind(id)
        .bind(manifest_json)
        .bind(Utc::now().to_rfc3339())
        .bind(pack_path)
        .bind(source_type)
        .execute(pool)
        .await
        .expect("insert skill");
    }

    #[tokio::test]
    async fn vendored_docx_reports_missing_dotnet_sdk_when_not_installed() {
        let pool = setup_memory_pool().await;
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("builtin-skills").join("docx");
        insert_skill(
            &pool,
            "builtin-docx",
            "DOCX",
            &root.to_string_lossy(),
            "vendored",
        )
        .await;

        let status = get_skill_runtime_environment_status_with_pool(&pool, "builtin-docx")
            .await
            .expect("status");

        assert_eq!(status.skill_id, "builtin-docx");
        assert!(status.checks.iter().any(|check| check.key == "dotnet-sdk"));
    }

    #[tokio::test]
    async fn vendored_pptx_includes_vendor_warning_for_missing_helper_scripts() {
        let pool = setup_memory_pool().await;
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("builtin-skills").join("pptx");
        insert_skill(
            &pool,
            "builtin-pptx",
            "PPTX",
            &root.to_string_lossy(),
            "vendored",
        )
        .await;

        let status = get_skill_runtime_environment_status_with_pool(&pool, "builtin-pptx")
            .await
            .expect("status");

        assert!(status
            .warnings
            .iter()
            .any(|warning| warning.contains("helper scripts")));
    }

    #[test]
    fn resolve_command_path_accepts_known_windows_commands_when_available() {
        if let Some(path) = resolve_command_path("node") {
            assert!(path.exists());
        }
    }
}
