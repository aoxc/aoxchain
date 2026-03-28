// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    config::{loader::load, settings::Settings},
    data_home::resolve_home,
    error::{AppError, ErrorCode},
    node::lifecycle::load_state,
    runtime::context::RuntimeContext,
};

/// Builds the canonical AOXC runtime context for operator-facing status and
/// diagnostics surfaces.
///
/// Behavioral policy:
/// - Existing validated settings are preferred.
/// - Missing configuration falls back to deterministic in-memory defaults for
///   the active AOXC home.
/// - Invalid configuration remains a hard failure.
/// - Node state is optional because early bootstrap surfaces may execute before
///   a canonical runtime-state payload exists.
///
/// Side-effect policy:
/// - This function is read-oriented and must not initialize or persist settings
///   merely to construct a runtime context view.
pub fn runtime_context() -> Result<RuntimeContext, AppError> {
    let settings = effective_settings_for_runtime_context()?;
    let node_state = load_state().ok();

    Ok(RuntimeContext::new(settings, node_state))
}

/// Resolves effective settings for read-only runtime context construction.
///
/// Resolution order:
/// 1. Load canonical persisted settings when present.
/// 2. If settings are missing, derive deterministic defaults from the active
///    AOXC home without writing to disk.
/// 3. Propagate invalid settings as an explicit error.
fn effective_settings_for_runtime_context() -> Result<Settings, AppError> {
    match load() {
        Ok(settings) => Ok(settings),
        Err(error) if error.code() == ErrorCode::ConfigMissing.as_str() => {
            let home = resolve_home()?;
            Ok(Settings::default_for(home.display().to_string()))
        }
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::runtime_context;
    use crate::test_support::{aoxc_home_test_lock, AoxcHomeGuard, TestHome};

    fn with_test_home<T>(label: &str, test: impl FnOnce(&TestHome) -> T) -> T {
        let _lock = aoxc_home_test_lock();
        let home = TestHome::new(label);
        let _guard = AoxcHomeGuard::install(home.path());
        test(&home)
    }

    #[test]
    fn runtime_context_uses_in_memory_defaults_when_config_is_missing() {
        with_test_home("runtime-context-missing-config", |home| {
            let context =
                runtime_context().expect("runtime context should resolve without persisted config");

            assert_eq!(context.settings.profile, "validation");
            assert_eq!(context.settings.home_dir, home.path().display().to_string());
        });
    }

    #[test]
    fn runtime_context_allows_absent_node_state() {
        with_test_home("runtime-context-missing-node-state", |_home| {
            let context =
                runtime_context().expect("runtime context should resolve without node state");

            assert!(context.node_state.is_none());
        });
    }
}
