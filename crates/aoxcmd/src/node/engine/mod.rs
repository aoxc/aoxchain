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
use aoxcdata::{BlockEnvelope, HybridDataStore, IndexBackend};
use aoxcunity::{
    Block, BlockBody, BlockSection, ConsensusMessage, LaneCommitment, LaneCommitmentSection,
    LaneType, Proposer,
};
use sha2::{Digest, Sha256};
use sha3::Sha3_256;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

mod consensus;
mod primitives;
mod produce;

#[cfg(test)]
mod tests;

use consensus::*;
use primitives::*;
use produce::*;

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

pub fn produce_once(tx: &str) -> Result<NodeState, AppError> {
    produce_once_impl(tx)
}

pub fn run_rounds(rounds: u64, tx_prefix: &str) -> Result<NodeState, AppError> {
    run_rounds_impl(rounds, tx_prefix)
}

pub fn run_rounds_with_observer<F>(
    rounds: u64,
    tx_prefix: &str,
    observer: F,
) -> Result<NodeState, AppError>
where
    F: FnMut(&RoundTelemetry),
{
    run_rounds_with_observer_impl(rounds, tx_prefix, observer)
}
