// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::error::{AppError, ErrorCode};
use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::{OnceLock, RwLock},
};

const LEGACY_HOME_CANDIDATES: [&str; 3] = [".AOXCData/home/default", ".AOXCData", ".AOXC"];
const MAIN_LAYOUT_DIRS: [&str; 14] = [
    "config",
    "identity",
    "keys",
    "wallets",
    "ledger",
    "runtime",
    "runtime/db",
    "runtime/bin",
    "runtime/logs",
    "runtime/snapshots",
    "telemetry",
    "reports",
    "support",
    "backups",
];
const NETWORKS: [&str; 3] = ["mainnet", "testnet", "devnet"];
const NETWORK_LAYOUT_DIRS: [&str; 5] = ["config", "db", "ledger", "logs", "wallets"];

/// Process-local AOXC home override registry.
///
/// Design rationale:
/// - Rust 2024 treats process environment mutation as unsafe because it is
///   shared mutable process-wide state.
/// - AOXC CLI still requires command-scoped home redirection for bootstrap
///   and test isolation.
/// - A guarded in-process override preserves deterministic path resolution
///   without mutating the operating-system environment.
///
/// Concurrency model:
/// - Reads are lock-protected and clone the effective path before use.
/// - Writes are serialized through the same lock.
/// - The override is intended for short-lived command-scoped use.
fn home_override_registry() -> &'static RwLock<Option<PathBuf>> {
    static REGISTRY: OnceLock<RwLock<Option<PathBuf>>> = OnceLock::new();
    REGISTRY.get_or_init(|| RwLock::new(None))
}

/// Process-scoped guard that installs a temporary AOXC home override and
/// restores the previous override state when dropped.
pub struct ScopedHomeOverride {
    previous_home: Option<PathBuf>,
}

impl ScopedHomeOverride {
    /// Installs a scoped AOXC home override for the current process.
    ///
    /// Behavioral contract:
    /// - Captures the previously active in-process override.
    /// - Does not mutate the operating-system environment.
    /// - Restores the prior override on drop.
    pub fn install(home: &Path) -> Self {
        let registry = home_override_registry();
        let mut guard = registry
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let previous_home = guard.clone();
        *guard = Some(home.to_path_buf());

        Self { previous_home }
    }
}

impl Drop for ScopedHomeOverride {
    fn drop(&mut self) {
        let registry = home_override_registry();
        let mut guard = registry
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        *guard = self.previous_home.take();
    }
}

/// Returns the active process-local AOXC home override, when present.
pub fn active_home_override() -> Option<PathBuf> {
    home_override_registry()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

/// Returns the canonical AOXC data root for the current user.
///
/// Canonical data root policy:
/// - Linux/macOS style environments resolve to `$HOME/.aoxc`.
///
/// This directory is the top-level AOXC-owned namespace and also the default
/// AOXC home unless explicitly overridden.
pub fn default_data_root() -> Result<PathBuf, AppError> {
    let home = env::var("HOME").map(PathBuf::from).map_err(|_| {
        AppError::new(
            ErrorCode::HomeResolutionFailed,
            "HOME environment variable is not set",
        )
    })?;

    Ok(home.join(".aoxc"))
}

/// Returns the canonical default AOXC home directory.
///
/// Canonical home policy:
/// - `$HOME/.aoxc`
pub fn default_home_dir() -> Result<PathBuf, AppError> {
    default_data_root()
}

/// Resolves the effective AOXC operator home.
///
/// Resolution order:
/// 1. in-process scoped override when present
/// 2. `AOXC_HOME` when present and non-empty
/// 3. canonical default returned by `default_home_dir()`
pub fn resolve_home() -> Result<PathBuf, AppError> {
    if let Some(home) = active_home_override() {
        return Ok(home);
    }

    match env::var("AOXC_HOME") {
        Ok(value) if !value.trim().is_empty() => Ok(PathBuf::from(value)),
        _ => default_home_dir(),
    }
}

/// Ensures the canonical AOXC directory layout exists under the supplied home.
///
/// Layout policy under the effective AOXC home:
/// - `config/`
/// - `identity/`
/// - `keys/`
/// - `wallets/`
/// - `ledger/`
/// - `runtime/{db,bin,logs,snapshots}`
/// - `networks/{mainnet,testnet,devnet}/{config,db,ledger,logs,wallets}`
/// - `telemetry/`, `reports/`, `support/`, `backups/`
///
/// Compatibility and safety policy:
/// - Migrates legacy `$HOME/.AOXCData/home/default`, `$HOME/.AOXCData`,
///   and `$HOME/.AOXC` content into the canonical default home on first
///   initialization.
/// - Writes a deletion guard marker in `support/delete-protection.md`.
pub fn ensure_layout(home: &Path) -> Result<(), AppError> {
    maybe_migrate_legacy_home(home)?;

    for relative in MAIN_LAYOUT_DIRS {
        let dir = home.join(relative);
        fs::create_dir_all(&dir).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to create directory {}", dir.display()),
                error,
            )
        })?;
    }

    for network in NETWORKS {
        for relative in NETWORK_LAYOUT_DIRS {
            let dir = home.join("networks").join(network).join(relative);
            fs::create_dir_all(&dir).map_err(|error| {
                AppError::with_source(
                    ErrorCode::FilesystemIoFailed,
                    format!("Failed to create directory {}", dir.display()),
                    error,
                )
            })?;
        }
    }

    let deletion_protection = home.join("support").join("delete-protection.md");
    if !deletion_protection.exists() {
        write_file(
            &deletion_protection,
            "AOXC deletion protection guard.\n\nDo not remove this home root recursively without verified backup snapshots and key export validation.\n",
        )?;
    }

    harden_directory_permissions(&home.join("keys"))?;
    harden_directory_permissions(&home.join("wallets"))?;

    Ok(())
}

/// Writes a UTF-8 AOXC file and hardens permissions where supported.
pub fn write_file(path: &Path, content: &str) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to create parent directory {}", parent.display()),
                error,
            )
        })?;
    }

    fs::write(path, content).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to write file {}", path.display()),
            error,
        )
    })?;

    harden_file_permissions(path)?;
    Ok(())
}

/// Reads a UTF-8 AOXC file from disk.
pub fn read_file(path: &Path) -> Result<String, AppError> {
    fs::read_to_string(path).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to read file {}", path.display()),
            error,
        )
    })
}

/// Returns whether the file permissions are hardened for sensitive AOXC data.
pub fn file_permissions_are_hardened(path: &Path) -> Result<bool, AppError> {
    let metadata = fs::metadata(path).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to read metadata for {}", path.display()),
            error,
        )
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mode = metadata.permissions().mode() & 0o777;
        Ok(mode & 0o077 == 0)
    }

    #[cfg(not(unix))]
    {
        let _ = metadata;
        Ok(true)
    }
}

fn maybe_migrate_legacy_home(target_home: &Path) -> Result<(), AppError> {
    let canonical_home = default_home_dir()?;
    if target_home != canonical_home {
        return Ok(());
    }

    if target_home.exists() {
        return Ok(());
    }

    let user_home = env::var("HOME").map(PathBuf::from).map_err(|_| {
        AppError::new(
            ErrorCode::HomeResolutionFailed,
            "HOME environment variable is not set",
        )
    })?;

    for candidate in LEGACY_HOME_CANDIDATES {
        let legacy = user_home.join(candidate);
        if legacy.is_dir() {
            copy_directory_tree(&legacy, target_home)?;
            break;
        }
    }

    Ok(())
}

fn copy_directory_tree(source: &Path, destination: &Path) -> Result<(), AppError> {
    fs::create_dir_all(destination).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to create directory {}", destination.display()),
            error,
        )
    })?;

    for entry in fs::read_dir(source).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to read directory {}", source.display()),
            error,
        )
    })? {
        let entry = entry.map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to read entry in {}", source.display()),
                error,
            )
        })?;

        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type = entry.file_type().map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to inspect {}", source_path.display()),
                error,
            )
        })?;

        if file_type.is_dir() {
            copy_directory_tree(&source_path, &destination_path)?;
            continue;
        }

        if file_type.is_file() && !destination_path.exists() {
            fs::copy(&source_path, &destination_path).map_err(|error| {
                AppError::with_source(
                    ErrorCode::FilesystemIoFailed,
                    format!(
                        "Failed to copy {} to {}",
                        source_path.display(),
                        destination_path.display()
                    ),
                    error,
                )
            })?;
        }
    }

    Ok(())
}

/// Hardens file permissions for sensitive AOXC artifacts.
fn harden_file_permissions(path: &Path) -> Result<(), AppError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to harden permissions on {}", path.display()),
                error,
            )
        })?;
    }

    Ok(())
}

fn harden_directory_permissions(path: &Path) -> Result<(), AppError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        if path.exists() {
            fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|error| {
                AppError::with_source(
                    ErrorCode::FilesystemIoFailed,
                    format!(
                        "Failed to harden directory permissions on {}",
                        path.display()
                    ),
                    error,
                )
            })?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        ScopedHomeOverride, active_home_override, default_data_root, default_home_dir,
        ensure_layout, file_permissions_are_hardened, read_file, resolve_home, write_file,
    };
    use std::env;

    #[test]
    fn default_data_root_resolves_to_hidden_aoxc_root() {
        let home = env::var("HOME").expect("HOME must be set for tests");
        assert_eq!(
            default_data_root().expect("data root should resolve"),
            std::path::PathBuf::from(home).join(".aoxc")
        );
    }

    #[test]
    fn default_home_dir_resolves_beneath_canonical_data_root() {
        let home = env::var("HOME").expect("HOME must be set for tests");
        assert_eq!(
            default_home_dir().expect("default home should resolve"),
            std::path::PathBuf::from(home).join(".aoxc")
        );
    }

    #[test]
    fn scoped_home_override_redirects_resolution_without_mutating_environment() {
        let target = std::env::temp_dir().join("aoxc-scoped-home-resolution");
        let previous = active_home_override();

        {
            let _guard = ScopedHomeOverride::install(&target);
            assert_eq!(resolve_home().expect("home should resolve"), target);
        }

        assert_eq!(active_home_override(), previous);
    }

    #[test]
    fn ensure_layout_creates_required_operator_directories() {
        let root = default_data_root()
            .expect("data root should resolve")
            .join(".test")
            .join("layout-check");

        let _ = std::fs::remove_dir_all(&root);
        ensure_layout(&root).expect("layout creation should succeed");

        for relative in [
            "config",
            "identity",
            "keys",
            "wallets",
            "ledger",
            "runtime",
            "runtime/db",
            "runtime/bin",
            "runtime/logs",
            "runtime/snapshots",
            "telemetry",
            "reports",
            "support",
            "backups",
            "networks/mainnet/config",
            "networks/mainnet/db",
            "networks/testnet/ledger",
            "networks/devnet/logs",
            "support/delete-protection.md",
        ] {
            assert!(root.join(relative).exists(), "missing {relative}");
        }

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn write_file_persists_content_and_hardens_permissions() {
        let root = default_data_root()
            .expect("data root should resolve")
            .join(".test")
            .join("write-file");
        let path = root.join("config").join("settings.json");

        let _ = std::fs::remove_dir_all(&root);

        write_file(&path, "{\"profile\":\"mainnet\"}")
            .expect("sensitive file write should succeed");

        let content = read_file(&path).expect("written file should be readable");
        assert_eq!(content, "{\"profile\":\"mainnet\"}");

        assert!(file_permissions_are_hardened(&path).expect("metadata should be readable"));

        let _ = std::fs::remove_dir_all(root);
    }
}
