// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    economy::ledger,
    error::AppError,
    node::lifecycle,
    telemetry::prometheus::{MetricsSnapshot, now, persist_metrics},
};

/// Refreshes the canonical AOXC runtime metrics snapshot.
///
/// Refresh contract:
/// - Loads the currently persisted canonical node state.
/// - Loads the currently persisted canonical ledger state.
/// - Derives a point-in-time metrics snapshot from those two authoritative
///   runtime surfaces.
/// - Persists the resulting metrics snapshot to the canonical telemetry path.
///
/// Failure policy:
/// - Node-state load failures are propagated without masking.
/// - Ledger load failures are propagated without masking.
/// - Metrics persistence failures are propagated without masking.
///
/// Operational rationale:
/// - This function is intentionally side-effecting because telemetry refresh is
///   meaningful only once the derived snapshot is durably written.
/// - The snapshot is derived exclusively from authoritative persisted runtime
///   state rather than transient in-memory estimates.
pub fn refresh_runtime_metrics() -> Result<(), AppError> {
    let snapshot = build_runtime_metrics_snapshot()?;
    persist_metrics(&snapshot)?;
    Ok(())
}

/// Builds a canonical runtime metrics snapshot from authoritative persisted
/// runtime surfaces.
///
/// Composition policy:
/// - `node_height` is sourced from canonical node state.
/// - `produced_blocks` is sourced from canonical node state.
/// - `treasury_balance` is sourced from canonical ledger state.
fn build_runtime_metrics_snapshot() -> Result<MetricsSnapshot, AppError> {
    let node_state = lifecycle::load_state()?;
    let ledger_state = ledger::load()?;

    Ok(now(
        node_state.current_height,
        node_state.produced_blocks,
        ledger_state.treasury_balance,
    ))
}

#[cfg(test)]
mod tests {
    use super::refresh_runtime_metrics;
    use crate::{
        economy::ledger::{LedgerState, persist},
        node::{lifecycle::persist_state, state::NodeState},
        telemetry::prometheus::{MetricsSnapshot, metrics_path},
        test_support::{AoxcHomeGuard, TestHome, aoxc_home_test_lock},
    };

    fn with_test_home<T>(label: &str, test: impl FnOnce(&TestHome) -> T) -> T {
        let _lock = aoxc_home_test_lock();
        let home = TestHome::new(label);
        let _guard = AoxcHomeGuard::install(home.path());
        test(&home)
    }

    #[test]
    fn refresh_runtime_metrics_persists_snapshot_from_canonical_state() {
        with_test_home("refresh-runtime-metrics", |_home| {
            let mut node_state = NodeState::bootstrap();
            node_state.current_height = 7;
            node_state.produced_blocks = 7;
            node_state.last_tx = "metrics-smoke".to_string();
            node_state.consensus.last_round = 7;
            node_state.consensus.last_message_kind = "block_proposal".to_string();
            persist_state(&node_state).expect("node state fixture should persist");

            let mut ledger_state = LedgerState::new();
            ledger_state.treasury_balance = 999_999_999_000;
            persist(&ledger_state).expect("ledger fixture should persist");

            refresh_runtime_metrics().expect("runtime metrics refresh should succeed");

            let path = metrics_path().expect("metrics path should resolve");
            assert!(path.is_file(), "metrics snapshot should be persisted");

            let raw = std::fs::read_to_string(&path).expect("metrics snapshot should be readable");
            let snapshot: MetricsSnapshot =
                serde_json::from_str(&raw).expect("metrics snapshot should decode");

            assert_eq!(snapshot.node_height, 7);
            assert_eq!(snapshot.produced_blocks, 7);
            assert_eq!(snapshot.treasury_balance, 999_999_999_000);
            assert!(
                chrono::DateTime::parse_from_rfc3339(&snapshot.recorded_at).is_ok(),
                "recorded_at should remain a valid RFC3339 timestamp"
            );
        });
    }

    #[test]
    fn refresh_runtime_metrics_fails_when_node_state_is_missing() {
        with_test_home("refresh-runtime-metrics-missing-node", |_home| {
            let error = refresh_runtime_metrics()
                .expect_err("missing canonical node state must fail telemetry refresh");

            assert_eq!(error.code(), "AOXC-NOD-001");
        });
    }
}
