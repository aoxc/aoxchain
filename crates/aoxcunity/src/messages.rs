use serde::{Deserialize, Serialize};

use crate::block::Block;
use crate::seal::BlockSeal;
use crate::vote::Vote;

/// Canonical consensus message surface.
///
/// Network transports remain free to wrap these messages as needed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsensusMessage {
    BlockProposal { block: Block },
    Vote(Vote),
    Finalize { seal: BlockSeal },
}

impl ConsensusMessage {
    /// Returns deterministic bytes for authenticated transport binding.
    ///
    /// This method does not perform signature verification. It provides the
    /// canonical payload foundation required for future authenticated network
    /// envelopes without relying on text serialization.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        match self {
            Self::BlockProposal { block } => {
                let mut bytes = Vec::with_capacity(1 + 32);
                bytes.push(0);
                bytes.extend_from_slice(&block.hash);
                bytes
            }
            Self::Vote(vote) => {
                let mut bytes = Vec::with_capacity(1 + vote.signing_bytes().len());
                bytes.push(1);
                bytes.extend_from_slice(&vote.signing_bytes());
                bytes
            }
            Self::Finalize { seal } => {
                let mut bytes = Vec::with_capacity(1 + 32 + 8 + 32);
                bytes.push(2);
                bytes.extend_from_slice(&seal.block_hash);
                bytes.extend_from_slice(&seal.finalized_round.to_le_bytes());
                bytes.extend_from_slice(&seal.attestation_root);
                bytes
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::seal::BlockSeal;
    use crate::vote::{Vote, VoteKind};

    use super::ConsensusMessage;

    #[test]
    fn canonical_vote_message_bytes_are_deterministic() {
        let vote = Vote {
            voter: [8u8; 32],
            block_hash: [9u8; 32],
            height: 10,
            round: 4,
            kind: VoteKind::Commit,
        };

        let a = ConsensusMessage::Vote(vote.clone()).canonical_bytes();
        let b = ConsensusMessage::Vote(vote).canonical_bytes();

        assert_eq!(a, b);
    }

    #[test]
    fn canonical_finalize_message_bytes_include_seal_material() {
        let seal = BlockSeal {
            block_hash: [3u8; 32],
            finalized_round: 5,
            attestation_root: [4u8; 32],
        };

        let bytes = ConsensusMessage::Finalize { seal }.canonical_bytes();

        assert!(bytes.ends_with(&[4u8; 32]));
    }
}
