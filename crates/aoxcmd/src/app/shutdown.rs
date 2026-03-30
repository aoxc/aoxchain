// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    error::AppError,
    node::{
        lifecycle::{load_state, persist_state},
        state::NodeState,
    },
};

/// Performs a canonical AOXC graceful shutdown transition.
///
/// Shutdown contract:
/// - Loads the currently persisted canonical node state.
/// - Transitions the node into a non-running state.
/// - Refreshes the operator-facing update timestamp.
/// - Persists the updated state back to the canonical runtime-state surface.
///
/// Operational rationale:
/// - This function is intentionally state-mutating because graceful shutdown is
///   complete only once the non-running state is durably recorded.
/// - The operation is designed to be idempotent; shutting down an already
///   stopped node remains a valid persistence action.
pub fn graceful_shutdown() -> Result<(), AppError> {
    let state = build_shutdown_state()?;
    persist_state(&state)?;
    Ok(())
}

/// Builds the canonical post-shutdown node-state snapshot.
///
/// Transition policy:
/// - `running` is always forced to `false`.
/// - `touch()` is always applied so the persisted state reflects the shutdown
///   operation timestamp, even when the node was already stopped.
fn build_shutdown_state() -> Result<NodeState, AppError> {
    let mut state = load_state()?;
    state.running = false;
    state.touch();
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::graceful_shutdown;
    use crate::{
        node::{lifecycle::persist_state, state::NodeState},
        test_support::{AoxcHomeGuard, TestHome, aoxc_home_test_lock},
    };

    fn with_test_home<T>(label: &str, test: impl FnOnce(&TestHome) -> T) -> T {
        let _lock = aoxc_home_test_lock();
        let home = TestHome::new(label);
        let _guard = AoxcHomeGuard::install(home.path());
        test(&home)
    }

    #[test]
    fn graceful_shutdown_persists_non_running_state() {
        with_test_home("graceful-shutdown-running", |_home| {
            let mut state = NodeState::bootstrap();
            state.running = true;
            state.current_height = 4;
            state.produced_blocks = 4;
            state.last_tx = "shutdown-smoke".to_string();
            state.consensus.last_round = 4;
            state.consensus.last_message_kind = "block_proposal".to_string();

            persist_state(&state).expect("running node state fixture should persist");
            graceful_shutdown().expect("graceful shutdown should succeed");

            let reloaded =
                crate::node::lifecycle::load_state().expect("shutdown state should reload");

            assert!(!reloaded.running);
            assert_eq!(reloaded.current_height, 4);
            assert_eq!(reloaded.produced_blocks, 4);
            assert_eq!(reloaded.last_tx, "shutdown-smoke");
        });
    }

    #[test]
    fn graceful_shutdown_is_idempotent_for_already_stopped_nodes() {
        with_test_home("graceful-shutdown-idempotent", |_home| {
            let mut state = NodeState::bootstrap();
            state.running = false;
            state.current_height = 1;
            state.produced_blocks = 1;
            state.last_tx = "already-stopped".to_string();
            state.consensus.last_round = 1;
            state.consensus.last_message_kind = "block_proposal".to_string();

            persist_state(&state).expect("stopped node state fixture should persist");

            graceful_shutdown().expect("first graceful shutdown should succeed");
            graceful_shutdown().expect("second graceful shutdown should also succeed");

            let reloaded =
                crate::node::lifecycle::load_state().expect("shutdown state should reload");

            assert!(!reloaded.running);
            assert_eq!(reloaded.current_height, 1);
            assert_eq!(reloaded.produced_blocks, 1);
        });
    }
}
