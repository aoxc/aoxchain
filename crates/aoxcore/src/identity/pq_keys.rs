//! core/identity/src/pq_keys.rs
//!
//! Post-quantum key utilities for AOXC identity system.
//!
//! Current algorithm:
//! Dilithium3 (NIST PQC standard candidate)

use pqcrypto_dilithium::dilithium3::{PublicKey, SecretKey, SignedMessage, keypair, open, sign};

use pqcrypto_traits::sign::{PublicKey as _, SecretKey as _, SignedMessage as _};

use sha3::{Digest, Sha3_256};

/// Generates a new post-quantum Dilithium3 keypair.
///
/// This uses secure randomness from the underlying pqcrypto library.
pub fn generate_keypair() -> (PublicKey, SecretKey) {
    keypair()
}

/// Signs a message using a Dilithium3 secret key.
///
/// Returns a serialized signed message.
pub fn sign_message(message: &[u8], sk: &SecretKey) -> Vec<u8> {
    sign(message, sk).as_bytes().to_vec()
}

/// Verifies a Dilithium3 signed message.
///
/// Returns the original message if verification succeeds.
pub fn verify_message(signed: &[u8], pk: &PublicKey) -> Result<Vec<u8>, String> {
    let signed_msg =
        SignedMessage::from_bytes(signed).map_err(|_| "INVALID_SIGNED_MESSAGE".to_string())?;

    let opened = open(&signed_msg, pk).map_err(|_| "SIGNATURE_VERIFICATION_FAILED".to_string())?;

    Ok(opened)
}

/// Serializes a public key into raw bytes.
pub fn serialize_public_key(pk: &PublicKey) -> Vec<u8> {
    pk.as_bytes().to_vec()
}

/// Serializes a secret key into raw bytes.
///
/// WARNING:
/// This exposes private key material and should only be used
/// for secure storage (e.g., encrypted keyfiles).
pub fn serialize_secret_key(sk: &SecretKey) -> Vec<u8> {
    sk.as_bytes().to_vec()
}

/// Restores a public key from raw bytes.
pub fn public_key_from_bytes(bytes: &[u8]) -> Result<PublicKey, String> {
    PublicKey::from_bytes(bytes).map_err(|_| "INVALID_PUBLIC_KEY".to_string())
}

/// Restores a secret key from raw bytes.
pub fn secret_key_from_bytes(bytes: &[u8]) -> Result<SecretKey, String> {
    SecretKey::from_bytes(bytes).map_err(|_| "INVALID_SECRET_KEY".to_string())
}

/// Computes a short deterministic fingerprint for a public key.
///
/// Useful for logging, identity display, and debugging.
pub fn fingerprint(pk: &PublicKey) -> String {
    let mut hasher = Sha3_256::new();

    hasher.update(pk.as_bytes());

    let digest = hasher.finalize();

    hex::encode_upper(&digest[..8])
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn keypair_generation_works() {
        let (_pk, _sk) = generate_keypair();
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
    fn public_key_serialization_roundtrip() {
        let (pk, _) = generate_keypair();

        let bytes = serialize_public_key(&pk);

        let restored = public_key_from_bytes(&bytes).unwrap();

        assert_eq!(bytes, restored.as_bytes());
    }
}
