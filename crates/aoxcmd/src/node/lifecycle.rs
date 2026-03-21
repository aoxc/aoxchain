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
    serde_json::from_str(&raw)
        .map_err(|e| AppError::with_source(ErrorCode::NodeStateInvalid, "Failed to parse node state", e))
}

pub fn persist_state(state: &NodeState) -> Result<(), AppError> {
    let path = state_path()?;
    let content = serde_json::to_string_pretty(state)
        .map_err(|e| AppError::with_source(ErrorCode::OutputEncodingFailed, "Failed to encode node state", e))?;
    write_file(&path, &content)
}

pub fn bootstrap_state() -> Result<NodeState, AppError> {
    let state = NodeState::bootstrap();
    persist_state(&state)?;
    Ok(state)
}
