use crate::runtime_bootstrap::{
    write_runtime_root_bootstrap_pending_migration, BootstrapMigrationStatus,
    RuntimeBootstrapError, RuntimeRootBootstrap, RuntimeRootBootstrapMigration,
    RuntimeRootBootstrapMigrationResult,
};
use crate::runtime_paths::{self, RuntimePathValidationError};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub enum RuntimeRootMigrationError {
    EmptyTargetRoot,
    TargetRootNotWritable {
        target_root: PathBuf,
        reason: String,
    },
    NestedTarget(RuntimePathValidationError),
    Bootstrap(RuntimeBootstrapError),
    PendingMigrationAlreadyScheduled {
        target_root: PathBuf,
    },
    NoPendingMigration,
    MigrationExecutionFailed {
        path: PathBuf,
        reason: String,
    },
}

impl fmt::Display for RuntimeRootMigrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTargetRoot => write!(f, "migration target root cannot be empty"),
            Self::TargetRootNotWritable {
                target_root,
                reason,
            } => {
                write!(
                    f,
                    "migration target root is not writable: {} ({reason})",
                    target_root.display()
                )
            }
            Self::NestedTarget(error) => write!(f, "{error}"),
            Self::Bootstrap(error) => write!(f, "{error}"),
            Self::PendingMigrationAlreadyScheduled { target_root } => write!(
                f,
                "migration is already pending for bootstrap target root {}",
                target_root.display()
            ),
            Self::NoPendingMigration => write!(f, "no pending runtime root migration is scheduled"),
            Self::MigrationExecutionFailed { path, reason } => {
                write!(f, "failed to migrate managed path {}: {reason}", path.display())
            }
        }
    }
}

impl std::error::Error for RuntimeRootMigrationError {}

impl From<RuntimeBootstrapError> for RuntimeRootMigrationError {
    fn from(value: RuntimeBootstrapError) -> Self {
        Self::Bootstrap(value)
    }
}

impl From<RuntimePathValidationError> for RuntimeRootMigrationError {
    fn from(value: RuntimePathValidationError) -> Self {
        Self::NestedTarget(value)
    }
}

fn migration_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:09}Z", now.as_secs(), now.subsec_nanos())
}

fn probe_directory_writable(directory: &Path) -> Result<(), RuntimeRootMigrationError> {
    let probe_name = format!(
        ".workclaw-root-migration-probe-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let probe_path = directory.join(probe_name);
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe_path)
    {
        Ok(file) => {
            drop(file);
            let _ = fs::remove_file(&probe_path);
            Ok(())
        }
        Err(error) => Err(RuntimeRootMigrationError::TargetRootNotWritable {
            target_root: directory.to_path_buf(),
            reason: error.to_string(),
        }),
    }
}

fn validate_target_root_writable(target_root: &Path) -> Result<(), RuntimeRootMigrationError> {
    let writable_directory = if target_root.exists() {
        if !target_root.is_dir() {
            return Err(RuntimeRootMigrationError::TargetRootNotWritable {
                target_root: target_root.to_path_buf(),
                reason: "target root already exists as a file".to_string(),
            });
        }
        target_root
    } else {
        target_root
            .parent()
            .ok_or_else(|| RuntimeRootMigrationError::TargetRootNotWritable {
                target_root: target_root.to_path_buf(),
                reason: "target root has no writable parent directory".to_string(),
            })?
    };

    if !writable_directory.exists() {
        return Err(RuntimeRootMigrationError::TargetRootNotWritable {
            target_root: target_root.to_path_buf(),
            reason: "target root parent directory does not exist".to_string(),
        });
    }

    probe_directory_writable(writable_directory)
}

fn build_runtime_paths(root: &str) -> runtime_paths::RuntimePaths {
    runtime_paths::RuntimePaths::new(PathBuf::from(root))
}

fn managed_runtime_paths(
    source_root: &str,
    target_root: &str,
) -> Vec<(PathBuf, PathBuf, bool)> {
    let source = build_runtime_paths(source_root);
    let target = build_runtime_paths(target_root);
    vec![
        (source.database.db_path, target.database.db_path, false),
        (source.database.wal_path, target.database.wal_path, false),
        (source.database.shm_path, target.database.shm_path, false),
        (source.diagnostics.root, target.diagnostics.root, true),
        (source.cache_dir, target.cache_dir, true),
        (source.sessions_dir, target.sessions_dir, true),
        (source.plugins.root, target.plugins.root, true),
        (source.plugins.cli_shim_dir, target.plugins.cli_shim_dir, true),
        (source.plugins.state_dir, target.plugins.state_dir, true),
        (
            source.plugins.skills_vendor_dir,
            target.plugins.skills_vendor_dir,
            true,
        ),
        (source.workspace_dir, target.workspace_dir, true),
    ]
}

fn copy_managed_path(source: &Path, target: &Path, is_directory: bool) -> Result<(), RuntimeRootMigrationError> {
    if !source.exists() {
        return Ok(());
    }

    if is_directory {
        if target.exists() && !target.is_dir() {
            return Err(RuntimeRootMigrationError::MigrationExecutionFailed {
                path: target.to_path_buf(),
                reason: "target path already exists as a file".to_string(),
            });
        }
        fs::create_dir_all(target).map_err(|error| RuntimeRootMigrationError::MigrationExecutionFailed {
            path: target.to_path_buf(),
            reason: error.to_string(),
        })?;

        for entry in fs::read_dir(source).map_err(|error| RuntimeRootMigrationError::MigrationExecutionFailed {
            path: source.to_path_buf(),
            reason: error.to_string(),
        })? {
            let entry = entry.map_err(|error| RuntimeRootMigrationError::MigrationExecutionFailed {
                path: source.to_path_buf(),
                reason: error.to_string(),
            })?;
            let child_source = entry.path();
            let child_target = target.join(entry.file_name());
            let child_is_directory = entry
                .file_type()
                .map_err(|error| RuntimeRootMigrationError::MigrationExecutionFailed {
                    path: child_source.clone(),
                    reason: error.to_string(),
                })?
                .is_dir();
            copy_managed_path(&child_source, &child_target, child_is_directory)?;
        }
        return Ok(());
    }

    if target.exists() && target.is_dir() {
        return Err(RuntimeRootMigrationError::MigrationExecutionFailed {
            path: target.to_path_buf(),
            reason: "target path already exists as a directory".to_string(),
        });
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|error| RuntimeRootMigrationError::MigrationExecutionFailed {
            path: parent.to_path_buf(),
            reason: error.to_string(),
        })?;
    }

    fs::copy(source, target).map_err(|error| RuntimeRootMigrationError::MigrationExecutionFailed {
        path: source.to_path_buf(),
        reason: error.to_string(),
    })?;
    Ok(())
}

fn remove_managed_path(path: &Path, is_directory: bool) {
    if !path.exists() {
        return;
    }

    if is_directory {
        let _ = fs::remove_dir_all(path);
    } else {
        let _ = fs::remove_file(path);
    }
}

fn remove_managed_runtime_paths(paths: &[(PathBuf, PathBuf, bool)], target_side: bool) {
    for (source, target, is_directory) in paths.iter().rev() {
        let path = if target_side { target } else { source };
        remove_managed_path(path, *is_directory);
    }
}

fn mark_migration_in_progress(bootstrap: &mut RuntimeRootBootstrap) {
    if let Some(pending) = bootstrap.pending_migration.as_mut() {
        pending.status = BootstrapMigrationStatus::InProgress;
        pending.last_error = None;
    }
}

fn record_migration_completion(
    bootstrap: &mut RuntimeRootBootstrap,
    from_root: &str,
    to_root: &str,
) {
    bootstrap.current_root = to_root.to_string();
    bootstrap.previous_root = Some(from_root.to_string());
    bootstrap.pending_migration = None;
    bootstrap.last_migration_result = Some(RuntimeRootBootstrapMigrationResult {
        from_root: from_root.to_string(),
        to_root: to_root.to_string(),
        status: BootstrapMigrationStatus::Completed,
        completed_at: migration_timestamp(),
        message: Some("runtime root migration completed".to_string()),
    });
}

fn record_migration_rollback(
    bootstrap: &mut RuntimeRootBootstrap,
    from_root: &str,
    to_root: &str,
    reason: &str,
) {
    bootstrap.current_root = from_root.to_string();
    bootstrap.previous_root = None;
    bootstrap.pending_migration = None;
    bootstrap.last_migration_result = Some(RuntimeRootBootstrapMigrationResult {
        from_root: from_root.to_string(),
        to_root: to_root.to_string(),
        status: BootstrapMigrationStatus::RolledBack,
        completed_at: migration_timestamp(),
        message: Some(reason.to_string()),
    });
}

pub fn schedule_runtime_root_migration(
    bootstrap_path: &Path,
    target_root: &Path,
) -> Result<RuntimeRootBootstrap, RuntimeRootMigrationError> {
    if target_root.as_os_str().is_empty() {
        return Err(RuntimeRootMigrationError::EmptyTargetRoot);
    }

    let default_root = runtime_paths::resolve_runtime_root();
    let mut bootstrap = crate::runtime_bootstrap::discover_runtime_root_bootstrap(
        bootstrap_path,
        None,
        &default_root,
    )?;
    if bootstrap.pending_migration.is_some() {
        return Err(
            RuntimeRootMigrationError::PendingMigrationAlreadyScheduled {
                target_root: target_root.to_path_buf(),
            },
        );
    }

    let current_root = PathBuf::from(&bootstrap.current_root);
    runtime_paths::validate_migration_target(&current_root, target_root)?;
    validate_target_root_writable(target_root)?;

    let pending_migration = RuntimeRootBootstrapMigration {
        from_root: bootstrap.current_root.clone(),
        to_root: target_root.to_string_lossy().to_string(),
        status: BootstrapMigrationStatus::Pending,
        created_at: migration_timestamp(),
        last_error: None,
    };

    write_runtime_root_bootstrap_pending_migration(
        bootstrap_path,
        &mut bootstrap,
        pending_migration,
    )?;

    Ok(bootstrap)
}

pub fn execute_runtime_root_migration(
    bootstrap_path: &Path,
) -> Result<RuntimeRootBootstrap, RuntimeRootMigrationError> {
    let mut bootstrap = crate::runtime_bootstrap::read_runtime_root_bootstrap(bootstrap_path)?;
    let pending = bootstrap
        .pending_migration
        .clone()
        .ok_or(RuntimeRootMigrationError::NoPendingMigration)?;

    let from_root = pending.from_root.clone();
    let target_root = pending.to_root.clone();
    let current_root = PathBuf::from(&from_root);
    let target_root_path = PathBuf::from(&target_root);

    runtime_paths::validate_migration_target(&current_root, &target_root_path)?;
    validate_target_root_writable(&target_root_path)?;

    mark_migration_in_progress(&mut bootstrap);
    crate::runtime_bootstrap::write_runtime_root_bootstrap(bootstrap_path, &bootstrap)?;

    let managed_paths = managed_runtime_paths(&from_root, &target_root);
    let copy_result: Result<(), RuntimeRootMigrationError> = (|| {
        for (source, target, is_directory) in &managed_paths {
            if source.exists() {
                copy_managed_path(source, target, *is_directory)?;
            }
        }
        Ok(())
    })();

    if let Err(error) = copy_result {
        remove_managed_runtime_paths(&managed_paths, true);
        record_migration_rollback(&mut bootstrap, &from_root, &target_root, &error.to_string());
        crate::runtime_bootstrap::write_runtime_root_bootstrap(bootstrap_path, &bootstrap)?;
        return Err(error);
    }

    record_migration_completion(&mut bootstrap, &from_root, &target_root);
    crate::runtime_bootstrap::write_runtime_root_bootstrap(bootstrap_path, &bootstrap)?;

    remove_managed_runtime_paths(&managed_paths, false);

    Ok(bootstrap)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime_bootstrap::{
        default_runtime_root_bootstrap, read_runtime_root_bootstrap, write_runtime_root_bootstrap,
        BootstrapMigrationStatus, RuntimeRootBootstrap, RuntimeRootBootstrapMigration,
    };
    use crate::runtime_paths::RuntimePaths;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn make_bootstrap_path() -> (tempfile::TempDir, PathBuf) {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let bootstrap_path = temp_dir.path().join("bootstrap-root.json");
        (temp_dir, bootstrap_path)
    }

    fn seed_runtime_tree(root: &Path) {
        let paths = RuntimePaths::new(root.to_path_buf());

        fs::create_dir_all(&paths.root).expect("create root");
        fs::write(paths.database.db_path, "db").expect("write db");
        fs::write(paths.database.wal_path, "wal").expect("write wal");
        fs::write(paths.database.shm_path, "shm").expect("write shm");

        fs::create_dir_all(&paths.diagnostics.logs_dir).expect("create diagnostics logs");
        fs::write(
            paths.diagnostics.logs_dir.join("runtime-2026-04-06.jsonl"),
            "log",
        )
        .expect("write diagnostics log");
        fs::create_dir_all(&paths.diagnostics.audit_dir).expect("create diagnostics audit");
        fs::write(
            paths.diagnostics.audit_dir.join("audit-2026-04-06.jsonl"),
            "audit",
        )
        .expect("write audit log");

        fs::create_dir_all(&paths.cache_dir).expect("create cache");
        fs::write(paths.cache_dir.join("runtime-cache.bin"), "cache").expect("write cache");

        fs::create_dir_all(&paths.sessions_dir).expect("create sessions");
        fs::create_dir_all(paths.sessions_dir.join("session-1")).expect("create session dir");
        fs::write(
            paths.sessions_dir.join("session-1").join("journal.json"),
            "journal",
        )
        .expect("write session journal");

        fs::create_dir_all(&paths.plugins.root).expect("create plugins root");
        fs::create_dir_all(paths.plugins.root.join("plugin-a")).expect("create plugin dir");
        fs::write(
            paths.plugins.root.join("plugin-a").join("manifest.json"),
            "plugin",
        )
        .expect("write plugin manifest");
        fs::create_dir_all(&paths.plugins.state_dir).expect("create plugin state");
        fs::write(paths.plugins.state_dir.join("registry.json"), "state")
            .expect("write plugin state");
        fs::create_dir_all(&paths.plugins.cli_shim_dir).expect("create plugin cli shim");
        fs::write(paths.plugins.cli_shim_dir.join("shim.json"), "shim")
            .expect("write cli shim");
        fs::create_dir_all(&paths.plugins.skills_vendor_dir).expect("create skills vendor");
        fs::create_dir_all(paths.plugins.skills_vendor_dir.join("skill-a"))
            .expect("create vendored skill dir");
        fs::write(
            paths.plugins.skills_vendor_dir.join("skill-a").join("SKILL.md"),
            "skill",
        )
        .expect("write vendored skill");

        fs::create_dir_all(&paths.workspace_dir).expect("create workspace");
        fs::write(paths.workspace_dir.join("notes.txt"), "notes").expect("write workspace file");
    }

    fn assert_runtime_tree_exists(root: &Path) {
        let paths = RuntimePaths::new(root.to_path_buf());

        assert!(paths.database.db_path.exists());
        assert!(paths.database.wal_path.exists());
        assert!(paths.database.shm_path.exists());
        assert!(paths.diagnostics.logs_dir.join("runtime-2026-04-06.jsonl").exists());
        assert!(paths.diagnostics.audit_dir.join("audit-2026-04-06.jsonl").exists());
        assert!(paths.cache_dir.join("runtime-cache.bin").exists());
        assert!(paths
            .sessions_dir
            .join("session-1")
            .join("journal.json")
            .exists());
        assert!(paths
            .plugins
            .root
            .join("plugin-a")
            .join("manifest.json")
            .exists());
        assert!(paths.plugins.state_dir.join("registry.json").exists());
        assert!(paths.plugins.cli_shim_dir.join("shim.json").exists());
        assert!(paths
            .plugins
            .skills_vendor_dir
            .join("skill-a")
            .join("SKILL.md")
            .exists());
        assert!(paths.workspace_dir.join("notes.txt").exists());
    }

    #[test]
    fn schedule_migration_records_pending_migration() {
        let (_temp_dir, bootstrap_path) = make_bootstrap_path();
        let current_root = PathBuf::from(r"D:\WorkClawData");
        let target_root = PathBuf::from(r"E:\WorkClawData");
        let bootstrap = default_runtime_root_bootstrap(&current_root);
        write_runtime_root_bootstrap(&bootstrap_path, &bootstrap).expect("seed bootstrap");

        let scheduled = schedule_runtime_root_migration(&bootstrap_path, &target_root)
            .expect("schedule migration");

        assert_eq!(scheduled.current_root, current_root.to_string_lossy());
        let pending = scheduled.pending_migration.expect("pending migration");
        assert_eq!(pending.from_root, current_root.to_string_lossy());
        assert_eq!(pending.to_root, target_root.to_string_lossy());
        assert_eq!(pending.status, BootstrapMigrationStatus::Pending);

        let persisted = read_runtime_root_bootstrap(&bootstrap_path).expect("read bootstrap");
        assert_eq!(persisted.pending_migration, Some(pending));
    }

    #[test]
    fn schedule_migration_recovers_from_malformed_bootstrap() {
        let (temp_dir, bootstrap_path) = make_bootstrap_path();
        std::fs::write(&bootstrap_path, "{ this is not valid json")
            .expect("write malformed bootstrap");
        let target_root = temp_dir.path().join("scheduled-target");
        let expected_current_root = runtime_paths::resolve_runtime_root();

        let scheduled = schedule_runtime_root_migration(&bootstrap_path, &target_root)
            .expect("schedule migration");

        assert_eq!(
            scheduled.current_root,
            expected_current_root.to_string_lossy()
        );
        assert!(scheduled.pending_migration.is_some());

        let persisted = read_runtime_root_bootstrap(&bootstrap_path).expect("read bootstrap");
        assert!(persisted.pending_migration.is_some());
    }

    #[test]
    fn schedule_migration_rejects_empty_target_root() {
        let (_temp_dir, bootstrap_path) = make_bootstrap_path();
        let current_root = PathBuf::from(r"D:\WorkClawData");
        let bootstrap = default_runtime_root_bootstrap(&current_root);
        write_runtime_root_bootstrap(&bootstrap_path, &bootstrap).expect("seed bootstrap");

        let result = schedule_runtime_root_migration(&bootstrap_path, &PathBuf::new());

        assert!(matches!(
            result,
            Err(RuntimeRootMigrationError::EmptyTargetRoot)
        ));
    }

    #[test]
    fn schedule_migration_rejects_non_writable_target_root() {
        let (_temp_dir, bootstrap_path) = make_bootstrap_path();
        let current_root = PathBuf::from(r"D:\WorkClawData");
        let bootstrap = default_runtime_root_bootstrap(&current_root);
        write_runtime_root_bootstrap(&bootstrap_path, &bootstrap).expect("seed bootstrap");

        let non_writable_target = bootstrap_path.with_file_name("target-root.txt");
        fs::write(&non_writable_target, "locked").expect("seed file target");

        let result = schedule_runtime_root_migration(&bootstrap_path, &non_writable_target);

        assert!(matches!(
            result,
            Err(RuntimeRootMigrationError::TargetRootNotWritable { .. })
        ));
    }

    #[test]
    fn schedule_migration_rejects_nested_target_roots() {
        let (_temp_dir, bootstrap_path) = make_bootstrap_path();
        let current_root = PathBuf::from(r"D:\WorkClawData");
        let bootstrap = default_runtime_root_bootstrap(&current_root);
        write_runtime_root_bootstrap(&bootstrap_path, &bootstrap).expect("seed bootstrap");

        let nested_target = current_root.join("child");
        let result = schedule_runtime_root_migration(&bootstrap_path, &nested_target);

        assert!(matches!(
            result,
            Err(RuntimeRootMigrationError::NestedTarget(_))
        ));
    }

    #[test]
    fn schedule_migration_rejects_second_pending_schedule() {
        let (_temp_dir, bootstrap_path) = make_bootstrap_path();
        let current_root = PathBuf::from(r"D:\WorkClawData");
        let target_root = PathBuf::from(r"E:\WorkClawData");
        let bootstrap = RuntimeRootBootstrap {
            pending_migration: Some(RuntimeRootBootstrapMigration {
                from_root: current_root.to_string_lossy().to_string(),
                to_root: r"F:\WorkClawData".to_string(),
                status: BootstrapMigrationStatus::Pending,
                created_at: "2026-04-06T10:00:00Z".to_string(),
                last_error: None,
            }),
            ..default_runtime_root_bootstrap(&current_root)
        };
        write_runtime_root_bootstrap(&bootstrap_path, &bootstrap).expect("seed bootstrap");

        let result = schedule_runtime_root_migration(&bootstrap_path, &target_root);

        assert!(matches!(
            result,
            Err(RuntimeRootMigrationError::PendingMigrationAlreadyScheduled { .. })
        ));
    }

    #[test]
    fn execute_migration_moves_managed_runtime_paths_and_records_completion_metadata() {
        let (temp_dir, bootstrap_path) = make_bootstrap_path();
        let current_root = temp_dir.path().join("old-root");
        let target_root = temp_dir.path().join("new-root");
        fs::create_dir_all(&current_root).expect("create old root");
        fs::create_dir_all(target_root.parent().expect("target parent")).expect("create target parent");
        seed_runtime_tree(&current_root);

        let bootstrap = default_runtime_root_bootstrap(&current_root);
        write_runtime_root_bootstrap(&bootstrap_path, &bootstrap).expect("seed bootstrap");
        schedule_runtime_root_migration(&bootstrap_path, &target_root).expect("schedule migration");

        let completed = execute_runtime_root_migration(&bootstrap_path).expect("execute migration");

        let persisted = read_runtime_root_bootstrap(&bootstrap_path).expect("read bootstrap");
        let current_root_text = current_root.to_string_lossy().to_string();
        let target_root_text = target_root.to_string_lossy().to_string();
        assert_eq!(completed.current_root, target_root.to_string_lossy());
        assert_eq!(persisted.current_root, target_root.to_string_lossy());
        assert_eq!(persisted.previous_root.as_deref(), Some(current_root_text.as_str()));
        assert!(persisted.pending_migration.is_none());
        let result = persisted
            .last_migration_result
            .expect("completion metadata");
        assert_eq!(result.from_root, current_root_text);
        assert_eq!(result.to_root, target_root_text);
        assert_eq!(result.status, BootstrapMigrationStatus::Completed);
        assert!(!result.completed_at.is_empty());

        assert_runtime_tree_exists(&target_root);
        assert!(!RuntimePaths::new(current_root.clone()).database.db_path.exists());
        assert!(!RuntimePaths::new(current_root.clone()).cache_dir.join("runtime-cache.bin").exists());
        assert!(!RuntimePaths::new(current_root.clone())
            .sessions_dir
            .join("session-1")
            .join("journal.json")
            .exists());
        assert!(!RuntimePaths::new(current_root.clone())
            .workspace_dir
            .join("notes.txt")
            .exists());
    }

    #[test]
    fn execute_migration_restores_bootstrap_after_partial_copy_failure() {
        let (temp_dir, bootstrap_path) = make_bootstrap_path();
        let current_root = temp_dir.path().join("old-root");
        let target_root = temp_dir.path().join("new-root");
        fs::create_dir_all(&current_root).expect("create old root");
        fs::create_dir_all(target_root.parent().expect("target parent")).expect("create target parent");
        seed_runtime_tree(&current_root);

        let bootstrap = default_runtime_root_bootstrap(&current_root);
        write_runtime_root_bootstrap(&bootstrap_path, &bootstrap).expect("seed bootstrap");
        schedule_runtime_root_migration(&bootstrap_path, &target_root).expect("schedule migration");

        let target_workspace_file = RuntimePaths::new(target_root.clone()).workspace_dir;
        fs::create_dir_all(target_workspace_file.parent().expect("workspace parent"))
            .expect("create workspace parent");
        fs::write(&target_workspace_file, "conflict").expect("seed conflicting target file");

        let result = execute_runtime_root_migration(&bootstrap_path);

        assert!(result.is_err());

        let persisted = read_runtime_root_bootstrap(&bootstrap_path).expect("read bootstrap");
        let current_root_text = current_root.to_string_lossy().to_string();
        let target_root_text = target_root.to_string_lossy().to_string();
        assert_eq!(persisted.current_root, current_root.to_string_lossy());
        assert!(persisted.previous_root.is_none());
        assert!(persisted.pending_migration.is_none());
        let result = persisted
            .last_migration_result
            .expect("failure metadata");
        assert_eq!(result.from_root, current_root_text);
        assert_eq!(result.to_root, target_root_text);
        assert_eq!(result.status, BootstrapMigrationStatus::RolledBack);
        assert!(!result.completed_at.is_empty());

        assert_runtime_tree_exists(&current_root);
        assert!(!RuntimePaths::new(target_root.clone())
            .database
            .db_path
            .exists());
        assert!(!RuntimePaths::new(target_root.clone())
            .cache_dir
            .join("runtime-cache.bin")
            .exists());
        assert!(!RuntimePaths::new(target_root.clone())
            .sessions_dir
            .join("session-1")
            .join("journal.json")
            .exists());
        assert!(!RuntimePaths::new(target_root.clone())
            .workspace_dir
            .join("notes.txt")
            .exists());
    }
}
