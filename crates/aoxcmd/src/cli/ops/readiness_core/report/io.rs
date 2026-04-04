use super::*;

pub(in crate::cli::ops) fn write_readiness_markdown_report(
    path: &Path,
    readiness: &Readiness,
    embedded_baseline: Option<&ProfileBaselineReport>,
    aoxhub_baseline: Option<&ProfileBaselineReport>,
) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to create report directory {}", parent.display()),
                error,
            )
        })?;
    }

    fs::write(
        path,
        readiness_markdown_report(readiness, embedded_baseline, aoxhub_baseline),
    )
    .map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to write readiness report {}", path.display()),
            error,
        )
    })
}

pub(in crate::cli::ops) fn write_full_surface_markdown_report(
    path: &Path,
    readiness: &FullSurfaceReadiness,
) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!(
                    "Failed to create full-surface report directory {}",
                    parent.display()
                ),
                error,
            )
        })?;
    }

    fs::write(path, full_surface_markdown_report(readiness)).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to write full-surface report {}", path.display()),
            error,
        )
    })
}
