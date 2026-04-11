// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    data_home::{read_file, resolve_home, write_file},
    error::{AppError, ErrorCode},
    node::state::NodeState,
    storage::RuntimeStateStore,
};
use std::path::PathBuf;

/// Returns the canonical legacy JSON runtime-state path.
///
/// Canonical path policy:
/// - Legacy JSON runtime state is stored at:
///   `<AOXC_HOME>/runtime/node_state.json`.
///
/// Operational note:
/// - This path exists for backward compatibility and migration support.
/// - The authoritative runtime state surface is the canonical redb-backed
///   storage layer.
pub fn runtime_state_json_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("runtime").join("node_state.json"))
}

/// Legacy JSON runtime-state store.
///
/// Design intent:
/// - Preserve compatibility with historical JSON runtime state payloads.
/// - Provide a narrow migration/read-write surface for legacy tooling.
/// - Keep semantic validation at the storage boundary.
#[derive(Debug, Default)]
pub struct JsonRuntimeStateStore;

impl RuntimeStateStore for JsonRuntimeStateStore {
    /// Loads and validates runtime state from the legacy JSON path.
    ///
    /// Error mapping policy:
    /// - Missing legacy JSON maps to `NodeStateInvalid` with an explicit
    ///   missing-file message.
    /// - Other filesystem read failures map to `FilesystemIoFailed`.
    /// - Decode and semantic validation failures map to `NodeStateInvalid`.
    fn load_state(&self) -> Result<NodeState, AppError> {
        let path = runtime_state_json_path()?;
        let raw = read_file(&path).map_err(|error| {
            if error.has_io_error_kind(std::io::ErrorKind::NotFound) {
                AppError::new(
                    ErrorCode::NodeStateInvalid,
                    format!("Legacy node state file is missing at {}", path.display()),
                )
            } else {
                AppError::with_source(
                    ErrorCode::FilesystemIoFailed,
                    format!("Failed to read legacy node state from {}", path.display()),
                    error,
                )
            }
        })?;

        let state: NodeState = serde_json::from_str(&raw).map_err(|error| {
            AppError::with_source(
                ErrorCode::NodeStateInvalid,
                format!("Failed to parse legacy node state at {}", path.display()),
                error,
            )
        })?;

        state
            .validate()
            .map_err(|error| AppError::new(ErrorCode::NodeStateInvalid, error.to_string()))?;

        Ok(state)
    }

    /// Persists validated runtime state to the legacy JSON path.
    ///
    /// Validation policy:
    /// - Semantic validation is enforced before serialization.
    /// - Only validated runtime state is encoded and written.
    fn persist_state(&self, state: &NodeState) -> Result<(), AppError> {
        state
            .validate()
            .map_err(|error| AppError::new(ErrorCode::NodeStateInvalid, error.to_string()))?;

        let path = runtime_state_json_path()?;
        let content = serde_json::to_string_pretty(state).map_err(|error| {
            AppError::with_source(
                ErrorCode::OutputEncodingFailed,
                format!("Failed to encode legacy node state for {}", path.display()),
                error,
            )
        })?;

        write_file(&path, &content)
    }
}

#[cfg(test)]
mod tests {
    use super::{JsonRuntimeStateStore, runtime_state_json_path};
    use crate::{
        error::ErrorCode,
        node::state::NodeState,
        storage::RuntimeStateStore,
        test_support::{AoxcHomeGuard, TestHome, aoxc_home_test_lock},
    };

    fn with_test_home<T>(label: &str, test: impl FnOnce(&TestHome) -> T) -> T {
        let _lock = aoxc_home_test_lock();
        let home = TestHome::new(label).expect("test home should be created");
        let _guard = AoxcHomeGuard::install(&_lock, home.path());
        test(&home)
    }

    #[test]
    fn runtime_state_json_path_resolves_inside_active_test_home() {
        with_test_home("json-runtime-path", |home| {
            let path = runtime_state_json_path().expect("runtime json path must resolve");

            assert_eq!(path, home.path().join("runtime").join("node_state.json"));
        });
    }

    #[test]
    fn persist_and_reload_legacy_runtime_state_round_trips() {
        with_test_home("json-runtime-roundtrip", |_home| {
            let store = JsonRuntimeStateStore;
            let mut state = NodeState::bootstrap();
            state.current_height = 4;
            state.produced_blocks = 4;
            state.last_tx = "smoke".to_string();
            state.consensus.last_round = 4;
            state.consensus.last_message_kind = "block_proposal".to_string();

            store
                .persist_state(&state)
                .expect("legacy runtime state should persist");

            let reloaded = store
                .load_state()
                .expect("persisted legacy runtime state should reload");

            assert_eq!(reloaded.current_height, 4);
            assert_eq!(reloaded.produced_blocks, 4);
            assert_eq!(reloaded.last_tx, "smoke");
            assert_eq!(reloaded.consensus.last_round, 4);
            assert_eq!(reloaded.consensus.last_message_kind, "block_proposal");
        });
    }

    #[test]
    fn load_state_rejects_invalid_json_payload() {
        with_test_home("json-runtime-invalid-json", |_home| {
            let path = runtime_state_json_path().expect("runtime json path must resolve");
            crate::data_home::write_file(&path, "{ invalid json")
                .expect("invalid fixture should be written");

            let store = JsonRuntimeStateStore;
            let error = store
                .load_state()
                .expect_err("invalid legacy runtime state must fail");

            assert_eq!(error.code(), ErrorCode::NodeStateInvalid.as_str());
        });
    }

    #[test]
    fn persist_state_rejects_invalid_semantic_payload() {
        with_test_home("json-runtime-invalid-semantic", |_home| {
            let store = JsonRuntimeStateStore;
            let mut state = NodeState::bootstrap();
            state.current_height = 1;
            state.produced_blocks = 5;

            let error = store
                .persist_state(&state)
                .expect_err("invalid semantic runtime state must fail");

            assert_eq!(error.code(), ErrorCode::NodeStateInvalid.as_str());
        });
    }
}
