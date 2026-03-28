// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::error::{AppError, ErrorCode};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

/// Returns the canonical AOXC data root for the current user.
///
/// Canonical data root policy:
/// - Linux/macOS style environments resolve to `$HOME/.AOXCData`.
///
/// This directory is the top-level AOXC-owned namespace. It is not, by itself,
/// the effective runtime home used by commands. The effective AOXC home is
/// derived beneath this root unless explicitly overridden.
pub fn default_data_root() -> Result<PathBuf, AppError> {
    let home = env::var("HOME").map(PathBuf::from).map_err(|_| {
        AppError::new(
            ErrorCode::HomeResolutionFailed,
            "HOME environment variable is not set",
        )
    })?;

    Ok(home.join(".AOXCData"))
}

/// Returns the canonical default AOXC home directory.
///
/// Canonical home policy:
/// - `$HOME/.AOXCData/home/default`
///
/// Design intent:
/// - Preserve a stable AOXC-owned root at `$HOME/.AOXCData`.
/// - Keep operator-specific runtime state under `home/<name>`.
/// - Align runtime defaults with packaging and Makefile conventions.
pub fn default_home_dir() -> Result<PathBuf, AppError> {
    Ok(default_data_root()?.join("home").join("default"))
}

/// Resolves the effective AOXC operator home.
///
/// Resolution order:
/// 1. `AOXC_HOME` when present and non-empty
/// 2. canonical default returned by `default_home_dir()`
pub fn resolve_home() -> Result<PathBuf, AppError> {
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
/// - `ledger/`
/// - `runtime/`
/// - `runtime/db/`
/// - `telemetry/`
/// - `reports/`
/// - `support/`
pub fn ensure_layout(home: &Path) -> Result<(), AppError> {
    let required_dirs = [
        "config",
        "identity",
        "keys",
        "ledger",
        "runtime",
        "runtime/db",
        "telemetry",
        "reports",
        "support",
    ];

    for relative in required_dirs {
        let dir = home.join(relative);
        fs::create_dir_all(&dir).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to create directory {}", dir.display()),
                error,
            )
        })?;
    }

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

#[cfg(test)]
mod tests {
    use super::{
        default_data_root, default_home_dir, ensure_layout, file_permissions_are_hardened,
        read_file, write_file,
    };
    use std::env;

    #[test]
    fn default_data_root_resolves_to_hidden_aoxcdata_root() {
        let home = env::var("HOME").expect("HOME must be set for tests");
        assert_eq!(
            default_data_root().expect("data root should resolve"),
            std::path::PathBuf::from(home).join(".AOXCData")
        );
    }

    #[test]
    fn default_home_dir_resolves_beneath_canonical_data_root() {
        let home = env::var("HOME").expect("HOME must be set for tests");
        assert_eq!(
            default_home_dir().expect("default home should resolve"),
            std::path::PathBuf::from(home)
                .join(".AOXCData")
                .join("home")
                .join("default")
        );
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
            "ledger",
            "runtime",
            "runtime/db",
            "telemetry",
            "reports",
            "support",
        ] {
            assert!(root.join(relative).is_dir());
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
