// Copyright (c) AOXC
// SPDX-License-Identifier: MIT

use aoxcore::genesis::config::GenesisBlock;
use aoxcore::genesis::loader::GenesisLoader;
use aoxcore::identity::hd_path::HdPath;
use aoxcore::identity::key_engine::KeyEngine;
use aoxcore::mempool::pool::{Mempool, MempoolConfig};

use aoxcunity::fork_choice::ForkChoice;
use aoxcunity::quorum::QuorumThreshold;
use aoxcunity::rotation::ValidatorRotation;
use aoxcunity::state::ConsensusState;
use aoxcunity::validator::{Validator, ValidatorRole};

use std::error::Error;
use std::fmt;
use std::time::Duration;

const GENESIS_PATH: &str = "AOXC_DATA/identity/genesis.json";

pub struct AOXCNode {
    pub mempool: Mempool,
    pub consensus: ConsensusState,
    pub fork_choice: ForkChoice,
    pub rotation: ValidatorRotation,
}

#[derive(Debug)]
pub enum NodeInitError {
    GenesisBootstrapFailed(String),
    GenesisValidationFailed(String),
    ValidatorSetInitializationFailed(String),
    QuorumInitializationFailed(String),
    MempoolInitializationFailed(String),
}

impl fmt::Display for NodeInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GenesisBootstrapFailed(reason) => write!(f, "genesis bootstrap failed: {reason}"),
            Self::GenesisValidationFailed(reason) => {
                write!(f, "genesis validation failed: {reason}")
            }
            Self::ValidatorSetInitializationFailed(reason) => {
                write!(f, "validator set initialization failed: {reason}")
            }
            Self::QuorumInitializationFailed(reason) => {
                write!(f, "quorum initialization failed: {reason}")
            }
            Self::MempoolInitializationFailed(reason) => {
                write!(f, "mempool initialization failed: {reason}")
            }
        }
    }
}

impl Error for NodeInitError {}

pub fn setup() -> Result<AOXCNode, NodeInitError> {
    let genesis = GenesisLoader::load_or_create(GENESIS_PATH)
        .map_err(|error| NodeInitError::GenesisBootstrapFailed(error.to_string()))?;

    genesis
        .config
        .validate()
        .map_err(NodeInitError::GenesisValidationFailed)?;

    let key_engine = KeyEngine::new(None);
    let validators = build_validators(&key_engine, &genesis);

    let rotation = ValidatorRotation::new(validators)
        .map_err(|error| NodeInitError::ValidatorSetInitializationFailed(error.to_string()))?;

    let quorum = QuorumThreshold::new(2, 3)
        .map_err(|error| NodeInitError::QuorumInitializationFailed(error.to_string()))?;

    let consensus = ConsensusState::new(rotation.clone(), quorum);
    let fork_choice = ForkChoice::new();

    let mempool = Mempool::new(default_mempool_config())
        .map_err(|error| NodeInitError::MempoolInitializationFailed(error.to_string()))?;

    Ok(AOXCNode {
        mempool,
        consensus,
        fork_choice,
        rotation,
    })
}

fn default_mempool_config() -> MempoolConfig {
    MempoolConfig {
        max_txs: 10_000,
        max_tx_size: 1024 * 1024,
        max_total_bytes: 64 * 1024 * 1024,
        tx_ttl: Duration::from_secs(60 * 30),
    }
}

fn build_validators(key_engine: &KeyEngine, genesis: &GenesisBlock) -> Vec<Validator> {
    let seed = key_engine.derive_entropy(&HdPath {
        chain: genesis.config.chain_num,
        role: 1,
        zone: 0,
        index: 0,
    });

    let mut validator_id = [0u8; 32];
    validator_id.copy_from_slice(&seed[..32]);

    vec![Validator::new(validator_id, 1, ValidatorRole::Proposer)]
}
