// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! core/identity/src/pq_keys.rs
//!
//! Post-quantum key utilities for the AOXC identity system.
//!
//! Current algorithm:
//! Dilithium3
//!
//! Security posture:
//! - preserves the legacy signed-message API for compatibility,
//! - adds domain-separated helpers for safer protocol usage,
//! - enforces explicit key decoding boundaries,
//! - uses deterministic domain-separated public-key fingerprints.

use pqcrypto_dilithium::dilithium3::{
    keypair, open, public_key_bytes, secret_key_bytes, sign, PublicKey, SecretKey, SignedMessage,
};

use pqcrypto_traits::sign::{PublicKey as _, SecretKey as _, SignedMessage as _};

use sha3::{Digest, Sha3_256};
use std::fmt;

/// Domain separator for AOXC PQ fingerprint derivation.
const AOXC_PQ_FINGERPRINT_DOMAIN: &[u8] = b"AOXC/IDENTITY/PQ_KEYS/FINGERPRINT/V1";

/// Domain separator for AOXC signed-message wrapping.
///
/// Compatibility note:
/// the legacy `sign_message` / `verify_message` surface remains raw in order to
/// avoid surprising existing callers. New code should prefer the domain-
/// separated helpers below where protocol binding matters.
const AOXC_PQ_SIGNING_DOMAIN: &[u8] = b"AOXC/IDENTITY/PQ_KEYS/SIGNED_MESSAGE/V1";

/// Short fingerprint output length in bytes.
const PQ_FINGERPRINT_LEN: usize = 8;

/// Canonical PQ key error surface used internally and mapped to stable strings.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PqKeyError {
    InvalidSignedMessage,
    SignatureVerificationFailed,
    InvalidPublicKey,
    InvalidSecretKey,
    InvalidPublicKeyHex,
    InvalidSecretKeyHex,
    InvalidWrappedMessageDomain,
}

impl PqKeyError {
    /// Returns a stable symbolic error code suitable for logs and telemetry.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidSignedMessage => "INVALID_SIGNED_MESSAGE",
            Self::SignatureVerificationFailed => "SIGNATURE_VERIFICATION_FAILED",
            Self::InvalidPublicKey => "INVALID_PUBLIC_KEY",
            Self::InvalidSecretKey => "INVALID_SECRET_KEY",
            Self::InvalidPublicKeyHex => "INVALID_PUBLIC_KEY_HEX",
            Self::InvalidSecretKeyHex => "INVALID_SECRET_KEY_HEX",
            Self::InvalidWrappedMessageDomain => "INVALID_WRAPPED_MESSAGE_DOMAIN",
        }
    }
}

impl fmt::Display for PqKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl std::error::Error for PqKeyError {}

/// Generates a new post-quantum Dilithium3 keypair.
///
/// This uses secure randomness from the underlying pqcrypto library.
#[must_use]
pub fn generate_keypair() -> (PublicKey, SecretKey) {
    keypair()
}

/// Returns the expected serialized public-key length in bytes.
#[must_use]
pub fn expected_public_key_len() -> usize {
    public_key_bytes()
}

/// Returns the expected serialized secret-key length in bytes.
#[must_use]
pub fn expected_secret_key_len() -> usize {
    secret_key_bytes()
}

/// Signs a message using a Dilithium3 secret key.
///
/// Compatibility note:
/// this preserves the legacy raw signed-message behavior without AOXC domain
/// wrapping. For new protocol-bound call paths, prefer
/// `sign_message_domain_separated`.
///
/// Returns a serialized signed message.
#[must_use]
pub fn sign_message(message: &[u8], sk: &SecretKey) -> Vec<u8> {
    sign(message, sk).as_bytes().to_vec()
}

/// Verifies a Dilithium3 signed message.
///
/// Compatibility note:
/// this verifies the legacy raw signed-message format and returns the original
/// message if verification succeeds.
pub fn verify_message(signed: &[u8], pk: &PublicKey) -> Result<Vec<u8>, String> {
    verify_message_internal(signed, pk).map_err(|error| error.code().to_string())
}

/// Signs a message using a Dilithium3 secret key under an AOXC domain wrapper.
///
/// New protocol-bound call sites should prefer this helper over `sign_message`
/// because it provides explicit AOXC message namespace separation.
#[must_use]
pub fn sign_message_domain_separated(message: &[u8], sk: &SecretKey) -> Vec<u8> {
    let wrapped = wrap_message_for_signing(message);
    sign(&wrapped, sk).as_bytes().to_vec()
}

/// Verifies an AOXC domain-separated signed message.
///
/// Returns the original unwrapped message if verification succeeds.
pub fn verify_message_domain_separated(
    signed: &[u8],
    pk: &PublicKey,
) -> Result<Vec<u8>, String> {
    let opened = verify_message_internal(signed, pk).map_err(|error| error.code().to_string())?;
    unwrap_verified_message(&opened).map_err(|error| error.code().to_string())
}

/// Internal verification helper shared by both signed-message surfaces.
fn verify_message_internal(signed: &[u8], pk: &PublicKey) -> Result<Vec<u8>, PqKeyError> {
    let signed_msg = SignedMessage::from_bytes(signed).map_err(|_| PqKeyError::InvalidSignedMessage)?;

    open(&signed_msg, pk).map_err(|_| PqKeyError::SignatureVerificationFailed)
}

/// Serializes a public key into raw bytes.
#[must_use]
pub fn serialize_public_key(pk: &PublicKey) -> Vec<u8> {
    pk.as_bytes().to_vec()
}

/// Serializes a secret key into raw bytes.
///
/// WARNING:
/// This exposes private key material and should only be used for protected
/// custody flows such as encrypted keyfiles or offline secure export.
#[must_use]
pub fn serialize_secret_key(sk: &SecretKey) -> Vec<u8> {
    sk.as_bytes().to_vec()
}

/// Serializes a public key into uppercase hexadecimal.
#[must_use]
pub fn serialize_public_key_hex(pk: &PublicKey) -> String {
    hex::encode_upper(pk.as_bytes())
}

/// Serializes a secret key into uppercase hexadecimal.
///
/// WARNING:
/// This exposes private key material and should only be used for protected
/// custody flows such as encrypted keyfiles or offline secure export.
#[must_use]
pub fn serialize_secret_key_hex(sk: &SecretKey) -> String {
    hex::encode_upper(sk.as_bytes())
}

/// Restores a public key from raw bytes.
pub fn public_key_from_bytes(bytes: &[u8]) -> Result<PublicKey, String> {
    PublicKey::from_bytes(bytes).map_err(|_| PqKeyError::InvalidPublicKey.code().to_string())
}

/// Restores a secret key from raw bytes.
pub fn secret_key_from_bytes(bytes: &[u8]) -> Result<SecretKey, String> {
    SecretKey::from_bytes(bytes).map_err(|_| PqKeyError::InvalidSecretKey.code().to_string())
}

/// Restores a public key from uppercase or lowercase hexadecimal.
pub fn public_key_from_hex(encoded: &str) -> Result<PublicKey, String> {
    let bytes = hex::decode(encoded).map_err(|_| PqKeyError::InvalidPublicKeyHex.code().to_string())?;
    public_key_from_bytes(&bytes)
}

/// Restores a secret key from uppercase or lowercase hexadecimal.
pub fn secret_key_from_hex(encoded: &str) -> Result<SecretKey, String> {
    let bytes = hex::decode(encoded).map_err(|_| PqKeyError::InvalidSecretKeyHex.code().to_string())?;
    secret_key_from_bytes(&bytes)
}

/// Computes a short deterministic fingerprint for a public key.
///
/// Security rationale:
/// - the fingerprint is domain-separated,
/// - the output is stable and short for operator-facing diagnostics,
/// - this is suitable for logging, identity display, and debugging, but not as
///   a substitute for the full public key.
#[must_use]
pub fn fingerprint(pk: &PublicKey) -> String {
    let mut hasher = Sha3_256::new();

    hasher.update(AOXC_PQ_FINGERPRINT_DOMAIN);
    hasher.update([0x00]);
    hasher.update(pk.as_bytes());

    let digest = hasher.finalize();

    hex::encode_upper(&digest[..PQ_FINGERPRINT_LEN])
}

/// Wraps a message under the canonical AOXC PQ signing domain.
fn wrap_message_for_signing(message: &[u8]) -> Vec<u8> {
    let mut wrapped = Vec::with_capacity(AOXC_PQ_SIGNING_DOMAIN.len() + 1 + message.len());

    wrapped.extend_from_slice(AOXC_PQ_SIGNING_DOMAIN);
    wrapped.push(0x00);
    wrapped.extend_from_slice(message);

    wrapped
}

/// Unwraps a previously AOXC-domain-wrapped verified message.
fn unwrap_verified_message(wrapped: &[u8]) -> Result<Vec<u8>, PqKeyError> {
    let prefix_len = AOXC_PQ_SIGNING_DOMAIN.len() + 1;

    if wrapped.len() < prefix_len {
        return Err(PqKeyError::InvalidWrappedMessageDomain);
    }

    if !wrapped.starts_with(AOXC_PQ_SIGNING_DOMAIN) || wrapped[AOXC_PQ_SIGNING_DOMAIN.len()] != 0x00 {
        return Err(PqKeyError::InvalidWrappedMessageDomain);
    }

    Ok(wrapped[prefix_len..].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keypair_generation_works() {
        let (pk, sk) = generate_keypair();

        assert_eq!(pk.as_bytes().len(), expected_public_key_len());
        assert_eq!(sk.as_bytes().len(), expected_secret_key_len());
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        let (pk, sk) = generate_keypair();

        let message = b"AOXC test message";
        let signed = sign_message(message, &sk);
        let opened = verify_message(&signed, &pk).unwrap();

        assert_eq!(opened, message);
    }

    #[test]
    fn sign_and_verify_domain_separated_roundtrip() {
        let (pk, sk) = generate_keypair();

        let message = b"AOXC domain separated message";
        let signed = sign_message_domain_separated(message, &sk);
        let opened = verify_message_domain_separated(&signed, &pk).unwrap();

        assert_eq!(opened, message);
    }

    #[test]
    fn raw_signed_message_is_not_accepted_by_domain_separated_verifier() {
        let (pk, sk) = generate_keypair();

        let message = b"AOXC raw message";
        let signed = sign_message(message, &sk);
        let result = verify_message_domain_separated(&signed, &pk);

        assert_eq!(result, Err("INVALID_WRAPPED_MESSAGE_DOMAIN".to_string()));
    }

    #[test]
    fn public_key_serialization_roundtrip() {
        let (pk, _) = generate_keypair();

        let bytes = serialize_public_key(&pk);
        let restored = public_key_from_bytes(&bytes).unwrap();

        assert_eq!(bytes, restored.as_bytes());
    }

    #[test]
    fn secret_key_serialization_roundtrip() {
        let (_, sk) = generate_keypair();

        let bytes = serialize_secret_key(&sk);
        let restored = secret_key_from_bytes(&bytes).unwrap();

        assert_eq!(bytes, restored.as_bytes());
    }

    #[test]
    fn public_key_hex_roundtrip() {
        let (pk, _) = generate_keypair();

        let encoded = serialize_public_key_hex(&pk);
        let restored = public_key_from_hex(&encoded).unwrap();

        assert_eq!(pk.as_bytes(), restored.as_bytes());
    }

    #[test]
    fn secret_key_hex_roundtrip() {
        let (_, sk) = generate_keypair();

        let encoded = serialize_secret_key_hex(&sk);
        let restored = secret_key_from_hex(&encoded).unwrap();

        assert_eq!(sk.as_bytes(), restored.as_bytes());
    }

    #[test]
    fn invalid_public_key_bytes_are_rejected() {
        let result = public_key_from_bytes(&[0u8; 8]);
        assert_eq!(result, Err("INVALID_PUBLIC_KEY".to_string()));
    }

    #[test]
    fn invalid_secret_key_bytes_are_rejected() {
        let result = secret_key_from_bytes(&[0u8; 8]);
        assert_eq!(result, Err("INVALID_SECRET_KEY".to_string()));
    }

    #[test]
    fn invalid_public_key_hex_is_rejected() {
        let result = public_key_from_hex("ZZ_NOT_HEX");
        assert_eq!(result, Err("INVALID_PUBLIC_KEY_HEX".to_string()));
    }

    #[test]
    fn invalid_secret_key_hex_is_rejected() {
        let result = secret_key_from_hex("ZZ_NOT_HEX");
        assert_eq!(result, Err("INVALID_SECRET_KEY_HEX".to_string()));
    }

    #[test]
    fn fingerprint_is_stable() {
        let (pk, _) = generate_keypair();

        let a = fingerprint(&pk);
        let b = fingerprint(&pk);

        assert_eq!(a, b);
        assert_eq!(a.len(), PQ_FINGERPRINT_LEN * 2);
    }

    #[test]
    fn different_public_keys_produce_different_fingerprints() {
        let (pk_a, _) = generate_keypair();
        let (pk_b, _) = generate_keypair();

        assert_ne!(fingerprint(&pk_a), fingerprint(&pk_b));
    }
}
