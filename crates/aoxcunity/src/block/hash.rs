// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use sha2::{Digest, Sha256};

use crate::block::types::{
    AiSection, BlockBody, BlockHeader, BlockSection, CAPABILITY_AI_ATTESTATION,
    CAPABILITY_CONSTITUTIONAL, CAPABILITY_EXECUTION, CAPABILITY_IDENTITY, CAPABILITY_PQ_ROTATION,
    CAPABILITY_SETTLEMENT, CAPABILITY_TIME_SEAL, ConstitutionalSection, ExecutionLaneRecord,
    ExternalNetwork, ExternalProofType, IdentitySection, LaneCommitment, LaneType,
    PostQuantumSection, TimeSealSection,
};

const BLOCK_HEADER_DOMAIN_V1: &[u8] = b"AOXC_BLOCK_HEADER_V1";
const BODY_ROOT_DOMAIN_V1: &[u8] = b"AOXC_BODY_ROOT_V1";
const AUTHORITY_ROOT_DOMAIN_V1: &[u8] = b"AOXC_AUTHORITY_ROOT_V1";
const LANE_ROOT_DOMAIN_V1: &[u8] = b"AOXC_LANE_ROOT_V1";
const PROOF_ROOT_DOMAIN_V1: &[u8] = b"AOXC_PROOF_ROOT_V1";
const FINALITY_ROOT_DOMAIN_V1: &[u8] = b"AOXC_FINALITY_ROOT_V1";
const IDENTITY_ROOT_DOMAIN_V1: &[u8] = b"AOXC_IDENTITY_ROOT_V1";
const AI_ROOT_DOMAIN_V1: &[u8] = b"AOXC_AI_ROOT_V1";
const PQ_ROOT_DOMAIN_V1: &[u8] = b"AOXC_PQ_ROOT_V1";
const EXTERNAL_SETTLEMENT_ROOT_DOMAIN_V1: &[u8] = b"AOXC_EXTERNAL_SETTLEMENT_ROOT_V1";
const POLICY_ROOT_DOMAIN_V1: &[u8] = b"AOXC_POLICY_ROOT_V1";
const TIME_SEAL_ROOT_DOMAIN_V1: &[u8] = b"AOXC_TIME_SEAL_ROOT_V1";
const SECTION_HASH_DOMAIN_V1: &[u8] = b"AOXC_SECTION_HASH_V1";
const SECTION_ORDER_DOMAIN_V1: &[u8] = b"AOXC_SECTION_ORDER_V1";
const LANE_TYPE_DOMAIN_V1: &[u8] = b"AOXC_LANE_TYPE_V1";
const NETWORK_TYPE_DOMAIN_V1: &[u8] = b"AOXC_NETWORK_TYPE_V1";
const PROOF_TYPE_DOMAIN_V1: &[u8] = b"AOXC_PROOF_TYPE_V1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BodyRoots {
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

    let mut finality_hasher = Sha256::new();
    finality_hasher.update(FINALITY_ROOT_DOMAIN_V1);

    let mut identity_hasher = Sha256::new();
    identity_hasher.update(IDENTITY_ROOT_DOMAIN_V1);

    let mut ai_hasher = Sha256::new();
    ai_hasher.update(AI_ROOT_DOMAIN_V1);

    let mut pq_hasher = Sha256::new();
    pq_hasher.update(PQ_ROOT_DOMAIN_V1);

    let mut external_settlement_hasher = Sha256::new();
    external_settlement_hasher.update(EXTERNAL_SETTLEMENT_ROOT_DOMAIN_V1);

    let mut policy_hasher = Sha256::new();
    policy_hasher.update(POLICY_ROOT_DOMAIN_V1);

    let mut time_seal_hasher = Sha256::new();
    time_seal_hasher.update(TIME_SEAL_ROOT_DOMAIN_V1);

    let mut capability_flags = 0u64;

    for section in &body.sections {
        let section_hash = hash_section(section);
        body_hasher.update(section_hash);

        match section {
            BlockSection::Execution(_) => {
                lane_hasher.update(section_hash);
                capability_flags |= CAPABILITY_EXECUTION;
            }
            BlockSection::LaneCommitment(_) => {
                lane_hasher.update(section_hash);
                capability_flags |= CAPABILITY_EXECUTION;
            }
            BlockSection::ExternalProof(_) => {
                proof_hasher.update(section_hash);
                capability_flags |= CAPABILITY_SETTLEMENT;
            }
            BlockSection::Identity(_) => {
                authority_hasher.update(section_hash);
                identity_hasher.update(section_hash);
                capability_flags |= CAPABILITY_IDENTITY;
            }
            BlockSection::PostQuantum(_) => {
                policy_hasher.update(section_hash);
                pq_hasher.update(section_hash);
                capability_flags |= CAPABILITY_PQ_ROTATION;
            }
            BlockSection::Ai(_) => {
                ai_hasher.update(section_hash);
                policy_hasher.update(section_hash);
                capability_flags |= CAPABILITY_AI_ATTESTATION;
            }
            BlockSection::ExternalSettlement(_) => {
                external_settlement_hasher.update(section_hash);
                capability_flags |= CAPABILITY_SETTLEMENT;
            }
            BlockSection::Constitutional(_) => {
                authority_hasher.update(section_hash);
                finality_hasher.update(section_hash);
                policy_hasher.update(section_hash);
                capability_flags |= CAPABILITY_CONSTITUTIONAL;
            }
            BlockSection::TimeSeal(_) => {
                time_seal_hasher.update(section_hash);
                capability_flags |= CAPABILITY_TIME_SEAL;
            }
        }
    }

    BodyRoots {
        body_root: body_hasher.finalize().into(),
        finality_root: finality_hasher.finalize().into(),
        authority_root: authority_hasher.finalize().into(),
        lane_root: lane_hasher.finalize().into(),
        proof_root: proof_hasher.finalize().into(),
        identity_root: identity_hasher.finalize().into(),
        ai_root: ai_hasher.finalize().into(),
        pq_root: pq_hasher.finalize().into(),
        external_settlement_root: external_settlement_hasher.finalize().into(),
        policy_root: policy_hasher.finalize().into(),
        time_seal_root: time_seal_hasher.finalize().into(),
        capability_flags,
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
    hasher.update(header.finality_root);
    hasher.update(header.authority_root);
    hasher.update(header.lane_root);
    hasher.update(header.proof_root);
    hasher.update(header.identity_root);
    hasher.update(header.ai_root);
    hasher.update(header.pq_root);
    hasher.update(header.external_settlement_root);
    hasher.update(header.policy_root);
    hasher.update(header.time_seal_root);
    hasher.update(header.capability_flags.to_le_bytes());
    hasher.update(header.crypto_epoch.to_le_bytes());
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
        BlockSection::Execution(section) => {
            hasher.update([0]);
            hasher.update((section.lanes.len() as u64).to_le_bytes());

            let mut lanes = section.lanes.clone();
            lanes.sort();

            for lane in &lanes {
                hasher.update(hash_execution_lane_record(lane));
            }
        }
        BlockSection::LaneCommitment(section) => {
            hasher.update([1]);
            hasher.update((section.lanes.len() as u64).to_le_bytes());

            let mut lanes = section.lanes.clone();
            lanes.sort();

            for lane in &lanes {
                hasher.update(hash_lane_commitment(lane));
            }
        }
        BlockSection::ExternalProof(section) => {
            hasher.update([2]);
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
        BlockSection::Identity(section) => {
            hasher.update([3]);
            hasher.update(hash_identity_section(section));
        }
        BlockSection::PostQuantum(section) => {
            hasher.update([4]);
            hasher.update(hash_post_quantum_section(section));
        }
        BlockSection::Ai(section) => {
            hasher.update([5]);
            hasher.update(hash_ai_section(section));
        }
        BlockSection::ExternalSettlement(section) => {
            hasher.update([6]);
            hasher.update((section.settlements.len() as u64).to_le_bytes());
            let mut settlements = section.settlements.clone();
            settlements.sort();
            for settlement in settlements {
                hasher.update(hash_external_network(&settlement.source_network));
                hasher.update(settlement.checkpoint_hash);
                hasher.update(settlement.settlement_commitment);
                hasher.update(settlement.admission_proof_commitment);
                hasher.update(settlement.finalized_at.to_le_bytes());
            }
        }
        BlockSection::Constitutional(section) => {
            hasher.update([7]);
            hasher.update(hash_constitutional_section(section));
        }
        BlockSection::TimeSeal(section) => {
            hasher.update([8]);
            hasher.update(hash_time_seal_section(section));
        }
    }

    hasher.finalize().into()
}

fn hash_execution_lane_record(record: &ExecutionLaneRecord) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"AOXC_EXECUTION_LANE_RECORD_V1");
    hasher.update(record.lane_id.to_le_bytes());
    hasher.update(hash_lane_type(&record.lane_type));
    hasher.update(record.tx_count.to_le_bytes());
    hasher.update(record.gas_commitment);
    hasher.update(record.fee_commitment);
    hasher.update(record.input_root);
    hasher.update(record.output_root);
    hasher.update(record.receipt_root);
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

fn hash_identity_section(section: &IdentitySection) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"AOXC_IDENTITY_SECTION_V1");
    hasher.update(section.validator_snapshot_root);
    hasher.update(section.session_keys_root);
    hasher.update(section.revocation_root);
    hasher.update(section.authority_epoch_proof);
    hasher.finalize().into()
}

fn hash_post_quantum_section(section: &PostQuantumSection) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"AOXC_POST_QUANTUM_SECTION_V1");
    hasher.update(section.scheme_registry_root);
    hasher.update(section.signer_set_root);
    hasher.update(section.hybrid_policy_root);
    hasher.update(section.signature_policy_id.to_le_bytes());
    hasher.update([u8::from(section.downgrade_prohibited)]);
    hasher.finalize().into()
}

fn hash_ai_section(section: &AiSection) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"AOXC_AI_SECTION_V1");
    hasher.update(section.request_hash);
    hasher.update(section.response_hash);
    hasher.update(section.policy_hash);
    hasher.update(section.confidence_commitment);
    hasher.update([u8::from(section.human_override)]);
    hasher.update([u8::from(section.fallback_mode)]);
    hasher.update(section.replay_nonce.to_le_bytes());
    hasher.finalize().into()
}

fn hash_constitutional_section(section: &ConstitutionalSection) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"AOXC_CONSTITUTIONAL_SECTION_V1");
    hasher.update(section.legitimacy_certificate_hash);
    hasher.update(section.continuity_certificate_hash);
    hasher.update(section.execution_certificate_hash);
    hasher.update(section.constitutional_seal_hash);
    hasher.finalize().into()
}

fn hash_time_seal_section(section: &TimeSealSection) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"AOXC_TIME_SEAL_SECTION_V1");
    hasher.update(section.valid_from.to_le_bytes());
    hasher.update(section.valid_until.to_le_bytes());
    hasher.update(section.epoch_action_root);
    hasher.update(section.delayed_effect_root);
    hasher.finalize().into()
}
