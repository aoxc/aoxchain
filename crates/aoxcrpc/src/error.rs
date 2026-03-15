use crate::types::RpcErrorResponse;
use thiserror::Error;

/// RPC subsystem errors.
#[derive(Debug, Error)]
pub enum RpcError {
    #[error("INVALID_REQUEST")]
    InvalidRequest,
    #[error("METHOD_NOT_FOUND")]
    MethodNotFound,
    #[error("RATE_LIMIT_EXCEEDED: retry_after_ms={retry_after_ms}")]
    RateLimitExceeded { retry_after_ms: u64 },
    #[error("MTLS_AUTH_FAILED")]
    MtlsAuthFailed,
    #[error("ZKP_VALIDATION_FAILED: {0}")]
    ZkpValidationFailed(String),
    #[error("INTERNAL_ERROR")]
    InternalError,
}

impl RpcError {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest => "INVALID_REQUEST",
            Self::MethodNotFound => "METHOD_NOT_FOUND",
            Self::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
            Self::MtlsAuthFailed => "MTLS_AUTH_FAILED",
            Self::ZkpValidationFailed(_) => "ZKP_VALIDATION_FAILED",
            Self::InternalError => "INTERNAL_ERROR",
        }
    }

    #[must_use]
    pub fn retry_after_ms(&self) -> Option<u64> {
        match self {
            Self::RateLimitExceeded { retry_after_ms } => Some(*retry_after_ms),
            _ => None,
        }
    }

    #[must_use]
    pub fn to_response(&self, request_id: Option<String>) -> RpcErrorResponse {
        RpcErrorResponse {
            code: self.code(),
            message: self.to_string(),
            retry_after_ms: self.retry_after_ms(),
            request_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limit_error_contains_retry_after_in_response() {
        let err = RpcError::RateLimitExceeded {
            retry_after_ms: 250,
        };
        let response = err.to_response(Some("req-42".to_string()));

        assert_eq!(response.code, "RATE_LIMIT_EXCEEDED");
        assert_eq!(response.retry_after_ms, Some(250));
        assert_eq!(response.request_id.as_deref(), Some("req-42"));
    }
}
