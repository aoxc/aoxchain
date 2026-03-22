use crate::error::{AppError, ErrorCode};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

pub fn default_home_dir() -> Result<PathBuf, AppError> {
    let home = env::var("HOME").map(PathBuf::from).map_err(|_| {
        AppError::new(
            ErrorCode::HomeResolutionFailed,
            "HOME environment variable is not set",
        )
    })?;
    Ok(home.join(".aoxc-data"))
}

pub fn resolve_home() -> Result<PathBuf, AppError> {
    match env::var("AOXC_HOME") {
        Ok(value) if !value.trim().is_empty() => Ok(PathBuf::from(value)),
        _ => default_home_dir(),
    }
}

pub fn ensure_layout(home: &Path) -> Result<(), AppError> {
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
        fs::create_dir_all(home.join(relative)).map_err(|e| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!(
                    "Failed to create directory {}",
                    home.join(relative).display()
                ),
                e,
            )
        })?;
    }
    Ok(())
}

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
    })
}

pub fn read_file(path: &Path) -> Result<String, AppError> {
    fs::read_to_string(path).map_err(|e| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to read file {}", path.display()),
            e,
        )
    })
}
