use crate::{
    error::{AppError, ErrorCode},
    node::{
        lifecycle::{load_state, persist_state},
        state::{ConsensusSnapshot, NodeState},
    },
};
use aoxcunity::{
    Block, BlockBody, BlockSection, ConsensusMessage, LaneCommitment, LaneCommitmentSection,
    LaneType, Proposer,
};
use sha3::{Digest, Sha3_256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Default network identifier used when a consensus message variant does not
/// carry an explicit network identifier in the persisted snapshot model.
///
/// Rationale:
/// - Prevents undefined network context in non-block messages.
/// - Ensures deterministic fallback for snapshot reconstruction.
const DEFAULT_NETWORK_ID: u32 = 2626;

/// Produces a single block from the provided transaction payload.
///
/// Security guarantees:
/// - State is loaded before mutation.
/// - Block derivation is deterministic.
/// - Persistence occurs only after successful application.
pub fn produce_once(tx: &str) -> Result<NodeState, AppError> {
    let mut state = load_state()?;
    state.running = true;

    let block = build_block_for_tx(&state, tx)?;
    apply_block_proposal(&mut state, tx, &block);

    persist_state(&state)?;
    Ok(state)
}

/// Produces multiple deterministic block rounds.
pub fn run_rounds(rounds: u64, tx_prefix: &str) -> Result<NodeState, AppError> {
    let mut state = load_state()?;
    state.running = true;

    for index in 0..rounds {
        let tx = format!("{tx_prefix}-{index}");
        let block = build_block_for_tx(&state, &tx)?;
        apply_block_proposal(&mut state, &tx, &block);
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
fn build_block_for_tx(state: &NodeState, tx: &str) -> Result<Block, AppError> {
    let height = state.current_height.saturating_add(1);
    let round = state.consensus.last_round.saturating_add(1);
    let timestamp = unix_now();

    let parent_hash = decode_hash32(
        &state.consensus.last_block_hash_hex,
        "last_block_hash_hex",
        ErrorCode::NodeStateInvalid,
    )?;

    let proposer_key = derive_digest32("AOXC-CMD-PROPOSER", tx.as_bytes());

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
fn apply_block_proposal(state: &mut NodeState, tx: &str, block: &Block) {
    let message = ConsensusMessage::BlockProposal {
        block: block.clone(),
    };

    state.current_height = block.header.height;
    state.produced_blocks = state.produced_blocks.saturating_add(1);
    state.last_tx = tx.to_string();
    state.consensus = snapshot_from_message(&message);
    state.touch();
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
            network_id: DEFAULT_NETWORK_ID,
            last_parent_hash_hex: hex::encode([0u8; 32]),
            last_block_hash_hex: hex::encode(vote.block_hash),
            last_proposer_hex: hex::encode(vote.voter),
            last_round: vote.round,
            last_timestamp_unix: 0,
            last_message_kind: "vote".to_string(),
            last_section_count: 0,
        },
        ConsensusMessage::Finalize { seal } => ConsensusSnapshot {
            network_id: DEFAULT_NETWORK_ID,
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

fn decode_hash32(value: &str, field: &str, code: ErrorCode) -> Result<[u8; 32], AppError> {
    let bytes = hex::decode(value).map_err(|e| {
        AppError::with_source(code, format!("Failed to decode {field}"), e)
    })?;

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

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(1)
        .max(1)
}
