// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    economy::ledger,
    error::AppError,
    node::{lifecycle, state::NodeState},
    telemetry::prometheus::{MetricsSnapshot, now, persist_metrics},
};

/// Refreshes the canonical AOXC runtime metrics snapshot.
///
/// Control flow:
/// 1. Load the authoritative canonical node state from persistence.
/// 2. Load the authoritative canonical ledger state from persistence.
/// 3. Derive an immutable point-in-time telemetry snapshot.
/// 4. Persist the derived snapshot to the canonical telemetry location.
///
/// Failure policy:
/// - Node-state loading failures are returned unchanged.
/// - Ledger-state loading failures are returned unchanged.
/// - Metrics persistence failures are returned unchanged.
///
/// Design rationale:
/// This function intentionally remains as a thin orchestration layer.
/// The actual snapshot derivation logic is delegated to a pure helper so that:
/// - business invariants can be unit-tested without filesystem coupling,
/// - persistence concerns stay localized,
/// - regression diagnostics remain precise and operationally meaningful.
pub fn refresh_runtime_metrics() -> Result<(), AppError> {
    let snapshot = build_runtime_metrics_snapshot()?;
    persist_metrics(&snapshot)?;
    Ok(())
}

/// Builds a canonical runtime metrics snapshot from authoritative persisted
/// runtime surfaces.
///
/// Data contract:
/// - `node_height` is sourced from canonical node state.
/// - `produced_blocks` is sourced from canonical node state.
/// - `treasury_balance` is sourced from canonical ledger state.
///
/// Operational note:
/// This function is still persistence-aware because it represents the canonical
/// runtime snapshot assembly path used by production telemetry refresh.
fn build_runtime_metrics_snapshot() -> Result<MetricsSnapshot, AppError> {
    let node_state = lifecycle::load_state()?;
    let ledger_state = ledger::load()?;

    Ok(derive_runtime_metrics_snapshot(
        &node_state,
        ledger_state.treasury_balance,
    ))
}

/// Derives a telemetry snapshot from already-loaded authoritative runtime data.
///
/// Purity contract:
/// - Performs no filesystem access.
/// - Performs no mutation of runtime state.
/// - Encodes only the mapping between authoritative runtime inputs and the
///   telemetry representation.
///
/// Testability rationale:
/// Keeping this logic pure allows strict unit testing of the business mapping
/// without dependence on persistence layout, environment variables, or test
/// home bootstrapping.
fn derive_runtime_metrics_snapshot(
    node_state: &NodeState,
    treasury_balance: u64,
) -> MetricsSnapshot {
    now(
        node_state.current_height,
        node_state.produced_blocks,
        treasury_balance,
    )
}

#[cfg(test)]
mod tests {
    use super::{derive_runtime_metrics_snapshot, refresh_runtime_metrics};
    use crate::{
        economy::ledger::{persist, LedgerState},
        node::{lifecycle::persist_state, state::NodeState},
        telemetry::prometheus::{metrics_path, MetricsSnapshot},
        test_support::{aoxc_home_test_lock, AoxcHomeGuard, TestHome},
    };

    fn with_test_home<T>(label: &str, test: impl FnOnce(&TestHome) -> T) -> T {
        let _lock = aoxc_home_test_lock();
        let home = TestHome::new(label);
        let _guard = AoxcHomeGuard::install(home.path());
        test(&home)
    }

    #[test]
    fn derive_runtime_metrics_snapshot_maps_authoritative_runtime_fields_exactly() {
        let mut node_state = NodeState::bootstrap();
        node_state.current_height = 42;
        node_state.produced_blocks = 39;

        let snapshot = derive_runtime_metrics_snapshot(&node_state, 1_500_000);

        assert_eq!(
            snapshot.node_height, 42,
            "node height must be sourced directly from authoritative node state"
        );
        assert_eq!(
            snapshot.produced_blocks, 39,
            "produced block count must be sourced directly from authoritative node state"
        );
        assert_eq!(
            snapshot.treasury_balance, 1_500_000,
            "treasury balance must be sourced directly from authoritative ledger state"
        );
        assert!(
            chrono::DateTime::parse_from_rfc3339(&snapshot.recorded_at).is_ok(),
            "derived snapshot timestamp must remain a valid RFC3339 instant"
        );
    }

    #[test]
    fn derive_runtime_metrics_snapshot_preserves_zero_valued_runtime_surfaces() {
        let mut node_state = NodeState::bootstrap();
        node_state.current_height = 0;
        node_state.produced_blocks = 0;

        let snapshot = derive_runtime_metrics_snapshot(&node_state, 0);

        assert_eq!(
            snapshot.node_height, 0,
            "zero node height must remain representable in telemetry output"
        );
        assert_eq!(
            snapshot.produced_blocks, 0,
            "zero produced block count must remain representable in telemetry output"
        );
        assert_eq!(
            snapshot.treasury_balance, 0,
            "zero treasury balance must remain representable in telemetry output"
        );
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

            let raw = std::fs::read_to_string(&path)
                .expect("persisted metrics snapshot should be readable");
            let snapshot: MetricsSnapshot = serde_json::from_str(&raw)
                .expect("persisted metrics snapshot should decode from canonical JSON");

            assert_eq!(
                snapshot.node_height, 7,
                "persisted node height must match the canonical node state fixture"
            );
            assert_eq!(
                snapshot.produced_blocks, 7,
                "persisted produced block count must match the canonical node state fixture"
            );
            assert_eq!(
                snapshot.treasury_balance, 999_999_999_000,
                "persisted treasury balance must match the canonical ledger fixture"
            );
            assert!(
                chrono::DateTime::parse_from_rfc3339(&snapshot.recorded_at).is_ok(),
                "persisted snapshot timestamp must remain a valid RFC3339 instant"
            );
        });
    }

    #[test]
    fn refresh_runtime_metrics_overwrites_stale_snapshot_with_current_canonical_state() {
        with_test_home("refresh-runtime-metrics-overwrite", |_home| {
            let mut stale_node_state = NodeState::bootstrap();
            stale_node_state.current_height = 1;
            stale_node_state.produced_blocks = 1;
            persist_state(&stale_node_state).expect("stale node state fixture should persist");

            let mut stale_ledger_state = LedgerState::new();
            stale_ledger_state.treasury_balance = 10;
            persist(&stale_ledger_state).expect("stale ledger state fixture should persist");

            refresh_runtime_metrics().expect("initial refresh should succeed");

            let mut current_node_state = NodeState::bootstrap();
            current_node_state.current_height = 88;
            current_node_state.produced_blocks = 77;
            current_node_state.last_tx = "canonical-update".to_string();
            current_node_state.consensus.last_round = 88;
            current_node_state.consensus.last_message_kind = "commit".to_string();
            persist_state(&current_node_state).expect("current node state fixture should persist");

            let mut current_ledger_state = LedgerState::new();
            current_ledger_state.treasury_balance = 123_456_789;
            persist(&current_ledger_state).expect("current ledger state fixture should persist");

            refresh_runtime_metrics().expect("second refresh should succeed");

            let path = metrics_path().expect("metrics path should resolve");
            let raw = std::fs::read_to_string(&path)
                .expect("updated metrics snapshot should be readable");
            let snapshot: MetricsSnapshot = serde_json::from_str(&raw)
                .expect("updated metrics snapshot should decode");

            assert_eq!(
                snapshot.node_height, 88,
                "refresh must overwrite stale telemetry with the current canonical node height"
            );
            assert_eq!(
                snapshot.produced_blocks, 77,
                "refresh must overwrite stale telemetry with the current canonical produced block count"
            );
            assert_eq!(
                snapshot.treasury_balance, 123_456_789,
                "refresh must overwrite stale telemetry with the current canonical treasury balance"
            );
        });
    }

    #[test]
    fn refresh_runtime_metrics_reports_ledger_error_when_node_state_bootstraps() {
        with_test_home("refresh-runtime-metrics-missing-node", |_home| {
            let error = refresh_runtime_metrics()
                .expect_err("missing canonical node state must fail telemetry refresh");

            assert_eq!(
                error.code(),
                "AOXC-LED-001",
                "when node state is absent, refresh bootstraps runtime state and must surface the unchanged ledger-layer failure code"
            );
        });
    }

    #[test]
    fn refresh_runtime_metrics_fails_when_ledger_state_is_missing() {
        with_test_home("refresh-runtime-metrics-missing-ledger", |_home| {
            let mut node_state = NodeState::bootstrap();
            node_state.current_height = 9;
            node_state.produced_blocks = 9;
            persist_state(&node_state).expect("node state fixture should persist");

            let error = refresh_runtime_metrics()
                .expect_err("missing canonical ledger state must fail telemetry refresh");

            assert_eq!(
                error.code(),
                "AOXC-LED-001",
                "missing canonical ledger state must surface the ledger-layer failure code unchanged"
            );
        });
    }
}
