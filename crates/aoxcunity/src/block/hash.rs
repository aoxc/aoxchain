use sha2::{Digest, Sha256};

use crate::block::types::{
    BlockBody, BlockHeader, BlockSection, ExternalNetwork, ExternalProofType, LaneCommitment,
    LaneType,
};

const BLOCK_HEADER_DOMAIN_V1: &[u8] = b"AOXC_BLOCK_HEADER_V1";
const BODY_ROOT_DOMAIN_V1: &[u8] = b"AOXC_BODY_ROOT_V1";
const AUTHORITY_ROOT_DOMAIN_V1: &[u8] = b"AOXC_AUTHORITY_ROOT_V1";
const LANE_ROOT_DOMAIN_V1: &[u8] = b"AOXC_LANE_ROOT_V1";
const PROOF_ROOT_DOMAIN_V1: &[u8] = b"AOXC_PROOF_ROOT_V1";
const SECTION_HASH_DOMAIN_V1: &[u8] = b"AOXC_SECTION_HASH_V1";
const SECTION_ORDER_DOMAIN_V1: &[u8] = b"AOXC_SECTION_ORDER_V1";
const LANE_TYPE_DOMAIN_V1: &[u8] = b"AOXC_LANE_TYPE_V1";
const NETWORK_TYPE_DOMAIN_V1: &[u8] = b"AOXC_NETWORK_TYPE_V1";
const PROOF_TYPE_DOMAIN_V1: &[u8] = b"AOXC_PROOF_TYPE_V1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BodyRoots {
    pub body_root: [u8; 32],
    pub authority_root: [u8; 32],
    pub lane_root: [u8; 32],
    pub proof_root: [u8; 32],
}

/// Computes canonical body roots.
///
/// The body is hashed after canonical section ordering and nested collection
/// ordering have been established by the builder.
pub fn compute_body_roots(body: &BlockBody) -> BodyRoots {
    let mut body_hasher = Sha256::new();
    body_hasher.update(BODY_ROOT_DOMAIN_V1);

    let mut authority_hasher = Sha256::new();
    authority_hasher.update(AUTHORITY_ROOT_DOMAIN_V1);

    let mut lane_hasher = Sha256::new();
    lane_hasher.update(LANE_ROOT_DOMAIN_V1);

    let mut proof_hasher = Sha256::new();
    proof_hasher.update(PROOF_ROOT_DOMAIN_V1);

    for section in &body.sections {
        let section_hash = hash_section(section);
        body_hasher.update(section_hash);

        match section {
            BlockSection::LaneCommitment(_) => lane_hasher.update(section_hash),
            BlockSection::ExternalProof(_) => proof_hasher.update(section_hash),
        }
    }

    BodyRoots {
        body_root: body_hasher.finalize().into(),
        authority_root: authority_hasher.finalize().into(),
        lane_root: lane_hasher.finalize().into(),
        proof_root: proof_hasher.finalize().into(),
    }
}

/// Computes the canonical block hash from the header.
pub fn compute_block_hash(header: &BlockHeader) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(BLOCK_HEADER_DOMAIN_V1);
    hasher.update(header.version.to_le_bytes());
    hasher.update(header.network_id.to_le_bytes());
    hasher.update(header.parent_hash);
    hasher.update(header.height.to_le_bytes());
    hasher.update(header.era.to_le_bytes());
    hasher.update(header.round.to_le_bytes());
    hasher.update(header.timestamp.to_le_bytes());
    hasher.update(header.proposer);
    hasher.update(header.body_root);
    hasher.update(header.authority_root);
    hasher.update(header.lane_root);
    hasher.update(header.proof_root);
    hasher.finalize().into()
}

/// Returns a stable section-order key.
///
/// This ensures that identical logical bodies produce identical body roots
/// even when callers provide sections in a different order.
pub fn canonical_section_sort_key(section: &BlockSection) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(SECTION_ORDER_DOMAIN_V1);
    hasher.update([section.discriminant()]);
    hasher.update(hash_section(section));
    hasher.finalize().into()
}

/// Computes the canonical section hash.
pub fn hash_section(section: &BlockSection) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(SECTION_HASH_DOMAIN_V1);

    match section {
        BlockSection::LaneCommitment(section) => {
            hasher.update([0]);
            hasher.update((section.lanes.len() as u64).to_le_bytes());

            let mut lanes = section.lanes.clone();
            lanes.sort();

            for lane in &lanes {
                hasher.update(hash_lane_commitment(lane));
            }
        }
        BlockSection::ExternalProof(section) => {
            hasher.update([1]);
            hasher.update((section.proofs.len() as u64).to_le_bytes());

            let mut proofs = section.proofs.clone();
            proofs.sort();

            for proof in &proofs {
                hasher.update(hash_external_network(&proof.source_network));
                hasher.update(hash_external_proof_type(&proof.proof_type));
                hasher.update(proof.subject_hash);
                hasher.update(proof.proof_commitment);
                hasher.update(proof.finalized_at.to_le_bytes());
            }
        }
    }

    hasher.finalize().into()
}

fn hash_lane_commitment(lane: &LaneCommitment) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"AOXC_LANE_COMMITMENT_V1");
    hasher.update(lane.lane_id.to_le_bytes());
    hasher.update(hash_lane_type(&lane.lane_type));
    hasher.update(lane.tx_count.to_le_bytes());
    hasher.update(lane.input_root);
    hasher.update(lane.output_root);
    hasher.update(lane.receipt_root);
    hasher.update(lane.state_commitment);
    hasher.update(lane.proof_commitment);
    hasher.finalize().into()
}

fn hash_lane_type(value: &LaneType) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(LANE_TYPE_DOMAIN_V1);

    match value {
        LaneType::Native => hasher.update(b"NATIVE"),
        LaneType::Evm => hasher.update(b"EVM"),
        LaneType::SuiMove => hasher.update(b"SUI_MOVE"),
        LaneType::CardanoUtxo => hasher.update(b"CARDANO_UTXO"),
        LaneType::ZkEvm => hasher.update(b"ZK_EVM"),
        LaneType::Wasm => hasher.update(b"WASM"),
        LaneType::External => hasher.update(b"EXTERNAL"),
    }

    hasher.finalize().into()
}

fn hash_external_network(value: &ExternalNetwork) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(NETWORK_TYPE_DOMAIN_V1);

    match value {
        ExternalNetwork::Ethereum => hasher.update(b"ETHEREUM"),
        ExternalNetwork::XLayer => hasher.update(b"XLAYER"),
        ExternalNetwork::Sui => hasher.update(b"SUI"),
        ExternalNetwork::Cardano => hasher.update(b"CARDANO"),
        ExternalNetwork::Bitcoin => hasher.update(b"BITCOIN"),
        ExternalNetwork::Other(id) => {
            hasher.update(b"OTHER");
            hasher.update(id.to_le_bytes());
        }
    }

    hasher.finalize().into()
}

fn hash_external_proof_type(value: &ExternalProofType) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(PROOF_TYPE_DOMAIN_V1);

    match value {
        ExternalProofType::Finality => hasher.update(b"FINALITY"),
        ExternalProofType::Inclusion => hasher.update(b"INCLUSION"),
        ExternalProofType::Checkpoint => hasher.update(b"CHECKPOINT"),
        ExternalProofType::StateCommitment => hasher.update(b"STATE_COMMITMENT"),
        ExternalProofType::Attestation => hasher.update(b"ATTESTATION"),
    }

    hasher.finalize().into()
}
