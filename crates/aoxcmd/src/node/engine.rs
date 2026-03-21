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
const DEFAULT_NETWORK_ID: u32 = 2626;

/// Produces a single block from the provided transaction payload, updates the
/// local node state, persists the result, and returns the refreshed snapshot.
pub fn produce_once(tx: &str) -> Result<NodeState, AppError> {
    let mut state = load_state()?;
    state.running = true;

    let block = build_block_for_tx(&state, tx)?;
    apply_block_proposal(&mut state, tx, &block);

    persist_state(&state)?;
    Ok(state)
}

/// Produces a deterministic sequence of block proposals using the supplied
/// prefix and persists the final node state after all rounds complete.
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

/// Constructs a synthetic block proposal using the current node state as the
/// parent reference and the provided transaction payload as deterministic input
/// material for lane commitments.
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
                format!("Failed to build consensus block at height {height}"),
                error,
            )
        })
}

/// Applies the proposed block to the mutable node state and refreshes the
/// persisted consensus snapshot metadata.
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

/// Converts a consensus message into the compact snapshot representation stored
/// by the node state subsystem.
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
            last_parent_hash_hex: hex::encode([0u8; 32]),
            last_block_hash_hex: hex::encode(seal.block_hash),
            last_proposer_hex: hex::encode([0u8; 32]),
            last_round: seal.finalized_round,
            last_timestamp_unix: 0,
            last_message_kind: "finalize".to_string(),
            last_section_count: 0,
        },
    }
}

/// Decodes a hex-encoded 32-byte value and rejects malformed or incorrectly
/// sized input with a structured application error.
fn decode_hash32(value: &str, field: &str, code: ErrorCode) -> Result<[u8; 32], AppError> {
    let bytes = hex::decode(value).map_err(|error| {
        AppError::with_source(code, format!("Failed to decode {field} as hex"), error)
    })?;

    if bytes.len() != 32 {
        return Err(AppError::new(
            code,
            format!("{field} must decode to exactly 32 bytes"),
        ));
    }

    let mut hash = [0u8; 32];
    hash.copy_from_slice(&bytes);
    Ok(hash)
}

/// Derives a deterministic 32-byte digest under a domain-separated label.
fn derive_digest32(domain: &str, payload: &[u8]) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(domain.as_bytes());
    hasher.update(payload);
    hasher.finalize().into()
}

/// Returns a non-zero UNIX timestamp for synthetic block production flows.
fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(1)
        .max(1)
}

#[cfg(test)]
mod tests {
    use super::{build_block_for_tx, produce_once, run_rounds};
    use crate::node::{lifecycle::bootstrap_state, state::NodeState};
    use std::env;
    use std::fs;
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .expect("environment lock must not be poisoned")
    }

    fn temp_home() -> String {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after UNIX_EPOCH")
            .as_nanos();

        let path = env::temp_dir().join(format!("aoxcmd-node-tests-{nonce}"));
        fs::create_dir_all(&path).expect("temporary test directory must be created");
        path.display().to_string()
    }

    #[test]
    fn produce_once_persists_consensus_block_metadata() {
        let _guard = env_lock();
        let home = temp_home();
        unsafe { env::set_var("AOXC_HOME", &home) };

        let state = bootstrap_state().expect("bootstrap must succeed");
        let block = build_block_for_tx(&state, "tx-1").expect("block construction must succeed");
        assert_eq!(block.header.height, 1);

        let next_state = produce_once("tx-1").expect("produce_once must succeed");
        assert_eq!(next_state.current_height, 1);
        assert_eq!(next_state.produced_blocks, 1);
        assert_eq!(next_state.consensus.last_message_kind, "block_proposal");
        assert_eq!(next_state.consensus.last_section_count, 1);
        assert_ne!(
            next_state.consensus.last_block_hash_hex,
            hex::encode([0u8; 32])
        );
    }

    #[test]
    fn run_rounds_chains_parent_hash_between_blocks() {
        let _guard = env_lock();
        let home = temp_home();
        unsafe { env::set_var("AOXC_HOME", &home) };

        bootstrap_state().expect("bootstrap must succeed");
        let state = run_rounds(3, "bench").expect("run_rounds must succeed");
        assert_eq!(state.current_height, 3);
        assert_eq!(state.produced_blocks, 3);
        assert_eq!(state.last_tx, "bench-2");
        assert_eq!(state.consensus.last_message_kind, "block_proposal");
    }

    #[test]
    fn block_builder_rejects_corrupted_legacy_consensus_hash() {
        let mut state = NodeState::bootstrap();
        state.consensus.last_block_hash_hex = "bad-hex".to_string();

        let error =
            build_block_for_tx(&state, "tx-corrupt").expect_err("invalid hex input must fail");
        assert!(error.to_string().contains("last_block_hash_hex"));
    }
}
