use thiserror::Error;

/// Domain error type for the AOXC AI runtime.
///
/// Error messages are intentionally operator-oriented. They are expected to be
/// suitable for logs, audits, and incident analysis.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AiError {
    #[error("I/O failure at '{path}': {reason}")]
    Io { path: String, reason: String },

    #[error("manifest parse failure: {0}")]
    ManifestParse(String),

    #[error("manifest validation failure: {0}")]
    ManifestValidation(String),

    #[error("model not found: {0}")]
    ModelNotFound(String),

    #[error("task binding not found: {0}")]
    BindingNotFound(String),

    #[error("unsupported backend type: {0}")]
    UnsupportedBackend(String),

    #[error("backend execution failed: {0}")]
    BackendFailure(String),

    #[error("backend unreachable: {0}")]
    BackendUnreachable(String),

    #[error("backend timed out: {0}")]
    BackendTimeout(String),

    #[error("backend schema failure: {0}")]
    BackendSchema(String),

    #[error("policy evaluation failed: {0}")]
    PolicyFailure(String),

    #[error("missing environment variable: {0}")]
    MissingEnvironment(String),

    #[error("HTTP failure: {0}")]
    Http(String),

    #[error("JSON failure: {0}")]
    Json(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),
}

impl AiError {
    /// Returns `true` when the error should be treated as a remote backend
    /// reachability failure for fallback policy purposes.
    #[must_use]
    pub fn is_backend_unreachable(&self) -> bool {
        matches!(self, Self::BackendUnreachable(_) | Self::Http(_))
    }

    /// Returns `true` when the error should be treated as a timeout for
    /// fallback policy purposes.
    #[must_use]
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::BackendTimeout(_))
    }

    /// Returns `true` when the error should be treated as a backend schema
    /// failure for fallback policy purposes.
    #[must_use]
    pub fn is_schema_error(&self) -> bool {
        matches!(self, Self::BackendSchema(_) | Self::Json(_))
    }
}
