use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Canonical protocol version for the AOXC coordination block format.
///
/// This value is committed into the block header hash in order to preserve
/// protocol-domain separation across future block format revisions.
pub const BLOCK_VERSION_V1: u32 = 1;

/// Canonical AOXC block.
///
/// Security and design notes:
/// - `hash` is derived exclusively from the header.
/// - The body may expand over time without destabilizing the hash model,
///   provided section hashing and body-root construction remain canonical.
/// - The block model is coordination-oriented rather than execution-only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub hash: [u8; 32],
    pub header: BlockHeader,
    pub body: BlockBody,
}

/// Canonical block header.
///
/// All fields in this structure participate directly in block hashing.
/// The header is intentionally compact and commitment-oriented.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHeader {
    pub version: u32,
    pub network_id: u32,
    pub parent_hash: [u8; 32],
    pub height: u64,
    pub era: u64,
    pub round: u64,
    pub timestamp: u64,
    pub proposer: [u8; 32],
    pub body_root: [u8; 32],
    pub authority_root: [u8; 32],
    pub lane_root: [u8; 32],
    pub proof_root: [u8; 32],
}

/// Canonical block body.
///
/// The body is section-based in order to support protocol growth without
/// forcing every future capability into a single monolithic payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BlockBody {
    pub sections: Vec<BlockSection>,
}

/// Typed block section.
///
/// The protocol permits multiple sections in a single block. Section order
/// must be canonicalized by the builder before hashing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockSection {
    LaneCommitment(LaneCommitmentSection),
    ExternalProof(ExternalProofSection),
}

/// Lane commitment section.
///
/// This section summarizes execution-oriented results or imported lane-level
/// commitments without forcing the consensus layer to understand every
/// execution engine in detail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LaneCommitmentSection {
    pub lanes: Vec<LaneCommitment>,
}

/// External proof section.
///
/// This section records verified or admitted commitments tied to external
/// networks such as Sui, Cardano, XLayer, Ethereum, or future networks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ExternalProofSection {
    pub proofs: Vec<ExternalProofRecord>,
}

/// Lane-level commitment record.
///
/// This structure is intentionally generic. It permits AOXC to model native
/// and external execution lanes while keeping the consensus core agnostic
/// to lane-specific state internals.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaneCommitment {
    pub lane_id: u32,
    pub lane_type: LaneType,
    pub tx_count: u32,
    pub input_root: [u8; 32],
    pub output_root: [u8; 32],
    pub receipt_root: [u8; 32],
    pub state_commitment: [u8; 32],
    pub proof_commitment: [u8; 32],
}

/// Supported lane families.
///
/// This enum should remain compact and stable. Chain-specific details belong
/// in higher-level adapters, not in the consensus core.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LaneType {
    Native,
    Evm,
    SuiMove,
    CardanoUtxo,
    ZkEvm,
    Wasm,
    External,
}

/// External proof record admitted into a block.
///
/// The record stores a proof commitment rather than the full proof blob.
/// Large proof material should live in a separate content-addressed store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalProofRecord {
    pub source_network: ExternalNetwork,
    pub proof_type: ExternalProofType,
    pub subject_hash: [u8; 32],
    pub proof_commitment: [u8; 32],
    pub finalized_at: u64,
}

/// Supported external network families.
///
/// This list can grow without changing the core block model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalNetwork {
    Ethereum,
    XLayer,
    Sui,
    Cardano,
    Bitcoin,
    Other(u32),
}

/// Supported proof categories.
///
/// AOXC does not require every network to expose the same proof model.
/// The adapter layer remains responsible for network-specific verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalProofType {
    Finality,
    Inclusion,
    Checkpoint,
    StateCommitment,
    Attestation,
}

/// Block construction failures.
///
/// This error type is intentionally strict and panic-free.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BlockBuildError {
    #[error("block timestamp must be greater than zero")]
    ZeroTimestamp,

    #[error("block proposer must not be zero")]
    ZeroProposer,

    #[error("section count exceeds supported u64 range")]
    SectionCountOverflow,
}
