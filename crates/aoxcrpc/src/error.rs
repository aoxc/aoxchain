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
