use thiserror::Error;

/// Canonical AOXCVM error domain.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AoxcvmError {
    #[error("transaction rejected during admission: {0}")]
    AdmissionRejected(&'static str),
    #[error("authorization failed: {0}")]
    AuthorizationFailed(&'static str),
    #[error("bytecode verification failed: {0}")]
    VerificationFailed(&'static str),
    #[error("policy violation: {0}")]
    PolicyViolation(&'static str),
    #[error("capability required: {0}")]
    MissingCapability(&'static str),
    #[error("determinism breach: {0}")]
    DeterminismBreach(&'static str),
    #[error("resource limit exceeded: {0}")]
    LimitExceeded(&'static str),
}
