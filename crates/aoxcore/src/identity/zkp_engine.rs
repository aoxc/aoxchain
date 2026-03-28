// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::fmt;

/// Current pseudo-proof schema version.
///
/// Compatibility note:
/// the external `ZkpProof` structure remains intentionally minimal, but the
/// proof byte layout emitted by this module follows the versioned policy
/// defined here.
pub const ZKP_PROOF_VERSION: u8 = 1;

/// Canonical pseudo-proof byte length.
///
/// Layout:
/// - bytes[0..8]   : circuit binding tag
/// - bytes[8..16]  : public input binding tag
/// - bytes[16..32] : witness-derived tag
pub const ZKP_PROOF_BYTES_LEN: usize = 32;

/// Canonical public-input hash length in hexadecimal characters.
pub const ZKP_PUBLIC_INPUTS_HASH_HEX_LEN: usize = 64;

/// Maximum accepted circuit identifier length.
pub const MAX_CIRCUIT_ID_LEN: usize = 128;

/// Domain separator for deterministic pseudo-proof construction.
const ZKP_PROOF_DOMAIN: &[u8] = b"AOXC/ZKP/PSEUDO_PROOF/V1";

/// Domain separator for public-input hashing.
const ZKP_PUBLIC_INPUTS_HASH_DOMAIN: &[u8] = b"AOXC/ZKP/PUBLIC_INPUTS_HASH/V1";

/// Domain separator for circuit tag derivation.
const ZKP_CIRCUIT_TAG_DOMAIN: &[u8] = b"AOXC/ZKP/CIRCUIT_TAG/V1";

/// Domain separator for public-input tag derivation.
const ZKP_PUBLIC_INPUT_TAG_DOMAIN: &[u8] = b"AOXC/ZKP/PUBLIC_INPUT_TAG/V1";

/// Domain separator for witness tag derivation.
const ZKP_WITNESS_TAG_DOMAIN: &[u8] = b"AOXC/ZKP/WITNESS_TAG/V1";

/// Domain separator for proof fingerprint derivation.
const ZKP_PROOF_FINGERPRINT_DOMAIN: &[u8] = b"AOXC/ZKP/PROOF_FINGERPRINT/V1";

/// Minimal deterministic ZKP envelope used by higher layers.
///
/// Important note:
/// this structure models a deterministic integration proof envelope and not a
/// real zero-knowledge proof transcript.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZkpProof {
    pub circuit_id: String,
    pub proof_bytes: Vec<u8>,
    pub public_inputs_hash: String,
}

impl ZkpProof {
    /// Validates the structural integrity of the pseudo-proof envelope.
    pub fn validate(&self) -> Result<(), ZkpError> {
        validate_circuit_id(&self.circuit_id)?;

        if self.proof_bytes.len() != ZKP_PROOF_BYTES_LEN {
            return Err(ZkpError::InvalidProofLength);
        }

        let canonical_hash = canonicalize_hash_hex(&self.public_inputs_hash)?;
        if canonical_hash.len() != ZKP_PUBLIC_INPUTS_HASH_HEX_LEN {
            return Err(ZkpError::InvalidPublicInputsHash);
        }

        Ok(())
    }

    /// Computes a short deterministic fingerprint of the full proof envelope.
    #[must_use]
    pub fn fingerprint(&self) -> String {
        let mut hasher = Sha3_256::new();
        hasher.update(ZKP_PROOF_FINGERPRINT_DOMAIN);
        hasher.update([0x00]);
        hasher.update(self.circuit_id.as_bytes());
        hasher.update([0x00]);
        hasher.update(&self.proof_bytes);
        hasher.update([0x00]);
        hasher.update(self.public_inputs_hash.as_bytes());

        let digest = hasher.finalize();
        hex::encode_upper(&digest[..8])
    }

    /// Serializes the proof to JSON after structural validation.
    pub fn to_json(&self) -> Result<String, ZkpError> {
        self.validate()?;

        serde_json::to_string(self)
            .map_err(|error| ZkpError::SerializationFailed(error.to_string()))
    }

    /// Restores the proof from JSON and validates it.
    pub fn from_json(data: &str) -> Result<Self, ZkpError> {
        let proof: Self = serde_json::from_str(data)
            .map_err(|error| ZkpError::ParseFailed(error.to_string()))?;

        proof.validate()?;
        Ok(proof)
    }
}

/// Canonical ZKP envelope error surface.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ZkpError {
    EmptyCircuitId,
    InvalidCircuitId,
    InvalidProofLength,
    InvalidPublicInputsHash,
    PublicInputMismatch,
    CircuitMismatch,
    InvalidProofEncoding,
    BackendRejected,
    SerializationFailed(String),
    ParseFailed(String),
}

impl ZkpError {
    /// Returns a stable symbolic error code suitable for logs and telemetry.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::EmptyCircuitId => "ZKP_EMPTY_CIRCUIT_ID",
            Self::InvalidCircuitId => "ZKP_INVALID_CIRCUIT_ID",
            Self::InvalidProofLength => "ZKP_INVALID_PROOF_LENGTH",
            Self::InvalidPublicInputsHash => "ZKP_INVALID_PUBLIC_INPUTS_HASH",
            Self::PublicInputMismatch => "ZKP_PUBLIC_INPUT_MISMATCH",
            Self::CircuitMismatch => "ZKP_CIRCUIT_MISMATCH",
            Self::InvalidProofEncoding => "ZKP_INVALID_PROOF_ENCODING",
            Self::BackendRejected => "ZKP_BACKEND_REJECTED",
            Self::SerializationFailed(_) => "ZKP_SERIALIZATION_FAILED",
            Self::ParseFailed(_) => "ZKP_PARSE_FAILED",
        }
    }
}

impl fmt::Display for ZkpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SerializationFailed(_) | Self::ParseFailed(_) => write!(f, "{}", self.code()),
            _ => write!(f, "{}", self.code()),
        }
    }
}

impl std::error::Error for ZkpError {}

/// Verification backend abstraction.
///
/// This trait allows the pseudo-proof envelope to be upgraded into a real
/// proving system integration without changing the higher-level call pattern.
///
/// A real backend may represent:
/// - Groth16,
/// - PLONK,
/// - Halo2,
/// - STARK,
/// - or any AOXC-specific proof verification adapter.
pub trait ZkpBackend {
    /// Verifies that the proof is cryptographically valid for the supplied
    /// expected public inputs.
    fn verify(&self, proof: &ZkpProof, expected_public_inputs: &[u8]) -> Result<(), ZkpError>;
}

/// Produces a deterministic pseudo-proof artifact for testing and integration wiring.
///
/// Compatibility note:
/// this function remains infallible for existing call sites. Invalid or messy
/// circuit identifiers are normalized into a safe canonical fallback form.
///
/// New code that wants strict validation should prefer `generate_proof_strict`.
#[must_use]
pub fn generate_proof(circuit_id: &str, witness: &[u8], public_inputs: &[u8]) -> ZkpProof {
    let canonical_circuit_id = canonicalize_circuit_id_lossy(circuit_id);
    build_pseudo_proof(&canonical_circuit_id, witness, public_inputs)
}

/// Produces a deterministic pseudo-proof artifact with strict input validation.
///
/// This is the preferred constructor for production code that wants strong
/// input discipline even when using the pseudo-proof path.
pub fn generate_proof_strict(
    circuit_id: &str,
    witness: &[u8],
    public_inputs: &[u8],
) -> Result<ZkpProof, ZkpError> {
    validate_circuit_id(circuit_id)?;
    Ok(build_pseudo_proof(circuit_id, witness, public_inputs))
}

/// Verifies a deterministic pseudo-proof artifact.
///
/// Verification scope:
/// - circuit_id must be canonical,
/// - proof byte layout must be canonical,
/// - public input hash must match,
/// - circuit binding tag must match,
/// - public input binding tag must match.
///
/// Important note:
/// this function validates envelope integrity only. It does not prove witness
/// correctness in the cryptographic zero-knowledge sense.
pub fn verify_proof(proof: &ZkpProof, expected_public_inputs: &[u8]) -> Result<(), String> {
    verify_proof_detailed(proof, expected_public_inputs).map_err(|error| error.code().to_string())
}

/// Detailed pseudo-proof verification returning a structured error.
pub fn verify_proof_detailed(
    proof: &ZkpProof,
    expected_public_inputs: &[u8],
) -> Result<(), ZkpError> {
    proof.validate()?;

    let expected_public_inputs_hash = compute_public_inputs_hash(expected_public_inputs);
    let stored_public_inputs_hash = canonicalize_hash_hex(&proof.public_inputs_hash)?;

    if stored_public_inputs_hash != expected_public_inputs_hash {
        return Err(ZkpError::PublicInputMismatch);
    }

    let expected_circuit_tag = compute_tag(ZKP_CIRCUIT_TAG_DOMAIN, proof.circuit_id.as_bytes(), 8);
    let expected_public_input_tag =
        compute_tag(ZKP_PUBLIC_INPUT_TAG_DOMAIN, expected_public_inputs, 8);

    if proof.proof_bytes[0..8] != expected_circuit_tag[..] {
        return Err(ZkpError::CircuitMismatch);
    }

    if proof.proof_bytes[8..16] != expected_public_input_tag[..] {
        return Err(ZkpError::PublicInputMismatch);
    }

    if proof.proof_bytes[16..32].iter().all(|byte| *byte == 0) {
        return Err(ZkpError::InvalidProofEncoding);
    }

    Ok(())
}

/// Verifies a proof using both local envelope checks and an external backend.
///
/// This is the preferred production-grade entry point once a real proof system
/// backend is available.
pub fn verify_proof_with_backend<B: ZkpBackend>(
    proof: &ZkpProof,
    expected_public_inputs: &[u8],
    backend: &B,
) -> Result<(), ZkpError> {
    verify_proof_detailed(proof, expected_public_inputs)?;
    backend.verify(proof, expected_public_inputs)
}

/// Computes the canonical public-input hash used by the pseudo-proof surface.
#[must_use]
pub fn compute_public_inputs_hash(public_inputs: &[u8]) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(ZKP_PUBLIC_INPUTS_HASH_DOMAIN);
    hasher.update([0x00]);
    hasher.update(public_inputs);

    let digest = hasher.finalize();
    hex::encode_upper(digest)
}

/// Builds the deterministic pseudo-proof bytes.
///
/// Layout:
/// - circuit tag,
/// - public-input tag,
/// - witness-derived tag.
fn build_pseudo_proof(circuit_id: &str, witness: &[u8], public_inputs: &[u8]) -> ZkpProof {
    let public_inputs_hash = compute_public_inputs_hash(public_inputs);

    let circuit_tag = compute_tag(ZKP_CIRCUIT_TAG_DOMAIN, circuit_id.as_bytes(), 8);
    let public_input_tag = compute_tag(ZKP_PUBLIC_INPUT_TAG_DOMAIN, public_inputs, 8);

    let mut witness_material = Vec::with_capacity(
        ZKP_PROOF_DOMAIN.len()
            + 1
            + circuit_id.len()
            + 1
            + public_inputs_hash.len()
            + 1
            + witness.len(),
    );
    witness_material.extend_from_slice(ZKP_PROOF_DOMAIN);
    witness_material.push(0x00);
    witness_material.extend_from_slice(circuit_id.as_bytes());
    witness_material.push(0x00);
    witness_material.extend_from_slice(public_inputs_hash.as_bytes());
    witness_material.push(0x00);
    witness_material.extend_from_slice(witness);

    let witness_tag = compute_tag(ZKP_WITNESS_TAG_DOMAIN, &witness_material, 16);

    let mut proof_bytes = Vec::with_capacity(ZKP_PROOF_BYTES_LEN);
    proof_bytes.extend_from_slice(&circuit_tag);
    proof_bytes.extend_from_slice(&public_input_tag);
    proof_bytes.extend_from_slice(&witness_tag);

    ZkpProof {
        circuit_id: circuit_id.to_string(),
        proof_bytes,
        public_inputs_hash,
    }
}

/// Validates a circuit identifier.
fn validate_circuit_id(circuit_id: &str) -> Result<(), ZkpError> {
    if circuit_id.is_empty() || circuit_id.trim().is_empty() {
        return Err(ZkpError::EmptyCircuitId);
    }

    if circuit_id != circuit_id.trim() {
        return Err(ZkpError::InvalidCircuitId);
    }

    if circuit_id.len() > MAX_CIRCUIT_ID_LEN {
        return Err(ZkpError::InvalidCircuitId);
    }

    if !circuit_id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
    {
        return Err(ZkpError::InvalidCircuitId);
    }

    Ok(())
}

/// Lossy canonicalization used only by the legacy infallible constructor.
///
/// New strict paths remain validation-based and do not use this fallback.
fn canonicalize_circuit_id_lossy(circuit_id: &str) -> String {
    let trimmed = circuit_id.trim();

    if trimmed.is_empty() {
        return "UNSPECIFIED_CIRCUIT".to_string();
    }

    let mut out = String::with_capacity(trimmed.len().min(MAX_CIRCUIT_ID_LEN));

    for ch in trimmed.chars() {
        if out.len() >= MAX_CIRCUIT_ID_LEN {
            break;
        }

        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.' {
            out.push(ch);
        }
    }

    if out.is_empty() {
        "UNSPECIFIED_CIRCUIT".to_string()
    } else {
        out
    }
}

/// Canonicalizes a hexadecimal hash string and validates its format.
fn canonicalize_hash_hex(value: &str) -> Result<String, ZkpError> {
    if value.is_empty() || value.trim().is_empty() || value != value.trim() {
        return Err(ZkpError::InvalidPublicInputsHash);
    }

    if value.len() != ZKP_PUBLIC_INPUTS_HASH_HEX_LEN {
        return Err(ZkpError::InvalidPublicInputsHash);
    }

    let decoded = hex::decode(value).map_err(|_| ZkpError::InvalidPublicInputsHash)?;
    Ok(hex::encode_upper(decoded))
}

/// Computes a deterministic truncated tag under an explicit domain.
fn compute_tag(domain: &[u8], data: &[u8], len: usize) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(domain);
    hasher.update([0x00]);
    hasher.update(data);

    let digest = hasher.finalize();
    digest[..len].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AcceptAllBackend;

    impl ZkpBackend for AcceptAllBackend {
        fn verify(&self, _proof: &ZkpProof, _expected_public_inputs: &[u8]) -> Result<(), ZkpError> {
            Ok(())
        }
    }

    struct RejectAllBackend;

    impl ZkpBackend for RejectAllBackend {
        fn verify(&self, _proof: &ZkpProof, _expected_public_inputs: &[u8]) -> Result<(), ZkpError> {
            Err(ZkpError::BackendRejected)
        }
    }

    #[test]
    fn proof_roundtrip() {
        let proof = generate_proof("transfer_v1", b"witness", b"public-inputs");
        assert!(verify_proof(&proof, b"public-inputs").is_ok());
    }

    #[test]
    fn strict_generation_rejects_invalid_circuit_id() {
        let result = generate_proof_strict(" bad circuit ", b"witness", b"public-inputs");
        assert_eq!(result, Err(ZkpError::InvalidCircuitId));
    }

    #[test]
    fn public_input_mismatch_is_rejected() {
        let proof = generate_proof("transfer_v1", b"witness", b"public-inputs");

        let error = verify_proof(&proof, b"other-public-inputs").unwrap_err();
        assert_eq!(error, "ZKP_PUBLIC_INPUT_MISMATCH");
    }

    #[test]
    fn circuit_tag_mismatch_is_rejected() {
        let mut proof = generate_proof("transfer_v1", b"witness", b"public-inputs");
        proof.circuit_id = "transfer_v2".to_string();

        let error = verify_proof_detailed(&proof, b"public-inputs").unwrap_err();
        assert_eq!(error, ZkpError::CircuitMismatch);
    }

    #[test]
    fn proof_length_is_strict() {
        let mut proof = generate_proof("transfer_v1", b"witness", b"public-inputs");
        proof.proof_bytes = vec![0u8; 16];

        let error = verify_proof_detailed(&proof, b"public-inputs").unwrap_err();
        assert_eq!(error, ZkpError::InvalidProofLength);
    }

    #[test]
    fn public_input_hash_is_uppercase_and_stable() {
        let a = compute_public_inputs_hash(b"public-inputs");
        let b = compute_public_inputs_hash(b"public-inputs");

        assert_eq!(a, b);
        assert_eq!(a, a.to_ascii_uppercase());
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn fingerprint_is_stable() {
        let proof = generate_proof("transfer_v1", b"witness", b"public-inputs");

        let a = proof.fingerprint();
        let b = proof.fingerprint();

        assert_eq!(a, b);
        assert_eq!(a.len(), 16);
    }

    #[test]
    fn json_roundtrip_preserves_proof() {
        let proof = generate_proof("transfer_v1", b"witness", b"public-inputs");
        let json = proof.to_json().expect("serialization must succeed");
        let restored = ZkpProof::from_json(&json).expect("parse must succeed");

        assert_eq!(proof, restored);
    }

    #[test]
    fn generated_proof_has_expected_length() {
        let proof = generate_proof("transfer_v1", b"witness", b"public-inputs");
        assert_eq!(proof.proof_bytes.len(), ZKP_PROOF_BYTES_LEN);
    }

    #[test]
    fn backend_verification_can_accept() {
        let proof = generate_proof("transfer_v1", b"witness", b"public-inputs");
        let backend = AcceptAllBackend;

        let result = verify_proof_with_backend(&proof, b"public-inputs", &backend);
        assert!(result.is_ok());
    }

    #[test]
    fn backend_verification_can_reject() {
        let proof = generate_proof("transfer_v1", b"witness", b"public-inputs");
        let backend = RejectAllBackend;

        let result = verify_proof_with_backend(&proof, b"public-inputs", &backend);
        assert_eq!(result, Err(ZkpError::BackendRejected));
    }
}
