use crate::{
    error::{AppError, ErrorCode},
    keys::manager::inspect_operator_key,
    node::state::NodeState,
    storage::{
        json_runtime::{runtime_state_json_path, JsonRuntimeStateStore},
        RuntimeStateStore,
    },
};
use std::path::PathBuf;

pub fn state_path() -> Result<PathBuf, AppError> {
    runtime_state_json_path()
}

fn runtime_store() -> impl RuntimeStateStore {
    JsonRuntimeStateStore
}

pub fn load_state() -> Result<NodeState, AppError> {
    runtime_store().load_state()
}

pub fn persist_state(state: &NodeState) -> Result<(), AppError> {
    runtime_store().persist_state(state)
}

pub fn bootstrap_state() -> Result<NodeState, AppError> {
    let mut state = NodeState::bootstrap();

    if let Ok(summary) = inspect_operator_key() {
        state.key_material.bundle_fingerprint = summary.bundle_fingerprint;
        state.key_material.operational_state = summary.operational_state;
        state.key_material.consensus_public_key_hex = summary.consensus_public_key;
        state.key_material.transport_public_key_hex = summary.transport_public_key;
    }

    state
        .validate()
        .map_err(|e| AppError::new(ErrorCode::NodeStateInvalid, e))?;

    persist_state(&state)?;
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::{bootstrap_state, load_state, persist_state, state_path};
    use crate::{error::ErrorCode, keys::manager::bootstrap_operator_key, node::state::NodeState};
    use std::{
        env, fs,
        path::PathBuf,
        sync::{Mutex, OnceLock},
        time::{SystemTime, UNIX_EPOCH},
    };

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn unique_test_home(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        env::temp_dir().join(format!("aoxcmd-{label}-{nanos}"))
    }

    #[test]
    fn bootstrap_state_persists_default_node_state() {
        let _guard = env_lock().lock().expect("test env mutex must lock");
        let home = unique_test_home("bootstrap-state");
        env::set_var("AOXC_HOME", &home);

        let bootstrapped = bootstrap_state().expect("bootstrap should persist node state");
        let reloaded = load_state().expect("bootstrapped state should load");
        let expected_path = home.join("runtime").join("node_state.json");

        assert_eq!(
            state_path().expect("state path should resolve"),
            expected_path
        );
        assert!(bootstrapped.initialized);
        assert_eq!(reloaded.consensus.last_message_kind, "bootstrap");
        assert_eq!(reloaded.current_height, 0);

        let _ = fs::remove_dir_all(&home);
        env::remove_var("AOXC_HOME");
    }

    #[test]
    fn bootstrap_state_enriches_key_material_when_operator_key_exists() {
        let _guard = env_lock().lock().expect("test env mutex must lock");
        let home = unique_test_home("bootstrap-state-keys");
        env::set_var("AOXC_HOME", &home);

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

        let _ = fs::remove_dir_all(&home);
        env::remove_var("AOXC_HOME");
    }

    #[test]
    fn persist_state_round_trips_custom_consensus_snapshot() {
        let _guard = env_lock().lock().expect("test env mutex must lock");
        let home = unique_test_home("persist-state");
        env::set_var("AOXC_HOME", &home);

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

        let _ = fs::remove_dir_all(&home);
        env::remove_var("AOXC_HOME");
    }

    #[test]
    fn load_state_rejects_invalid_semantic_payload() {
        let _guard = env_lock().lock().expect("test env mutex must lock");
        let home = unique_test_home("invalid-state");
        env::set_var("AOXC_HOME", &home);

        let mut state = NodeState::bootstrap();
        state.produced_blocks = 5;
        state.current_height = 1;
        persist_state(&state).expect("invalid semantic payload should still persist for test");

        let error = load_state().expect_err("invalid semantic payload should be rejected");
        assert_eq!(error.code(), ErrorCode::NodeStateInvalid.as_str());

        let _ = fs::remove_dir_all(&home);
        env::remove_var("AOXC_HOME");
    }
}
