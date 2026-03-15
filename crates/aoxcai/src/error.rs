use thiserror::Error;

/// Represents all domain errors emitted by the AOXC AI runtime.
#[derive(Debug, Error)]
pub enum AiError {
    /// Returned when a supplied path cannot be read from storage.
    #[error("I/O failure at '{path}': {reason}")]
    Io { path: String, reason: String },

    /// Returned when a YAML manifest cannot be deserialized.
    #[error("manifest parse failure: {0}")]
    ManifestParse(String),

    /// Returned when a manifest violates runtime validation rules.
    #[error("manifest validation failure: {0}")]
    ManifestValidation(String),

    /// Returned when a requested model identifier is not present in the registry.
    #[error("model not found: {0}")]
    ModelNotFound(String),

    /// Returned when no model binding is available for the requested task.
    #[error("no model binding found for task '{0}'")]
    BindingNotFound(String),

    /// Returned when an unsupported backend type is requested.
    #[error("unsupported backend type: {0}")]
    UnsupportedBackend(String),

    /// Returned when backend execution fails.
    #[error("backend execution failed: {0}")]
    BackendFailure(String),

    /// Returned when policy evaluation fails.
    #[error("policy evaluation failed: {0}")]
    PolicyFailure(String),

    /// Returned when a required environment variable is not available.
    #[error("missing environment variable: {0}")]
    MissingEnvironment(String),

    /// Returned when an outbound HTTP call fails.
    #[error("HTTP failure: {0}")]
    Http(String),

    /// Returned when JSON serialization or deserialization fails.
    #[error("JSON failure: {0}")]
    Json(String),

    /// Returned when the engine receives malformed input.
    #[error("invalid input: {0}")]
    InvalidInput(String),
}
