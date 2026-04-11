// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use libcrux_ml_dsa::ml_dsa_65::{MLDSA65Signature, MLDSA65VerificationKey, verify};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::block::PQ_MANDATORY_START_EPOCH;
use crate::validator::ValidatorId;

const VOTE_SIGNING_DOMAIN_V1: &[u8] = b"AOXC_VOTE_SIGNING_V1";
const AUTHENTICATED_VOTE_SIGNING_DOMAIN_V1: &[u8] = b"AOXC_AUTHENTICATED_VOTE_V1";
const PQ_VALIDATOR_ID_BINDING_DOMAIN_V1: &[u8] = b"AOXC_PQ_VALIDATOR_ID_BINDING_V1";

pub const SIGNATURE_SCHEME_ED25519: u16 = 1;
pub const SIGNATURE_SCHEME_HYBRID_ED25519_DILITHIUM3: u16 = 2;
pub const SIGNATURE_SCHEME_DILITHIUM3: u16 = 3;

const ML_DSA_CONTEXT: &[u8] = b"";
const ML_DSA_65_SIGNATURE_SIZE: usize = 3309;
const ML_DSA_65_VERIFICATION_KEY_SIZE: usize = 1952;

/// Vote kind classification.
///
/// The discriminant is part of the canonical signing payload and must remain
/// stable across all implementations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VoteKind {
    Prepare,
    Commit,
}

impl VoteKind {
    #[must_use]
    pub fn discriminant(self) -> u8 {
        match self {
            Self::Prepare => 0,
            Self::Commit => 1,
        }
    }
}

/// Canonical consensus vote.
///
/// The vote commits to a specific block hash at a specific height and round.
/// The payload is intentionally compact and deterministic because it is used
/// as the root message body for both classical and authenticated vote forms.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Vote {
    pub voter: ValidatorId,
    pub block_hash: [u8; 32],
    pub height: u64,
    pub round: u64,
    pub kind: VoteKind,
}

/// Legacy classical vote envelope.
///
/// This structure is intentionally retained as a distinct verification path in
/// order to prevent policy confusion between legacy votes and context-bound
/// authenticated votes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedVote {
    pub vote: Vote,
    pub signature: Vec<u8>,
}

/// Authentication context that binds a vote to network, epoch, validator-set,
/// PQ attestation state, and the claimed signature scheme.
///
/// Security note:
/// This structure is fully committed into the authenticated vote signing bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoteAuthenticationContext {
    pub network_id: u32,
    pub epoch: u64,
    pub validator_set_root: [u8; 32],
    pub pq_attestation_root: [u8; 32],
    pub signature_scheme: u16,
}

/// Consensus identity profile derived from the claimed signature scheme.
///
/// This abstraction exists to make downstream policy enforcement explicit and
/// auditable, rather than scattering raw scheme constants throughout the code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsensusIdentityProfile {
    Classical,
    Hybrid,
    PostQuantum,
}

impl VoteAuthenticationContext {
    /// Resolves the signature scheme claim into the corresponding consensus
    /// identity profile.
    ///
    /// Security rationale:
    /// - The mapping is authoritative for downstream policy enforcement.
    /// - Unknown values must fail closed to prevent silent acceptance of an
    ///   unsupported or downgraded signature profile.
    pub fn identity_profile(&self) -> Result<ConsensusIdentityProfile, VoteAuthenticationError> {
        match self.signature_scheme {
            SIGNATURE_SCHEME_ED25519 => Ok(ConsensusIdentityProfile::Classical),
            SIGNATURE_SCHEME_HYBRID_ED25519_DILITHIUM3 => Ok(ConsensusIdentityProfile::Hybrid),
            SIGNATURE_SCHEME_DILITHIUM3 => Ok(ConsensusIdentityProfile::PostQuantum),
            _ => Err(VoteAuthenticationError::UnknownSignatureScheme),
        }
    }
}

/// Context-bound authenticated vote envelope.
///
/// Encoding model:
/// - `signature` always carries the primary signature field of the envelope.
/// - In hybrid mode, `signature` carries the Ed25519 signature and
///   `pq_signature` carries the ML-DSA-65 signature.
/// - In pure post-quantum mode, `signature` may carry the ML-DSA-65 signature
///   when `pq_signature` is not separately populated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthenticatedVote {
    pub vote: Vote,
    pub context: VoteAuthenticationContext,
    pub signature: Vec<u8>,
    #[serde(default)]
    pub pq_public_key: Option<Vec<u8>>,
    #[serde(default)]
    pub pq_signature: Option<Vec<u8>>,
}

/// Verified legacy vote wrapper.
///
/// This type ensures that downstream code cannot accidentally treat an
/// unverified legacy vote as trusted input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedVote {
    pub vote: Vote,
}

/// Verified authenticated vote wrapper.
///
/// This type ensures that downstream code cannot accidentally treat an
/// unverified authenticated vote as trusted input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedAuthenticatedVote {
    pub vote: Vote,
    pub context: VoteAuthenticationContext,
}

/// Deterministic vote authentication failure surface.
///
/// Error categories are intentionally coarse enough to avoid ambiguous behavior
/// while still preserving operational debuggability.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum VoteAuthenticationError {
    #[error("vote signature scheme is unknown")]
    UnknownSignatureScheme,

    #[error("vote signature verifier does not support the claimed scheme")]
    UnsupportedVerifierForSignatureScheme,

    #[error("vote requires post-quantum signature policy for this epoch")]
    PostQuantumPolicyRequired,

    #[error("vote public key is malformed")]
    MalformedPublicKey,

    #[error("vote signature is invalid")]
    InvalidSignature,

    #[error("vote requires an explicit post-quantum public key")]
    MissingPostQuantumPublicKey,

    #[error("vote requires an explicit post-quantum signature")]
    MissingPostQuantumSignature,

    #[error(
        "vote requires a non-zero post-quantum attestation root for hybrid/post-quantum identity profiles"
    )]
    MissingPostQuantumAttestationRoot,

    #[error("vote post-quantum identity binding does not match validator identifier")]
    PostQuantumIdentityBindingMismatch,
}

impl Vote {
    /// Returns the fully unique tuple for the vote.
    ///
    /// This key is suitable when the block hash must remain part of identity.
    #[must_use]
    pub fn unique_key(&self) -> ([u8; 32], ValidatorId, u64, u64, VoteKind) {
        (
            self.block_hash,
            self.voter,
            self.height,
            self.round,
            self.kind,
        )
    }

    /// Returns the conflict-detection key for the vote.
    ///
    /// This key intentionally excludes the block hash so that conflicting votes
    /// by the same validator for the same height/round/kind can be detected.
    #[must_use]
    pub fn conflict_key(&self) -> (ValidatorId, u64, u64, VoteKind) {
        (self.voter, self.height, self.round, self.kind)
    }

    /// Returns deterministic domain-separated signing bytes for the canonical
    /// vote payload.
    ///
    /// Security rationale:
    /// - Domain separation prevents cross-message and cross-protocol replay.
    /// - Field ordering is canonical and must remain stable for all signing
    ///   and verification implementations.
    #[must_use]
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(17 + 32 + 32 + 8 + 8 + 1);
        bytes.extend_from_slice(VOTE_SIGNING_DOMAIN_V1);
        bytes.extend_from_slice(&self.block_hash);
        bytes.extend_from_slice(&self.voter);
        bytes.extend_from_slice(&self.height.to_le_bytes());
        bytes.extend_from_slice(&self.round.to_le_bytes());
        bytes.push(self.kind.discriminant());
        bytes
    }
}

impl AuthenticatedVote {
    /// Returns deterministic domain-separated signing bytes for the
    /// authenticated vote envelope.
    ///
    /// Security rationale:
    /// - The full authentication context is cryptographically bound to the
    ///   canonical vote body.
    /// - This prevents replay across networks, epochs, validator sets,
    ///   attestation roots, or claimed signature schemes.
    #[must_use]
    pub fn signing_bytes(&self) -> Vec<u8> {
        let vote_bytes = self.vote.signing_bytes();
        let mut bytes = Vec::with_capacity(
            AUTHENTICATED_VOTE_SIGNING_DOMAIN_V1.len() + vote_bytes.len() + 4 + 8 + 32 + 32 + 2,
        );

        bytes.extend_from_slice(AUTHENTICATED_VOTE_SIGNING_DOMAIN_V1);
        bytes.extend_from_slice(&self.context.network_id.to_le_bytes());
        bytes.extend_from_slice(&self.context.epoch.to_le_bytes());
        bytes.extend_from_slice(&self.context.validator_set_root);
        bytes.extend_from_slice(&self.context.pq_attestation_root);
        bytes.extend_from_slice(&self.context.signature_scheme.to_le_bytes());
        bytes.extend_from_slice(&vote_bytes);
        bytes
    }

    /// Verifies the authenticated vote against the claimed identity profile
    /// and epoch policy.
    ///
    /// Verification order is intentionally fail closed:
    /// 1. Recognize the claimed signature scheme.
    /// 2. Resolve the identity profile.
    /// 3. Enforce PQ attestation-root policy.
    /// 4. Enforce epoch-level mandatory PQ policy.
    /// 5. Execute cryptographic verification.
    ///
    /// Security rationale:
    /// - Unknown schemes must never degrade into permissive handling.
    /// - Hybrid and post-quantum profiles require non-zero PQ attestation
    ///   material.
    /// - After the mandatory PQ epoch, only PQ-only signatures are accepted.
    pub fn verify(&self) -> Result<VerifiedAuthenticatedVote, VoteAuthenticationError> {
        if !is_known_signature_scheme(self.context.signature_scheme) {
            return Err(VoteAuthenticationError::UnknownSignatureScheme);
        }

        let identity_profile = self.context.identity_profile()?;
        if matches!(
            identity_profile,
            ConsensusIdentityProfile::Hybrid | ConsensusIdentityProfile::PostQuantum
        ) && is_zero_hash32(&self.context.pq_attestation_root)
        {
            return Err(VoteAuthenticationError::MissingPostQuantumAttestationRoot);
        }

        if self.context.epoch >= PQ_MANDATORY_START_EPOCH
            && self.context.signature_scheme != SIGNATURE_SCHEME_DILITHIUM3
        {
            return Err(VoteAuthenticationError::PostQuantumPolicyRequired);
        }

        let signing_bytes = self.signing_bytes();

        match self.context.signature_scheme {
            SIGNATURE_SCHEME_ED25519 => {
                self.verify_ed25519_only(&signing_bytes)?;
            }
            SIGNATURE_SCHEME_HYBRID_ED25519_DILITHIUM3 => {
                self.verify_ed25519_only(&signing_bytes)?;
                self.verify_mldsa65_only(&signing_bytes)?;
            }
            SIGNATURE_SCHEME_DILITHIUM3 => {
                self.verify_pq_identity_binding()?;
                self.verify_mldsa65_only(&signing_bytes)?;
            }
            _ => {
                return Err(VoteAuthenticationError::UnsupportedVerifierForSignatureScheme);
            }
        }

        Ok(VerifiedAuthenticatedVote {
            vote: self.vote.clone(),
            context: self.context,
        })
    }

    /// Verifies the Ed25519 component carried in `self.signature`.
    ///
    /// Security rationale:
    /// - The validator identifier is interpreted as the canonical Ed25519
    ///   verifying key for the vote author.
    /// - Malformed keys and invalid signatures are reported through explicit,
    ///   deterministic verification errors.
    fn verify_ed25519_only(&self, signing_bytes: &[u8]) -> Result<(), VoteAuthenticationError> {
        let key = VerifyingKey::from_bytes(&self.vote.voter)
            .map_err(|_| VoteAuthenticationError::MalformedPublicKey)?;

        let signature = Signature::from_slice(&self.signature)
            .map_err(|_| VoteAuthenticationError::InvalidSignature)?;

        key.verify(signing_bytes, &signature)
            .map_err(|_| VoteAuthenticationError::InvalidSignature)
    }

    /// Verifies the ML-DSA-65 component using the explicit post-quantum public
    /// key and the scheme-resolved signature payload.
    ///
    /// Encoding model:
    /// - In hybrid mode, the PQ signature must be carried in `pq_signature`.
    /// - In pure post-quantum mode, `self.signature` may be interpreted as the
    ///   PQ signature payload when `pq_signature` is not separately populated.
    ///
    /// Security rationale:
    /// - The PQ public key is mandatory for all ML-DSA verification paths.
    /// - Key and signature sizes are validated before cryptographic
    ///   verification to fail closed on malformed input.
    /// - Signature source selection is explicit and centralized to prevent
    ///   ambiguous envelope interpretation.
    fn verify_mldsa65_only(&self, signing_bytes: &[u8]) -> Result<(), VoteAuthenticationError> {
        let public_key = self.decode_mldsa65_public_key()?;
        let signature = self.decode_mldsa65_signature()?;

        verify(&public_key, signing_bytes, ML_DSA_CONTEXT, &signature)
            .map_err(|_| VoteAuthenticationError::InvalidSignature)?;

        Ok(())
    }

    /// Decodes and validates the ML-DSA-65 verification key.
    ///
    /// Security rationale:
    /// - The PQ verification key must be explicitly present.
    /// - Structural validation is completed before constructing the typed
    ///   verification key consumed by the cryptographic backend.
    fn decode_mldsa65_public_key(&self) -> Result<MLDSA65VerificationKey, VoteAuthenticationError> {
        let public_key_bytes = self
            .pq_public_key
            .as_deref()
            .ok_or(VoteAuthenticationError::MissingPostQuantumPublicKey)?;

        if public_key_bytes.len() != ML_DSA_65_VERIFICATION_KEY_SIZE {
            return Err(VoteAuthenticationError::MalformedPublicKey);
        }

        let mut public_key_array = [0u8; ML_DSA_65_VERIFICATION_KEY_SIZE];
        public_key_array.copy_from_slice(public_key_bytes);

        Ok(MLDSA65VerificationKey::new(public_key_array))
    }

    /// Decodes and validates the ML-DSA-65 signature.
    ///
    /// Security rationale:
    /// - Signature source resolution is explicit and deterministic.
    /// - Hybrid envelopes require the dedicated `pq_signature` field.
    /// - Pure post-quantum envelopes may reuse `self.signature` as the PQ
    ///   signature payload when no dedicated PQ field is supplied.
    fn decode_mldsa65_signature(&self) -> Result<MLDSA65Signature, VoteAuthenticationError> {
        let signature_bytes = self.resolve_mldsa65_signature_bytes()?;

        if signature_bytes.len() != ML_DSA_65_SIGNATURE_SIZE {
            return Err(VoteAuthenticationError::InvalidSignature);
        }

        let mut signature_array = [0u8; ML_DSA_65_SIGNATURE_SIZE];
        signature_array.copy_from_slice(signature_bytes);

        Ok(MLDSA65Signature::new(signature_array))
    }

    /// Resolves the byte slice that must be interpreted as the ML-DSA-65
    /// signature payload for the current authenticated vote.
    ///
    /// Resolution policy:
    /// - Prefer `pq_signature` when explicitly present.
    /// - Otherwise, in pure post-quantum mode, interpret `self.signature` as
    ///   the PQ signature payload.
    /// - All other cases fail closed.
    ///
    /// Security rationale:
    /// - This function centralizes envelope interpretation so that signature
    ///   source selection remains consistent across all verification paths.
    fn resolve_mldsa65_signature_bytes(&self) -> Result<&[u8], VoteAuthenticationError> {
        if let Some(signature_bytes) = self.pq_signature.as_deref() {
            return Ok(signature_bytes);
        }

        if self.context.signature_scheme == SIGNATURE_SCHEME_DILITHIUM3 {
            return Ok(self.signature.as_slice());
        }

        Err(VoteAuthenticationError::MissingPostQuantumSignature)
    }

    /// Enforces deterministic validator identity binding for post-quantum-only
    /// signatures.
    ///
    /// Security rationale:
    /// - In PQ-only mode, the validator identifier can no longer rely on an
    ///   Ed25519 key relationship.
    /// - Binding `vote.voter` to a deterministic digest of the PQ public key
    ///   prevents arbitrary voter-identifier claims with unrelated PQ key
    ///   material.
    fn verify_pq_identity_binding(&self) -> Result<(), VoteAuthenticationError> {
        let public_key_bytes = self
            .pq_public_key
            .as_deref()
            .ok_or(VoteAuthenticationError::MissingPostQuantumPublicKey)?;

        if public_key_bytes.len() != ML_DSA_65_VERIFICATION_KEY_SIZE {
            return Err(VoteAuthenticationError::MalformedPublicKey);
        }

        let expected_validator_id = derive_pq_validator_id(public_key_bytes);
        if self.vote.voter != expected_validator_id {
            return Err(VoteAuthenticationError::PostQuantumIdentityBindingMismatch);
        }

        Ok(())
    }
}

/// Derives the deterministic validator identifier for PQ-only identities.
///
/// Security rationale:
/// - Domain separation prevents digest reuse across unrelated derivation
///   contexts.
/// - The serialized length is included to preserve an unambiguous transcript.
#[must_use]
fn derive_pq_validator_id(public_key_bytes: &[u8]) -> ValidatorId {
    let mut hasher = Sha256::new();
    hasher.update(PQ_VALIDATOR_ID_BINDING_DOMAIN_V1);
    hasher.update((public_key_bytes.len() as u64).to_le_bytes());
    hasher.update(public_key_bytes);
    hasher.finalize().into()
}

/// Returns `true` when the supplied 32-byte value is all-zero.
///
/// This is used as an explicit structural policy guard for attestation roots.
#[must_use]
fn is_zero_hash32(value: &[u8; 32]) -> bool {
    value.iter().all(|byte| *byte == 0)
}

/// Returns `true` if the supplied signature scheme identifier is recognized by
/// the current verifier implementation.
///
/// Unknown values are intentionally rejected by policy before any signature
/// verification path is selected.
#[must_use]
fn is_known_signature_scheme(signature_scheme: u16) -> bool {
    matches!(
        signature_scheme,
        SIGNATURE_SCHEME_ED25519
            | SIGNATURE_SCHEME_HYBRID_ED25519_DILITHIUM3
            | SIGNATURE_SCHEME_DILITHIUM3
    )
}

impl SignedVote {
    /// Verifies the classical vote signature format.
    ///
    /// Security rationale:
    /// - This verification path is intentionally scoped to the legacy
    ///   classical vote envelope.
    /// - It remains separate from authenticated-vote verification to avoid
    ///   policy confusion between legacy and context-bound vote forms.
    pub fn verify(&self) -> Result<VerifiedVote, VoteAuthenticationError> {
        let key = VerifyingKey::from_bytes(&self.vote.voter)
            .map_err(|_| VoteAuthenticationError::MalformedPublicKey)?;

        let signature = Signature::from_slice(&self.signature)
            .map_err(|_| VoteAuthenticationError::InvalidSignature)?;

        key.verify(&self.vote.signing_bytes(), &signature)
            .map_err(|_| VoteAuthenticationError::InvalidSignature)?;

        Ok(VerifiedVote {
            vote: self.vote.clone(),
        })
    }
}

impl VerifiedVote {
    #[must_use]
    pub fn into_vote(self) -> Vote {
        self.vote
    }
}
