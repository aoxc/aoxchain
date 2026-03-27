// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::error::MobError;
use crate::util::sha3_hex_upper;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::Serialize;

/// Returns the uppercase hex-encoded public key.
#[must_use]
pub fn public_key_hex(signing_key: &SigningKey) -> String {
    hex::encode_upper(signing_key.verifying_key().to_bytes())
}

/// Returns a short stable public key fingerprint for operator visibility.
#[must_use]
pub fn public_key_fingerprint(verifying_key: &VerifyingKey) -> String {
    let digest = sha3_hex_upper(&verifying_key.to_bytes());
    digest[..24].to_string()
}

/// Signs a canonical JSON payload and returns `(signature_hex, payload_hash_hex)`.
pub fn sign_json_payload<T: Serialize>(
    signing_key: &SigningKey,
    payload: &T,
) -> Result<(String, String), MobError> {
    let bytes = serde_json::to_vec(payload)?;
    let signature: Signature = signing_key.sign(&bytes);
    let payload_hash_hex = sha3_hex_upper(&bytes);
    Ok((hex::encode_upper(signature.to_bytes()), payload_hash_hex))
}

/// Verifies a canonical JSON payload signature.
pub fn verify_json_payload<T: Serialize>(
    verifying_key: &VerifyingKey,
    payload: &T,
    signature_hex: &str,
) -> Result<(), MobError> {
    let bytes = serde_json::to_vec(payload)?;
    let raw = hex::decode(signature_hex)
        .map_err(|error| MobError::Crypto(format!("signature hex decode failed: {}", error)))?;
    let signature = Signature::from_slice(&raw)
        .map_err(|error| MobError::Crypto(format!("signature parse failed: {}", error)))?;
    verifying_key
        .verify(&bytes, &signature)
        .map_err(|error| MobError::Crypto(format!("signature verification failed: {}", error)))
}
