// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! Typed error boundary for the contract RPC surface.

use thiserror::Error;

use crate::types::RpcErrorResponse;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ContractRpcError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("validation failed: {0}")]
    ValidationFailed(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("unsupported operation: {0}")]
    UnsupportedOperation(String),
    #[error("internal mapping error: {0}")]
    InternalMappingError(String),
    #[error("registry error: {0}")]
    RegistryError(String),
    #[error("runtime resolution error: {0}")]
    RuntimeResolutionError(String),
}

impl ContractRpcError {
    pub fn http_status(&self) -> u16 {
        match self {
            Self::BadRequest(_) => 400,
            Self::NotFound(_) => 404,
            Self::Conflict(_) => 409,
            Self::ValidationFailed(_) => 422,
            Self::UnsupportedOperation(_) => 422,
            Self::InternalMappingError(_)
            | Self::RegistryError(_)
            | Self::RuntimeResolutionError(_) => 500,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::ValidationFailed(_) => "VALIDATION_FAILED",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::UnsupportedOperation(_) => "UNSUPPORTED_OPERATION",
            Self::InternalMappingError(_) => "INTERNAL_MAPPING_ERROR",
            Self::RegistryError(_) => "REGISTRY_ERROR",
            Self::RuntimeResolutionError(_) => "RUNTIME_RESOLUTION_ERROR",
        }
    }

    pub fn to_response(&self, request_id: String) -> RpcErrorResponse {
        RpcErrorResponse {
            code: self.code(),
            message: self.to_string(),
            retry_after_ms: None,
            request_id: Some(request_id),
            user_hint: Some(
                match self {
                    Self::BadRequest(_) => "Fix request shape, identifiers, or required fields.",
                    Self::ValidationFailed(_) => {
                        "Correct the contract manifest or descriptor and retry."
                    }
                    Self::NotFound(_) => "Verify the contract id exists before retrying.",
                    Self::Conflict(_) => {
                        "Check duplicate registration or lifecycle transition state."
                    }
                    Self::UnsupportedOperation(_) => {
                        "Enable the endpoint or use a supported contract operation."
                    }
                    Self::InternalMappingError(_) => {
                        "Inspect mapper coverage between domain and API models."
                    }
                    Self::RegistryError(_) => "Inspect registry state and lifecycle policies.",
                    Self::RuntimeResolutionError(_) => {
                        "Verify VM target and runtime configuration compatibility."
                    }
                }
                .to_string(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_mapping_matches_contract_expectations() {
        assert_eq!(ContractRpcError::BadRequest("x".into()).http_status(), 400);
        assert_eq!(ContractRpcError::NotFound("x".into()).http_status(), 404);
        assert_eq!(ContractRpcError::Conflict("x".into()).http_status(), 409);
        assert_eq!(
            ContractRpcError::ValidationFailed("x".into()).http_status(),
            422
        );
    }
}
