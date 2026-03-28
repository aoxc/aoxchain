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

/// Supported AOXC runtime-state persistence backends.
///
/// Backend policy:
/// - `Json` exists for legacy compatibility and migration workflows.
/// - `Redb` is the canonical structured storage backend for active AOXC runtime
///   persistence surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeStateBackend {
    Json,
    Redb,
}

impl RuntimeStateBackend {
    /// Parses a runtime-state backend selector from operator or configuration input.
    ///
    /// Parsing policy:
    /// - Leading and trailing whitespace are ignored.
    /// - Input is matched case-insensitively.
    /// - Only canonical backend names are accepted.
    ///
    /// Accepted values:
    /// - `json`
    /// - `redb`
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

    /// Returns the canonical lowercase string form of the backend.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Redb => "redb",
        }
    }
}

/// Canonical AOXC runtime-state storage contract.
///
/// Trait responsibilities:
/// - `load_state()` must return only semantically valid `NodeState` payloads.
/// - `persist_state()` must reject semantically invalid `NodeState` payloads
///   before committing them.
/// - Implementations are responsible for storage-specific encode/decode and
///   durability behavior, while preserving the same semantic runtime-state
///   contract at the boundary.
pub trait RuntimeStateStore {
    fn load_state(&self) -> Result<NodeState, AppError>;
    fn persist_state(&self, state: &NodeState) -> Result<(), AppError>;
}

#[cfg(test)]
mod tests {
    use super::RuntimeStateBackend;
    use crate::error::ErrorCode;

    #[test]
    fn runtime_state_backend_parse_accepts_json() {
        let backend =
            RuntimeStateBackend::parse("json").expect("json backend should parse successfully");

        assert_eq!(backend, RuntimeStateBackend::Json);
        assert_eq!(backend.as_str(), "json");
    }

    #[test]
    fn runtime_state_backend_parse_accepts_redb_case_insensitively() {
        let backend =
            RuntimeStateBackend::parse(" ReDb ").expect("redb backend should parse successfully");

        assert_eq!(backend, RuntimeStateBackend::Redb);
        assert_eq!(backend.as_str(), "redb");
    }

    #[test]
    fn runtime_state_backend_parse_rejects_unsupported_values() {
        let error =
            RuntimeStateBackend::parse("sqlite").expect_err("unsupported backend must be rejected");

        assert_eq!(error.code(), ErrorCode::ConfigInvalid.as_str());
    }
}
