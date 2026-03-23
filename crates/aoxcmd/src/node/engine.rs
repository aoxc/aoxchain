use crate::{
    error::{AppError, ErrorCode},
    keys::material::KeyMaterial,
    node::{
        lifecycle::{load_state, persist_state},
        state::{ConsensusSnapshot, KeyMaterialSnapshot, NodeState},
    },
};
use aoxcore::{
    block::{AssemblyLane, CanonicalBlockAssemblyPlan},
    receipts::Receipt,
    transaction::Transaction,
};
use aoxcunity::{
    Block, BlockBody, BlockSection, ConsensusMessage, LaneCommitment, LaneCommitmentSection,
    LaneType, Proposer,
};
use ed25519_dalek::{Signer, SigningKey};
use sha3::{Digest, Sha3_256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Produces a single block from the provided transaction payload.
///
/// Security guarantees:
/// - State is loaded before mutation.
/// - Block derivation is deterministic.
/// - Persistence occurs only after successful application.
pub fn produce_once(tx: &str) -> Result<NodeState, AppError> {
    let mut state = load_state()?;
    state.running = true;

    let key_material = crate::keys::loader::load_operator_key()?;
    let block = build_block_for_tx(&state, tx, &key_material)?;
    apply_block_proposal(&mut state, tx, &block, &key_material);

    persist_state(&state)?;
    Ok(state)
}

/// Produces multiple deterministic block rounds.
pub fn run_rounds(rounds: u64, tx_prefix: &str) -> Result<NodeState, AppError> {
    let mut state = load_state()?;
    state.running = true;
    let key_material = crate::keys::loader::load_operator_key()?;

    for index in 0..rounds {
        let tx = format!("{tx_prefix}-{index}");
        let block = build_block_for_tx(&state, &tx, &key_material)?;
        apply_block_proposal(&mut state, &tx, &block, &key_material);
    }

    persist_state(&state)?;
    Ok(state)
}

/// Constructs a deterministic block proposal.
///
/// Audit considerations:
/// - Saturating arithmetic prevents overflow.
/// - Parent hash strictly validated.
/// - Domain separation enforced.
fn build_block_for_tx(
    state: &NodeState,
    tx: &str,
    key_material: &KeyMaterial,
) -> Result<Block, AppError> {
    let height = state.current_height.saturating_add(1);
    let round = state.consensus.last_round.saturating_add(1);
    let timestamp = unix_now();

    let parent_hash = decode_hash32(
        &state.consensus.last_block_hash_hex,
        "last_block_hash_hex",
        ErrorCode::NodeStateInvalid,
    )?;

    let proposer_key = proposer_key_from_material(key_material)?;

    let lane_commitment = LaneCommitment {
        lane_id: 1,
        lane_type: LaneType::Native,
        tx_count: 1,
        input_root: derive_digest32("AOXC-CMD-INPUT", tx.as_bytes()),
        output_root: derive_digest32("AOXC-CMD-OUTPUT", tx.as_bytes()),
        receipt_root: derive_digest32("AOXC-CMD-RECEIPT", tx.as_bytes()),
        state_commitment: derive_digest32("AOXC-CMD-STATE", format!("{height}:{tx}").as_bytes()),
        proof_commitment: derive_digest32("AOXC-CMD-PROOF", format!("{round}:{tx}").as_bytes()),
    };

    let body = BlockBody {
        sections: vec![BlockSection::LaneCommitment(LaneCommitmentSection {
            lanes: vec![lane_commitment],
        })],
    };

    let proposer = Proposer::new(state.consensus.network_id.max(1), proposer_key);

    proposer
        .propose(parent_hash, height, 0, round, timestamp, body)
        .map_err(|error| {
            AppError::with_source(
                ErrorCode::NodeStateInvalid,
                format!("Failed to construct block at height {height}"),
                error,
            )
        })
}

/// Applies block to state safely.
fn apply_block_proposal(
    state: &mut NodeState,
    tx: &str,
    block: &Block,
    key_material: &KeyMaterial,
) {
    let message = ConsensusMessage::BlockProposal {
        block: block.clone(),
    };

    state.current_height = block.header.height;
    state.produced_blocks = state.produced_blocks.saturating_add(1);
    state.last_tx = tx.to_string();
    state.key_material = snapshot_from_key_material(key_material);
    state.consensus = snapshot_from_message(&message);
    state.touch();
}

fn snapshot_from_key_material(key_material: &KeyMaterial) -> KeyMaterialSnapshot {
    let summary = key_material
        .summary()
        .expect("key material summary must remain derivable after successful validation");

    KeyMaterialSnapshot {
        bundle_fingerprint: summary.bundle_fingerprint,
        operational_state: summary.operational_state,
        consensus_public_key_hex: summary.consensus_public_key,
        transport_public_key_hex: summary.transport_public_key,
    }
}

/// Snapshot builder.
fn snapshot_from_message(message: &ConsensusMessage) -> ConsensusSnapshot {
    match message {
        ConsensusMessage::BlockProposal { block } => ConsensusSnapshot {
            network_id: block.header.network_id,
            last_parent_hash_hex: hex::encode(block.header.parent_hash),
            last_block_hash_hex: hex::encode(block.hash),
            last_proposer_hex: hex::encode(block.header.proposer),
            last_round: block.header.round,
            last_timestamp_unix: block.header.timestamp,
            last_message_kind: "block_proposal".to_string(),
            last_section_count: block.body.sections.len(),
        },
        ConsensusMessage::Vote(vote) => ConsensusSnapshot {
            network_id: vote.context.network_id,
            last_parent_hash_hex: hex::encode([0u8; 32]),
            last_block_hash_hex: hex::encode(vote.vote.block_hash),
            last_proposer_hex: hex::encode(vote.vote.voter),
            last_round: vote.vote.round,
            last_timestamp_unix: 0,
            last_message_kind: "vote".to_string(),
            last_section_count: 0,
        },
        ConsensusMessage::Finalize { seal, certificate } => ConsensusSnapshot {
            network_id: certificate.network_id,
            last_block_hash_hex: hex::encode(seal.block_hash),
            last_parent_hash_hex: hex::encode([0u8; 32]),
            last_proposer_hex: hex::encode([0u8; 32]),
            last_round: seal.finalized_round,
            last_timestamp_unix: 0,
            last_message_kind: "finalize".to_string(),
            last_section_count: 0,
        },
    }
}

fn proposer_key_from_material(key_material: &KeyMaterial) -> Result<[u8; 32], AppError> {
    let summary = key_material.summary()?;
    decode_hash32(
        &summary.consensus_public_key,
        "consensus_public_key",
        ErrorCode::KeyMaterialInvalid,
    )
}

fn decode_hash32(value: &str, field: &str, code: ErrorCode) -> Result<[u8; 32], AppError> {
    let bytes = hex::decode(value)
        .map_err(|e| AppError::with_source(code, format!("Failed to decode {field}"), e))?;

    if bytes.len() != 32 {
        return Err(AppError::new(code, format!("{field} must be 32 bytes")));
    }

    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn derive_digest32(domain: &str, payload: &[u8]) -> [u8; 32] {
    let mut h = Sha3_256::new();
    h.update(domain.as_bytes());
    h.update(payload);
    h.finalize().into()
}

#[allow(dead_code)]
fn lane_commitment_from_assembly(
    assembly_plan: &CanonicalBlockAssemblyPlan,
    tx: &str,
    round: u64,
) -> Result<LaneCommitment, AppError> {
    let lane = assembly_plan.lanes.first().ok_or_else(|| {
        AppError::new(
            ErrorCode::NodeStateInvalid,
            "Canonical block assembly plan produced no execution lanes",
        )
    })?;

    Ok(LaneCommitment {
        lane_id: lane.lane_id,
        lane_type: unity_lane_type(lane.lane),
        tx_count: lane.task_count,
        input_root: lane.task_root,
        output_root: derive_digest32("AOXC-CMD-ASSEMBLY-OUTPUT", &assembly_plan.execution_root),
        receipt_root: assembly_plan.receipts_root,
        state_commitment: assembly_plan.execution_root,
        proof_commitment: derive_digest32(
            "AOXC-CMD-ASSEMBLY-PROOF",
            format!("{round}:{}:{tx}", assembly_plan.height).as_bytes(),
        ),
    })
}

#[allow(dead_code)]
fn unity_lane_type(lane: AssemblyLane) -> LaneType {
    match lane {
        AssemblyLane::Native => LaneType::Native,
        AssemblyLane::EthereumSettlement | AssemblyLane::BaseSettlement => LaneType::Evm,
        AssemblyLane::SolanaReward => LaneType::External,
    }
}

#[allow(dead_code)]
fn transaction_from_payload(tx: &str) -> Result<Transaction, AppError> {
    let seed = derive_digest32("AOXC-CMD-TX-SIGNER", tx.as_bytes());
    let signing_key = SigningKey::from_bytes(&seed);
    let sender = signing_key.verifying_key().to_bytes();

    let unsigned = Transaction {
        sender,
        nonce: 0,
        capability: aoxcore::block::Capability::UserSigned,
        target: aoxcore::block::TargetOutpost::AovmNative,
        payload: tx.as_bytes().to_vec(),
        signature: [0u8; 64],
    };

    let message = unsigned.signing_message();
    let signature = signing_key.sign(&message).to_bytes();

    Ok(Transaction {
        signature,
        ..unsigned
    })
}

#[allow(dead_code)]
fn receipt_for_transaction(transaction: &Transaction, tx: &str) -> Result<Receipt, AppError> {
    let tx_id = transaction.tx_id();
    let mut receipt = Receipt::success(tx_id, tx.len() as u64);
    receipt.push_event(aoxcore::receipts::Event {
        event_type: 1,
        data: tx.as_bytes().to_vec(),
    });
    Ok(receipt)
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(1)
        .max(1)
}

#[cfg(test)]
mod tests {
    use super::proposer_key_from_material;
    use crate::keys::material::KeyMaterial;
    use aoxcore::identity::key_bundle::NodeKeyRole;

    #[test]
    fn proposer_key_uses_consensus_public_key_from_bundle() {
        let material = KeyMaterial::generate("validator-01", "validator", "Test#2026!")
            .expect("key material generation should succeed");
        let proposer = proposer_key_from_material(&material)
            .expect("active key material should produce proposer key");
        let expected = material
            .bundle
            .keys
            .iter()
            .find(|record| matches!(record.role, NodeKeyRole::Consensus))
            .and_then(|record| hex::decode(&record.public_key).ok())
            .map(|bytes| {
                let mut out = [0u8; 32];
                out.copy_from_slice(&bytes[..32]);
                out
            })
            .expect("consensus key must decode");

        assert_eq!(proposer, expected);
    }
}
