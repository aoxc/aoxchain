// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! Canonical AOXC application error surface.
//!
//! This module provides the public error entry point for the AOXC command
//! plane. Callers should import `AppError` and `ErrorCode` from this module
//! rather than from internal submodules so the operator-facing error contract
//! remains centralized and stable.

pub mod app_error;

/// Canonical AOXC application error envelope.
///
/// Re-export policy:
/// - `AppError` is the standard operator-facing error type used across the
///   AOXC command plane.
/// - `ErrorCode` is the stable machine-readable taxonomy used for deterministic
///   exit codes, CLI diagnostics, and test assertions.
pub use app_error::{AppError, ErrorCode};

#[cfg(test)]
mod tests {
    use super::{AppError, ErrorCode};

    #[test]
    fn error_module_reexports_app_error_surface() {
        let error = AppError::new(ErrorCode::ConfigMissing, "Configuration file is missing");

        assert_eq!(error.code(), "AOXC-CFG-001");
        assert_eq!(error.exit_code(), 4);
    }
}
