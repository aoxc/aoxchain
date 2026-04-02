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
    /// Auth profile id does not exist in canonical registry.
    UnknownAuthProfile {
        /// Typed auth profile identifier as numeric wire value.
        profile_id: u32,
    },
    /// Auth profile version does not exist for a known profile id.
    UnknownAuthProfileVersion {
        /// Typed auth profile identifier as numeric wire value.
        profile_id: u32,
        /// Requested profile version.
        version: u16,
    },
    /// Attempted to reinsert an existing profile version.
    DuplicateAuthProfileVersion {
        /// Typed auth profile identifier as numeric wire value.
        profile_id: u32,
        /// Duplicate version value.
        version: u16,
    },
    /// Governance lane is not valid for requested action.
    GovernanceLaneViolation(&'static str),
    /// Runtime capability gate denied operation.
    CapabilityDenied(&'static str),
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
            Self::UnknownAuthProfile { profile_id } => {
                write!(f, "unknown auth profile id: {profile_id}")
            }
            Self::UnknownAuthProfileVersion {
                profile_id,
                version,
            } => write!(
                f,
                "unknown auth profile version: id={profile_id}, version={version}",
            ),
            Self::DuplicateAuthProfileVersion {
                profile_id,
                version,
            } => write!(
                f,
                "duplicate auth profile version: id={profile_id}, version={version}",
            ),
            Self::GovernanceLaneViolation(msg) => {
                write!(f, "governance lane violation: {msg}")
            }
            Self::CapabilityDenied(msg) => write!(f, "capability denied: {msg}"),
        }
    }
}

impl std::error::Error for AoxcvmError {}

/// Result alias for error-producing AOXCVM scaffold logic.
pub type AoxcvmResult<T> = Result<T, AoxcvmError>;
