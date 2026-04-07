// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    data_home::{ensure_layout, resolve_home},
    error::{AppError, ErrorCode},
    keys::material::KeyMaterial,
    node::{
        lifecycle::{load_state, persist_state},
        state::{ConsensusSnapshot, KeyMaterialSnapshot, NodeState},
    },
};
use aoxcdata::{BlockEnvelope, HybridDataStore, IndexBackend, canonical_block_envelope_hash_hex};
use aoxcunity::{
    Block, BlockBody, BlockSection, ConsensusMessage, LaneCommitment, LaneCommitmentSection,
    LaneType, Proposer,
};
use sha3::{Digest, Sha3_256};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const BLOCK_PROPOSAL_MESSAGE_KIND: &str = "block_proposal";
const MINIMUM_RUNTIME_TIMESTAMP_UNIX: u64 = 1;

#[derive(Debug, Clone)]
pub struct RoundTelemetry {
    pub round_index: u64,
    pub tx_id: String,
    pub height: u64,
    pub produced_blocks: u64,
    pub consensus_round: u64,
    pub section_count: usize,
    pub block_hash_hex: String,
    pub parent_hash_hex: String,
    pub timestamp_unix: u64,
}

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
    apply_block_proposal(&mut state, tx, &block, &key_material)?;
    persist_block_envelope(&block)?;

    persist_state(&state)?;
    Ok(state)
}

/// Produces multiple deterministic block rounds.
pub fn run_rounds(rounds: u64, tx_prefix: &str) -> Result<NodeState, AppError> {
    run_rounds_with_observer(rounds, tx_prefix, |_| {})
}

/// Produces deterministic block rounds while emitting round telemetry.
///
/// Validation policy:
/// - `rounds` must be greater than zero.
/// - The observer is invoked only after a round is successfully applied.
pub fn run_rounds_with_observer<F>(
    rounds: u64,
    tx_prefix: &str,
    mut observer: F,
) -> Result<NodeState, AppError>
where
    F: FnMut(&RoundTelemetry),
{
    if rounds == 0 {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Round count must be greater than zero",
        ));
    }

    let mut state = load_state()?;
    state.running = true;
    let key_material = crate::keys::loader::load_operator_key()?;

    for index in 0..rounds {
        let tx = format!("{tx_prefix}-{index}");
        let block = build_block_for_tx(&state, &tx, &key_material)?;
        apply_block_proposal(&mut state, &tx, &block, &key_material)?;
        persist_block_envelope(&block)?;

        let telemetry = RoundTelemetry {
            round_index: index + 1,
            tx_id: tx,
            height: state.current_height,
            produced_blocks: state.produced_blocks,
            consensus_round: state.consensus.last_round,
            section_count: state.consensus.last_section_count,
            block_hash_hex: state.consensus.last_block_hash_hex.clone(),
            parent_hash_hex: state.consensus.last_parent_hash_hex.clone(),
            timestamp_unix: state.consensus.last_timestamp_unix,
        };
        observer(&telemetry);
    }

    persist_state(&state)?;
    Ok(state)
}

fn persist_block_envelope(block: &Block) -> Result<(), AppError> {
    let db_root = runtime_db_root()?;
    let store = HybridDataStore::new(&db_root, IndexBackend::Redb).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to open block index store at {}", db_root.display()),
            error,
        )
    })?;

    let payload = serde_json::to_vec(block).map_err(|error| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to serialize block payload for historical storage",
            error,
        )
    })?;

    let parent_hash_hex = hex::encode(block.header.parent_hash);
    let block_hash_hex = block_envelope_hash_hex(block)?;

    let envelope = BlockEnvelope {
        height: block.header.height,
        block_hash_hex,
        parent_hash_hex,
        payload,
    };

    store.put_block(&envelope).map_err(|error| {
        AppError::with_source(
            ErrorCode::LedgerInvalid,
            format!(
                "Failed to persist historical block at height {}",
                envelope.height
            ),
            error,
        )
    })?;

    Ok(())
}

fn runtime_db_root() -> Result<PathBuf, AppError> {
    let home = resolve_home()?;
    ensure_layout(&home)?;
    Ok(home.join("runtime").join("db"))
}

/// Constructs a deterministic block proposal.
///
/// Audit considerations:
/// - Saturating arithmetic prevents overflow.
/// - Parent hash is strictly validated.
/// - Proposer material is bound to the active consensus key.
/// - Domain separation is enforced for every derived digest.
pub(super) fn build_block_for_tx(
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

/// Applies a validated block proposal to state.
///
/// Invariant policy:
/// - Block height must advance exactly by one.
/// - Block round must advance exactly by one.
/// - Parent hash must equal the previously recorded canonical block hash.
/// - Proposer identity must match the active operator consensus key.
/// - Runtime consensus snapshot is derived from the validated proposal.
fn apply_block_proposal(
    state: &mut NodeState,
    tx: &str,
    block: &Block,
    key_material: &KeyMaterial,
) -> Result<(), AppError> {
    apply_block_proposal_with_message(state, tx, block, key_material)
}

pub(super) fn apply_block_proposal_with_message(
    state: &mut NodeState,
    tx: &str,
    block: &Block,
    key_material: &KeyMaterial,
) -> Result<(), AppError> {
    validate_block_proposal_against_state(state, block, key_material)?;

    let message = ConsensusMessage::BlockProposal {
        block: block.clone(),
    };

    state.current_height = block.header.height;
    state.produced_blocks = state.produced_blocks.saturating_add(1);
    state.last_tx = tx.to_string();
    state.key_material = snapshot_from_key_material(key_material)?;
    state.consensus = snapshot_from_message_kind(&message, BLOCK_PROPOSAL_MESSAGE_KIND);
    state.consensus.last_block_hash_hex = block_envelope_hash_hex(block)?;
    state.touch();

    Ok(())
}

fn validate_block_proposal_against_state(
    state: &NodeState,
    block: &Block,
    key_material: &KeyMaterial,
) -> Result<(), AppError> {
    let expected_height = state.current_height.saturating_add(1);
    if block.header.height != expected_height {
        return Err(AppError::new(
            ErrorCode::NodeStateInvalid,
            format!(
                "Block height invariant violated: expected {}, found {}",
                expected_height, block.header.height
            ),
        ));
    }

    let expected_round = state.consensus.last_round.saturating_add(1);
    if block.header.round != expected_round {
        return Err(AppError::new(
            ErrorCode::NodeStateInvalid,
            format!(
                "Block round invariant violated: expected {}, found {}",
                expected_round, block.header.round
            ),
        ));
    }

    let expected_parent_hash = decode_hash32(
        &state.consensus.last_block_hash_hex,
        "last_block_hash_hex",
        ErrorCode::NodeStateInvalid,
    )?;
    if block.header.parent_hash != expected_parent_hash {
        return Err(AppError::new(
            ErrorCode::NodeStateInvalid,
            "Block parent hash does not match the canonical prior block hash",
        ));
    }

    let expected_proposer = proposer_key_from_material(key_material)?;
    if block.header.proposer != expected_proposer {
        return Err(AppError::new(
            ErrorCode::NodeStateInvalid,
            "Block proposer does not match active consensus key material",
        ));
    }

    if block.header.timestamp < MINIMUM_RUNTIME_TIMESTAMP_UNIX {
        return Err(AppError::new(
            ErrorCode::NodeStateInvalid,
            "Block timestamp must be a positive unix timestamp",
        ));
    }

    Ok(())
}

fn snapshot_from_key_material(key_material: &KeyMaterial) -> Result<KeyMaterialSnapshot, AppError> {
    let summary = key_material.summary()?;

    Ok(KeyMaterialSnapshot {
        bundle_fingerprint: summary.bundle_fingerprint,
        operational_state: summary.operational_state,
        consensus_public_key_hex: summary.consensus_public_key,
        transport_public_key_hex: summary.transport_public_key,
    })
}

/// Snapshot builder for test-only direct message assertions.
#[cfg(test)]
pub(super) fn snapshot_from_message(message: &ConsensusMessage) -> ConsensusSnapshot {
    snapshot_from_message_kind(message, BLOCK_PROPOSAL_MESSAGE_KIND)
}

fn snapshot_from_message_kind(
    message: &ConsensusMessage,
    block_proposal_kind: &str,
) -> ConsensusSnapshot {
    match message {
        ConsensusMessage::BlockProposal { block } => ConsensusSnapshot {
            network_id: block.header.network_id,
            last_parent_hash_hex: hex::encode(block.header.parent_hash),
            last_block_hash_hex: hex::encode(block.hash),
            last_proposer_hex: hex::encode(block.header.proposer),
            last_round: block.header.round,
            last_timestamp_unix: block.header.timestamp,
            last_message_kind: block_proposal_kind.to_string(),
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

pub(super) fn block_envelope_hash_hex(block: &Block) -> Result<String, AppError> {
    let payload = serde_json::to_vec(block).map_err(|error| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to serialize block payload for historical hash derivation",
            error,
        )
    })?;
    canonical_block_envelope_hash_hex(
        block.header.height,
        &hex::encode(block.header.parent_hash),
        &payload,
    )
    .map_err(|error| {
        AppError::with_source(
            ErrorCode::LedgerInvalid,
            format!(
                "Failed to derive historical block hash at height {}",
                block.header.height
            ),
            error,
        )
    })
}

pub(super) fn proposer_key_from_material(key_material: &KeyMaterial) -> Result<[u8; 32], AppError> {
    let summary = key_material.summary()?;
    decode_hash32(
        &summary.consensus_public_key,
        "consensus_public_key",
        ErrorCode::KeyMaterialInvalid,
    )
}

pub(super) fn decode_hash32(
    value: &str,
    field: &str,
    code: ErrorCode,
) -> Result<[u8; 32], AppError> {
    let bytes = hex::decode(value)
        .map_err(|error| AppError::with_source(code, format!("Failed to decode {field}"), error))?;

    if bytes.len() != 32 {
        return Err(AppError::new(code, format!("{field} must be 32 bytes")));
    }

    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn derive_digest32(domain: &str, payload: &[u8]) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(domain.as_bytes());
    hasher.update([0]);
    hasher.update(payload);
    hasher.finalize().into()
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(MINIMUM_RUNTIME_TIMESTAMP_UNIX)
        .max(MINIMUM_RUNTIME_TIMESTAMP_UNIX)
}
