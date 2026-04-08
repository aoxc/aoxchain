// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! AOXC deterministic Ed25519 key derivation helpers.
//!
//! This module converts AOXC role-scoped key material into Ed25519 operational
//! keys suitable for:
//! - node identity,
//! - consensus participation,
//! - transport identity,
//! - operator signing,
//! - recovery workflows,
//! - future hybrid augmentation entry points.
//!
//! The implementation intentionally derives a 32-byte Ed25519 seed from the
//! upstream AOXC deterministic key material using explicit domain separation.
//!
//! Compatibility posture:
//! - existing primary helper names are preserved,
//! - deterministic derivation remains role-scoped,
//! - the role label contract is now stricter in order to reduce ambiguity and
//!   silent derivation drift.

use ed25519_dalek::{SigningKey, VerifyingKey};
use sha3::{Digest, Sha3_256};
use std::fmt;

use crate::identity::key_engine::DERIVED_ENTROPY_LEN;

/// Canonical AOXC Ed25519 derivation domain.
///
/// This domain prevents downstream Ed25519 seed reuse across protocols and
/// protects the operational seed space from generic entropy reinterpretation.
const AOXC_ED25519_ROLE_SEED_DOMAIN: &[u8] = b"AOXC/IDENTITY/ED25519/ROLE_SEED/V1";

/// Canonical AOXC Ed25519 public-key fingerprint domain.
const AOXC_ED25519_PUBLIC_KEY_FINGERPRINT_DOMAIN: &[u8] =
    b"AOXC/IDENTITY/ED25519/PUBLIC_KEY_FINGERPRINT/V1";

/// Canonical AOXC Ed25519 public-key length in bytes.
pub const AOXC_ED25519_PUBLIC_KEY_LEN: usize = 32;

/// Canonical AOXC Ed25519 seed length in bytes.
pub const AOXC_ED25519_SEED_LEN: usize = 32;

/// Maximum accepted canonical role-label length.
///
/// This bound is intentionally conservative. It is sufficient for operational
/// role labels while preventing unbounded labels from affecting deterministic
/// derivation inputs.
pub const AOXC_ED25519_MAX_ROLE_LABEL_LEN: usize = 64;

/// Canonical AOXC Ed25519 short fingerprint length in bytes.
pub const AOXC_ED25519_FINGERPRINT_LEN: usize = 8;

/// Error surface for AOXC Ed25519 deterministic key derivation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Ed25519KeyError {
    EmptyRoleLabel,
    InvalidRoleLabel,
    InvalidPublicKeyHex,
    InvalidPublicKeyLength,
}

impl Ed25519KeyError {
    /// Returns a stable symbolic error code suitable for logs and telemetry.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::EmptyRoleLabel => "ED25519_KEY_EMPTY_ROLE_LABEL",
            Self::InvalidRoleLabel => "ED25519_KEY_INVALID_ROLE_LABEL",
            Self::InvalidPublicKeyHex => "ED25519_KEY_INVALID_PUBLIC_KEY_HEX",
            Self::InvalidPublicKeyLength => "ED25519_KEY_INVALID_PUBLIC_KEY_LENGTH",
        }
    }
}

impl fmt::Display for Ed25519KeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRoleLabel => {
                write!(
                    f,
                    "ed25519 key derivation failed: role label must not be empty"
                )
            }
            Self::InvalidRoleLabel => {
                write!(
                    f,
                    "ed25519 key derivation failed: role label is not canonical"
                )
            }
            Self::InvalidPublicKeyHex => {
                write!(
                    f,
                    "ed25519 public key decoding failed: public key must be valid hexadecimal"
                )
            }
            Self::InvalidPublicKeyLength => {
                write!(
                    f,
                    "ed25519 public key decoding failed: public key length is invalid"
                )
            }
        }
    }
}

impl std::error::Error for Ed25519KeyError {}

/// Validates a canonical AOXC role label.
///
/// Validation policy:
/// - the label must not be empty,
/// - surrounding whitespace is rejected rather than normalized,
/// - internal whitespace is forbidden,
/// - only ASCII alphanumeric characters plus `_`, `-`, and `.` are accepted,
/// - length must remain bounded.
fn validate_role_label(role_label: &str) -> Result<(), Ed25519KeyError> {
    if role_label.is_empty() {
        return Err(Ed25519KeyError::EmptyRoleLabel);
    }

    if role_label.trim().is_empty() {
        return Err(Ed25519KeyError::EmptyRoleLabel);
    }

    if role_label != role_label.trim() {
        return Err(Ed25519KeyError::InvalidRoleLabel);
    }

    if role_label.len() > AOXC_ED25519_MAX_ROLE_LABEL_LEN {
        return Err(Ed25519KeyError::InvalidRoleLabel);
    }

    if role_label.chars().any(|ch| ch.is_ascii_whitespace()) {
        return Err(Ed25519KeyError::InvalidRoleLabel);
    }

    if !role_label
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
    {
        return Err(Ed25519KeyError::InvalidRoleLabel);
    }

    Ok(())
}

/// Derives a canonical 32-byte Ed25519 seed from AOXC role-scoped material.
///
/// Security properties:
/// - explicit domain separation,
/// - role separation,
/// - deterministic reproducibility,
/// - no direct reuse of upstream 64-byte material as a signing key.
pub fn derive_ed25519_seed(
    material: &[u8; DERIVED_ENTROPY_LEN],
    role_label: &str,
) -> Result<[u8; AOXC_ED25519_SEED_LEN], Ed25519KeyError> {
    validate_role_label(role_label)?;

    let mut hasher = Sha3_256::new();
    hasher.update(AOXC_ED25519_ROLE_SEED_DOMAIN);
    hasher.update([0x00]);
    hasher.update(role_label.as_bytes());
    hasher.update([0x00]);
    hasher.update(material);

    let digest = hasher.finalize();

    let mut seed = [0u8; AOXC_ED25519_SEED_LEN];
    seed.copy_from_slice(&digest[..AOXC_ED25519_SEED_LEN]);

    Ok(seed)
}

/// Derives a canonical Ed25519 signing key from AOXC role-scoped material.
pub fn derive_ed25519_signing_key(
    material: &[u8; DERIVED_ENTROPY_LEN],
    role_label: &str,
) -> Result<SigningKey, Ed25519KeyError> {
    let seed = derive_ed25519_seed(material, role_label)?;
    Ok(SigningKey::from_bytes(&seed))
}

/// Derives a canonical Ed25519 verifying key from AOXC role-scoped material.
pub fn derive_ed25519_verifying_key(
    material: &[u8; DERIVED_ENTROPY_LEN],
    role_label: &str,
) -> Result<VerifyingKey, Ed25519KeyError> {
    let signing_key = derive_ed25519_signing_key(material, role_label)?;
    Ok(signing_key.verifying_key())
}

/// Derives both the canonical Ed25519 signing key and verifying key from
/// AOXC role-scoped material.
///
/// This helper preserves a single deterministic derivation path for callers
/// that need both operational surfaces together.
pub fn derive_ed25519_keypair(
    material: &[u8; DERIVED_ENTROPY_LEN],
    role_label: &str,
) -> Result<(SigningKey, VerifyingKey), Ed25519KeyError> {
    let signing_key = derive_ed25519_signing_key(material, role_label)?;
    let verifying_key = signing_key.verifying_key();
    Ok((signing_key, verifying_key))
}

/// Returns the hex-encoded Ed25519 public key.
#[must_use]
pub fn encode_ed25519_public_key_hex(public_key: &VerifyingKey) -> String {
    hex::encode_upper(public_key.to_bytes())
}

/// Decodes a hex-encoded Ed25519 public key into a verifying key.
///
/// Validation policy:
/// - input must be valid hexadecimal,
/// - decoded byte length must equal the canonical Ed25519 public-key length.
pub fn decode_ed25519_public_key_hex(
    encoded_public_key: &str,
) -> Result<VerifyingKey, Ed25519KeyError> {
    let decoded =
        hex::decode(encoded_public_key).map_err(|_| Ed25519KeyError::InvalidPublicKeyHex)?;

    if decoded.len() != AOXC_ED25519_PUBLIC_KEY_LEN {
        return Err(Ed25519KeyError::InvalidPublicKeyLength);
    }

    let mut bytes = [0u8; AOXC_ED25519_PUBLIC_KEY_LEN];
    bytes.copy_from_slice(&decoded);

    VerifyingKey::from_bytes(&bytes).map_err(|_| Ed25519KeyError::InvalidPublicKeyHex)
}

/// Returns a stable short fingerprint for an Ed25519 public key.
///
/// The fingerprint is derived under an explicit AOXC fingerprint domain rather
/// than hashing the raw public-key bytes without context.
#[must_use]
pub fn fingerprint_ed25519_public_key(public_key: &VerifyingKey) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(AOXC_ED25519_PUBLIC_KEY_FINGERPRINT_DOMAIN);
    hasher.update([0x00]);
    hasher.update(public_key.to_bytes());

    let digest = hasher.finalize();
    hex::encode_upper(&digest[..AOXC_ED25519_FINGERPRINT_LEN])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_material() -> [u8; DERIVED_ENTROPY_LEN] {
        [0x42; DERIVED_ENTROPY_LEN]
    }

    #[test]
    fn ed25519_seed_derivation_is_deterministic() {
        let material = sample_material();

        let a = derive_ed25519_seed(&material, "consensus").unwrap();
        let b = derive_ed25519_seed(&material, "consensus").unwrap();

        assert_eq!(a, b);
    }

    #[test]
    fn ed25519_seed_derivation_changes_by_role() {
        let material = sample_material();

        let a = derive_ed25519_seed(&material, "consensus").unwrap();
        let b = derive_ed25519_seed(&material, "transport").unwrap();

        assert_ne!(a, b);
    }

    #[test]
    fn ed25519_signing_and_verifying_key_derivation_are_consistent() {
        let material = sample_material();

        let signing_key = derive_ed25519_signing_key(&material, "identity").unwrap();
        let verifying_key = derive_ed25519_verifying_key(&material, "identity").unwrap();

        assert_eq!(signing_key.verifying_key(), verifying_key);
    }

    #[test]
    fn ed25519_keypair_derivation_returns_consistent_pair() {
        let material = sample_material();

        let (signing_key, verifying_key) = derive_ed25519_keypair(&material, "operator").unwrap();

        assert_eq!(signing_key.verifying_key(), verifying_key);
    }

    #[test]
    fn verifying_key_has_expected_length() {
        let material = sample_material();

        let key = derive_ed25519_verifying_key(&material, "identity").unwrap();
        assert_eq!(key.to_bytes().len(), AOXC_ED25519_PUBLIC_KEY_LEN);
    }

    #[test]
    fn empty_role_label_is_rejected() {
        let material = sample_material();

        let result = derive_ed25519_seed(&material, "");
        assert_eq!(result, Err(Ed25519KeyError::EmptyRoleLabel));
    }

    #[test]
    fn whitespace_only_role_label_is_rejected() {
        let material = sample_material();

        let result = derive_ed25519_seed(&material, "   ");
        assert_eq!(result, Err(Ed25519KeyError::EmptyRoleLabel));
    }

    #[test]
    fn non_canonical_role_label_with_surrounding_whitespace_is_rejected() {
        let material = sample_material();

        let result = derive_ed25519_seed(&material, " consensus ");
        assert_eq!(result, Err(Ed25519KeyError::InvalidRoleLabel));
    }

    #[test]
    fn non_canonical_role_label_with_internal_whitespace_is_rejected() {
        let material = sample_material();

        let result = derive_ed25519_seed(&material, "consensus role");
        assert_eq!(result, Err(Ed25519KeyError::InvalidRoleLabel));
    }

    #[test]
    fn invalid_role_label_characters_are_rejected() {
        let material = sample_material();

        let result = derive_ed25519_seed(&material, "consensus!");
        assert_eq!(result, Err(Ed25519KeyError::InvalidRoleLabel));
    }

    #[test]
    fn public_key_hex_roundtrip_succeeds() {
        let material = sample_material();

        let verifying_key = derive_ed25519_verifying_key(&material, "identity").unwrap();
        let encoded = encode_ed25519_public_key_hex(&verifying_key);
        let decoded = decode_ed25519_public_key_hex(&encoded).unwrap();

        assert_eq!(decoded, verifying_key);
    }

    #[test]
    fn invalid_public_key_hex_is_rejected() {
        let result = decode_ed25519_public_key_hex("ZZ_NOT_HEX");
        assert_eq!(result, Err(Ed25519KeyError::InvalidPublicKeyHex));
    }

    #[test]
    fn invalid_public_key_length_is_rejected() {
        let result = decode_ed25519_public_key_hex("A1B2");
        assert_eq!(result, Err(Ed25519KeyError::InvalidPublicKeyLength));
    }

    #[test]
    fn public_key_fingerprint_is_stable() {
        let material = sample_material();

        let key = derive_ed25519_verifying_key(&material, "identity").unwrap();
        let a = fingerprint_ed25519_public_key(&key);
        let b = fingerprint_ed25519_public_key(&key);

        assert_eq!(a, b);
        assert_eq!(a.len(), AOXC_ED25519_FINGERPRINT_LEN * 2);
    }

    #[test]
    fn public_key_fingerprint_changes_across_roles() {
        let material = sample_material();

        let a = derive_ed25519_verifying_key(&material, "identity").unwrap();
        let b = derive_ed25519_verifying_key(&material, "consensus").unwrap();

        assert_ne!(
            fingerprint_ed25519_public_key(&a),
            fingerprint_ed25519_public_key(&b)
        );
    }
}
