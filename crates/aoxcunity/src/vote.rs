// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::block::PQ_MANDATORY_START_EPOCH;
use crate::validator::ValidatorId;

const VOTE_SIGNING_DOMAIN_V1: &[u8] = b"AOXC_VOTE_SIGNING_V1";
const AUTHENTICATED_VOTE_SIGNING_DOMAIN_V1: &[u8] = b"AOXC_AUTHENTICATED_VOTE_V1";
const SIGNATURE_SCHEME_ED25519: u16 = 1;
const SIGNATURE_SCHEME_HYBRID_ED25519_DILITHIUM3: u16 = 2;
const SIGNATURE_SCHEME_DILITHIUM3: u16 = 3;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthenticatedVote {
    pub vote: Vote,
    pub context: VoteAuthenticationContext,
    pub signature: Vec<u8>,
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

        if self.context.epoch >= PQ_MANDATORY_START_EPOCH
            && !is_post_quantum_hardened_scheme(self.context.signature_scheme)
        {
            return Err(VoteAuthenticationError::PostQuantumPolicyRequired);
        }

        if self.context.signature_scheme == SIGNATURE_SCHEME_DILITHIUM3 {
            return Err(VoteAuthenticationError::UnsupportedVerifierForSignatureScheme);
        }

        let key = VerifyingKey::from_bytes(&self.vote.voter)
            .map_err(|_| VoteAuthenticationError::MalformedPublicKey)?;
        let signature = Signature::from_slice(&self.signature)
            .map_err(|_| VoteAuthenticationError::InvalidSignature)?;
        key.verify(&self.signing_bytes(), &signature)
            .map_err(|_| VoteAuthenticationError::InvalidSignature)?;
        Ok(VerifiedAuthenticatedVote {
            vote: self.vote.clone(),
            context: self.context,
        })
    }
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

#[cfg(test)]
mod tests {
    use crate::block::PQ_MANDATORY_START_EPOCH;
    use ed25519_dalek::{Signer, SigningKey};

    use super::{
        AuthenticatedVote, SIGNATURE_SCHEME_ED25519, SIGNATURE_SCHEME_HYBRID_ED25519_DILITHIUM3,
        SignedVote, Vote, VoteAuthenticationContext, VoteAuthenticationError, VoteKind,
    };

    fn make_vote(block_hash: [u8; 32], round: u64, kind: VoteKind) -> Vote {
        Vote {
            voter: [7u8; 32],
            block_hash,
            height: 9,
            round,
            kind,
        }
    }

    #[test]
    fn signing_bytes_are_deterministic_for_identical_votes() {
        let a = make_vote([1u8; 32], 2, VoteKind::Prepare);
        let b = make_vote([1u8; 32], 2, VoteKind::Prepare);

        assert_eq!(a.signing_bytes(), b.signing_bytes());
    }

    #[test]
    fn signing_bytes_change_with_block_hash() {
        let a = make_vote([1u8; 32], 2, VoteKind::Prepare);
        let b = make_vote([2u8; 32], 2, VoteKind::Prepare);

        assert_ne!(a.signing_bytes(), b.signing_bytes());
    }

    #[test]
    fn signing_bytes_change_with_round() {
        let a = make_vote([1u8; 32], 2, VoteKind::Prepare);
        let b = make_vote([1u8; 32], 3, VoteKind::Prepare);

        assert_ne!(a.signing_bytes(), b.signing_bytes());
    }

    #[test]
    fn signing_bytes_change_with_kind() {
        let a = make_vote([1u8; 32], 2, VoteKind::Prepare);
        let b = make_vote([1u8; 32], 2, VoteKind::Commit);

        assert_ne!(a.signing_bytes(), b.signing_bytes());
    }

    #[test]
    fn signed_vote_verifies_with_matching_public_key() {
        let signing_key = SigningKey::from_bytes(&[7u8; 32]);
        let vote = Vote {
            voter: signing_key.verifying_key().to_bytes(),
            block_hash: [1u8; 32],
            height: 9,
            round: 2,
            kind: VoteKind::Commit,
        };
        let signature = signing_key.sign(&vote.signing_bytes()).to_bytes().to_vec();

        let verified = SignedVote { vote, signature }.verify();
        assert!(verified.is_ok());
    }

    #[test]
    fn modified_vote_payload_breaks_signature() {
        let signing_key = SigningKey::from_bytes(&[7u8; 32]);
        let mut vote = Vote {
            voter: signing_key.verifying_key().to_bytes(),
            block_hash: [1u8; 32],
            height: 9,
            round: 2,
            kind: VoteKind::Commit,
        };
        let signature = signing_key.sign(&vote.signing_bytes()).to_bytes().to_vec();
        vote.round = 3;

        let verified = SignedVote { vote, signature }.verify();
        assert_eq!(verified, Err(VoteAuthenticationError::InvalidSignature));
    }

    #[test]
    fn authenticated_vote_binds_context_into_signature() {
        let signing_key = SigningKey::from_bytes(&[7u8; 32]);
        let vote = Vote {
            voter: signing_key.verifying_key().to_bytes(),
            block_hash: [1u8; 32],
            height: 9,
            round: 2,
            kind: VoteKind::Commit,
        };
        let context = VoteAuthenticationContext {
            network_id: 2626,
            epoch: 4,
            validator_set_root: [5u8; 32],
            pq_attestation_root: [11u8; 32],
            signature_scheme: SIGNATURE_SCHEME_ED25519,
        };
        let mut authenticated = AuthenticatedVote {
            vote,
            context,
            signature: Vec::new(),
        };
        authenticated.signature = signing_key
            .sign(&authenticated.signing_bytes())
            .to_bytes()
            .to_vec();

        assert!(authenticated.verify().is_ok());

        authenticated.context.epoch = 5;
        assert_eq!(
            authenticated.verify(),
            Err(VoteAuthenticationError::InvalidSignature)
        );
    }

    #[test]
    fn authenticated_vote_rejects_unknown_signature_scheme() {
        let signing_key = SigningKey::from_bytes(&[7u8; 32]);
        let vote = Vote {
            voter: signing_key.verifying_key().to_bytes(),
            block_hash: [1u8; 32],
            height: 9,
            round: 2,
            kind: VoteKind::Commit,
        };
        let mut authenticated = AuthenticatedVote {
            vote,
            context: VoteAuthenticationContext {
                network_id: 2626,
                epoch: 4,
                validator_set_root: [5u8; 32],
                pq_attestation_root: [11u8; 32],
                signature_scheme: 999,
            },
            signature: Vec::new(),
        };
        authenticated.signature = signing_key
            .sign(&authenticated.signing_bytes())
            .to_bytes()
            .to_vec();

        assert_eq!(
            authenticated.verify(),
            Err(VoteAuthenticationError::UnknownSignatureScheme)
        );
    }

    #[test]
    fn authenticated_vote_requires_pq_hardened_scheme_after_cutover_epoch() {
        let signing_key = SigningKey::from_bytes(&[7u8; 32]);
        let vote = Vote {
            voter: signing_key.verifying_key().to_bytes(),
            block_hash: [1u8; 32],
            height: 9,
            round: 2,
            kind: VoteKind::Commit,
        };
        let mut authenticated = AuthenticatedVote {
            vote,
            context: VoteAuthenticationContext {
                network_id: 2626,
                epoch: PQ_MANDATORY_START_EPOCH,
                validator_set_root: [5u8; 32],
                pq_attestation_root: [11u8; 32],
                signature_scheme: SIGNATURE_SCHEME_ED25519,
            },
            signature: Vec::new(),
        };
        authenticated.signature = signing_key
            .sign(&authenticated.signing_bytes())
            .to_bytes()
            .to_vec();

        assert_eq!(
            authenticated.verify(),
            Err(VoteAuthenticationError::PostQuantumPolicyRequired)
        );
    }

    #[test]
    fn authenticated_vote_accepts_hybrid_scheme_after_cutover_epoch() {
        let signing_key = SigningKey::from_bytes(&[7u8; 32]);
        let vote = Vote {
            voter: signing_key.verifying_key().to_bytes(),
            block_hash: [1u8; 32],
            height: 9,
            round: 2,
            kind: VoteKind::Commit,
        };
        let mut authenticated = AuthenticatedVote {
            vote,
            context: VoteAuthenticationContext {
                network_id: 2626,
                epoch: PQ_MANDATORY_START_EPOCH,
                validator_set_root: [5u8; 32],
                pq_attestation_root: [11u8; 32],
                signature_scheme: SIGNATURE_SCHEME_HYBRID_ED25519_DILITHIUM3,
            },
            signature: Vec::new(),
        };
        authenticated.signature = signing_key
            .sign(&authenticated.signing_bytes())
            .to_bytes()
            .to_vec();

        assert!(authenticated.verify().is_ok());
    }
}
