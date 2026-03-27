// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! AOXC deterministic Ed25519 key derivation helpers.
//!
//! This module converts AOXC role-scoped key material into real Ed25519
//! operational keypairs suitable for:
//! - node identity,
//! - consensus participation,
//! - transport identity,
//! - operator signing,
//! - recovery workflows,
//! - future hybrid augmentation entry points.
//!
//! The implementation intentionally derives a 32-byte Ed25519 seed from the
//! upstream AOXC deterministic key material using explicit domain separation.

use ed25519_dalek::{SigningKey, VerifyingKey};
use sha3::{Digest, Sha3_256};
use std::fmt;

use crate::identity::key_engine::DERIVED_ENTROPY_LEN;

/// Canonical AOXC Ed25519 derivation domain.
///
/// This domain prevents downstream Ed25519 seed reuse across protocols and
/// protects the operational seed space from generic entropy reinterpretation.
const AOXC_ED25519_ROLE_SEED_DOMAIN: &[u8] = b"AOXC-ED25519-ROLE-SEED-V1";

/// Canonical AOXC Ed25519 public-key length in bytes.
pub const AOXC_ED25519_PUBLIC_KEY_LEN: usize = 32;

/// Canonical AOXC Ed25519 seed length in bytes.
pub const AOXC_ED25519_SEED_LEN: usize = 32;

/// Error surface for AOXC Ed25519 deterministic key derivation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Ed25519KeyError {
    EmptyRoleLabel,
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
        }
    }
}

impl std::error::Error for Ed25519KeyError {}

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
    if role_label.trim().is_empty() {
        return Err(Ed25519KeyError::EmptyRoleLabel);
    }

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

/// Returns the hex-encoded Ed25519 public key.
#[must_use]
pub fn encode_ed25519_public_key_hex(public_key: &VerifyingKey) -> String {
    hex::encode_upper(public_key.to_bytes())
}

/// Returns a stable short fingerprint for an Ed25519 public key.
#[must_use]
pub fn fingerprint_ed25519_public_key(public_key: &VerifyingKey) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(public_key.to_bytes());
    let digest = hasher.finalize();
    hex::encode_upper(&digest[..8])
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
}
