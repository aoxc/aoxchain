use crate::{
    error::{AppError, ErrorCode},
<<<<<<< HEAD
=======
    node::{
        lifecycle::{load_state, persist_state},
        state::{ConsensusSnapshot, NodeState},
    error::AppError,
>>>>>>> origin/main
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

const DEFAULT_NETWORK_ID: u32 = 2626;

pub fn produce_once(tx: &str) -> Result<NodeState, AppError> {
    let mut state = load_state()?;
    state.running = true;

    let block = build_block_for_tx(&state, tx)?;
    apply_block_proposal(&mut state, tx, &block);

    persist_state(&state)?;
    Ok(state)
}

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

fn build_block_for_tx(state: &NodeState, tx: &str) -> Result<Block, AppError> {
    let height = state.current_height.saturating_add(1);
    let round = state.consensus.last_round.saturating_add(1);
    let timestamp = unix_now();
<<<<<<< HEAD

=======
>>>>>>> origin/main
    let parent_hash = decode_hash32(
        &state.consensus.last_block_hash_hex,
        "last_block_hash_hex",
        ErrorCode::NodeStateInvalid,
    )?;
<<<<<<< HEAD

    let proposer_key = derive_digest32("AOXC-CMD-PROPOSER", tx.as_bytes());

=======
    let proposer = derive_digest32("AOXC-CMD-PROPOSER", tx.as_bytes());
>>>>>>> origin/main
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

<<<<<<< HEAD
    let proposer = Proposer::new(state.consensus.network_id.max(1), proposer_key);

=======
    let proposer = Proposer::new(state.consensus.network_id.max(1), proposer);
>>>>>>> origin/main
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

fn derive_digest32(domain: &str, payload: &[u8]) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(domain.as_bytes());
    hasher.update(payload);
    hasher.finalize().into()
}

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
            .expect("env lock should not be poisoned")
    }

    fn temp_home() -> String {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
<<<<<<< HEAD

=======
>>>>>>> origin/main
        let path = env::temp_dir().join(format!("aoxcmd-node-tests-{nonce}"));
        fs::create_dir_all(&path).expect("temp dir should be created");
        path.display().to_string()
    }

    #[test]
    fn produce_once_persists_consensus_block_metadata() {
        let _guard = env_lock();
        let home = temp_home();
        unsafe { env::set_var("AOXC_HOME", &home) };

        let state = bootstrap_state().expect("bootstrap should succeed");
        let block = build_block_for_tx(&state, "tx-1").expect("block should build");
        assert_eq!(block.header.height, 1);

        let next_state = produce_once("tx-1").expect("produce_once should succeed");
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

        bootstrap_state().expect("bootstrap should succeed");
        let state = run_rounds(3, "bench").expect("run_rounds should succeed");
        assert_eq!(state.current_height, 3);
        assert_eq!(state.produced_blocks, 3);
        assert_eq!(state.last_tx, "bench-2");
        assert_eq!(state.consensus.last_message_kind, "block_proposal");
    }

    #[test]
    fn block_builder_rejects_corrupted_legacy_consensus_hash() {
        let mut state = NodeState::bootstrap();
        state.consensus.last_block_hash_hex = "bad-hex".to_string();

        let error = build_block_for_tx(&state, "tx-corrupt").expect_err("invalid hex must fail");
        assert!(error.to_string().contains("last_block_hash_hex"));
    }
}
