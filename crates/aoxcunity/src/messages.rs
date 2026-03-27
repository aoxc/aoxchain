// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};

use crate::block::Block;
use crate::seal::{AuthenticatedQuorumCertificate, BlockSeal};
use crate::vote::AuthenticatedVote;

/// Canonical consensus message surface.
///
/// # Design Intent
/// This enum defines the transport-facing message classes exchanged by the
/// consensus subsystem. The message model remains intentionally compact and
/// consensus-oriented.
///
/// # Security Note
/// These messages are canonical protocol artifacts. Network transports may wrap
/// them in authenticated envelopes, but they must not alter the deterministic
/// payload semantics defined here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsensusMessage {
    BlockProposal {
        block: Block,
    },
    Vote(AuthenticatedVote),
    Finalize {
        seal: BlockSeal,
        certificate: AuthenticatedQuorumCertificate,
    },
}

impl ConsensusMessage {
    /// Returns deterministic bytes for authenticated transport binding.
    ///
    /// # Security Rationale
    /// This method provides the canonical byte representation required for
    /// authenticated transport and future signature binding. It intentionally
    /// avoids text-based serialization in order to prevent ambiguity, encoding
    /// drift, or implementation-dependent payload variation.
    ///
    /// # Encoding Rules
    /// - The first byte is a stable message discriminator.
    /// - All numeric fields use little-endian encoding.
    /// - Hash-like fields are appended in fixed-width canonical order.
    /// - Finalization messages bind both attestation material and certificate
    ///   identity so that transport authentication commits to the full
    ///   finality artifact.
    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        match self {
            Self::BlockProposal { block } => {
                let mut bytes = Vec::with_capacity(1 + 32);
                bytes.push(0);
                bytes.extend_from_slice(&block.hash);
                bytes
            }
            Self::Vote(vote) => {
                let signing_bytes = vote.signing_bytes();
                let mut bytes = Vec::with_capacity(1 + signing_bytes.len());
                bytes.push(1);
                bytes.extend_from_slice(&signing_bytes);
                bytes
            }
            Self::Finalize { seal, certificate } => {
                let mut bytes = Vec::with_capacity(1 + 32 + 8 + 32 + 32 + 32);
                bytes.push(2);
                bytes.extend_from_slice(&seal.block_hash);
                bytes.extend_from_slice(&seal.finalized_round.to_le_bytes());
                bytes.extend_from_slice(&seal.attestation_root);
                bytes.extend_from_slice(&seal.certificate.certificate_hash);
                bytes.extend_from_slice(&certificate.authenticated_hash);
                bytes
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::seal::{AuthenticatedQuorumCertificate, BlockSeal, QuorumCertificate};
    use crate::vote::{AuthenticatedVote, Vote, VoteAuthenticationContext, VoteKind};

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
        let authenticated = AuthenticatedVote {
            vote,
            context: VoteAuthenticationContext {
                network_id: 2626,
                epoch: 1,
                validator_set_root: [7u8; 32],
                signature_scheme: 1,
            },
            signature: vec![5u8; 64],
        };

        let a = ConsensusMessage::Vote(authenticated.clone()).canonical_bytes();
        let b = ConsensusMessage::Vote(authenticated).canonical_bytes();

        assert_eq!(a, b);
    }

    #[test]
    fn canonical_finalize_message_bytes_include_seal_material() {
        let certificate =
            QuorumCertificate::new([3u8; 32], 5, 5, vec![[1u8; 32], [2u8; 32]], 2, 3, 2, 3);

        let seal = BlockSeal {
            block_hash: [3u8; 32],
            finalized_round: 5,
            attestation_root: [4u8; 32],
            certificate,
        };
        let authenticated_certificate =
            AuthenticatedQuorumCertificate::new(seal.certificate.clone(), 2626, 1, [6u8; 32], 1);

        let certificate_hash = seal.certificate.certificate_hash;
        let authenticated_hash = authenticated_certificate.authenticated_hash;
        let bytes = ConsensusMessage::Finalize {
            seal,
            certificate: authenticated_certificate,
        }
        .canonical_bytes();

        assert!(bytes.windows(32).any(|window| window == [4u8; 32]));
        assert!(bytes.windows(32).any(|window| window == certificate_hash));
        assert!(bytes.windows(32).any(|window| window == authenticated_hash));
    }
}
