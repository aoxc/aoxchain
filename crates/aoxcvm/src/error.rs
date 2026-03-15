use core::fmt;

/// Canonical runtime error surface shared by all execution lanes.
///
/// The error model is intentionally small and stable. Lane-specific
/// failures are normalized into this type so the host can reason about
/// them deterministically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AovmError {
    InvalidTransaction(&'static str),
    GasExhausted,
    StateAccessViolation(&'static str),
    NotFound(&'static str),
    DecodeError(&'static str),
    ExecutionFailure(String),
    UnsupportedOperation(&'static str),
}

impl fmt::Display for AovmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransaction(msg) => write!(f, "invalid transaction: {msg}"),
            Self::GasExhausted => write!(f, "gas exhausted"),
            Self::StateAccessViolation(msg) => write!(f, "state access violation: {msg}"),
            Self::NotFound(msg) => write!(f, "not found: {msg}"),
            Self::DecodeError(msg) => write!(f, "decode error: {msg}"),
            Self::ExecutionFailure(msg) => write!(f, "execution failure: {msg}"),
            Self::UnsupportedOperation(msg) => write!(f, "unsupported operation: {msg}"),
        }
    }
}

impl std::error::Error for AovmError {}
