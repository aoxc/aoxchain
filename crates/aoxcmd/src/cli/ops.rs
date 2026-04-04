// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::data_home::resolve_home;
use crate::{
    app::{
        bootstrap::bootstrap_operator_home, runtime::refresh_runtime_metrics,
        shutdown::graceful_shutdown,
    },
    cli_support::{arg_value, emit_serialized, has_flag, output_format, text_envelope},
    config::{loader::load, settings::Settings},
    economy::ledger,
    error::{AppError, ErrorCode},
    node::{engine, lifecycle},
    runtime::{
        core::runtime_context, handles::default_handles, node::health_status, unity::unity_status,
    },
};
use aoxcdata::{BlockEnvelope, DataError, HybridDataStore, IndexBackend};
use chrono::{Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpStream, ToSocketAddrs},
    path::{Path, PathBuf},
    time::Duration,
};

mod consensus_ops;
mod faucet;
mod node_ops;
mod readiness_commands;
mod vm_ops;

pub use consensus_ops::{
    cmd_consensus_commits, cmd_consensus_evidence, cmd_consensus_finality, cmd_consensus_proposer,
    cmd_consensus_round, cmd_consensus_status, cmd_consensus_validators,
};
pub use faucet::{
    cmd_faucet_audit, cmd_faucet_balance, cmd_faucet_claim, cmd_faucet_config,
    cmd_faucet_config_show, cmd_faucet_disable, cmd_faucet_enable, cmd_faucet_history,
    cmd_faucet_reset, cmd_faucet_status,
};
pub use node_ops::{
    cmd_network_smoke, cmd_node_bootstrap, cmd_node_health, cmd_node_run, cmd_produce_once,
    cmd_real_network, cmd_storage_smoke,
};
pub use readiness_commands::{
    cmd_full_surface_gate, cmd_full_surface_readiness, cmd_level_score, cmd_load_benchmark,
    cmd_mainnet_readiness, cmd_profile_baseline, cmd_testnet_readiness,
};
pub use vm_ops::{
    cmd_vm_call, cmd_vm_code_get, cmd_vm_contract_get, cmd_vm_estimate_gas, cmd_vm_simulate,
    cmd_vm_status, cmd_vm_storage_get, cmd_vm_trace,
};

const FAUCET_MAX_CLAIM_AMOUNT: u64 = 10_000;
const FAUCET_COOLDOWN_SECS: u64 = 3_600;
const FAUCET_DAILY_LIMIT_PER_ACCOUNT: u64 = 50_000;
const FAUCET_DAILY_GLOBAL_LIMIT: u64 = 1_000_000;
const FAUCET_MIN_RESERVE_BALANCE: u64 = 100_000;
const FAUCET_AUDIT_RETENTION_HOURS: i64 = 168;
const TX_INDEX_FILE: &str = "tx-index.v1.json";

#[cfg(test)]
mod tests;
