// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    error::{AppError, ErrorCode},
    keys::manager::inspect_operator_key,
    node::state::NodeState,
    storage::{
        RuntimeStateStore,
        json_runtime::JsonRuntimeStateStore,
        redb_chain::{append_chain_log, load_node_state, main_redb_path, persist_node_state},
    },
};
use std::path::PathBuf;

/// Returns the canonical runtime state database path.
///
/// Canonical storage policy:
/// - Runtime state is persisted in the primary redb-backed state store.
/// - Callers must treat this path as the authoritative runtime state location.
pub fn state_path() -> Result<PathBuf, AppError> {
    main_redb_path()
}

/// Loads runtime state from the canonical redb store.
///
/// Fallback policy:
/// - The redb store remains the authoritative runtime source of truth.
/// - If the canonical redb file does not yet exist, a first-run bootstrap state
///   is created, persisted, and returned.
/// - If canonical redb loading fails while the canonical store already exists,
///   a one-time legacy JSON migration is attempted.
/// - Migrated state is validated and re-persisted into redb before it is returned.
///
/// Security rationale:
/// - Bootstrap is allowed only when canonical storage is physically absent.
/// - Existing-but-invalid canonical state must never be silently overwritten.
/// - Migration must not mask semantically invalid payloads.
/// - Returned state must always satisfy `NodeState::validate()`.
pub fn load_state() -> Result<NodeState, AppError> {
    match load_node_state() {
        Ok(state) => {
            validate_state(&state)?;
            Ok(state)
        }
        Err(primary_error) => recover_state(primary_error),
    }
}

/// Persists runtime state to the canonical redb store.
///
/// Audit guarantees:
/// - Semantic validation is enforced before persistence.
/// - Successful persistence emits a runtime chain-log breadcrumb.
/// - The persistence log is best-effort and must never override the primary
///   persistence outcome.
pub fn persist_state(state: &NodeState) -> Result<(), AppError> {
    validate_state(state)?;
    persist_node_state(state)?;

    let _ = append_chain_log(
        "runtime",
        "persist_node_state",
        &format!(
            "height={} produced_blocks={} message_kind={}",
            state.current_height, state.produced_blocks, state.consensus.last_message_kind
        ),
    );

    Ok(())
}

/// Bootstraps a canonical runtime state document.
///
/// Bootstrap behavior:
/// - Starts from `NodeState::bootstrap()`.
/// - Enriches key-material snapshot fields when validated operator key material
///   already exists in the active AOXC home.
/// - Validates and persists the resulting state before returning it.
pub fn bootstrap_state() -> Result<NodeState, AppError> {
    let mut state = NodeState::bootstrap();

    if let Ok(summary) = inspect_operator_key() {
        state.key_material.bundle_fingerprint = summary.bundle_fingerprint;
        state.key_material.operational_state = summary.operational_state;
        state.key_material.consensus_public_key_hex = summary.consensus_public_key;
        state.key_material.transport_public_key_hex = summary.transport_public_key;
    }

    validate_state(&state)?;
    persist_state(&state)?;

    let _ = append_chain_log("runtime", "bootstrap_state", "node state bootstrapped");
    Ok(state)
}

/// Validates runtime state under the canonical AOXC node-state contract.
fn validate_state(state: &NodeState) -> Result<(), AppError> {
    state
        .validate()
        .map_err(|error| AppError::new(ErrorCode::NodeStateInvalid, error))
}

/// Recovers runtime state after canonical redb loading failed.
///
/// Recovery order:
/// - If the canonical redb file is absent from disk, first-run bootstrap is allowed.
/// - Otherwise a one-time legacy JSON migration is attempted.
/// - If migration is unavailable, the original canonical node error is returned unchanged.
///
/// Security rationale:
/// - Canonical node failures must remain observable when recovery is not possible.
/// - Bootstrap must never replace an already-existing canonical store.
fn recover_state(primary_error: AppError) -> Result<NodeState, AppError> {
    if should_bootstrap_from_filesystem()? {
        return bootstrap_state();
    }

    try_load_legacy_state(primary_error)
}

/// Returns whether the canonical runtime redb store is physically absent.
///
/// Decision rule:
/// - Bootstrap is permitted only when the canonical redb file does not exist.
/// - Filesystem resolution failures must surface explicitly and must not be
///   reinterpreted as bootstrap-eligible conditions.
fn should_bootstrap_from_filesystem() -> Result<bool, AppError> {
    let path = state_path()?;
    Ok(!path.exists())
}

/// Attempts a one-time legacy JSON runtime-state migration.
///
/// Migration contract:
/// - Migration is attempted only after the canonical redb load path failed.
/// - Legacy JSON must itself load successfully and pass semantic validation.
/// - The migrated state is persisted into redb and logged before being returned.
/// - If legacy JSON is unavailable or invalid, the original redb load error is preserved.
fn try_load_legacy_state(primary_error: AppError) -> Result<NodeState, AppError> {
    let legacy_store = JsonRuntimeStateStore;

    let legacy_state = match legacy_store.load_state() {
        Ok(state) => state,
        Err(_) => return Err(primary_error),
    };

    validate_state(&legacy_state)?;
    persist_node_state(&legacy_state)?;

    let _ = append_chain_log(
        "runtime",
        "migrate_json_to_redb",
        "legacy node_state.json migrated to canonical redb state",
    );

    Ok(legacy_state)
}

#[cfg(test)]
mod tests {
    use super::{bootstrap_state, load_state, persist_state, state_path};
    use crate::{
        error::ErrorCode,
        keys::manager::bootstrap_operator_key,
        node::state::NodeState,
        test_support::{AoxcHomeGuard, TestHome, aoxc_home_test_lock},
    };

    /// Executes a lifecycle test inside a process-safe isolated AOXC home.
    ///
    /// Isolation policy:
    /// - Reuses the shared crate-level AOXC home lock so every test that mutates
    ///   `AOXC_HOME` participates in the same serialization domain.
    /// - Reuses the shared RAII home guard so environment restoration occurs
    ///   even when a test fails or panics.
    /// - Reuses the shared `TestHome` helper so disposable state remains under
    ///   the canonical AOXC test namespace.
    fn with_test_home<T>(label: &str, test: impl FnOnce(&TestHome) -> T) -> T {
        let _lock = aoxc_home_test_lock();
        let home = TestHome::new(label).expect("test home should be created");
        let _guard = AoxcHomeGuard::install(&_lock, home.path());
        test(&home)
    }

    #[test]
    fn bootstrap_state_persists_default_node_state() {
        with_test_home("lifecycle-bootstrap-state", |home| {
            let bootstrapped = bootstrap_state().expect("bootstrap should persist node state");
            let reloaded = load_state().expect("bootstrapped state should load");
            let expected_path = home.path().join("runtime").join("db").join("main.redb");

            assert_eq!(
                state_path().expect("state path should resolve"),
                expected_path
            );
            assert!(bootstrapped.initialized);
            assert_eq!(reloaded.consensus.last_message_kind, "bootstrap");
            assert_eq!(reloaded.current_height, 0);
        });
    }

    #[test]
    fn bootstrap_state_enriches_key_material_when_operator_key_exists() {
        with_test_home("lifecycle-bootstrap-state-keys", |_home| {
            bootstrap_operator_key("validator-01", "devnet", "StrongPass123!")
                .expect("operator key bootstrap should succeed");

            let state = bootstrap_state().expect("bootstrap should persist enriched node state");

            assert!(!state.key_material.bundle_fingerprint.is_empty());
            assert_eq!(state.key_material.operational_state, "active");
            assert!(!state.key_material.consensus_public_key_hex.is_empty());
            assert!(!state.key_material.transport_public_key_hex.is_empty());

            let reloaded = load_state().expect("enriched node state should load");
            assert_eq!(
                reloaded.key_material.bundle_fingerprint,
                state.key_material.bundle_fingerprint
            );
            assert_eq!(
                reloaded.key_material.consensus_public_key_hex,
                state.key_material.consensus_public_key_hex
            );
            assert_eq!(
                reloaded.key_material.transport_public_key_hex,
                state.key_material.transport_public_key_hex
            );
        });
    }

    #[test]
    fn persist_state_round_trips_custom_consensus_snapshot() {
        with_test_home("lifecycle-persist-state", |_home| {
            let mut state = NodeState::bootstrap();
            state.current_height = 9;
            state.produced_blocks = 9;
            state.last_tx = "smoke".to_string();
            state.consensus.last_round = 4;
            state.consensus.last_message_kind = "block_proposal".to_string();

            persist_state(&state).expect("custom state should persist");
            let reloaded = load_state().expect("custom state should reload");

            assert_eq!(reloaded.current_height, 9);
            assert_eq!(reloaded.produced_blocks, 9);
            assert_eq!(reloaded.last_tx, "smoke");
            assert_eq!(reloaded.consensus.last_round, 4);
            assert_eq!(reloaded.consensus.last_message_kind, "block_proposal");
        });
    }

    #[test]
    fn persist_state_rejects_invalid_semantic_payload() {
        with_test_home("lifecycle-invalid-state", |_home| {
            let mut state = NodeState::bootstrap();
            state.produced_blocks = 5;
            state.current_height = 1;

            let error = persist_state(&state)
                .expect_err("invalid semantic payload must be rejected before persistence");

            assert_eq!(error.code(), ErrorCode::NodeStateInvalid.as_str());
        });
    }

    #[test]
    fn load_state_bootstraps_on_first_run_when_store_is_empty() {
        with_test_home("lifecycle-first-run-bootstrap", |_home| {
            let loaded = load_state().expect("first-run load should bootstrap state");

            assert!(loaded.initialized);
            assert_eq!(loaded.current_height, 0);
            assert_eq!(loaded.produced_blocks, 0);
            assert_eq!(loaded.consensus.last_message_kind, "bootstrap");

            let reloaded = load_state().expect("restarted load should return persisted state");
            assert_eq!(reloaded, loaded);
        });
    }
}
