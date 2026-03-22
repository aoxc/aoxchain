use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::{Vote, VoteKind};

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
}
