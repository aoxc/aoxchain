use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct FaucetClaimRecord {
    pub(super) account_id: String,
    pub(super) amount: u64,
    #[serde(alias = "timestamp_unix")]
    pub(super) claimed_at: u64,
    #[serde(alias = "tx_id")]
    pub(super) tx_hash: String,
    pub(super) status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct FaucetAuditRecord {
    pub(super) at_unix: u64,
    pub(super) action: String,
    pub(super) actor: String,
    pub(super) detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub(super) struct FaucetState {
    pub(super) enabled: bool,
    pub(super) max_claim_amount: u64,
    pub(super) cooldown_secs: u64,
    pub(super) daily_limit_per_account: u64,
    pub(super) daily_global_limit: u64,
    pub(super) min_reserve_balance: u64,
    pub(super) claims: Vec<FaucetClaimRecord>,
    pub(super) banned_accounts: Vec<String>,
    pub(super) allowlisted_accounts: Vec<String>,
    pub(super) audit_log: Vec<FaucetAuditRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct FaucetClaimDecision {
    pub(super) allowed: bool,
    pub(super) cooldown_remaining_secs: u64,
    pub(super) claimed_last_24h: u64,
    pub(super) daily_remaining: u64,
    pub(super) global_distributed_last_24h: u64,
    pub(super) global_remaining: u64,
    pub(super) next_eligible_claim_at: Option<u64>,
    pub(super) denied_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct TxIndex {
    pub(super) entries: BTreeMap<String, TxIndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct TxIndexEntry {
    pub(super) tx_payload: String,
    pub(super) block_height: u64,
    pub(super) block_hash_hex: String,
    pub(super) execution_status: String,
    pub(super) gas_used: u64,
    pub(super) fee_paid: u64,
    pub(super) events: Vec<String>,
    pub(super) state_change_summary: String,
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
pub(super) struct ReadinessCheck {
    pub(super) name: &'static str,
    pub(super) area: &'static str,
    pub(super) passed: bool,
    pub(super) weight: u8,
    pub(super) detail: String,
}

#[derive(Serialize)]
pub(super) struct Readiness {
    pub(super) profile: String,
    pub(super) stage: &'static str,
    pub(super) readiness_score: u8,
    pub(super) max_score: u8,
    pub(super) completed_weight: u8,
    pub(super) remaining_weight: u8,
    pub(super) verdict: &'static str,
    pub(super) blockers: Vec<String>,
    pub(super) remediation_plan: Vec<String>,
    pub(super) next_focus: Vec<String>,
    pub(super) area_progress: Vec<ReadinessAreaProgress>,
    pub(super) track_progress: Vec<ReadinessTrackProgress>,
    pub(super) checks: Vec<ReadinessCheck>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct SurfaceCheck {
    pub(super) name: &'static str,
    pub(super) passed: bool,
    pub(super) detail: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct SurfaceReadiness {
    pub(super) surface: &'static str,
    pub(super) owner: &'static str,
    pub(super) status: &'static str,
    pub(super) score: u8,
    pub(super) blockers: Vec<String>,
    pub(super) evidence: Vec<String>,
    pub(super) checks: Vec<SurfaceCheck>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct FullSurfaceReadiness {
    pub(super) release_line: &'static str,
    pub(super) matrix_path: String,
    pub(super) matrix_loaded: bool,
    pub(super) matrix_release_line: Option<String>,
    pub(super) matrix_surface_count: u8,
    pub(super) matrix_warnings: Vec<String>,
    pub(super) overall_status: &'static str,
    pub(super) overall_score: u8,
    pub(super) candidate_surfaces: u8,
    pub(super) total_surfaces: u8,
    pub(super) surfaces: Vec<SurfaceReadiness>,
    pub(super) blockers: Vec<String>,
    pub(super) next_focus: Vec<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct SurfaceGateFailure {
    pub(super) surface: String,
    pub(super) check: String,
    pub(super) code: String,
    pub(super) detail: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct FullSurfaceGateReport {
    pub(super) profile: String,
    pub(super) enforced: bool,
    pub(super) passed: bool,
    pub(super) overall_status: String,
    pub(super) overall_score: u8,
    pub(super) failure_count: usize,
    pub(super) failures: Vec<SurfaceGateFailure>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct PlatformLevelScore {
    pub(super) profile: String,
    pub(super) mainnet_readiness_score: u8,
    pub(super) full_surface_score: u8,
    pub(super) block_production_score: u8,
    pub(super) net_level_score: u8,
    pub(super) level_verdict: &'static str,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct ReadinessAreaProgress {
    pub(super) area: &'static str,
    pub(super) completed_weight: u8,
    pub(super) max_weight: u8,
    pub(super) ratio: u8,
    pub(super) passed_checks: u8,
    pub(super) total_checks: u8,
    pub(super) status: &'static str,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct ReadinessTrackProgress {
    pub(super) name: &'static str,
    pub(super) completed_weight: u8,
    pub(super) max_weight: u8,
    pub(super) ratio: u8,
    pub(super) status: &'static str,
    pub(super) objective: &'static str,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub(super) struct FullSurfaceMatrixModel {
    pub(super) release_line: String,
    pub(super) surfaces: Vec<FullSurfaceMatrixSurface>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub(super) struct FullSurfaceMatrixSurface {
    pub(super) id: String,
    pub(super) owner: String,
    pub(super) required_evidence: Vec<String>,
    pub(super) verification_command: String,
    pub(super) blocker: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct ProfileBaselineReport {
    pub(super) mainnet_path: String,
    pub(super) testnet_path: String,
    pub(super) passed: bool,
    pub(super) shared_controls: Vec<BaselineControl>,
    pub(super) drift: Vec<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct BaselineControl {
    pub(super) name: &'static str,
    pub(super) mainnet: String,
    pub(super) testnet: String,
    pub(super) passed: bool,
    pub(super) expectation: &'static str,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct NetworkProfileConfig {
    pub(super) chain_id: String,
    pub(super) listen_addr: String,
    pub(super) rpc_addr: String,
    pub(super) peers: Vec<String>,
    pub(super) security_mode: String,
}
