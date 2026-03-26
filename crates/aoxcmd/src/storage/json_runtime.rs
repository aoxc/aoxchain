use crate::{
    data_home::{read_file, resolve_home, write_file},
    error::{AppError, ErrorCode},
    node::state::NodeState,
    storage::RuntimeStateStore,
};
use std::path::PathBuf;

pub fn runtime_state_json_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("runtime").join("node_state.json"))
}

#[derive(Debug, Default)]
pub struct JsonRuntimeStateStore;

impl RuntimeStateStore for JsonRuntimeStateStore {
    fn load_state(&self) -> Result<NodeState, AppError> {
        let path = runtime_state_json_path()?;
        let raw = read_file(&path).map_err(|_| {
            AppError::new(
                ErrorCode::NodeStateInvalid,
                format!("Node state file is missing at {}", path.display()),
            )
        })?;

        let state: NodeState = serde_json::from_str(&raw).map_err(|e| {
            AppError::with_source(ErrorCode::NodeStateInvalid, "Failed to parse node state", e)
        })?;

        state
            .validate()
            .map_err(|e| AppError::new(ErrorCode::NodeStateInvalid, e))?;

        Ok(state)
    }

    fn persist_state(&self, state: &NodeState) -> Result<(), AppError> {
        let path = runtime_state_json_path()?;
        let content = serde_json::to_string_pretty(state).map_err(|e| {
            AppError::with_source(
                ErrorCode::OutputEncodingFailed,
                "Failed to encode node state",
                e,
            )
        })?;

        write_file(&path, &content)
    }
}
