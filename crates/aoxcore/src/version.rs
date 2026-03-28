// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::Serialize;
use sha3::{Digest, Sha3_256};
use std::fmt;

/// Canonical AOXC core identity schema generation.
///
/// Rationale:
/// - the struct below intentionally remains small and stable,
/// - the externally visible protocol lines already carry explicit `V1` markers,
/// - a dedicated schema constant preserves future migration space without forcing
///   unnecessary type renaming at every internal call site.
pub const AOXC_CORE_IDENTITY_SCHEMA_VERSION: u8 = 1;

/// Canonical AOXC core identity name.
///
/// This value identifies the root canonical AOXC protocol surface.
pub const AOXC_CANONICAL_CORE_NAME: &str = "AOXC Canonical Core";

/// Canonical AOXC core line.
///
/// This line is intended to remain stable and externally recognizable.
pub const AOXC_CORE_LINE: &str = "AOXC-CORE-V1";

/// Canonical AOXC block format line.
///
/// The current value explicitly signals that the block format contract remains
/// in draft state even though its generation marker is versioned.
pub const AOXC_BLOCK_FORMAT_LINE: &str = "AOXC-BLOCK-FMT-V1-draft";

/// Canonical AOXC genesis authority line.
///
/// The current value explicitly signals that the genesis-authority surface
/// remains in draft state even though its generation marker is versioned.
pub const AOXC_GENESIS_AUTH_LINE: &str = "AOXC-GENESIS-AUTH-V1-draft";

/// Domain separator used for canonical core-identity commitment hashing.
///
/// Security rationale:
/// - explicit domain separation prevents accidental cross-protocol digest reuse,
/// - SHA3-256 provides a conservative hash choice aligned with modern
///   post-quantum-aware engineering preferences for hash-based commitments.
const AOXC_CORE_IDENTITY_DOMAIN: &[u8] = b"AOXC/CORE_IDENTITY/V1";

/// Canonical AOXC core identity contract.
///
/// Design properties:
/// - small immutable public surface,
/// - stable serialization shape,
/// - suitable for display, diagnostics, capability negotiation, and
///   protocol-line commitment generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct CoreIdentity {
    pub name: &'static str,
    pub line: &'static str,
    pub block_format: &'static str,
    pub genesis_authority: &'static str,
}

/// Canonical error surface for core-identity validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CoreIdentityError {
    InvalidName,
    InvalidLine,
    InvalidBlockFormat,
    InvalidGenesisAuthority,
}

impl CoreIdentityError {
    /// Returns a stable symbolic error code suitable for logs and telemetry.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidName => "CORE_IDENTITY_INVALID_NAME",
            Self::InvalidLine => "CORE_IDENTITY_INVALID_LINE",
            Self::InvalidBlockFormat => "CORE_IDENTITY_INVALID_BLOCK_FORMAT",
            Self::InvalidGenesisAuthority => "CORE_IDENTITY_INVALID_GENESIS_AUTHORITY",
        }
    }
}

impl fmt::Display for CoreIdentityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName => {
                write!(f, "core identity validation failed: canonical name is invalid")
            }
            Self::InvalidLine => {
                write!(f, "core identity validation failed: canonical line is invalid")
            }
            Self::InvalidBlockFormat => {
                write!(f, "core identity validation failed: block format line is invalid")
            }
            Self::InvalidGenesisAuthority => {
                write!(
                    f,
                    "core identity validation failed: genesis authority line is invalid"
                )
            }
        }
    }
}

impl std::error::Error for CoreIdentityError {}

impl CoreIdentity {
    /// Returns the canonical schema generation for this identity contract.
    #[must_use]
    pub const fn schema_version(&self) -> u8 {
        AOXC_CORE_IDENTITY_SCHEMA_VERSION
    }

    /// Returns whether the currently exposed core surface still contains draft lines.
    ///
    /// Current policy:
    /// - block format and genesis authority are draft-tagged,
    /// - the core line itself is canonical and versioned.
    #[must_use]
    pub fn is_draft_surface(&self) -> bool {
        self.block_format.ends_with("-draft") || self.genesis_authority.ends_with("-draft")
    }

    /// Validates that the identity matches the canonical AOXC public contract.
    ///
    /// Validation policy:
    /// - every line must exactly match the current canonical constant,
    /// - this function is intentionally strict because the object is meant to
    ///   represent one exact protocol identity rather than a loose profile.
    pub fn validate(&self) -> Result<(), CoreIdentityError> {
        if self.name != AOXC_CANONICAL_CORE_NAME {
            return Err(CoreIdentityError::InvalidName);
        }

        if self.line != AOXC_CORE_LINE {
            return Err(CoreIdentityError::InvalidLine);
        }

        if self.block_format != AOXC_BLOCK_FORMAT_LINE {
            return Err(CoreIdentityError::InvalidBlockFormat);
        }

        if self.genesis_authority != AOXC_GENESIS_AUTH_LINE {
            return Err(CoreIdentityError::InvalidGenesisAuthority);
        }

        Ok(())
    }

    /// Returns a canonical SHA3-256 commitment over the full public identity surface.
    ///
    /// The digest covers:
    /// - the explicit domain separator,
    /// - schema version,
    /// - canonical name,
    /// - canonical core line,
    /// - block format line,
    /// - genesis authority line.
    #[must_use]
    pub fn commitment(&self) -> [u8; 32] {
        compute_core_identity_commitment(self)
    }

    /// Returns the full commitment encoded as uppercase hexadecimal.
    #[must_use]
    pub fn commitment_hex(&self) -> String {
        hex::encode_upper(self.commitment())
    }

    /// Returns a short stable fingerprint for operator-facing diagnostics.
    ///
    /// This helper is intended for:
    /// - logs,
    /// - dashboards,
    /// - audit notes,
    /// - compatibility summaries.
    #[must_use]
    pub fn fingerprint(&self) -> String {
        let digest = self.commitment();
        hex::encode_upper(&digest[..8])
    }
}

/// Returns the canonical AOXC core identity.
///
/// This function is intentionally `const` so that downstream code may rely on
/// a compile-time stable identity object.
#[must_use]
pub const fn core_identity() -> CoreIdentity {
    CoreIdentity {
        name: AOXC_CANONICAL_CORE_NAME,
        line: AOXC_CORE_LINE,
        block_format: AOXC_BLOCK_FORMAT_LINE,
        genesis_authority: AOXC_GENESIS_AUTH_LINE,
    }
}

/// Computes the canonical commitment for the supplied core identity.
///
/// Security rationale:
/// - the hash is domain-separated,
/// - the schema generation is bound into the digest,
/// - the resulting commitment is deterministic and externally reproducible.
#[must_use]
pub fn compute_core_identity_commitment(identity: &CoreIdentity) -> [u8; 32] {
    let mut hasher = Sha3_256::new();

    hasher.update(AOXC_CORE_IDENTITY_DOMAIN);
    hasher.update([0x00]);

    hasher.update([AOXC_CORE_IDENTITY_SCHEMA_VERSION]);
    hasher.update([0x00]);

    hasher.update(identity.name.as_bytes());
    hasher.update([0x00]);

    hasher.update(identity.line.as_bytes());
    hasher.update([0x00]);

    hasher.update(identity.block_format.as_bytes());
    hasher.update([0x00]);

    hasher.update(identity.genesis_authority.as_bytes());

    let digest = hasher.finalize();

    let mut out = [0u8; 32];
    out.copy_from_slice(&digest[..32]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_core_identity_validates_successfully() {
        let identity = core_identity();

        assert_eq!(identity.schema_version(), AOXC_CORE_IDENTITY_SCHEMA_VERSION);
        assert!(identity.validate().is_ok());
    }

    #[test]
    fn canonical_core_identity_reports_draft_surface() {
        let identity = core_identity();

        assert!(identity.is_draft_surface());
    }

    #[test]
    fn canonical_commitment_is_stable() {
        let identity = core_identity();

        let a = identity.commitment();
        let b = identity.commitment();

        assert_eq!(a, b);
        assert_eq!(a.len(), 32);
    }

    #[test]
    fn canonical_fingerprint_is_stable_and_short() {
        let identity = core_identity();

        let a = identity.fingerprint();
        let b = identity.fingerprint();

        assert_eq!(a, b);
        assert_eq!(a.len(), 16);
    }

    #[test]
    fn invalid_name_is_rejected() {
        let identity = CoreIdentity {
            name: "AOXC Experimental Core",
            ..core_identity()
        };

        let error = identity.validate().expect_err("validation must fail");
        assert_eq!(error, CoreIdentityError::InvalidName);
        assert_eq!(error.code(), "CORE_IDENTITY_INVALID_NAME");
    }

    #[test]
    fn invalid_core_line_is_rejected() {
        let identity = CoreIdentity {
            line: "AOXC-CORE-V2",
            ..core_identity()
        };

        let error = identity.validate().expect_err("validation must fail");
        assert_eq!(error, CoreIdentityError::InvalidLine);
    }

    #[test]
    fn commitment_changes_when_public_surface_changes() {
        let canonical = core_identity();
        let modified = CoreIdentity {
            genesis_authority: "AOXC-GENESIS-AUTH-V2-draft",
            ..canonical
        };

        assert_ne!(canonical.commitment(), modified.commitment());
    }

    #[test]
    fn commitment_hex_is_uppercase_and_full_length() {
        let identity = core_identity();
        let encoded = identity.commitment_hex();

        assert_eq!(encoded, encoded.to_ascii_uppercase());
        assert_eq!(encoded.len(), 64);
    }
}
