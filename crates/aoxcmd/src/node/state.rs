// Copyright (c) AOXC
// SPDX-License-Identifier: MIT

// AOXC Core
use aoxcore::genesis::loader::GenesisLoader;
use aoxcore::genesis::GenesisBlock;
use aoxcore::identity::hd_path::HdPath;
use aoxcore::identity::key_engine::KeyEngine;
use aoxcore::mempool::pool::{Mempool, MempoolConfig};

// AOXC Unity
use aoxcunity::fork_choice::ForkChoice;
use aoxcunity::rotation::ValidatorRotation;
use aoxcunity::state::ConsensusState;
use aoxcunity::validator::{Validator, ValidatorRole};

// Standard library
use std::error::Error;
use std::fmt;
use std::time::Duration;

const GENESIS_PATH: &str = "AOXC_DATA/identity/genesis.json";

/// AOXC node runtime container.
///
/// This structure owns the core runtime subsystems required to bootstrap
/// and operate a local node process. The container intentionally excludes
/// transport-specific or CLI-specific concerns in order to preserve a
/// clean separation between:
/// - domain state initialization,
/// - consensus coordination,
/// - runtime orchestration,
/// - external I/O integration.
///
/// Network transport and observability layers should be composed by the
/// outer runtime boundary rather than hard-wired into state bootstrap.
pub struct AOXCNode {
    pub mempool: Mempool,
    pub consensus: ConsensusState,
    pub fork_choice: ForkChoice,
    pub rotation: ValidatorRotation,
}

/// Error type returned when node bootstrap cannot be completed safely.
///
/// The variants are string-backed by design so this module remains decoupled
/// from lower-level error type volatility during the ongoing architecture
/// migration.
#[derive(Debug)]
pub enum NodeInitError {
    GenesisBootstrapFailed(String),
    GenesisValidationFailed(String),
    MempoolInitializationFailed(String),
}

impl fmt::Display for NodeInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GenesisBootstrapFailed(reason) => {
                write!(f, "genesis bootstrap failed: {reason}")
            }
            Self::GenesisValidationFailed(reason) => {
                write!(f, "genesis validation failed: {reason}")
            }
            Self::MempoolInitializationFailed(reason) => {
                write!(f, "mempool initialization failed: {reason}")
            }
        }
    }
}

impl Error for NodeInitError {}

/// Initializes the AOXC node runtime.
///
/// Bootstrap sequence:
/// 1. Load or create the genesis state.
/// 2. Validate the genesis configuration.
/// 3. Derive local identity material.
/// 4. Build the initial validator set.
/// 5. Initialize consensus state, rotation policy, fork choice, and mempool.
///
/// Design note:
/// This function returns a typed error rather than panicking. Bootstrap is a
/// critical control path, and a caller should retain the ability to decide
/// whether to abort, retry, or surface structured diagnostics.
pub fn setup() -> Result<AOXCNode, NodeInitError> {
    // ---------------------------------------------------------------------
    // 1. Genesis bootstrap
    // ---------------------------------------------------------------------
    let genesis = GenesisLoader::load_or_create(GENESIS_PATH)
        .map_err(|error| NodeInitError::GenesisBootstrapFailed(error.to_string()))?;

    genesis
        .config
        .validate()
        .map_err(|error| NodeInitError::GenesisValidationFailed(error.to_string()))?;

    // ---------------------------------------------------------------------
    // 2. Identity engine
    // ---------------------------------------------------------------------
    let key_engine = KeyEngine::new(None);

    // ---------------------------------------------------------------------
    // 3. Validator set construction
    // ---------------------------------------------------------------------
    let validators = build_validators(&key_engine, &genesis);

    // ---------------------------------------------------------------------
    // 4. Consensus state
    // ---------------------------------------------------------------------
    let consensus = ConsensusState::new();

    // ---------------------------------------------------------------------
    // 5. Validator rotation
    // ---------------------------------------------------------------------
    let rotation = ValidatorRotation::new(&validators);

    // ---------------------------------------------------------------------
    // 6. Fork choice
    // ---------------------------------------------------------------------
    let fork_choice = ForkChoice::new();

    // ---------------------------------------------------------------------
    // 7. Mempool
    // ---------------------------------------------------------------------
    let mempool = Mempool::new(default_mempool_config())
        .map_err(|error| NodeInitError::MempoolInitializationFailed(error.to_string()))?;

    Ok(AOXCNode {
        mempool,
        consensus,
        fork_choice,
        rotation,
    })
}

/// Returns the default mempool configuration used during node bootstrap.
///
/// Design rationale:
/// - `max_txs` bounds queue cardinality,
/// - `max_tx_size` bounds per-transaction memory exposure,
/// - `max_total_bytes` bounds aggregate resident payload memory,
/// - `tx_ttl` bounds stale transaction retention.
///
/// These defaults are intentionally conservative and suitable for local
/// development or controlled validator environments. Production deployments
/// should externalize these parameters and apply environment-specific policy.
fn default_mempool_config() -> MempoolConfig {
    MempoolConfig {
        max_txs: 10_000,
        max_tx_size: 1024 * 1024,
        max_total_bytes: 64 * 1024 * 1024,
        tx_ttl: Duration::from_secs(60 * 30),
    }
}

/// Builds the validator set from locally derived identity material.
///
/// Current behavior:
/// - derives deterministic entropy from the configured HD path,
/// - converts the first 32 bytes into a public key payload,
/// - constructs a single local validator entry.
///
/// Security note:
/// This implementation is appropriate for local development, deterministic
/// testing, or a single-node bootstrap environment. A production-grade
/// validator set should be sourced from authenticated genesis state, a
/// governance-controlled registry, or an equivalent trusted authority model.
fn build_validators(key_engine: &KeyEngine, genesis: &GenesisBlock) -> Vec<Validator> {
    let seed = key_engine.derive_entropy(&HdPath {
        chain: genesis.config.chain_num,
        role: 1,
        zone: 0,
        index: 0,
    });

    let public_key = seed[..32].to_vec();

    vec![Validator {
        actor_id: "AOXC-NODE-0001".to_string(),
        role: ValidatorRole::Node,
        public_key,
    }]
}
