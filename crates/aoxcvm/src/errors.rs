//! Shared error definitions for AOXCVM scaffolds that already have executable logic.

use core::fmt;

/// Crate-level error type used by deterministic validation paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AoxcvmError {
    /// Authentication envelope exceeded one of configured limits.
    AuthLimitExceeded {
        /// Name of the violated limit.
        limit: &'static str,
        /// Actual observed value.
        got: usize,
        /// Maximum allowed value.
        max: usize,
    },
    /// Envelope contained no signatures.
    EmptySignatureSet,
    /// Signature metadata is malformed.
    InvalidSignatureMetadata(&'static str),
    /// Signature set fails active policy profile checks.
    PolicyViolation(&'static str),
}

impl fmt::Display for AoxcvmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthLimitExceeded { limit, got, max } => {
                write!(f, "auth limit exceeded: {limit} (got {got}, max {max})")
            }
            Self::EmptySignatureSet => write!(f, "auth envelope has no signatures"),
            Self::InvalidSignatureMetadata(msg) => {
                write!(f, "invalid signature metadata: {msg}")
            }
            Self::PolicyViolation(msg) => write!(f, "authentication policy violation: {msg}"),
        }
    }
}

impl std::error::Error for AoxcvmError {}

/// Result alias for error-producing AOXCVM scaffold logic.
pub type AoxcvmResult<T> = Result<T, AoxcvmError>;
