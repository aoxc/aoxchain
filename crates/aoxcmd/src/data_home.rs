// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::error::{AppError, ErrorCode};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

/// Returns the canonical AOXC operator data root for the current user.
///
/// Default path policy:
/// - Linux/macOS style environments resolve to `$HOME/.AOXCData`.
///
/// Operational rationale:
/// - Preserve a single AOXC-owned root directory under the user home.
/// - Avoid scattering AOXC state across unrelated user-visible locations.
/// - Provide a stable anchor for configuration, identity material, keys,
///   runtime state, telemetry output, and operator-facing reports.
///
/// Security note:
/// - This path is a storage convention, not a trust boundary by itself.
/// - Profile separation, path validation, and permission controls remain the
///   responsibility of the caller and the surrounding workflow.
pub fn default_home_dir() -> Result<PathBuf, AppError> {
    let home = env::var("HOME").map(PathBuf::from).map_err(|_| {
        AppError::new(
            ErrorCode::HomeResolutionFailed,
            "HOME environment variable is not set",
        )
    })?;

    Ok(home.join(".AOXCData"))
}

/// Resolves the effective AOXC operator data root.
///
/// Resolution order:
/// 1. `AOXC_HOME` when present and non-empty
/// 2. canonical default returned by `default_home_dir()`
///
/// Path policy:
/// - Empty or whitespace-only override values are ignored.
/// - Non-empty override values are accepted as provided by the operator.
pub fn resolve_home() -> Result<PathBuf, AppError> {
    match env::var("AOXC_HOME") {
        Ok(value) if !value.trim().is_empty() => Ok(PathBuf::from(value)),
        _ => default_home_dir(),
    }
}

/// Ensures the canonical AOXC directory layout exists under the supplied root.
///
/// Layout policy:
/// - `config/`      => operator configuration surfaces
/// - `identity/`    => identity-bound metadata and declarations
/// - `keys/`        => sensitive key material and related artifacts
/// - `ledger/`      => economy and accounting state
/// - `runtime/`     => live runtime state
/// - `runtime/db/`  => local database storage
/// - `telemetry/`   => metrics, snapshots, and observability output
/// - `reports/`     => readiness, audit, and operator-facing reports
/// - `support/`     => support bundles and auxiliary diagnostic artifacts
///
/// Design objective:
/// - Preserve a stable and operator-readable storage contract under a single
///   AOXC-owned root directory.
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
///
/// Write behavior:
/// - Parent directories are created automatically when absent.
/// - Existing files are overwritten with the provided content.
/// - On Unix targets, permissions are reduced to `0600` after write.
///
/// Operational intent:
/// - Sensitive AOXC artifacts must not rely on permissive default visibility
///   in operator environments.
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
///
/// Unix policy:
/// - The check passes only when no group or world permission bits are present.
///
/// Non-Unix policy:
/// - The function currently returns `true` as a compatibility fallback because
///   equivalent portable permission semantics are not uniformly exposed through
///   the same standard library surface.
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
///
/// Unix policy:
/// - Files are reduced to mode `0600`.
///
/// Non-Unix policy:
/// - The function is currently a no-op.
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
        default_home_dir, ensure_layout, file_permissions_are_hardened, read_file, write_file,
    };
    use std::{env, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

    fn unique_test_root(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after unix epoch")
            .as_nanos();

        env::temp_dir().join(format!("aoxc-home-tests-{label}-{nanos}"))
    }

    #[test]
    fn default_home_dir_resolves_to_hidden_aoxcdata_root() {
        let home = env::var("HOME").expect("HOME must be set for tests");
        let expected = PathBuf::from(home).join(".AOXCData");

        assert_eq!(
            default_home_dir().expect("default home should resolve"),
            expected
        );
    }

    #[test]
    fn ensure_layout_creates_required_operator_directories() {
        let root = unique_test_root("layout");

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
            assert!(
                root.join(relative).is_dir(),
                "expected directory {} to exist",
                root.join(relative).display()
            );
        }

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn write_file_persists_content_and_hardens_permissions() {
        let root = unique_test_root("write-file");
        let path = root.join("config").join("settings.json");

        write_file(&path, "{\"profile\":\"mainnet\"}")
            .expect("sensitive file write should succeed");

        let content = read_file(&path).expect("written file should be readable");
        assert_eq!(content, "{\"profile\":\"mainnet\"}");

        assert!(
            file_permissions_are_hardened(&path).expect("metadata should be readable"),
            "written files must be hardened for operator environments"
        );

        let _ = std::fs::remove_dir_all(root);
    }
}
