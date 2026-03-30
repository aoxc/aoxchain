// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{data_home::resolve_home, error::AppError};
use std::path::PathBuf;

/// Returns the canonical AOXC operator-key path.
///
/// Canonical storage policy:
/// - Operator key material is persisted at:
///   `<AOXC_HOME>/keys/operator_key.json`.
///
/// Design rationale:
/// - The path is derived strictly from the effective AOXC home so that
///   command-scoped home overrides, isolated test homes, and production
///   operator homes all resolve consistently.
/// - This function is path-only; directory creation and persistence semantics
///   remain the responsibility of higher-level filesystem helpers.
pub fn operator_key_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("keys").join("operator_key.json"))
}

#[cfg(test)]
mod tests {
    use super::operator_key_path;
    use crate::test_support::{AoxcHomeGuard, TestHome, aoxc_home_test_lock};

    fn with_test_home<T>(label: &str, test: impl FnOnce(&TestHome) -> T) -> T {
        let _lock = aoxc_home_test_lock();
        let home = TestHome::new(label);
        let _guard = AoxcHomeGuard::install(home.path());
        test(&home)
    }

    #[test]
    fn operator_key_path_resolves_inside_active_test_home() {
        with_test_home("keys-path-resolution", |home| {
            let path = operator_key_path().expect("operator key path must resolve");

            assert_eq!(path, home.path().join("keys").join("operator_key.json"));
        });
    }

    #[test]
    fn operator_key_path_uses_canonical_operator_key_filename() {
        with_test_home("keys-path-filename", |_home| {
            let path = operator_key_path().expect("operator key path must resolve");

            assert!(
                path.display()
                    .to_string()
                    .ends_with("keys/operator_key.json")
            );
        });
    }
}
