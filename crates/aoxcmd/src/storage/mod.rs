// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

pub mod json_runtime;
pub mod redb_chain;
pub mod redb_runtime;

use crate::{
    error::{AppError, ErrorCode},
    node::state::NodeState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeStateBackend {
    Json,
    Redb,
}

impl RuntimeStateBackend {
    pub fn parse(value: &str) -> Result<Self, AppError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "redb" => Ok(Self::Redb),
            other => Err(AppError::new(
                ErrorCode::ConfigInvalid,
                format!("Unsupported runtime state backend `{other}`"),
            )),
        }
    }
}

pub trait RuntimeStateStore {
    fn load_state(&self) -> Result<NodeState, AppError>;
    fn persist_state(&self, state: &NodeState) -> Result<(), AppError>;
}
