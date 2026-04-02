//! Shared error definitions for AOXCVM runtime components that already enforce
//! deterministic validation and constitutional policy boundaries.

use core::fmt;

/// Crate-level error type used by deterministic validation, policy enforcement,
/// and constitutional runtime guardrails.
///
/// Design notes:
/// - This type is intentionally compact and audit-friendly.
/// - Variants are structured around deterministic rejection provenance.
/// - New runtime/law layers should prefer explicit variants over ambiguous
///   string-only error multiplexing whenever the failure class is stable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AoxcvmError {
    /// Authentication envelope exceeded one of the configured structural limits.
    AuthLimitExceeded {
        /// Name of the violated limit.
        limit: &'static str,
        /// Actual observed value.
        got: usize,
        /// Maximum allowed value.
        max: usize,
    },

    /// Authentication envelope contained no signature entries.
    EmptySignatureSet,

    /// Signature metadata is malformed or structurally inconsistent.
    InvalidSignatureMetadata(&'static str),

    /// A deterministic authentication or policy rule was violated.
    PolicyViolation(&'static str),

    /// Runtime capability law rejected the requested operation.
    ///
    /// This variant is intended for execution-time or policy-time denials where
    /// a requested action is forbidden by the active capability model.
    CapabilityDenied(&'static str),

    /// Governance action attempted to use an invalid, unauthorized, or
    /// constitutionally forbidden governance lane.
    GovernanceLaneViolation(&'static str),

    /// Auth profile id does not exist in canonical registry state.
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

    /// Attempted to insert an already existing profile version.
    DuplicateAuthProfileVersion {
        /// Typed auth profile identifier as numeric wire value.
        profile_id: u32,
        /// Duplicate version value.
        version: u16,
    },
}

impl fmt::Display for AoxcvmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthLimitExceeded { limit, got, max } => {
                write!(f, "auth limit exceeded: {limit} (got {got}, max {max})")
            }
            Self::EmptySignatureSet => {
                write!(f, "auth envelope has no signatures")
            }
            Self::InvalidSignatureMetadata(msg) => {
                write!(f, "invalid signature metadata: {msg}")
            }
            Self::PolicyViolation(msg) => {
                write!(f, "authentication policy violation: {msg}")
            }
            Self::CapabilityDenied(msg) => {
                write!(f, "capability denied: {msg}")
            }
            Self::GovernanceLaneViolation(msg) => {
                write!(f, "governance lane violation: {msg}")
            }
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
        }
    }
}

impl std::error::Error for AoxcvmError {}

/// Result alias for error-producing AOXCVM runtime logic.
pub type AoxcvmResult<T> = Result<T, AoxcvmError>;
