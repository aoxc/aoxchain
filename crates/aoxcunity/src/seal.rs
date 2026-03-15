use serde::{Deserialize, Serialize};

/// Finalized block seal.
///
/// This type models the minimum cryptographic or quorum-backed evidence
/// required to mark a block as finalized in the fork-choice view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockSeal {
    pub block_hash: [u8; 32],
    pub finalized_round: u64,
    pub attestation_root: [u8; 32],
}
