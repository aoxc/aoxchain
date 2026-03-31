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
/// - Node-state loading is treated as best-effort for context construction:
///   when canonical runtime state can be materialized successfully, it is
///   attached to the context; otherwise the context remains available without
///   a node-state payload.
///
/// Operational rationale:
/// - Operator-facing runtime status surfaces should remain readable even before
///   the node has completed a full bootstrap sequence.
/// - Context construction must never create or persist configuration artifacts.
/// - Node-state availability is intentionally non-fatal for read-oriented
///   runtime inspection paths.
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
///
/// Audit note:
/// This function is intentionally read-oriented. It must not initialize,
/// mutate, or persist configuration merely to satisfy status/reporting
/// surfaces.
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
    use crate::{
        config::{loader::persist, settings::Settings},
        node::{lifecycle::persist_state, state::NodeState},
        test_support::{AoxcHomeGuard, TestHome, aoxc_home_test_lock},
    };

    /// Executes a runtime-context test inside a process-safe isolated AOXC home.
    ///
    /// Isolation guarantees:
    /// - Serializes AOXC_HOME mutation through the shared crate-level lock.
    /// - Restores the previous environment via the shared RAII home guard.
    /// - Provides a disposable filesystem namespace per test label.
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

            assert_eq!(
                context.settings.profile, "validation",
                "missing configuration must fall back to the deterministic validation profile"
            );
            assert_eq!(
                context.settings.home_dir,
                home.path().display().to_string(),
                "in-memory default settings must bind to the active AOXC home"
            );
        });
    }

    #[test]
    fn runtime_context_uses_persisted_settings_when_configuration_exists() {
        with_test_home("runtime-context-persisted-config", |home| {
            let mut settings = Settings::default_for(home.path().display().to_string());
            settings.profile = "mainnet".to_string();
            settings.logging.json = true;
            settings.network.bind_host = "0.0.0.0".to_string();

            persist(&settings).expect("persisted settings fixture should be written");

            let context =
                runtime_context().expect("runtime context should load persisted settings");

            assert_eq!(
                context.settings.profile, "mainnet",
                "persisted configuration must take precedence over in-memory defaults"
            );
            assert_eq!(
                context.settings.home_dir,
                home.path().display().to_string(),
                "persisted configuration must remain scoped to the active AOXC home"
            );
            assert!(
                context.settings.logging.json,
                "persisted structured logging preference must be preserved"
            );
            assert_eq!(
                context.settings.network.bind_host, "0.0.0.0",
                "persisted network settings must be surfaced unchanged"
            );
        });
    }

    #[test]
    fn runtime_context_materializes_bootstrap_node_state_when_runtime_store_is_empty() {
        with_test_home("runtime-context-bootstrap-node-state", |_home| {
            let context = runtime_context()
                .expect("runtime context should resolve when the runtime store is empty");

            let node_state = context
                .node_state
                .as_ref()
                .expect("runtime context should surface a bootstrap node state on first run");

            assert!(
                node_state.initialized,
                "bootstrap node state must be marked as initialized"
            );
            assert_eq!(
                node_state.current_height, 0,
                "bootstrap node state must start at canonical height zero"
            );
            assert_eq!(
                node_state.produced_blocks, 0,
                "bootstrap node state must start with zero produced blocks"
            );
            assert_eq!(
                node_state.consensus.last_message_kind, "bootstrap",
                "bootstrap node state must expose the canonical bootstrap consensus marker"
            );
        });
    }

    #[test]
    fn runtime_context_surfaces_persisted_node_state_when_runtime_state_exists() {
        with_test_home("runtime-context-persisted-node-state", |_home| {
            let mut node_state = NodeState::bootstrap();
            node_state.current_height = 17;
            node_state.produced_blocks = 17;
            node_state.last_tx = "runtime-context-smoke".to_string();
            node_state.consensus.last_round = 17;
            node_state.consensus.last_message_kind = "commit".to_string();

            persist_state(&node_state).expect("runtime state fixture should persist");

            let context =
                runtime_context().expect("runtime context should load persisted node state");
            let loaded = context
                .node_state
                .as_ref()
                .expect("persisted runtime state must be attached to the context");

            assert_eq!(
                loaded.current_height, 17,
                "runtime context must expose the canonical persisted node height"
            );
            assert_eq!(
                loaded.produced_blocks, 17,
                "runtime context must expose the canonical persisted produced block count"
            );
            assert_eq!(
                loaded.last_tx, "runtime-context-smoke",
                "runtime context must expose the canonical persisted last transaction marker"
            );
            assert_eq!(
                loaded.consensus.last_round, 17,
                "runtime context must expose the canonical persisted consensus round"
            );
            assert_eq!(
                loaded.consensus.last_message_kind, "commit",
                "runtime context must expose the canonical persisted consensus message kind"
            );
        });
    }

    #[test]
    fn runtime_context_keeps_settings_resolution_independent_from_node_state_materialization() {
        with_test_home("runtime-context-settings-node-independence", |home| {
            let mut settings = Settings::default_for(home.path().display().to_string());
            settings.profile = "testnet".to_string();
            settings.logging.json = true;
            persist(&settings).expect("settings fixture should persist");

            let context = runtime_context()
                .expect("runtime context should resolve with mixed persisted state");

            assert_eq!(
                context.settings.profile, "testnet",
                "settings resolution must still honor persisted configuration"
            );

            let node_state = context
                .node_state
                .as_ref()
                .expect("runtime context should still attach a materialized node state");

            assert!(
                node_state.initialized,
                "node-state materialization must remain available alongside persisted settings"
            );
        });
    }
}
