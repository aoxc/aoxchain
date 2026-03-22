use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::validator::ValidatorId;

const VOTE_SIGNING_DOMAIN_V1: &[u8] = b"AOXC_VOTE_SIGNING_V1";

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedVote {
    pub vote: Vote,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum VoteAuthenticationError {
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
    use ed25519_dalek::{Signer, SigningKey};

    use super::{SignedVote, Vote, VoteAuthenticationError, VoteKind};

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
}
