// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    config::loader::load_or_init,
    data_home::{ensure_layout, resolve_home},
    error::AppError,
};

/// Ensures that the canonical AOXC operator environment exists and is usable.
///
/// Environment contract:
/// - The effective AOXC home must resolve successfully.
/// - The canonical AOXC directory layout must exist.
/// - A canonical settings document must be loadable after the operation.
/// - If settings are missing, safe defaults are materialized through the
///   canonical configuration bootstrap surface.
///
/// Operational rationale:
/// - This function is intentionally bootstrap-oriented rather than read-only.
/// - Callers use this surface when they want the local operator environment to
///   be ready for subsequent commands, not merely inspected.
///
/// Side effects:
/// - May create the AOXC home layout on disk.
/// - May initialize the canonical settings document when it is absent.
pub fn ensure_operator_environment() -> Result<(), AppError> {
    let home = resolve_home()?;
    ensure_layout(&home)?;

    let _settings = load_or_init()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::ensure_operator_environment;
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
    fn ensure_operator_environment_creates_canonical_layout_and_settings() {
        with_test_home("ensure-operator-environment", |home| {
            ensure_operator_environment().expect("operator environment bootstrap should succeed");

            assert!(home.path().join("config").is_dir());
            assert!(home.path().join("keys").is_dir());
            assert!(home.path().join("runtime").is_dir());
            assert!(settings_path()
                .expect("settings path should resolve")
                .is_file());
        });
    }

    #[test]
    fn ensure_operator_environment_is_idempotent() {
        with_test_home("ensure-operator-environment-idempotent", |_home| {
            ensure_operator_environment()
                .expect("first operator environment bootstrap should succeed");
            ensure_operator_environment()
                .expect("second operator environment bootstrap should also succeed");
        });
    }
}
