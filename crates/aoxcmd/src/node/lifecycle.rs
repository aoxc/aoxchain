use crate::{
    data_home::{read_file, resolve_home, write_file},
    error::{AppError, ErrorCode},
    node::state::NodeState,
};
use std::path::PathBuf;

pub fn state_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("runtime").join("node_state.json"))
}

pub fn load_state() -> Result<NodeState, AppError> {
    let path = state_path()?;
    let raw = read_file(&path).map_err(|_| {
        AppError::new(
            ErrorCode::NodeStateInvalid,
            format!("Node state file is missing at {}", path.display()),
        )
    })?;
    serde_json::from_str(&raw).map_err(|e| {
        AppError::with_source(ErrorCode::NodeStateInvalid, "Failed to parse node state", e)
    })
}

pub fn persist_state(state: &NodeState) -> Result<(), AppError> {
    let path = state_path()?;
    let content = serde_json::to_string_pretty(state).map_err(|e| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode node state",
            e,
        )
    })?;
    write_file(&path, &content)
}

pub fn bootstrap_state() -> Result<NodeState, AppError> {
    let state = NodeState::bootstrap();
    persist_state(&state)?;
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::{bootstrap_state, load_state, persist_state, state_path};
    use crate::node::state::NodeState;
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
}
