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

mod chain_ops;
mod consensus_ops;
mod economy_runtime_ops;
mod faucet;
mod metrics_ops;
mod network_read_ops;
mod node_ops;
mod readiness_commands;
mod readiness_core;
mod rpc_status_ops;
mod shared_helpers;
mod tx_account_ops;
mod vm_ops;

pub use chain_ops::{cmd_block_get, cmd_chain_status};
pub use consensus_ops::{
    cmd_consensus_commits, cmd_consensus_evidence, cmd_consensus_finality, cmd_consensus_proposer,
    cmd_consensus_round, cmd_consensus_status, cmd_consensus_validators,
};
pub use economy_runtime_ops::{
    cmd_economy_init, cmd_economy_status, cmd_runtime_status, cmd_stake_delegate,
    cmd_stake_undelegate, cmd_treasury_transfer,
};
pub use faucet::{
    cmd_faucet_audit, cmd_faucet_balance, cmd_faucet_claim, cmd_faucet_config,
    cmd_faucet_config_show, cmd_faucet_disable, cmd_faucet_enable, cmd_faucet_history,
    cmd_faucet_reset, cmd_faucet_status,
};
pub use metrics_ops::cmd_metrics;
pub use network_read_ops::{cmd_network_full, cmd_network_status, cmd_peer_list, cmd_state_root};
pub use node_ops::{
    cmd_network_smoke, cmd_node_bootstrap, cmd_node_health, cmd_node_run, cmd_produce_once,
    cmd_real_network, cmd_storage_smoke,
};
pub use readiness_commands::{
    cmd_full_surface_gate, cmd_full_surface_readiness, cmd_level_score, cmd_load_benchmark,
    cmd_mainnet_readiness, cmd_profile_baseline, cmd_testnet_readiness,
};
pub use rpc_status_ops::{cmd_rpc_curl_smoke, cmd_rpc_status};
pub use tx_account_ops::{cmd_account_get, cmd_balance_get, cmd_tx_get, cmd_tx_receipt};
pub use vm_ops::{
    cmd_vm_call, cmd_vm_code_get, cmd_vm_contract_get, cmd_vm_estimate_gas, cmd_vm_simulate,
    cmd_vm_status, cmd_vm_storage_get, cmd_vm_trace,
};

use readiness_core::*;
use shared_helpers::*;

const FAUCET_MAX_CLAIM_AMOUNT: u64 = 10_000;
const FAUCET_COOLDOWN_SECS: u64 = 3_600;
const FAUCET_DAILY_LIMIT_PER_ACCOUNT: u64 = 50_000;
const FAUCET_DAILY_GLOBAL_LIMIT: u64 = 1_000_000;
const FAUCET_MIN_RESERVE_BALANCE: u64 = 100_000;
const FAUCET_AUDIT_RETENTION_HOURS: i64 = 168;
const TX_INDEX_FILE: &str = "tx-index.v1.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct FaucetClaimRecord {
    account_id: String,
    amount: u64,
    #[serde(alias = "timestamp_unix")]
    claimed_at: u64,
    #[serde(alias = "tx_id")]
    tx_hash: String,
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct FaucetAuditRecord {
    at_unix: u64,
    action: String,
    actor: String,
    detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
struct FaucetState {
    enabled: bool,
    max_claim_amount: u64,
    cooldown_secs: u64,
    daily_limit_per_account: u64,
    daily_global_limit: u64,
    min_reserve_balance: u64,
    claims: Vec<FaucetClaimRecord>,
    banned_accounts: Vec<String>,
    allowlisted_accounts: Vec<String>,
    audit_log: Vec<FaucetAuditRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct FaucetClaimDecision {
    allowed: bool,
    cooldown_remaining_secs: u64,
    claimed_last_24h: u64,
    daily_remaining: u64,
    global_distributed_last_24h: u64,
    global_remaining: u64,
    next_eligible_claim_at: Option<u64>,
    denied_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TxIndex {
    entries: BTreeMap<String, TxIndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TxIndexEntry {
    tx_payload: String,
    block_height: u64,
    block_hash_hex: String,
    execution_status: String,
    gas_used: u64,
    fee_paid: u64,
    events: Vec<String>,
    state_change_summary: String,
}

impl Default for FaucetState {
    fn default() -> Self {
        Self {
            enabled: true,
            max_claim_amount: FAUCET_MAX_CLAIM_AMOUNT,
            cooldown_secs: FAUCET_COOLDOWN_SECS,
            daily_limit_per_account: FAUCET_DAILY_LIMIT_PER_ACCOUNT,
            daily_global_limit: FAUCET_DAILY_GLOBAL_LIMIT,
            min_reserve_balance: FAUCET_MIN_RESERVE_BALANCE,
            claims: Vec::new(),
            banned_accounts: Vec::new(),
            allowlisted_accounts: Vec::new(),
            audit_log: Vec::new(),
        }
    }
}

#[derive(Serialize)]
struct ReadinessCheck {
    name: &'static str,
    area: &'static str,
    passed: bool,
    weight: u8,
    detail: String,
}

#[derive(Serialize)]
struct Readiness {
    profile: String,
    stage: &'static str,
    readiness_score: u8,
    max_score: u8,
    completed_weight: u8,
    remaining_weight: u8,
    verdict: &'static str,
    blockers: Vec<String>,
    remediation_plan: Vec<String>,
    next_focus: Vec<String>,
    area_progress: Vec<ReadinessAreaProgress>,
    track_progress: Vec<ReadinessTrackProgress>,
    checks: Vec<ReadinessCheck>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct SurfaceCheck {
    name: &'static str,
    passed: bool,
    detail: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct SurfaceReadiness {
    surface: &'static str,
    owner: &'static str,
    status: &'static str,
    score: u8,
    blockers: Vec<String>,
    evidence: Vec<String>,
    checks: Vec<SurfaceCheck>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct FullSurfaceReadiness {
    release_line: &'static str,
    matrix_path: String,
    matrix_loaded: bool,
    matrix_release_line: Option<String>,
    matrix_surface_count: u8,
    matrix_warnings: Vec<String>,
    overall_status: &'static str,
    overall_score: u8,
    candidate_surfaces: u8,
    total_surfaces: u8,
    surfaces: Vec<SurfaceReadiness>,
    blockers: Vec<String>,
    next_focus: Vec<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct SurfaceGateFailure {
    surface: String,
    check: String,
    code: String,
    detail: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct FullSurfaceGateReport {
    profile: String,
    enforced: bool,
    passed: bool,
    overall_status: String,
    overall_score: u8,
    failure_count: usize,
    failures: Vec<SurfaceGateFailure>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct PlatformLevelScore {
    profile: String,
    mainnet_readiness_score: u8,
    full_surface_score: u8,
    block_production_score: u8,
    net_level_score: u8,
    level_verdict: &'static str,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct ReadinessAreaProgress {
    area: &'static str,
    completed_weight: u8,
    max_weight: u8,
    ratio: u8,
    passed_checks: u8,
    total_checks: u8,
    status: &'static str,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct ReadinessTrackProgress {
    name: &'static str,
    completed_weight: u8,
    max_weight: u8,
    ratio: u8,
    status: &'static str,
    objective: &'static str,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct FullSurfaceMatrixModel {
    release_line: String,
    surfaces: Vec<FullSurfaceMatrixSurface>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct FullSurfaceMatrixSurface {
    id: String,
    owner: String,
    required_evidence: Vec<String>,
    verification_command: String,
    blocker: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct ProfileBaselineReport {
    mainnet_path: String,
    testnet_path: String,
    passed: bool,
    shared_controls: Vec<BaselineControl>,
    drift: Vec<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct BaselineControl {
    name: &'static str,
    mainnet: String,
    testnet: String,
    passed: bool,
    expectation: &'static str,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct NetworkProfileConfig {
    chain_id: String,
    listen_addr: String,
    rpc_addr: String,
    peers: Vec<String>,
    security_mode: String,
}

#[cfg(test)]
mod tests;
