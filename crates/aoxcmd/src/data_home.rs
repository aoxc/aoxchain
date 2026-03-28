// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::error::{AppError, ErrorCode};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

/// Returns the canonical default AOXC operator home directory.
///
/// The default path is intentionally namespaced under:
/// `$HOME/.AOXCData/home/default`
///
/// This avoids mixing AOXC state with unrelated hidden folders and makes it
/// operationally clearer that a single home directory should represent a
/// single runtime environment. Operators are expected to override this path
/// with `AOXC_HOME` or CLI `--home` for explicit per-network separation.
pub fn default_home_dir() -> Result<PathBuf, AppError> {
    let home = env::var("HOME").map(PathBuf::from).map_err(|_| {
        AppError::new(
            ErrorCode::HomeResolutionFailed,
            "HOME environment variable is not set",
        )
    })?;

    Ok(home.join(".AOXCData").join("home").join("default"))
}

/// Resolves the effective AOXC operator home directory.
///
/// Resolution order:
/// 1. `AOXC_HOME` environment variable when present and non-empty
/// 2. canonical default operator home returned by `default_home_dir()`
pub fn resolve_home() -> Result<PathBuf, AppError> {
    match env::var("AOXC_HOME") {
        Ok(value) if !value.trim().is_empty() => Ok(PathBuf::from(value)),
        _ => default_home_dir(),
    }
}

/// Ensures the canonical AOXC directory layout exists under the supplied home.
///
/// The layout intentionally separates configuration, key material, genesis,
/// runtime state, telemetry, and operator reports into dedicated subtrees.
pub fn ensure_layout(home: &Path) -> Result<(), AppError> {
    let required_dirs = [
        "config",
        "identity",
        "keys",
        "ledger",
        "runtime",
        "telemetry",
        "reports",
        "support",
    ];

    for relative in required_dirs {
        let dir = home.join(relative);
        fs::create_dir_all(&dir).map_err(|e| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to create directory {}", dir.display()),
                e,
            )
        })?;
    }

    Ok(())
}

/// Writes a sensitive AOXC file and hardens its permissions when supported.
///
/// Parent directories are created automatically before writing.
/// On Unix platforms, file permissions are reduced to `0600`.
pub fn write_file(path: &Path, content: &str) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to create parent directory {}", parent.display()),
                e,
            )
        })?;
    }

    fs::write(path, content).map_err(|e| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to write file {}", path.display()),
            e,
        )
    })?;

    harden_file_permissions(path)?;
    Ok(())
}

/// Reads a UTF-8 AOXC file from disk.
pub fn read_file(path: &Path) -> Result<String, AppError> {
    fs::read_to_string(path).map_err(|e| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to read file {}", path.display()),
            e,
        )
    })
}

/// Returns whether the file permissions are hardened for sensitive operator data.
///
/// On Unix, the check requires that no group/world permission bits are present.
/// On non-Unix platforms, this function currently returns `true`.
pub fn file_permissions_are_hardened(path: &Path) -> Result<bool, AppError> {
    let metadata = fs::metadata(path).map_err(|e| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to read metadata for {}", path.display()),
            e,
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
///
/// On Unix, written files are reduced to mode `0600`.
/// On non-Unix targets, the function is a no-op.
fn harden_file_permissions(path: &Path) -> Result<(), AppError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|e| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to harden permissions on {}", path.display()),
                e,
            )
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        default_home_dir, ensure_layout, file_permissions_are_hardened, resolve_home, write_file,
    };
    use crate::test_support::TestHome;
    use std::{env, path::PathBuf};

    #[test]
    fn default_home_dir_is_namespaced_under_hidden_aoxcdata_home() {
        let home = env::var("HOME").expect("HOME must be set for tests");
        let expected = PathBuf::from(home).join(".AOXCData").join("home").join("default");

        assert_eq!(
            default_home_dir().expect("default home should resolve"),
            expected
        );
    }

    #[test]
    fn resolve_home_prefers_aoxc_home_override() {
        let test_home = TestHome::new("resolve-home-override");
        let override_home = test_home.path().join("custom-home");

        env::set_var("AOXC_HOME", &override_home);
        let resolved = resolve_home().expect("AOXC_HOME override should resolve");
        env::remove_var("AOXC_HOME");

        assert_eq!(resolved, override_home);
    }

    #[test]
    fn ensure_layout_creates_required_operator_directories() {
        let home = TestHome::new("ensure-layout");
        let root = home.path().join("operator-home");

        ensure_layout(&root).expect("layout creation should succeed");

        for relative in [
            "config",
            "identity",
            "keys",
            "ledger",
            "runtime",
            "telemetry",
            "reports",
            "support",
        ] {
            assert!(
                root.join(relative).is_dir(),
                "expected directory {} to exist",
                root.join(relative).display()
            );
        }
    }

    #[test]
    fn write_file_hardens_sensitive_file_permissions() {
        let home = TestHome::new("permission-hardening");
        let path = home.path().join("config").join("settings.json");

        write_file(&path, "{\"profile\":\"mainnet\"}")
            .expect("sensitive file write should succeed");

        assert!(
            file_permissions_are_hardened(&path).expect("metadata should be readable"),
            "written files must be hardened for production operator environments"
        );
    }
}
