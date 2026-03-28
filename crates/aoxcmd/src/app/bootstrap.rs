// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    config::loader::load_or_init,
    data_home::{ensure_layout, resolve_home},
    error::AppError,
};

/// Bootstraps the canonical AOXC operator home.
///
/// Bootstrap contract:
/// - Resolves the effective AOXC home.
/// - Ensures the canonical AOXC directory layout exists.
/// - Ensures a canonical settings document is available.
/// - Returns successfully only when the operator home is usable for
///   subsequent AOXC command surfaces.
///
/// Side-effect policy:
/// - This function is intentionally bootstrap-oriented.
/// - It may create directories under the effective AOXC home.
/// - It may initialize the canonical settings document when configuration is
///   currently absent.
///
/// Safety rationale:
/// - Existing valid configuration is preserved.
/// - Existing invalid configuration remains a hard failure through
///   `load_or_init()`, preventing silent overwrite of operator-authored state.
pub fn bootstrap_operator_home() -> Result<(), AppError> {
    let home = resolve_home()?;
    ensure_layout(&home)?;

    let _settings = load_or_init()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::bootstrap_operator_home;
    use crate::{
        config::loader::settings_path,
        test_support::{aoxc_home_test_lock, AoxcHomeGuard, TestHome},
    };

    fn with_test_home<T>(label: &str, test: impl FnOnce(&TestHome) -> T) -> T {
        let _lock = aoxc_home_test_lock();
        let home = TestHome::new(label);
        let _guard = AoxcHomeGuard::install(home.path());
        test(&home)
    }

    #[test]
    fn bootstrap_operator_home_creates_canonical_layout_and_settings() {
        with_test_home("app-bootstrap-home", |home| {
            bootstrap_operator_home().expect("operator home bootstrap should succeed");

            assert!(home.path().join("config").is_dir());
            assert!(home.path().join("keys").is_dir());
            assert!(home.path().join("runtime").is_dir());
            assert!(home.path().join("telemetry").is_dir());
            assert!(settings_path()
                .expect("settings path should resolve")
                .is_file());
        });
    }

    #[test]
    fn bootstrap_operator_home_is_idempotent() {
        with_test_home("app-bootstrap-home-idempotent", |_home| {
            bootstrap_operator_home().expect("first operator home bootstrap should succeed");
            bootstrap_operator_home().expect("second operator home bootstrap should also succeed");
        });
    }
}
