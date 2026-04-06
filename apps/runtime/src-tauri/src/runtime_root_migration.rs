use crate::runtime_bootstrap::{
    write_runtime_root_bootstrap_pending_migration, BootstrapMigrationStatus,
    RuntimeBootstrapError, RuntimeRootBootstrap, RuntimeRootBootstrapMigration,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime_bootstrap::{
        default_runtime_root_bootstrap, read_runtime_root_bootstrap, write_runtime_root_bootstrap,
        BootstrapMigrationStatus, RuntimeRootBootstrap, RuntimeRootBootstrapMigration,
    };
    use std::fs;
    use std::path::PathBuf;

    fn make_bootstrap_path() -> (tempfile::TempDir, PathBuf) {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let bootstrap_path = temp_dir.path().join("bootstrap-root.json");
        (temp_dir, bootstrap_path)
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
}
