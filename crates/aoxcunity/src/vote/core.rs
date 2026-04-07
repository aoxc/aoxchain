// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use pqcrypto_mldsa::mldsa65::{PublicKey as DilithiumPublicKey, SignedMessage, open};
use pqcrypto_traits::sign::{PublicKey as _, SignedMessage as _};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::block::PQ_MANDATORY_START_EPOCH;
use crate::validator::ValidatorId;

const VOTE_SIGNING_DOMAIN_V1: &[u8] = b"AOXC_VOTE_SIGNING_V1";
const AUTHENTICATED_VOTE_SIGNING_DOMAIN_V1: &[u8] = b"AOXC_AUTHENTICATED_VOTE_V1";
pub const SIGNATURE_SCHEME_ED25519: u16 = 1;
pub const SIGNATURE_SCHEME_HYBRID_ED25519_DILITHIUM3: u16 = 2;
pub const SIGNATURE_SCHEME_DILITHIUM3: u16 = 3;

/// Vote kind classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VoteKind {
    Prepare,
    Commit,
}

impl VoteKind {
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Vote {
    pub voter: ValidatorId,
    pub block_hash: [u8; 32],
    pub height: u64,
    pub round: u64,
    pub kind: VoteKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedVote {
    pub vote: Vote,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoteAuthenticationContext {
    pub network_id: u32,
    pub epoch: u64,
    pub validator_set_root: [u8; 32],
    pub pq_attestation_root: [u8; 32],
    pub signature_scheme: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsensusIdentityProfile {
    Classical,
    Hybrid,
    PostQuantum,
}

impl VoteAuthenticationContext {
    pub fn identity_profile(&self) -> Result<ConsensusIdentityProfile, VoteAuthenticationError> {
        match self.signature_scheme {
            SIGNATURE_SCHEME_ED25519 => Ok(ConsensusIdentityProfile::Classical),
            SIGNATURE_SCHEME_HYBRID_ED25519_DILITHIUM3 => Ok(ConsensusIdentityProfile::Hybrid),
            SIGNATURE_SCHEME_DILITHIUM3 => Ok(ConsensusIdentityProfile::PostQuantum),
            _ => Err(VoteAuthenticationError::UnknownSignatureScheme),
        }
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedVote {
    pub vote: Vote,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedAuthenticatedVote {
    pub vote: Vote,
    pub context: VoteAuthenticationContext,
}

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
}

impl Vote {
    pub fn unique_key(&self) -> ([u8; 32], ValidatorId, u64, u64, VoteKind) {
        (
            self.block_hash,
            self.voter,
            self.height,
            self.round,
            self.kind,
        )
    }

    pub fn conflict_key(&self) -> (ValidatorId, u64, u64, VoteKind) {
        (self.voter, self.height, self.round, self.kind)
    }

    /// Returns deterministic domain-separated signing bytes for authenticated
    /// vote envelopes.
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
            && !is_post_quantum_hardened_scheme(self.context.signature_scheme)
        {
            return Err(VoteAuthenticationError::PostQuantumPolicyRequired);
        }

        let signing_bytes = self.signing_bytes();
        match self.context.signature_scheme {
            SIGNATURE_SCHEME_ED25519 => self.verify_ed25519_only(&signing_bytes)?,
            SIGNATURE_SCHEME_HYBRID_ED25519_DILITHIUM3 => {
                self.verify_ed25519_only(&signing_bytes)?;
                self.verify_dilithium_only(&signing_bytes)?;
            }
            SIGNATURE_SCHEME_DILITHIUM3 => self.verify_dilithium_only(&signing_bytes)?,
            _ => return Err(VoteAuthenticationError::UnsupportedVerifierForSignatureScheme),
        }

        Ok(VerifiedAuthenticatedVote {
            vote: self.vote.clone(),
            context: self.context,
        })
    }

    fn verify_ed25519_only(&self, signing_bytes: &[u8]) -> Result<(), VoteAuthenticationError> {
        let key = VerifyingKey::from_bytes(&self.vote.voter)
            .map_err(|_| VoteAuthenticationError::MalformedPublicKey)?;
        let signature = Signature::from_slice(&self.signature)
            .map_err(|_| VoteAuthenticationError::InvalidSignature)?;
        key.verify(signing_bytes, &signature)
            .map_err(|_| VoteAuthenticationError::InvalidSignature)
    }

    fn verify_dilithium_only(&self, signing_bytes: &[u8]) -> Result<(), VoteAuthenticationError> {
        let public_key_bytes = self
            .pq_public_key
            .as_deref()
            .ok_or(VoteAuthenticationError::MissingPostQuantumPublicKey)?;
        let public_key = DilithiumPublicKey::from_bytes(public_key_bytes)
            .map_err(|_| VoteAuthenticationError::MalformedPublicKey)?;

        let signature_bytes = self
            .pq_signature
            .as_deref()
            .or({
                if self.context.signature_scheme == SIGNATURE_SCHEME_DILITHIUM3 {
                    Some(self.signature.as_slice())
                } else {
                    None
                }
            })
            .ok_or(VoteAuthenticationError::MissingPostQuantumSignature)?;

        let mut signed_message_bytes =
            Vec::with_capacity(signature_bytes.len() + signing_bytes.len());
        signed_message_bytes.extend_from_slice(signature_bytes);
        signed_message_bytes.extend_from_slice(signing_bytes);
        let signed_message = SignedMessage::from_bytes(&signed_message_bytes)
            .map_err(|_| VoteAuthenticationError::InvalidSignature)?;

        let opened = open(&signed_message, &public_key)
            .map_err(|_| VoteAuthenticationError::InvalidSignature)?;
        if opened.as_slice() != signing_bytes {
            return Err(VoteAuthenticationError::InvalidSignature);
        }

        Ok(())
    }
}

fn is_zero_hash32(value: &[u8; 32]) -> bool {
    value.iter().all(|byte| *byte == 0)
}

fn is_known_signature_scheme(signature_scheme: u16) -> bool {
    matches!(
        signature_scheme,
        SIGNATURE_SCHEME_ED25519
            | SIGNATURE_SCHEME_HYBRID_ED25519_DILITHIUM3
            | SIGNATURE_SCHEME_DILITHIUM3
    )
}

fn is_post_quantum_hardened_scheme(signature_scheme: u16) -> bool {
    matches!(
        signature_scheme,
        SIGNATURE_SCHEME_HYBRID_ED25519_DILITHIUM3 | SIGNATURE_SCHEME_DILITHIUM3
    )
}

impl SignedVote {
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
