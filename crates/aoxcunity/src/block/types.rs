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
    pub finality_root: [u8; 32],
    pub authority_root: [u8; 32],
    pub lane_root: [u8; 32],
    pub proof_root: [u8; 32],
    pub identity_root: [u8; 32],
    pub ai_root: [u8; 32],
    pub pq_root: [u8; 32],
    pub external_settlement_root: [u8; 32],
    pub policy_root: [u8; 32],
    pub time_seal_root: [u8; 32],
    pub capability_flags: u64,
    pub crypto_epoch: u64,
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
/// The current protocol permits at most one section of each type per block.
/// The builder enforces that boundary before root calculation so that
/// canonical block construction does not depend on caller-provided section
/// duplication or ordering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockSection {
    Execution(ExecutionSection),
    LaneCommitment(LaneCommitmentSection),
    ExternalProof(ExternalProofSection),
    Identity(IdentitySection),
    PostQuantum(PostQuantumSection),
    Ai(AiSection),
    ExternalSettlement(ExternalSettlementSection),
    Constitutional(ConstitutionalSection),
    TimeSeal(TimeSealSection),
}

impl BlockSection {
    pub fn discriminant(&self) -> u8 {
        match self {
            Self::Execution(_) => 0,
            Self::LaneCommitment(_) => 1,
            Self::ExternalProof(_) => 2,
            Self::Identity(_) => 3,
            Self::PostQuantum(_) => 4,
            Self::Ai(_) => 5,
            Self::ExternalSettlement(_) => 6,
            Self::Constitutional(_) => 7,
            Self::TimeSeal(_) => 8,
        }
    }
}

/// Execution section.
///
/// Captures lane-oriented execution summaries as deterministic commitments.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ExecutionSection {
    pub lanes: Vec<ExecutionLaneRecord>,
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

/// Identity section.
///
/// Carries authority and identity commitments without embedding key material.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct IdentitySection {
    pub validator_snapshot_root: [u8; 32],
    pub session_keys_root: [u8; 32],
    pub revocation_root: [u8; 32],
    pub authority_epoch_proof: [u8; 32],
}

/// Post-quantum section.
///
/// Carries signature-policy commitments for crypto migration epochs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PostQuantumSection {
    pub scheme_registry_root: [u8; 32],
    pub signer_set_root: [u8; 32],
    pub hybrid_policy_root: [u8; 32],
    pub signature_policy_id: u32,
    pub downgrade_prohibited: bool,
}

/// AI attestation section.
///
/// Stores attestable decision commitments, not full prompts/responses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AiSection {
    pub request_hash: [u8; 32],
    pub response_hash: [u8; 32],
    pub policy_hash: [u8; 32],
    pub confidence_commitment: [u8; 32],
    pub human_override: bool,
    pub fallback_mode: bool,
    pub replay_nonce: u64,
}

/// External settlement section.
///
/// Captures settled cross-network outcomes and admission proofs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ExternalSettlementSection {
    pub settlements: Vec<ExternalSettlementRecord>,
}

/// Constitutional section.
///
/// Carries constitutional certificates as fixed commitments.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ConstitutionalSection {
    pub legitimacy_certificate_hash: [u8; 32],
    pub continuity_certificate_hash: [u8; 32],
    pub execution_certificate_hash: [u8; 32],
    pub constitutional_seal_hash: [u8; 32],
}

/// Time seal section.
///
/// Defines block validity envelope and delayed-effect commitments.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TimeSealSection {
    pub valid_from: u64,
    pub valid_until: u64,
    pub epoch_action_root: [u8; 32],
    pub delayed_effect_root: [u8; 32],
}

/// Execution lane record for the execution section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ExecutionLaneRecord {
    pub lane_id: u32,
    pub lane_type: LaneType,
    pub tx_count: u32,
    pub gas_commitment: [u8; 32],
    pub fee_commitment: [u8; 32],
    pub input_root: [u8; 32],
    pub output_root: [u8; 32],
    pub receipt_root: [u8; 32],
}

/// Lane-level commitment record.
///
/// This structure is intentionally generic. It permits AOXC to model native
/// and external execution lanes while keeping the consensus core agnostic
/// to lane-specific state internals.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ExternalProofRecord {
    pub source_network: ExternalNetwork,
    pub proof_type: ExternalProofType,
    pub subject_hash: [u8; 32],
    pub proof_commitment: [u8; 32],
    pub finalized_at: u64,
}

/// External settlement record admitted into a block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ExternalSettlementRecord {
    pub source_network: ExternalNetwork,
    pub checkpoint_hash: [u8; 32],
    pub settlement_commitment: [u8; 32],
    pub admission_proof_commitment: [u8; 32],
    pub finalized_at: u64,
}

/// Supported external network families.
///
/// This list can grow without changing the core block model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
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

    #[error("block contains duplicate section type")]
    DuplicateSectionType,

    #[error("section count exceeds supported u64 range")]
    SectionCountOverflow,
}
