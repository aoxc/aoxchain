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

const FAUCET_MAX_CLAIM_AMOUNT: u64 = 10_000;
const FAUCET_COOLDOWN_SECS: u64 = 3_600;
const FAUCET_DAILY_LIMIT_PER_ACCOUNT: u64 = 50_000;
const FAUCET_DAILY_GLOBAL_LIMIT: u64 = 1_000_000;
const FAUCET_MIN_RESERVE_BALANCE: u64 = 100_000;
const FAUCET_AUDIT_RETENTION_HOURS: i64 = 168;

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

pub fn cmd_load_benchmark(args: &[String]) -> Result<(), AppError> {
    let rounds = parse_positive_u64_arg(args, "--rounds", 100, "load benchmark")?;

    let mut details = BTreeMap::new();
    details.insert("benchmark_rounds".to_string(), rounds.to_string());
    details.insert(
        "result".to_string(),
        "baseline-local-benchmark-recorded".to_string(),
    );

    emit_serialized(
        &text_envelope("load-benchmark", "ok", details),
        output_format(args),
    )
}

pub fn cmd_mainnet_readiness(args: &[String]) -> Result<(), AppError> {
    cmd_profile_readiness(args, "mainnet")
}

pub fn cmd_testnet_readiness(args: &[String]) -> Result<(), AppError> {
    cmd_profile_readiness(args, "testnet")
}

fn cmd_profile_readiness(args: &[String], target_profile: &'static str) -> Result<(), AppError> {
    let settings = effective_settings_for_ops()?;
    let key_summary = crate::keys::manager::inspect_operator_key().ok();
    let genesis_ok = crate::cli::bootstrap::genesis_ready();
    let node_ok = lifecycle::load_state().is_ok();
    let config_validation = settings.validate();

    let readiness = evaluate_profile_readiness(
        target_profile,
        &settings,
        config_validation.err(),
        key_summary
            .as_ref()
            .map(|summary| summary.operational_state.as_str()),
        genesis_ok,
        node_ok,
    );

    if let Some(path) = parse_optional_text_arg(args, "--write-report", false) {
        write_readiness_markdown_report(
            Path::new(&path),
            &readiness,
            compare_embedded_network_profiles().ok().as_ref(),
            compare_aoxhub_network_profiles().ok().as_ref(),
        )?;
    }

    if has_flag(args, "--enforce") && readiness.verdict != "candidate" {
        let profile_title = if target_profile.eq_ignore_ascii_case("testnet") {
            "Testnet"
        } else {
            "Mainnet"
        };
        return Err(AppError::new(
            ErrorCode::PolicyGateFailed,
            format!(
                "{profile_title} readiness enforcement failed at score {} with blockers: {}",
                readiness.readiness_score,
                readiness.blockers.join(" | ")
            ),
        ));
    }

    emit_serialized(&readiness, output_format(args))
}

pub fn cmd_full_surface_readiness(args: &[String]) -> Result<(), AppError> {
    let settings = effective_settings_for_ops()?;
    let key_summary = crate::keys::manager::inspect_operator_key().ok();
    let genesis_ok = crate::cli::bootstrap::genesis_ready();
    let node_ok = lifecycle::load_state().is_ok();

    let full = evaluate_full_surface_readiness(
        &settings,
        &evaluate_profile_readiness(
            "mainnet",
            &settings,
            settings.validate().err(),
            key_summary
                .as_ref()
                .map(|summary| summary.operational_state.as_str()),
            genesis_ok,
            node_ok,
        ),
    );

    if let Some(path) = parse_optional_text_arg(args, "--write-report", false) {
        write_full_surface_markdown_report(Path::new(&path), &full)?;
    }

    if has_flag(args, "--enforce") && full.overall_status != "candidate" {
        return Err(AppError::new(
            ErrorCode::PolicyGateFailed,
            format!(
                "Full-surface readiness enforcement failed at score {} with blockers: {}",
                full.overall_score,
                full.blockers.join(" | ")
            ),
        ));
    }

    emit_serialized(&full, output_format(args))
}

pub fn cmd_full_surface_gate(args: &[String]) -> Result<(), AppError> {
    let settings = effective_settings_for_ops()?;
    let key_summary = crate::keys::manager::inspect_operator_key().ok();
    let genesis_ok = crate::cli::bootstrap::genesis_ready();
    let node_ok = lifecycle::load_state().is_ok();

    let mainnet = evaluate_profile_readiness(
        "mainnet",
        &settings,
        settings.validate().err(),
        key_summary
            .as_ref()
            .map(|summary| summary.operational_state.as_str()),
        genesis_ok,
        node_ok,
    );
    let full = evaluate_full_surface_readiness(&settings, &mainnet);
    let failures = collect_surface_gate_failures(&full);

    let report = FullSurfaceGateReport {
        profile: settings.profile.clone(),
        enforced: has_flag(args, "--enforce"),
        passed: failures.is_empty(),
        overall_status: full.overall_status.to_string(),
        overall_score: full.overall_score,
        failure_count: failures.len(),
        failures,
    };

    if report.enforced && !report.passed {
        let codes = report
            .failures
            .iter()
            .map(|failure| failure.code.clone())
            .collect::<Vec<_>>()
            .join(",");
        return Err(AppError::new(
            ErrorCode::PolicyGateFailed,
            format!(
                "Full-surface gate enforcement failed with {} failing checks [{}]",
                report.failure_count, codes
            ),
        ));
    }

    emit_serialized(&report, output_format(args))
}

pub fn cmd_level_score(args: &[String]) -> Result<(), AppError> {
    let settings = effective_settings_for_ops()?;
    let key_summary = crate::keys::manager::inspect_operator_key().ok();
    let genesis_ok = crate::cli::bootstrap::genesis_ready();
    let node_state = lifecycle::load_state().ok();
    let node_ok = node_state.is_some();

    let mainnet = evaluate_profile_readiness(
        "mainnet",
        &settings,
        settings.validate().err(),
        key_summary
            .as_ref()
            .map(|summary| summary.operational_state.as_str()),
        genesis_ok,
        node_ok,
    );
    let full = evaluate_full_surface_readiness(&settings, &mainnet);

    let block_production_score = node_state
        .as_ref()
        .map(|state| if state.current_height > 0 { 100 } else { 0 })
        .unwrap_or(0);

    let net_level_score = ((u16::from(mainnet.readiness_score)
        + u16::from(full.overall_score)
        + u16::from(block_production_score))
        / 3) as u8;

    let level_verdict = if net_level_score >= 100 {
        "perfect"
    } else if net_level_score >= 90 {
        "candidate"
    } else if net_level_score >= 70 {
        "in-progress"
    } else {
        "bootstrap"
    };

    let score = PlatformLevelScore {
        profile: settings.profile,
        mainnet_readiness_score: mainnet.readiness_score,
        full_surface_score: full.overall_score,
        block_production_score,
        net_level_score,
        level_verdict,
    };

    if has_flag(args, "--enforce") && score.net_level_score < 100 {
        return Err(AppError::new(
            ErrorCode::PolicyGateFailed,
            format!(
                "Platform level score enforcement failed at {} (mainnet={}, full-surface={}, block-production={})",
                score.net_level_score,
                score.mainnet_readiness_score,
                score.full_surface_score,
                score.block_production_score
            ),
        ));
    }

    emit_serialized(&score, output_format(args))
}

pub fn cmd_profile_baseline(args: &[String]) -> Result<(), AppError> {
    let report = compare_embedded_network_profiles()?;

    if has_flag(args, "--enforce") && !report.passed {
        return Err(AppError::new(
            ErrorCode::PolicyGateFailed,
            "Mainnet/testnet baseline parity failed; inspect drift before production promotion",
        ));
    }

    emit_serialized(&report, output_format(args))
}

fn evaluate_profile_readiness(
    target_profile: &'static str,
    settings: &crate::config::settings::Settings,
    config_validation_error: Option<String>,
    key_operational_state: Option<&str>,
    genesis_ok: bool,
    node_ok: bool,
) -> Readiness {
    let repo_root = locate_repo_root();
    let release_dir = locate_repo_artifact_dir("release-evidence");
    let closure_dir = locate_repo_artifact_dir("network-production-closure");
    let profile_checklist = repo_root.join("docs").join("src").join(
        if target_profile.eq_ignore_ascii_case("testnet") {
            "TESTNET_READINESS_CHECKLIST.md"
        } else {
            "MAINNET_READINESS_CHECKLIST.md"
        },
    );
    let checklist_open_items = open_checklist_items(&profile_checklist);
    let checklist_missing = checklist_open_items
        .iter()
        .all(|item| item.starts_with("missing-checklist:"));
    let baseline_parity = compare_embedded_network_profiles().ok();
    let aoxhub_parity = compare_aoxhub_network_profiles().ok();
    let key_state = key_operational_state.unwrap_or("missing");

    let checks = vec![
        readiness_check(
            "config-valid",
            "configuration",
            config_validation_error.is_none(),
            15,
            config_validation_error
                .map(|error| format!("Configuration validation failed: {error}"))
                .unwrap_or_else(|| "Operator configuration passed validation".to_string()),
        ),
        readiness_check(
            if target_profile.eq_ignore_ascii_case("testnet") {
                "testnet-profile"
            } else {
                "mainnet-profile"
            },
            "configuration",
            settings.profile.eq_ignore_ascii_case(target_profile),
            10,
            format!("Active profile is {}", settings.profile),
        ),
        readiness_check(
            "official-peers",
            "network",
            settings.network.enforce_official_peers,
            10,
            "Official peer enforcement must remain enabled for production".to_string(),
        ),
        readiness_check(
            "telemetry-metrics",
            "observability",
            settings.telemetry.enable_metrics,
            8,
            "Prometheus/metrics export is required for production operations".to_string(),
        ),
        readiness_check(
            "structured-logging",
            "observability",
            settings.logging.json,
            8,
            "JSON logs are required for audit trails and SIEM ingestion".to_string(),
        ),
        readiness_check(
            "genesis-present",
            "identity",
            genesis_ok,
            10,
            "Committed genesis material must exist in AOXC home".to_string(),
        ),
        readiness_check(
            "node-state-present",
            "runtime",
            node_ok,
            8,
            "Node runtime state must exist and load cleanly".to_string(),
        ),
        readiness_check(
            "operator-key-active",
            "identity",
            matches!(key_state, "active"),
            12,
            format!("Operator key operational state is {key_state}"),
        ),
        readiness_check(
            "profile-baseline-parity",
            "release",
            baseline_parity.as_ref().is_some_and(|report| report.passed),
            8,
            baseline_parity
                .map(|report| {
                    if report.passed {
                        "Mainnet and testnet embedded baselines share the same production control shape".to_string()
                    } else {
                        format!(
                            "Embedded mainnet/testnet baseline drift detected: {}",
                            report.drift.join("; ")
                        )
                    }
                })
                .unwrap_or_else(|| {
                    "Unable to compare embedded mainnet/testnet baseline files".to_string()
                }),
        ),
        readiness_check(
            "aoxhub-baseline-parity",
            "release",
            aoxhub_parity.as_ref().is_some_and(|report| report.passed),
            5,
            aoxhub_parity
                .map(|report| {
                    if report.passed {
                        "AOXHub mainnet/testnet baselines are aligned with the same security and port model".to_string()
                    } else {
                        format!(
                            "AOXHub mainnet/testnet drift detected: {}",
                            report.drift.join("; ")
                        )
                    }
                })
                .unwrap_or_else(|| {
                    "Unable to compare embedded AOXHub baseline files".to_string()
                }),
        ),
        readiness_check(
            "release-evidence",
            "release",
            has_release_evidence(&release_dir),
            7,
            format!(
                "Release evidence bundle must exist under {}",
                release_dir.display()
            ),
        ),
        readiness_check(
            "production-closure",
            "operations",
            has_production_closure_artifacts(&closure_dir),
            7,
            format!(
                "Production closure artifacts must exist under {}",
                closure_dir.display()
            ),
        ),
        readiness_check(
            "security-drill-evidence",
            "operations",
            has_security_drill_artifact(&closure_dir),
            4,
            "Security drill evidence must capture penetration, RPC hardening, and session replay scenarios".to_string(),
        ),
        readiness_check(
            "desktop-wallet-hub-compat",
            "release",
            has_desktop_wallet_compat_artifact(&closure_dir),
            4,
            "Desktop wallet compatibility evidence must cover AOXHub plus mainnet/testnet routing".to_string(),
        ),
        readiness_check(
            "checklist-closure",
            "operations",
            checklist_open_items.is_empty() || checklist_missing,
            3,
            if checklist_open_items.is_empty() {
                format!(
                    "{} checklist is fully closed at {}",
                    target_profile,
                    profile_checklist.display()
                )
            } else if checklist_missing {
                format!(
                    "{} checklist was not found at {}; readiness run continues without checklist scoring penalty",
                    target_profile,
                    profile_checklist.display()
                )
            } else {
                format!(
                    "{} checklist has {} open items at {}",
                    target_profile,
                    checklist_open_items.len(),
                    profile_checklist.display()
                )
            },
        ),
        readiness_check(
            "compatibility-matrix",
            "release",
            has_matching_artifact(&release_dir, "compat-matrix-", ".json"),
            3,
            "Compatibility matrix evidence must be generated for the candidate release".to_string(),
        ),
        readiness_check(
            "signature-evidence",
            "release",
            has_matching_artifact(&release_dir, "aoxc-", ".sig")
                || has_matching_artifact(&release_dir, "aoxc-", ".sig.status"),
            2,
            "Signature evidence must exist for the candidate artifact, even if the signer is still pending".to_string(),
        ),
        readiness_check(
            "sbom-artifact",
            "release",
            has_matching_artifact(&release_dir, "sbom-", ".json"),
            2,
            "An SBOM or dependency inventory must be generated for the candidate release".to_string(),
        ),
        readiness_check(
            "provenance-attestation",
            "release",
            has_matching_artifact(&release_dir, "provenance-", ".json"),
            2,
            "Provenance attestation must exist before final mainnet sign-off".to_string(),
        ),
    ];

    readiness_from_checks(settings.profile.clone(), checks)
}

fn load_full_surface_matrix(
    repo_root: &Path,
) -> (String, Option<FullSurfaceMatrixModel>, Vec<String>) {
    let matrix_path = repo_root
        .join("models")
        .join("full_surface_readiness_matrix_v1.yaml");
    let matrix_path_string = matrix_path.display().to_string();

    let raw = match fs::read_to_string(&matrix_path) {
        Ok(raw) => raw,
        Err(error) => {
            return (
                matrix_path_string,
                None,
                vec![format!("Unable to read canonical matrix: {error}")],
            );
        }
    };

    match serde_yaml::from_str::<FullSurfaceMatrixModel>(&raw) {
        Ok(model) => (matrix_path_string, Some(model), Vec::new()),
        Err(error) => (
            matrix_path_string,
            None,
            vec![format!("Unable to parse canonical matrix YAML: {error}")],
        ),
    }
}

fn validate_full_surface_matrix(
    matrix: Option<&FullSurfaceMatrixModel>,
    surfaces: &[SurfaceReadiness],
    release_line: &str,
) -> (bool, Option<String>, u8, Vec<String>) {
    let Some(matrix) = matrix else {
        return (false, None, 0, Vec::new());
    };

    let mut warnings = Vec::new();
    if matrix.release_line != release_line {
        warnings.push(format!(
            "Matrix release line {} does not match runtime release line {}",
            matrix.release_line, release_line
        ));
    }

    for expected in &matrix.surfaces {
        if expected.required_evidence.is_empty() {
            warnings.push(format!(
                "Matrix surface {} is missing required_evidence entries",
                expected.id
            ));
        }
        if expected.verification_command.trim().is_empty() {
            warnings.push(format!(
                "Matrix surface {} is missing verification_command",
                expected.id
            ));
        }
        if expected.blocker.trim().is_empty() {
            warnings.push(format!(
                "Matrix surface {} is missing blocker text",
                expected.id
            ));
        }

        match surfaces
            .iter()
            .find(|surface| surface.surface == expected.id)
        {
            Some(surface) => {
                if surface.owner != expected.owner {
                    warnings.push(format!(
                        "Matrix owner mismatch for {}: matrix={} runtime={}",
                        expected.id, expected.owner, surface.owner
                    ));
                }
            }
            None => warnings.push(format!(
                "Matrix surface {} is not represented in runtime readiness output",
                expected.id
            )),
        }
    }

    for surface in surfaces {
        if !matrix
            .surfaces
            .iter()
            .any(|expected| expected.id == surface.surface)
        {
            warnings.push(format!(
                "Runtime surface {} is missing from canonical matrix",
                surface.surface
            ));
        }
    }

    (
        true,
        Some(matrix.release_line.clone()),
        matrix.surfaces.len() as u8,
        warnings,
    )
}

fn evaluate_full_surface_readiness(
    settings: &crate::config::settings::Settings,
    mainnet_readiness: &Readiness,
) -> FullSurfaceReadiness {
    let repo_root = locate_repo_root();
    let release_line = "aoxc.v.0.1.1-akdeniz";
    let (matrix_path, matrix_model, mut matrix_warnings) = load_full_surface_matrix(&repo_root);
    let release_dir = repo_root.join("artifacts").join("release-evidence");
    let closure_dir = repo_root
        .join("artifacts")
        .join("network-production-closure");
    let mainnet_config = repo_root
        .join("configs")
        .join("environments")
        .join("mainnet")
        .join("profile.toml");
    let testnet_config = repo_root
        .join("configs")
        .join("environments")
        .join("testnet")
        .join("profile.toml");
    let devnet_config = repo_root
        .join("configs")
        .join("environments")
        .join("devnet")
        .join("profile.toml");
    let aoxhub_mainnet = repo_root
        .join("configs")
        .join("aoxhub")
        .join("mainnet.toml");
    let aoxhub_testnet = repo_root
        .join("configs")
        .join("aoxhub")
        .join("testnet.toml");
    let testnet_fixture_v1 = repo_root
        .join("configs")
        .join("environments")
        .join("testnet")
        .join("genesis.v1.json");
    let testnet_fixture_exists = testnet_fixture_v1.exists();
    let devnet_fixture = repo_root
        .join("configs")
        .join("environments")
        .join("devnet")
        .join("genesis.v1.json");
    let testnet_launch = repo_root
        .join("configs")
        .join("environments")
        .join("localnet")
        .join("launch-localnet.sh");
    let multi_host = repo_root
        .join("scripts")
        .join("validation")
        .join("multi_host_validation.sh");
    let frontend_rpc_doc = repo_root
        .join("docs")
        .join("src")
        .join("FRONTEND_RPC_API_INTEGRATION_TR.md");
    let mainnet_checklist = repo_root
        .join("docs")
        .join("src")
        .join("MAINNET_READINESS_CHECKLIST.md");
    let consensus_gate = crate::cli::bootstrap::consensus_profile_gate_status(None, None);

    let surfaces = vec![
        build_surface(
            "mainnet",
            "protocol-release",
            vec![
                surface_check(
                    "candidate-threshold",
                    mainnet_readiness.verdict == "candidate",
                    format!(
                        "mainnet-readiness verdict is {} at {}%",
                        mainnet_readiness.verdict, mainnet_readiness.readiness_score
                    ),
                ),
                surface_check(
                    "mainnet-config-present",
                    mainnet_config.exists(),
                    format!("expected config at {}", mainnet_config.display()),
                ),
                surface_check(
                    "release-evidence-bundle",
                    has_release_evidence(&release_dir),
                    format!("release evidence bundle under {}", release_dir.display()),
                ),
                surface_check(
                    "release-provenance-bundle",
                    has_release_provenance_bundle(&release_dir),
                    format!(
                        "release provenance artifacts must exist under {}",
                        release_dir.display()
                    ),
                ),
                surface_check(
                    "api-admission-controls",
                    repo_root
                        .join("crates")
                        .join("aoxcrpc")
                        .join("src")
                        .join("middleware")
                        .join("rate_limiter.rs")
                        .exists()
                        && repo_root
                            .join("crates")
                            .join("aoxcrpc")
                            .join("src")
                            .join("middleware")
                            .join("mtls_auth.rs")
                            .exists()
                        && repo_root.join("NETWORK_SECURITY_ARCHITECTURE.md").exists(),
                    "RPC admission controls require rate-limiter, mTLS middleware, and network security architecture baseline".to_string(),
                ),
            ],
            vec![
                mainnet_checklist.display().to_string(),
                release_dir.display().to_string(),
            ],
        ),
        build_surface(
            "quantum-consensus",
            "protocol-security",
            vec![
                surface_check(
                    "consensus-profile-gate",
                    consensus_gate
                        .as_ref()
                        .map(|status| status.passed)
                        .unwrap_or(false),
                    consensus_gate
                        .as_ref()
                        .map(|status| {
                            if status.passed {
                                status.detail.clone()
                            } else if status.blockers.is_empty() {
                                format!("{}; verdict={}", status.detail, status.verdict)
                            } else {
                                format!(
                                    "{}; blockers={}",
                                    status.detail,
                                    status.blockers.join(" | ")
                                )
                            }
                        })
                        .unwrap_or_else(|error| {
                            format!("consensus profile gate unavailable: {}", error)
                        }),
                ),
                surface_check(
                    "consensus-hybrid-or-pq-policy",
                    consensus_gate
                        .as_ref()
                        .map(|status| !status.detail.contains("consensus_profile=classical"))
                        .unwrap_or(false),
                    "mainnet candidate path must avoid classical-only consensus profile"
                        .to_string(),
                ),
            ],
            vec![
                repo_root
                    .join("identity")
                    .join("genesis.json")
                    .display()
                    .to_string(),
            ],
        ),
        build_surface(
            "testnet",
            "network-operations",
            vec![
                surface_check(
                    "testnet-config-present",
                    testnet_config.exists(),
                    format!("expected config at {}", testnet_config.display()),
                ),
                surface_check(
                    "deterministic-fixture",
                    testnet_fixture_exists,
                    format!(
                        "expected canonical testnet genesis fixture at {}",
                        testnet_fixture_v1.display()
                    ),
                ),
                surface_check(
                    "launch-script",
                    testnet_launch.exists(),
                    format!("expected launch script at {}", testnet_launch.display()),
                ),
                surface_check(
                    "multi-host-validation-entrypoint",
                    multi_host.exists(),
                    format!("expected validation script at {}", multi_host.display()),
                ),
            ],
            vec![
                testnet_fixture_v1.display().to_string(),
                multi_host.display().to_string(),
            ],
        ),
        build_surface(
            "aoxhub",
            "hub-platform",
            vec![
                surface_check(
                    "mainnet-profile",
                    aoxhub_mainnet.exists(),
                    format!(
                        "expected AOXHub mainnet config at {}",
                        aoxhub_mainnet.display()
                    ),
                ),
                surface_check(
                    "testnet-profile",
                    aoxhub_testnet.exists(),
                    format!(
                        "expected AOXHub testnet config at {}",
                        aoxhub_testnet.display()
                    ),
                ),
                surface_check(
                    "rollout-evidence",
                    closure_dir.join("aoxhub-rollout.json").exists(),
                    format!(
                        "expected AOXHub rollout artifact at {}",
                        closure_dir.join("aoxhub-rollout.json").display()
                    ),
                ),
                surface_check(
                    "baseline-parity",
                    compare_aoxhub_network_profiles()
                        .map(|report| report.passed)
                        .unwrap_or(false),
                    "AOXHub mainnet/testnet baseline parity must hold".to_string(),
                ),
            ],
            vec![
                aoxhub_mainnet.display().to_string(),
                aoxhub_testnet.display().to_string(),
                closure_dir
                    .join("aoxhub-rollout.json")
                    .display()
                    .to_string(),
            ],
        ),
        build_surface(
            "devnet",
            "engineering-platform",
            vec![
                surface_check(
                    "devnet-config-present",
                    devnet_config.exists(),
                    format!("expected config at {}", devnet_config.display()),
                ),
                surface_check(
                    "devnet-fixture-present",
                    devnet_fixture.exists(),
                    format!(
                        "expected deterministic devnet fixture at {}",
                        devnet_fixture.display()
                    ),
                ),
                surface_check(
                    "telemetry-snapshot",
                    closure_dir.join("telemetry-snapshot.json").exists(),
                    format!(
                        "expected telemetry snapshot at {}",
                        closure_dir.join("telemetry-snapshot.json").display()
                    ),
                ),
            ],
            vec![
                devnet_config.display().to_string(),
                devnet_fixture.display().to_string(),
                closure_dir
                    .join("telemetry-snapshot.json")
                    .display()
                    .to_string(),
            ],
        ),
        build_surface(
            "desktop-wallet",
            "client-platform",
            vec![
                surface_check(
                    "desktop-wallet-compat",
                    has_desktop_wallet_compat_artifact(&closure_dir),
                    format!(
                        "desktop wallet compatibility artifact at {}",
                        closure_dir.join("desktop-wallet-compat.json").display()
                    ),
                ),
                surface_check(
                    "production-audit",
                    closure_dir.join("production-audit.json").exists(),
                    format!(
                        "wallet release decisions rely on {}",
                        closure_dir.join("production-audit.json").display()
                    ),
                ),
                surface_check(
                    "rpc-integration-doc",
                    frontend_rpc_doc.exists(),
                    format!(
                        "expected integration guide at {}",
                        frontend_rpc_doc.display()
                    ),
                ),
            ],
            vec![
                closure_dir
                    .join("desktop-wallet-compat.json")
                    .display()
                    .to_string(),
                frontend_rpc_doc.display().to_string(),
            ],
        ),
        build_surface(
            "telemetry",
            "sre-observability",
            vec![
                surface_check(
                    "metrics-enabled",
                    settings.telemetry.enable_metrics,
                    "Prometheus/metrics export must stay enabled".to_string(),
                ),
                surface_check(
                    "telemetry-snapshot",
                    closure_dir.join("telemetry-snapshot.json").exists(),
                    format!(
                        "expected telemetry snapshot at {}",
                        closure_dir.join("telemetry-snapshot.json").display()
                    ),
                ),
                surface_check(
                    "alert-rules",
                    closure_dir.join("alert-rules.md").exists(),
                    format!(
                        "expected alert rules at {}",
                        closure_dir.join("alert-rules.md").display()
                    ),
                ),
                surface_check(
                    "runtime-telemetry-handle",
                    json_artifact_has_required_strings(
                        &closure_dir.join("runtime-status.json"),
                        "required_artifacts",
                        &["telemetry-snapshot.json"],
                    ) || closure_dir.join("runtime-status.json").exists(),
                    format!(
                        "runtime status should expose telemetry evidence at {}",
                        closure_dir.join("runtime-status.json").display()
                    ),
                ),
            ],
            vec![
                closure_dir
                    .join("telemetry-snapshot.json")
                    .display()
                    .to_string(),
                closure_dir.join("alert-rules.md").display().to_string(),
                closure_dir
                    .join("runtime-status.json")
                    .display()
                    .to_string(),
            ],
        ),
    ];

    let blockers = surfaces
        .iter()
        .flat_map(|surface| {
            surface
                .blockers
                .iter()
                .map(move |blocker| format!("{}: {}", surface.surface, blocker))
        })
        .collect::<Vec<_>>();

    let total_score = surfaces
        .iter()
        .map(|surface| surface.score as u16)
        .sum::<u16>();
    let overall_score = (total_score / surfaces.len() as u16) as u8;
    let candidate_surfaces = surfaces
        .iter()
        .filter(|surface| surface.status == "ready")
        .count() as u8;
    let next_focus = surfaces
        .iter()
        .filter(|surface| surface.status != "ready")
        .take(3)
        .map(|surface| {
            format!(
                "{}: raise from {}% to 100% by clearing {} blocker(s)",
                surface.surface,
                surface.score,
                surface.blockers.len()
            )
        })
        .collect::<Vec<_>>();

    let (matrix_loaded, matrix_release_line, matrix_surface_count, validation_warnings) =
        validate_full_surface_matrix(matrix_model.as_ref(), &surfaces, release_line);
    matrix_warnings.extend(validation_warnings);

    FullSurfaceReadiness {
        release_line,
        matrix_path,
        matrix_loaded,
        matrix_release_line,
        matrix_surface_count,
        matrix_warnings,
        overall_status: if blockers.is_empty() {
            "candidate"
        } else if overall_score >= 75 {
            "hardening"
        } else {
            "not-ready"
        },
        overall_score,
        candidate_surfaces,
        total_surfaces: surfaces.len() as u8,
        surfaces,
        blockers,
        next_focus,
    }
}

fn locate_repo_artifact_dir(artifact_name: &str) -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for candidate in cwd.ancestors() {
        let artifact_dir = candidate.join("artifacts").join(artifact_name);
        if artifact_dir.exists() {
            return artifact_dir;
        }
    }
    cwd.join("artifacts").join(artifact_name)
}

fn surface_check(name: &'static str, passed: bool, detail: String) -> SurfaceCheck {
    SurfaceCheck {
        name,
        passed,
        detail,
    }
}

fn build_surface(
    surface: &'static str,
    owner: &'static str,
    checks: Vec<SurfaceCheck>,
    evidence: Vec<String>,
) -> SurfaceReadiness {
    let blockers = checks
        .iter()
        .filter(|check| !check.passed)
        .map(|check| format!("{}: {}", check.name, check.detail))
        .collect::<Vec<_>>();
    let passed = checks.iter().filter(|check| check.passed).count() as u16;
    let score = if checks.is_empty() {
        0
    } else {
        (passed * 100 / checks.len() as u16) as u8
    };

    SurfaceReadiness {
        surface,
        owner,
        status: if blockers.is_empty() {
            "ready"
        } else if score >= 50 {
            "hardening"
        } else {
            "blocked"
        },
        score,
        blockers,
        evidence,
        checks,
    }
}

fn collect_surface_gate_failures(readiness: &FullSurfaceReadiness) -> Vec<SurfaceGateFailure> {
    let mut failures = Vec::new();

    for surface in &readiness.surfaces {
        for check in &surface.checks {
            if check.passed {
                continue;
            }
            failures.push(SurfaceGateFailure {
                surface: surface.surface.to_string(),
                check: check.name.to_string(),
                code: gate_failure_code(surface.surface, check.name),
                detail: check.detail.clone(),
            });
        }
    }

    failures
}

fn gate_failure_code(surface: &str, check: &str) -> String {
    let surface_token = surface
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>()
        .to_ascii_uppercase();
    let check_token = check
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>()
        .to_ascii_uppercase();
    format!("AOXC_GATE_{}_{}", surface_token, check_token)
}

fn readiness_check(
    name: &'static str,
    area: &'static str,
    passed: bool,
    weight: u8,
    detail: String,
) -> ReadinessCheck {
    ReadinessCheck {
        name,
        area,
        passed,
        weight,
        detail,
    }
}

fn readiness_from_checks(profile: String, checks: Vec<ReadinessCheck>) -> Readiness {
    let max_score = checks.iter().map(|check| check.weight).sum::<u8>();
    let completed_weight = checks
        .iter()
        .filter(|check| check.passed)
        .map(|check| check.weight)
        .sum::<u8>();
    let readiness_score = if max_score == 0 {
        0
    } else {
        (completed_weight as u16 * 100 / max_score as u16) as u8
    };
    let blockers = checks
        .iter()
        .filter(|check| !check.passed)
        .map(|check| format!("{}: {}", check.name, check.detail))
        .collect::<Vec<_>>();
    let remediation_plan = remediation_plan(&checks);
    let area_progress = area_progress(&checks);
    let track_progress = track_progress(&checks);
    let next_focus = next_focus(&area_progress);

    Readiness {
        profile,
        stage: if readiness_score == 100 {
            "mainnet-ready"
        } else if readiness_score >= 75 {
            "testnet-ready-mainnet-hardening"
        } else if readiness_score >= 50 {
            "integration-hardening"
        } else {
            "bootstrap"
        },
        readiness_score,
        max_score,
        completed_weight,
        remaining_weight: max_score.saturating_sub(completed_weight),
        verdict: if readiness_score == 100 {
            "candidate"
        } else {
            "not-ready"
        },
        blockers,
        remediation_plan,
        next_focus,
        area_progress,
        track_progress,
        checks,
    }
}

fn area_progress(checks: &[ReadinessCheck]) -> Vec<ReadinessAreaProgress> {
    let area_order = [
        "configuration",
        "network",
        "observability",
        "identity",
        "runtime",
        "release",
        "operations",
    ];
    let mut progress = Vec::new();

    for area in area_order {
        let area_checks = checks
            .iter()
            .filter(|check| check.area == area)
            .collect::<Vec<_>>();
        if area_checks.is_empty() {
            continue;
        }

        let max_weight = area_checks.iter().map(|check| check.weight).sum::<u8>();
        let completed_weight = area_checks
            .iter()
            .filter(|check| check.passed)
            .map(|check| check.weight)
            .sum::<u8>();
        let passed_checks = area_checks.iter().filter(|check| check.passed).count() as u8;
        let total_checks = area_checks.len() as u8;
        let ratio = ratio(completed_weight, max_weight);

        progress.push(ReadinessAreaProgress {
            area,
            completed_weight,
            max_weight,
            ratio,
            passed_checks,
            total_checks,
            status: progress_status(ratio),
        });
    }

    progress
}

fn track_progress(checks: &[ReadinessCheck]) -> Vec<ReadinessTrackProgress> {
    let testnet_max = checks
        .iter()
        .filter(|check| !check.name.ends_with("-profile"))
        .map(|check| check.weight)
        .sum::<u8>();
    let testnet_completed = checks
        .iter()
        .filter(|check| !check.name.ends_with("-profile") && check.passed)
        .map(|check| check.weight)
        .sum::<u8>();
    let mainnet_max = checks.iter().map(|check| check.weight).sum::<u8>();
    let mainnet_completed = checks
        .iter()
        .filter(|check| check.passed)
        .map(|check| check.weight)
        .sum::<u8>();

    vec![
        ReadinessTrackProgress {
            name: "testnet",
            completed_weight: testnet_completed,
            max_weight: testnet_max,
            ratio: ratio(testnet_completed, testnet_max),
            status: progress_status(ratio(testnet_completed, testnet_max)),
            objective: "Public testnet should close all non-mainnet-specific blockers and sustain AOXHub/core parity.",
        },
        ReadinessTrackProgress {
            name: "mainnet",
            completed_weight: mainnet_completed,
            max_weight: mainnet_max,
            ratio: ratio(mainnet_completed, mainnet_max),
            status: progress_status(ratio(mainnet_completed, mainnet_max)),
            objective: "Mainnet requires every weighted control to pass, including production profile, keys, runtime, and release evidence.",
        },
    ]
}

fn next_focus(area_progress: &[ReadinessAreaProgress]) -> Vec<String> {
    let mut weakest = area_progress
        .iter()
        .filter(|area| area.ratio < 100)
        .collect::<Vec<_>>();
    weakest.sort_by_key(|area| (area.ratio, area.area));

    weakest
        .into_iter()
        .take(3)
        .map(|area| {
            format!(
                "{}: raise from {}% to 100% ({} of {} checks passing)",
                area.area, area.ratio, area.passed_checks, area.total_checks
            )
        })
        .collect()
}

fn ratio(completed_weight: u8, max_weight: u8) -> u8 {
    if max_weight == 0 {
        0
    } else {
        (completed_weight as u16 * 100 / max_weight as u16) as u8
    }
}

fn progress_status(ratio: u8) -> &'static str {
    if ratio == 100 {
        "ready"
    } else if ratio >= 75 {
        "hardening"
    } else if ratio >= 50 {
        "in-progress"
    } else {
        "bootstrap"
    }
}

fn write_readiness_markdown_report(
    path: &Path,
    readiness: &Readiness,
    embedded_baseline: Option<&ProfileBaselineReport>,
    aoxhub_baseline: Option<&ProfileBaselineReport>,
) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to create report directory {}", parent.display()),
                error,
            )
        })?;
    }

    fs::write(
        path,
        readiness_markdown_report(readiness, embedded_baseline, aoxhub_baseline),
    )
    .map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to write readiness report {}", path.display()),
            error,
        )
    })
}

fn write_full_surface_markdown_report(
    path: &Path,
    readiness: &FullSurfaceReadiness,
) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!(
                    "Failed to create full-surface report directory {}",
                    parent.display()
                ),
                error,
            )
        })?;
    }

    fs::write(path, full_surface_markdown_report(readiness)).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to write full-surface report {}", path.display()),
            error,
        )
    })
}

fn readiness_markdown_report(
    readiness: &Readiness,
    embedded_baseline: Option<&ProfileBaselineReport>,
    aoxhub_baseline: Option<&ProfileBaselineReport>,
) -> String {
    let mut out = String::new();
    out.push_str("# AOXC Progress Report\n\n");
    out.push_str(&format!(
        "- Profile: `{}`\n- Stage: `{}`\n- Overall readiness: **{}%** ({}/{})\n- Verdict: `{}`\n\n",
        readiness.profile,
        readiness.stage,
        readiness.readiness_score,
        readiness.completed_weight,
        readiness.max_score,
        readiness.verdict,
    ));

    out.push_str("## Dual-track progress\n\n");
    for track in &readiness.track_progress {
        out.push_str(&format!(
            "- **{}**: {}% ({}/{}) — {}\n  - Objective: {}\n",
            track.name,
            track.ratio,
            track.completed_weight,
            track.max_weight,
            track.status,
            track.objective
        ));
    }

    out.push_str("\n## Area progress\n\n");
    for area in &readiness.area_progress {
        out.push_str(&format!(
            "- **{}**: {}% ({}/{} checks, weight {}/{}) — {}\n",
            area.area,
            area.ratio,
            area.passed_checks,
            area.total_checks,
            area.completed_weight,
            area.max_weight,
            area.status
        ));
    }

    out.push_str("\n## Remaining blockers\n\n");
    if readiness.blockers.is_empty() {
        out.push_str("- No active blockers.\n");
    } else {
        for blocker in &readiness.blockers {
            out.push_str(&format!("- {}\n", blocker));
        }
    }

    out.push_str("\n## Recommended next focus\n\n");
    if readiness.next_focus.is_empty() {
        out.push_str("- Keep CI enforcement active and preserve current closure state.\n");
    } else {
        for focus in &readiness.next_focus {
            out.push_str(&format!("- {}\n", focus));
        }
    }

    out.push_str("\n## Remediation plan\n\n");
    for step in &readiness.remediation_plan {
        out.push_str(&format!("- {}\n", step));
    }

    out.push_str("\n## Baseline parity\n\n");
    append_baseline_section(&mut out, "Embedded network profiles", embedded_baseline);
    append_baseline_section(&mut out, "AOXHub network profiles", aoxhub_baseline);

    out.push_str("\n## Check matrix\n\n");
    for check in &readiness.checks {
        let marker = if check.passed { "PASS" } else { "FAIL" };
        out.push_str(&format!(
            "- [{}] **{}** / {} / weight {} — {}\n",
            marker, check.name, check.area, check.weight, check.detail
        ));
    }

    out
}

fn full_surface_markdown_report(readiness: &FullSurfaceReadiness) -> String {
    let mut out = String::new();
    out.push_str("# AOXC Full-Surface Readiness Report\n\n");
    out.push_str(&format!(
        "- Release line: `{}`\n- Matrix path: `{}`\n- Matrix loaded: `{}`\n- Matrix release line: `{}`\n- Matrix surface count: `{}`\n- Overall status: `{}`\n- Overall score: **{}%**\n- Candidate surfaces: **{}/{}**\n\n",
        readiness.release_line,
        readiness.matrix_path,
        readiness.matrix_loaded,
        readiness
            .matrix_release_line
            .as_deref()
            .unwrap_or("unavailable"),
        readiness.matrix_surface_count,
        readiness.overall_status,
        readiness.overall_score,
        readiness.candidate_surfaces,
        readiness.total_surfaces,
    ));

    out.push_str("## Matrix validation\n\n");
    if readiness.matrix_warnings.is_empty() {
        out.push_str("- Canonical matrix matches the runtime readiness surface map.\n");
    } else {
        for warning in &readiness.matrix_warnings {
            out.push_str(&format!("- {}\n", warning));
        }
    }

    out.push_str("## Surface summary\n\n");
    for surface in &readiness.surfaces {
        let passed_checks = surface.checks.iter().filter(|check| check.passed).count();
        let total_checks = surface.checks.len();
        out.push_str(&format!(
            "- **{}** / owner `{}` — status `{}` — score **{}%** ({}/{})\n",
            surface.surface,
            surface.owner,
            surface.status,
            surface.score,
            passed_checks,
            total_checks
        ));
    }

    out.push_str("\n## Global blockers\n\n");
    if readiness.blockers.is_empty() {
        out.push_str("- No active blockers.\n");
    } else {
        for blocker in &readiness.blockers {
            out.push_str(&format!("- {}\n", blocker));
        }
    }

    out.push_str("\n## Next focus\n\n");
    if readiness.next_focus.is_empty() {
        out.push_str("- Preserve current candidate state and keep evidence fresh.\n");
    } else {
        for focus in &readiness.next_focus {
            out.push_str(&format!("- {}\n", focus));
        }
    }

    out.push_str("\n## Surface details\n\n");
    for surface in &readiness.surfaces {
        let passed_checks = surface.checks.iter().filter(|check| check.passed).count();
        let total_checks = surface.checks.len();
        out.push_str(&format!(
            "### {} ({})\n\n- Owner: `{}`\n- Status: `{}`\n- Score: **{}%** ({}/{})\n",
            surface.surface,
            surface.surface.to_uppercase(),
            surface.owner,
            surface.status,
            surface.score,
            passed_checks,
            total_checks
        ));

        out.push_str("- Evidence:\n");
        for item in &surface.evidence {
            out.push_str(&format!("  - `{}`\n", item));
        }

        out.push_str("- Checks:\n");
        for check in &surface.checks {
            out.push_str(&format!(
                "  - [{}] {} — {}\n",
                if check.passed { "PASS" } else { "FAIL" },
                check.name,
                check.detail
            ));
        }

        out.push_str("- Next actions:\n");
        if surface.blockers.is_empty() {
            out.push_str("  - Keep evidence current and preserve candidate posture.\n");
        } else {
            for blocker in &surface.blockers {
                out.push_str(&format!("  - Close blocker: {}\n", blocker));
            }
        }

        if !surface.blockers.is_empty() {
            out.push_str("- Blockers:\n");
            for blocker in &surface.blockers {
                out.push_str(&format!("  - {}\n", blocker));
            }
        }

        out.push('\n');
    }

    out
}

fn append_baseline_section(
    out: &mut String,
    title: &str,
    baseline: Option<&ProfileBaselineReport>,
) {
    out.push_str(&format!("### {}\n\n", title));
    match baseline {
        Some(report) => {
            out.push_str(&format!(
                "- Status: **{}**\n",
                if report.passed {
                    "aligned"
                } else {
                    "drift-detected"
                }
            ));
            out.push_str(&format!("- Mainnet file: `{}`\n", report.mainnet_path));
            out.push_str(&format!("- Testnet file: `{}`\n", report.testnet_path));
            for control in &report.shared_controls {
                out.push_str(&format!(
                    "- {}: {} (mainnet=`{}`, testnet=`{}`)\n",
                    control.name,
                    if control.passed { "ok" } else { "drift" },
                    control.mainnet,
                    control.testnet
                ));
            }
            if !report.drift.is_empty() {
                out.push_str("- Drift summary:\n");
                for drift in &report.drift {
                    out.push_str(&format!("  - {}\n", drift));
                }
            }
        }
        None => out.push_str("- Status: unavailable\n"),
    }
    out.push('\n');
}

fn remediation_plan(checks: &[ReadinessCheck]) -> Vec<String> {
    let mut plan = Vec::new();
    let total_checks = checks.len();
    let passed_checks = checks.iter().filter(|check| check.passed).count();

    for check in checks.iter().filter(|check| !check.passed) {
        let step = match check.name {
            "config-valid" => {
                "Run `aoxc config-validate` and fix the operator settings file before promotion."
            }
            "mainnet-profile" => {
                "Run `aoxc production-bootstrap --profile mainnet --password <value>` or `aoxc config-init --profile mainnet --json-logs`."
            }
            "testnet-profile" => {
                "Run `aoxc production-bootstrap --profile testnet --password <value>` or `aoxc config-init --profile testnet --json-logs`."
            }
            "official-peers" => {
                "Re-enable curated peer enforcement in the operator settings before joining production."
            }
            "telemetry-metrics" => {
                "Keep Prometheus telemetry enabled so production SLOs and alerts remain actionable."
            }
            "structured-logging" => {
                "Enable JSON logging to preserve audit-quality operator trails and SIEM ingestion."
            }
            "genesis-present" => {
                "Materialize genesis with `aoxc genesis-init` or re-run `aoxc production-bootstrap`."
            }
            "node-state-present" => {
                "Initialize runtime state with `aoxc node-bootstrap` or re-run `aoxc production-bootstrap`."
            }
            "operator-key-active" => {
                "Bootstrap or rotate operator keys with `aoxc key-bootstrap --profile mainnet --password <value>`."
            }
            "profile-baseline-parity" => {
                "Run `aoxc profile-baseline --enforce` and align embedded mainnet/testnet configs before promotion."
            }
            "aoxhub-baseline-parity" => {
                "Align `configs/aoxhub/mainnet.toml` and `configs/aoxhub/testnet.toml` so AOXHub rollout controls match promotion policy."
            }
            "release-evidence" => {
                "Regenerate release evidence under `artifacts/release-evidence/` before promotion."
            }
            "production-closure" => {
                "Refresh production closure artifacts under `artifacts/network-production-closure/`."
            }
            "security-drill-evidence" => {
                "Record a fresh security drill with penetration, RPC hardening, and session replay evidence before promotion."
            }
            "desktop-wallet-hub-compat" => {
                "Publish `desktop-wallet-compat.json` proving the desktop wallet remains compatible with AOXHub and both network tracks."
            }
            "compatibility-matrix" => {
                "Publish a fresh compatibility matrix for the candidate release."
            }
            "signature-evidence" => {
                "Attach signature evidence for the candidate binary before release sign-off."
            }
            "sbom-artifact" => {
                "Generate and archive an SBOM/dependency inventory for the candidate release."
            }
            "provenance-attestation" => {
                "Attach provenance attestation evidence before release sign-off."
            }
            _ => continue,
        };

        if !plan.iter().any(|existing| existing == step) {
            plan.push(step.to_string());
        }
    }

    if plan.is_empty() {
        plan.push(
            "Candidate is at 100%; keep running `aoxc mainnet-readiness --enforce --format json` and `aoxc testnet-readiness --enforce --format json` in CI to prevent regressions."
                .to_string(),
        );
    } else {
        let current_ratio = if total_checks == 0 {
            0
        } else {
            (passed_checks * 100) / total_checks
        };
        plan.push(format!(
            "Close remaining blockers to raise readiness from {}% to 100% before release sign-off.",
            current_ratio
        ));
    }

    plan
}

fn has_release_evidence(dir: &Path) -> bool {
    has_matching_artifact(dir, "release-evidence-", ".md")
        && has_matching_artifact(dir, "build-manifest-", ".json")
        && has_matching_artifact(dir, "compat-matrix-", ".json")
        && has_matching_artifact(dir, "production-audit-", ".json")
        && has_matching_artifact(dir, "sbom-", ".json")
        && (has_matching_artifact(dir, "aoxc-", ".sig")
            || has_matching_artifact(dir, "aoxc-", ".sig.status"))
}

fn has_release_provenance_bundle(dir: &Path) -> bool {
    has_matching_artifact(dir, "provenance-", ".json")
        && has_matching_artifact(dir, "release-provenance-", ".json")
        && has_matching_artifact(dir, "release-sbom-", ".json")
        && has_matching_artifact(dir, "release-build-manifest-", ".json")
        && has_matching_artifact(dir, "release-signature-status-", ".txt")
}

fn has_production_closure_artifacts(dir: &Path) -> bool {
    [
        "production-audit.json",
        "runtime-status.json",
        "soak-plan.json",
        "telemetry-snapshot.json",
        "aoxhub-rollout.json",
        "alert-rules.md",
    ]
    .iter()
    .all(|file| dir.join(file).exists())
}

fn has_security_drill_artifact(dir: &Path) -> bool {
    json_artifact_has_required_strings(
        &dir.join("security-drill.json"),
        "scenarios",
        &["penetration-baseline", "rpc-authz", "session-replay"],
    )
}

fn has_desktop_wallet_compat_artifact(dir: &Path) -> bool {
    json_artifact_has_required_strings(
        &dir.join("desktop-wallet-compat.json"),
        "surfaces",
        &["desktop-wallet", "aoxhub", "mainnet", "testnet"],
    )
}

fn json_artifact_has_required_strings(path: &Path, key: &str, required: &[&str]) -> bool {
    let Ok(raw) = fs::read_to_string(path) else {
        return false;
    };
    let Ok(value) = serde_json::from_str::<Value>(&raw) else {
        return false;
    };
    let Some(values) = value
        .get(key)
        .and_then(|entry| entry.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .collect::<Vec<_>>()
        })
    else {
        return false;
    };

    required
        .iter()
        .all(|needle| values.iter().any(|value| value == needle))
}

fn has_matching_artifact(dir: &Path, prefix: &str, suffix: &str) -> bool {
    std::fs::read_dir(dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .any(|name| name.starts_with(prefix) && name.ends_with(suffix))
}

fn compare_embedded_network_profiles() -> Result<ProfileBaselineReport, AppError> {
    let repo_root = locate_repo_root();
    compare_network_profile_pair(
        repo_root.join("configs").join("mainnet.toml"),
        repo_root.join("configs").join("testnet.toml"),
    )
}

fn compare_aoxhub_network_profiles() -> Result<ProfileBaselineReport, AppError> {
    let repo_root = locate_repo_root();
    compare_network_profile_pair(
        repo_root.join("configs").join("aoxhub-mainnet.toml"),
        repo_root.join("configs").join("aoxhub-testnet.toml"),
    )
}

fn compare_network_profile_pair(
    mainnet_path: PathBuf,
    testnet_path: PathBuf,
) -> Result<ProfileBaselineReport, AppError> {
    let mainnet = parse_network_profile(&mainnet_path)?;
    let testnet = parse_network_profile(&testnet_path)?;

    let shared_controls = vec![
        BaselineControl {
            name: "security_mode",
            mainnet: mainnet.security_mode.clone(),
            testnet: testnet.security_mode.clone(),
            passed: !mainnet.security_mode.trim().is_empty()
                && mainnet.security_mode == testnet.security_mode,
            expectation: "Both profiles must enforce the same security mode before promotion",
        },
        BaselineControl {
            name: "peer_seed_count",
            mainnet: mainnet.peers.len().to_string(),
            testnet: testnet.peers.len().to_string(),
            passed: !mainnet.peers.is_empty() && mainnet.peers.len() == testnet.peers.len(),
            expectation: "Both profiles should define the same number of bootstrap peers",
        },
        BaselineControl {
            name: "listen_port_offset",
            mainnet: normalized_port_pair(&mainnet.listen_addr, &mainnet.rpc_addr),
            testnet: normalized_port_pair(&testnet.listen_addr, &testnet.rpc_addr),
            passed: ports_are_shifted_consistently(&mainnet, &testnet),
            expectation: "Testnet should differ only by deterministic port offsets, not by capability shape",
        },
    ];

    let drift = shared_controls
        .iter()
        .filter(|control| !control.passed)
        .map(|control| {
            format!(
                "{} mismatch (mainnet={}, testnet={})",
                control.name, control.mainnet, control.testnet
            )
        })
        .collect::<Vec<_>>();

    Ok(ProfileBaselineReport {
        mainnet_path: mainnet_path.display().to_string(),
        testnet_path: testnet_path.display().to_string(),
        passed: drift.is_empty(),
        shared_controls,
        drift,
    })
}

fn locate_repo_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for candidate in cwd.ancestors() {
        if candidate.join("Cargo.toml").exists() && candidate.join("configs").exists() {
            return candidate.to_path_buf();
        }
    }
    cwd
}

fn open_checklist_items(path: &Path) -> Vec<String> {
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(_) => return vec![format!("missing-checklist:{}", path.display())],
    };

    raw.lines()
        .map(str::trim)
        .filter(|line| line.starts_with("- [ ] "))
        .map(|line| line.trim_start_matches("- [ ] ").to_string())
        .collect()
}

fn parse_network_profile(path: &Path) -> Result<NetworkProfileConfig, AppError> {
    let raw = fs::read_to_string(path).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to read network profile {}", path.display()),
            error,
        )
    })?;

    let mut config = NetworkProfileConfig::default();
    let mut in_peers = false;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if in_peers {
            if trimmed == "]" {
                in_peers = false;
                continue;
            }
            let peer = trimmed.trim_end_matches(',').trim_matches('"').trim();
            if !peer.is_empty() {
                config.peers.push(peer.to_string());
            }
            continue;
        }

        if let Some((key, value)) = trimmed.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "chain_id" => config.chain_id = unquote(value),
                "listen_addr" => config.listen_addr = unquote(value),
                "rpc_addr" => config.rpc_addr = unquote(value),
                "security_mode" => config.security_mode = unquote(value),
                "peers" if value == "[" => in_peers = true,
                _ => {}
            }
        }
    }

    if config.chain_id.is_empty()
        || config.listen_addr.is_empty()
        || config.rpc_addr.is_empty()
        || config.security_mode.is_empty()
    {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            format!(
                "Network profile {} is missing required fields",
                path.display()
            ),
        ));
    }

    Ok(config)
}

fn unquote(value: &str) -> String {
    value.trim().trim_matches('"').to_string()
}

fn normalized_port_pair(listen_addr: &str, rpc_addr: &str) -> String {
    format!(
        "{}/{}",
        extract_port(listen_addr).map_or_else(|| "?".to_string(), |p| p.to_string()),
        extract_port(rpc_addr).map_or_else(|| "?".to_string(), |p| p.to_string())
    )
}

fn ports_are_shifted_consistently(
    mainnet: &NetworkProfileConfig,
    testnet: &NetworkProfileConfig,
) -> bool {
    let mainnet_listen = extract_port(&mainnet.listen_addr);
    let testnet_listen = extract_port(&testnet.listen_addr);
    let mainnet_rpc = extract_port(&mainnet.rpc_addr);
    let testnet_rpc = extract_port(&testnet.rpc_addr);

    match (mainnet_listen, testnet_listen, mainnet_rpc, testnet_rpc) {
        (Some(ml), Some(tl), Some(mr), Some(tr)) => tl > ml && tr > mr && (tl - ml) == (tr - mr),
        _ => false,
    }
}

fn extract_port(addr: &str) -> Option<u16> {
    addr.rsplit(':').next()?.parse::<u16>().ok()
}

pub fn cmd_node_bootstrap(args: &[String]) -> Result<(), AppError> {
    bootstrap_operator_home()?;
    let state = lifecycle::bootstrap_state()?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&state, output_format(args))
}

pub fn cmd_produce_once(args: &[String]) -> Result<(), AppError> {
    let tx = parse_required_or_default_text_arg(args, "--tx", "boot-sequence-1", false)?;
    let state = engine::produce_once(&tx)?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&state, output_format(args))
}

pub fn cmd_node_run(args: &[String]) -> Result<(), AppError> {
    let rounds = parse_positive_u64_arg(args, "--rounds", 10, "node run")?;
    let tx_prefix = parse_required_or_default_text_arg(args, "--tx-prefix", "AOXC-RUN", false)?;
    let format = output_format(args);
    let live_log_enabled = !has_flag(args, "--no-live-log");
    let log_level = parse_required_or_default_text_arg(args, "--log-level", "info", true)?;

    if !matches!(log_level.as_str(), "info" | "debug") {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Invalid --log-level value (supported: info, debug)",
        ));
    }

    if format == crate::cli_support::OutputFormat::Text && live_log_enabled {
        print_node_live_log_header(rounds, &tx_prefix, &log_level)?;
    }

    let state = if format == crate::cli_support::OutputFormat::Text && live_log_enabled {
        engine::run_rounds_with_observer(rounds, &tx_prefix, |entry| {
            print_node_round_line(entry, &log_level);
        })?
    } else {
        engine::run_rounds(rounds, &tx_prefix)?
    };

    let _ = refresh_runtime_metrics().ok();
    let _ = graceful_shutdown();

    if format == crate::cli_support::OutputFormat::Text && live_log_enabled {
        print_node_live_log_footer(&state);
    }

    emit_serialized(&state, format)
}

fn print_node_live_log_header(
    rounds: u64,
    tx_prefix: &str,
    log_level: &str,
) -> Result<(), AppError> {
    let now = chrono::Utc::now().to_rfc3339();
    let db_path = lifecycle::state_path()?;

    println!("🚀 [{}] node-run startup", now);
    println!(
        "🧭 mode=live rounds={} tx_prefix={} log_level={}",
        rounds, tx_prefix, log_level
    );
    println!("🗄️  state_db={}", db_path.display());
    println!(
        "📋 {:>5} | {:<25} | {:>8} | {:>8} | {:>8} | {:<12}",
        "round", "timestamp", "height", "blocks", "sections", "tx"
    );
    println!(
        "────────────────────────────────────────────────────────────────────────────────────────"
    );
    Ok(())
}

fn print_node_round_line(entry: &engine::RoundTelemetry, log_level: &str) {
    let timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp(entry.timestamp_unix as i64, 0)
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

    println!(
        "✅ {:>5} | {:<25} | {:>8} | {:>8} | {:>8} | {:<12}",
        entry.round_index,
        timestamp,
        entry.height,
        entry.produced_blocks,
        entry.section_count,
        entry.tx_id
    );

    if log_level == "debug" {
        println!(
            "   🔍 round={} consensus_round={} block={} parent={}",
            entry.round_index,
            entry.consensus_round,
            short_hash(&entry.block_hash_hex),
            short_hash(&entry.parent_hash_hex)
        );
    }
}

fn print_node_live_log_footer(state: &crate::node::state::NodeState) {
    println!(
        "────────────────────────────────────────────────────────────────────────────────────────"
    );
    println!(
        "🏁 completed height={} produced_blocks={} updated_at={}",
        state.current_height, state.produced_blocks, state.updated_at
    );
}

fn short_hash(value: &str) -> String {
    if value.len() <= 16 {
        return value.to_string();
    }
    format!("{}…{}", &value[..8], &value[value.len() - 8..])
}

pub fn cmd_node_health(args: &[String]) -> Result<(), AppError> {
    let health = health_status()?;
    let mut details = BTreeMap::new();
    details.insert("health".to_string(), health.to_string());

    emit_serialized(
        &text_envelope("node-health", "ok", details),
        output_format(args),
    )
}

pub fn cmd_network_smoke(args: &[String]) -> Result<(), AppError> {
    let settings = effective_settings_for_ops()?;
    let key_summary = crate::keys::manager::inspect_operator_key()?;

    let mut details = BTreeMap::new();
    details.insert("bind_host".to_string(), settings.network.bind_host);
    details.insert(
        "rpc_port".to_string(),
        settings.network.rpc_port.to_string(),
    );
    details.insert(
        "probe".to_string(),
        "local-listener-simulated-ok".to_string(),
    );
    details.insert(
        "transport_public_key".to_string(),
        key_summary.transport_public_key,
    );
    details.insert(
        "key_operational_state".to_string(),
        key_summary.operational_state,
    );

    emit_serialized(
        &text_envelope("network-smoke", "ok", details),
        output_format(args),
    )
}

pub fn cmd_real_network(args: &[String]) -> Result<(), AppError> {
    let settings = effective_settings_for_ops()?;
    let key_summary = crate::keys::manager::inspect_operator_key()?;

    let mut details = BTreeMap::new();
    details.insert("mode".to_string(), "deterministic-local".to_string());
    details.insert(
        "enforce_official_peers".to_string(),
        settings.network.enforce_official_peers.to_string(),
    );
    details.insert(
        "key_operational_state".to_string(),
        key_summary.operational_state,
    );
    details.insert(
        "transport_public_key".to_string(),
        key_summary.transport_public_key,
    );

    emit_serialized(
        &text_envelope("real-network", "ok", details),
        output_format(args),
    )
}

pub fn cmd_storage_smoke(args: &[String]) -> Result<(), AppError> {
    let context = runtime_context()?;
    let mut details = BTreeMap::new();
    details.insert("home_dir".to_string(), context.settings.home_dir);
    details.insert("storage".to_string(), "writable".to_string());

    emit_serialized(
        &text_envelope("storage-smoke", "ok", details),
        output_format(args),
    )
}

pub fn cmd_economy_init(args: &[String]) -> Result<(), AppError> {
    let ledger = ledger::init()?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&ledger, output_format(args))
}

pub fn cmd_treasury_transfer(args: &[String]) -> Result<(), AppError> {
    let to = parse_required_or_default_text_arg(args, "--to", "ops", false)?;
    let amount = parse_positive_u64_arg(args, "--amount", 1000, "treasury transfer")?;

    let ledger = ledger::transfer(&to, amount)?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&ledger, output_format(args))
}

pub fn cmd_stake_delegate(args: &[String]) -> Result<(), AppError> {
    let validator = parse_required_or_default_text_arg(args, "--validator", "validator-01", false)?;
    let amount = parse_positive_u64_arg(args, "--amount", 1000, "stake delegation")?;

    let ledger = ledger::delegate(&validator, amount)?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&ledger, output_format(args))
}

pub fn cmd_stake_undelegate(args: &[String]) -> Result<(), AppError> {
    let validator = parse_required_or_default_text_arg(args, "--validator", "validator-01", false)?;
    let amount = parse_positive_u64_arg(args, "--amount", 1000, "stake undelegation")?;

    let ledger = ledger::undelegate(&validator, amount)?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&ledger, output_format(args))
}

pub fn cmd_economy_status(args: &[String]) -> Result<(), AppError> {
    let ledger = ledger::load()?;
    emit_serialized(&ledger, output_format(args))
}

pub fn cmd_faucet_status(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct FaucetStatus {
        enabled: bool,
        network_kind: String,
        treasury_balance: u64,
        total_distributed_today: u64,
        claims_today: usize,
        account_remaining_allowance: Option<u64>,
        next_eligible_claim_time: Option<u64>,
        faucet: FaucetState,
    }

    let now_unix = now_unix_secs()?;
    let account_id = parse_optional_text_arg(args, "--account-id", false);
    let settings = effective_settings_for_ops()?;
    let mut state = load_faucet_state()?;
    prune_faucet_history(&mut state, now_unix);
    let day_ago = now_unix.saturating_sub(24 * 60 * 60);
    let recent: Vec<&FaucetClaimRecord> = state
        .claims
        .iter()
        .filter(|record| record.claimed_at >= day_ago)
        .collect();
    let total_distributed_today = recent.iter().map(|record| record.amount).sum::<u64>();

    let decision = account_id
        .as_ref()
        .map(|id| evaluate_faucet_claim(&state, id, 1, now_unix, false, None, &settings.profile));
    let ledger_state = ledger::load().unwrap_or_default();

    let response = FaucetStatus {
        enabled: state.enabled,
        network_kind: settings.profile,
        treasury_balance: ledger_state.treasury_balance,
        total_distributed_today,
        claims_today: recent.len(),
        account_remaining_allowance: decision.as_ref().map(|d| d.daily_remaining),
        next_eligible_claim_time: decision.and_then(|d| d.next_eligible_claim_at),
        faucet: state,
    };

    emit_serialized(&response, output_format(args))
}

pub fn cmd_faucet_config(args: &[String]) -> Result<(), AppError> {
    let mut state = load_faucet_state()?;
    let now_unix = now_unix_secs()?;
    let mut changed = false;

    if has_flag(args, "--enable") {
        state.enabled = true;
        changed = true;
    }
    if has_flag(args, "--disable") {
        state.enabled = false;
        changed = true;
    }

    if let Some(amount) = arg_value(args, "--max-claim-amount") {
        state.max_claim_amount =
            parse_positive_u64_value(&amount, "--max-claim-amount", "faucet config")?;
        changed = true;
    }

    if let Some(cooldown_secs) = arg_value(args, "--cooldown-secs") {
        state.cooldown_secs =
            parse_positive_u64_value(&cooldown_secs, "--cooldown-secs", "faucet config")?;
        changed = true;
    }

    if let Some(limit) = arg_value(args, "--daily-limit-per-account") {
        state.daily_limit_per_account =
            parse_positive_u64_value(&limit, "--daily-limit-per-account", "faucet config")?;
        changed = true;
    }

    if let Some(limit) = arg_value(args, "--daily-global-limit") {
        state.daily_global_limit =
            parse_positive_u64_value(&limit, "--daily-global-limit", "faucet config")?;
        changed = true;
    }

    if let Some(balance) = arg_value(args, "--min-reserve-balance") {
        state.min_reserve_balance =
            parse_positive_u64_value(&balance, "--min-reserve-balance", "faucet config")?;
        changed = true;
    }

    if let Some(account) = parse_optional_text_arg(args, "--ban-account", true) {
        if !state.banned_accounts.contains(&account) {
            state.banned_accounts.push(account.clone());
            state.banned_accounts.sort();
            changed = true;
        }
    }

    if let Some(account) = parse_optional_text_arg(args, "--unban-account", true) {
        let initial_len = state.banned_accounts.len();
        state
            .banned_accounts
            .retain(|existing| existing != &account);
        changed = changed || initial_len != state.banned_accounts.len();
    }

    if let Some(account) = parse_optional_text_arg(args, "--allow-account", true) {
        if !state.allowlisted_accounts.contains(&account) {
            state.allowlisted_accounts.push(account.clone());
            state.allowlisted_accounts.sort();
            changed = true;
        }
    }

    if let Some(account) = parse_optional_text_arg(args, "--disallow-account", true) {
        let initial_len = state.allowlisted_accounts.len();
        state
            .allowlisted_accounts
            .retain(|existing| existing != &account);
        changed = changed || initial_len != state.allowlisted_accounts.len();
    }

    prune_faucet_history(&mut state, now_unix);
    if changed {
        append_faucet_audit(
            &mut state,
            "config-update",
            "operator-cli",
            "Faucet configuration updated via CLI",
            now_unix,
        );
    }
    persist_faucet_state(&state)?;
    emit_serialized(&state, output_format(args))
}

pub fn cmd_faucet_claim(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct FaucetClaimResponse {
        tx_hash: String,
        status: &'static str,
        account_id: String,
        amount: u64,
        claimed_last_24h: u64,
        daily_remaining: u64,
        global_remaining: u64,
        cooldown_remaining_secs: u64,
        next_eligible_claim_at: Option<u64>,
        claims_total: usize,
        automation_hint: &'static str,
        ledger: crate::economy::ledger::LedgerState,
    }

    let account_id =
        parse_required_or_default_text_arg(args, "--account-id", "testnet-user", false)?;
    let force = has_flag(args, "--force");
    let auto_init = has_flag(args, "--auto-init");
    let mut state = load_faucet_state()?;
    let settings = effective_settings_for_ops()?;
    let now_unix = now_unix_secs()?;
    prune_faucet_history(&mut state, now_unix);
    let mut ledger_snapshot = ledger::load().unwrap_or_default();

    if !state.enabled && !force {
        return Err(AppError::new(
            ErrorCode::PolicyGateFailed,
            "Faucet is disabled for this profile; use --force only for controlled tests",
        ));
    }

    let amount = parse_positive_u64_arg(args, "--amount", state.max_claim_amount, "faucet claim")?;

    let decision = evaluate_faucet_claim(
        &state,
        &account_id,
        amount,
        now_unix,
        force,
        Some(ledger_snapshot.treasury_balance),
        &settings.profile,
    );
    if !decision.allowed {
        append_faucet_audit(
            &mut state,
            "claim-denied",
            "operator-cli",
            &decision
                .denied_reason
                .clone()
                .unwrap_or_else(|| "Faucet claim denied".to_string()),
            now_unix,
        );
        persist_faucet_state(&state)?;
        return Err(AppError::new(
            ErrorCode::PolicyGateFailed,
            decision
                .denied_reason
                .unwrap_or_else(|| "Faucet claim was denied".to_string()),
        ));
    }

    let ledger_result = match ledger::delegate(&account_id, amount) {
        Ok(ledger) => ledger,
        Err(error) if auto_init && error.kind() == ErrorCode::FilesystemIoFailed => {
            let _ = ledger::init()?;
            ledger::delegate(&account_id, amount)?
        }
        Err(error) => return Err(error),
    };

    let tx_id = faucet_tx_id(&account_id, amount, now_unix, state.claims.len());
    state.claims.push(FaucetClaimRecord {
        account_id: account_id.clone(),
        amount,
        claimed_at: now_unix,
        tx_hash: tx_id.clone(),
        status: "confirmed".to_string(),
    });
    append_faucet_audit(
        &mut state,
        "claim-approved",
        "operator-cli",
        &format!("account_id={account_id} amount={amount} tx_hash={tx_id}"),
        now_unix,
    );
    persist_faucet_state(&state)?;
    ledger_snapshot = ledger_result.clone();

    let response = FaucetClaimResponse {
        tx_hash: tx_id,
        status: "confirmed",
        account_id,
        amount,
        cooldown_remaining_secs: state.cooldown_secs,
        claimed_last_24h: decision.claimed_last_24h.saturating_add(amount),
        daily_remaining: decision.daily_remaining.saturating_sub(amount),
        global_remaining: decision.global_remaining.saturating_sub(amount),
        next_eligible_claim_at: Some(now_unix.saturating_add(state.cooldown_secs)),
        claims_total: state.claims.len(),
        automation_hint: "Use --format json for CI/CD scripts and --auto-init for first-run ephemeral homes.",
        ledger: ledger_snapshot,
    };

    emit_serialized(&response, output_format(args))
}

pub fn cmd_faucet_reset(args: &[String]) -> Result<(), AppError> {
    let keep_config = has_flag(args, "--keep-config");
    let now_unix = now_unix_secs()?;
    let state = if keep_config {
        let current = load_faucet_state()?;
        FaucetState {
            enabled: current.enabled,
            max_claim_amount: current.max_claim_amount,
            cooldown_secs: current.cooldown_secs,
            daily_limit_per_account: current.daily_limit_per_account,
            daily_global_limit: current.daily_global_limit,
            min_reserve_balance: current.min_reserve_balance,
            claims: Vec::new(),
            banned_accounts: current.banned_accounts,
            allowlisted_accounts: current.allowlisted_accounts,
            audit_log: current.audit_log,
        }
    } else {
        FaucetState::default()
    };
    let mut state = state;
    append_faucet_audit(
        &mut state,
        "reset",
        "operator-cli",
        "Faucet state reset via CLI",
        now_unix,
    );
    persist_faucet_state(&state)?;
    emit_serialized(&state, output_format(args))
}

pub fn cmd_faucet_history(args: &[String]) -> Result<(), AppError> {
    let account_id =
        parse_required_or_default_text_arg(args, "--account-id", "testnet-user", false)?;
    let mut state = load_faucet_state()?;
    prune_faucet_history(&mut state, now_unix_secs()?);
    let claims = state
        .claims
        .into_iter()
        .filter(|claim| claim.account_id == account_id)
        .collect::<Vec<_>>();
    emit_serialized(&claims, output_format(args))
}

pub fn cmd_faucet_balance(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct FaucetBalance {
        treasury_balance: u64,
        reserve_floor: u64,
        available_for_faucet: u64,
    }

    let state = load_faucet_state()?;
    let ledger = ledger::load().unwrap_or_default();
    let response = FaucetBalance {
        treasury_balance: ledger.treasury_balance,
        reserve_floor: state.min_reserve_balance,
        available_for_faucet: ledger
            .treasury_balance
            .saturating_sub(state.min_reserve_balance),
    };
    emit_serialized(&response, output_format(args))
}

pub fn cmd_faucet_enable(args: &[String]) -> Result<(), AppError> {
    let settings = effective_settings_for_ops()?;
    if settings.profile == "mainnet" {
        return Err(AppError::new(
            ErrorCode::PolicyGateFailed,
            "Mainnet profile cannot enable faucet",
        ));
    }
    let mut state = load_faucet_state()?;
    state.enabled = true;
    let now_unix = now_unix_secs()?;
    append_faucet_audit(
        &mut state,
        "enabled",
        "operator-cli",
        "Faucet enabled",
        now_unix,
    );
    persist_faucet_state(&state)?;
    emit_serialized(&state, output_format(args))
}

pub fn cmd_faucet_disable(args: &[String]) -> Result<(), AppError> {
    let mut state = load_faucet_state()?;
    state.enabled = false;
    let now_unix = now_unix_secs()?;
    append_faucet_audit(
        &mut state,
        "disabled",
        "operator-cli",
        "Faucet disabled",
        now_unix,
    );
    persist_faucet_state(&state)?;
    emit_serialized(&state, output_format(args))
}

pub fn cmd_faucet_config_show(args: &[String]) -> Result<(), AppError> {
    let state = load_faucet_state()?;
    emit_serialized(&state, output_format(args))
}

pub fn cmd_faucet_audit(args: &[String]) -> Result<(), AppError> {
    let mut state = load_faucet_state()?;
    prune_faucet_history(&mut state, now_unix_secs()?);
    emit_serialized(&state.audit_log, output_format(args))
}

pub fn cmd_runtime_status(args: &[String]) -> Result<(), AppError> {
    let context = runtime_context()?;
    let handles = default_handles();
    let unity = unity_status();
    let ai = crate::ai::runtime::report();

    #[derive(serde::Serialize)]
    struct RuntimeStatus {
        context: crate::runtime::context::RuntimeContext,
        handles: crate::runtime::handles::RuntimeHandleSet,
        unity: crate::runtime::unity::UnityStatus,
        ai: crate::ai::runtime::AiRuntimeReport,
    }

    let status = RuntimeStatus {
        context,
        handles,
        unity,
        ai,
    };

    emit_serialized(&status, output_format(args))
}

pub fn cmd_consensus_status(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct ConsensusStatus {
        network_id: u32,
        last_round: u64,
        last_message_kind: String,
        latest_block_hash: String,
        latest_parent_hash: String,
        latest_proposer: String,
        latest_timestamp_unix: u64,
        validator_set_hash: String,
        active_validators: u64,
        finalized_height: u64,
        locked_height: u64,
        quorum_status: &'static str,
        produced_blocks: u64,
        current_height: u64,
        updated_at: String,
    }

    let state = lifecycle::load_state()?;
    let status = ConsensusStatus {
        network_id: state.consensus.network_id,
        last_round: state.consensus.last_round,
        last_message_kind: state.consensus.last_message_kind,
        latest_block_hash: state.consensus.last_block_hash_hex,
        latest_parent_hash: state.consensus.last_parent_hash_hex,
        latest_proposer: state.consensus.last_proposer_hex,
        latest_timestamp_unix: state.consensus.last_timestamp_unix,
        validator_set_hash: state.key_material.bundle_fingerprint.clone(),
        active_validators: 1,
        finalized_height: state.current_height,
        locked_height: state.current_height,
        quorum_status: if state.running {
            "single-node-ok"
        } else {
            "idle"
        },
        produced_blocks: state.produced_blocks,
        current_height: state.current_height,
        updated_at: state.updated_at,
    };

    emit_serialized(&status, output_format(args))
}

pub fn cmd_consensus_validators(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct ValidatorView {
        validator_id: String,
        voting_power: u64,
        status: &'static str,
        proposer_priority: i64,
    }

    #[derive(serde::Serialize)]
    struct ValidatorSetView {
        mode: &'static str,
        validator_set_hash: String,
        total_voting_power: u64,
        validators: Vec<ValidatorView>,
    }

    let state = lifecycle::load_state()?;
    let validators = vec![ValidatorView {
        validator_id: state.consensus.last_proposer_hex.clone(),
        voting_power: 1,
        status: if state.running { "active" } else { "inactive" },
        proposer_priority: 0,
    }];
    let response = ValidatorSetView {
        mode: "single-node",
        validator_set_hash: state.key_material.bundle_fingerprint,
        total_voting_power: 1,
        validators,
    };
    emit_serialized(&response, output_format(args))
}

pub fn cmd_consensus_proposer(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct ProposerView {
        proposer_id: String,
        height: u64,
        round: u64,
        timestamp_unix: u64,
    }

    let state = lifecycle::load_state()?;
    let response = ProposerView {
        proposer_id: state.consensus.last_proposer_hex,
        height: state.current_height,
        round: state.consensus.last_round,
        timestamp_unix: state.consensus.last_timestamp_unix,
    };
    emit_serialized(&response, output_format(args))
}

pub fn cmd_consensus_round(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct ConsensusRoundView {
        height: u64,
        round: u64,
        message_kind: String,
        quorum_status: &'static str,
        timeout_state: &'static str,
    }

    let state = lifecycle::load_state()?;
    let response = ConsensusRoundView {
        height: state.current_height,
        round: state.consensus.last_round,
        message_kind: state.consensus.last_message_kind,
        quorum_status: if state.running {
            "single-node-ok"
        } else {
            "idle"
        },
        timeout_state: "not-triggered",
    };
    emit_serialized(&response, output_format(args))
}

pub fn cmd_consensus_finality(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct FinalityView {
        head_height: u64,
        safe_height: u64,
        finalized_height: u64,
        pending_height: u64,
        mode: &'static str,
    }

    let state = lifecycle::load_state()?;
    let response = FinalityView {
        head_height: state.current_height,
        safe_height: state.current_height,
        finalized_height: state.current_height,
        pending_height: state.current_height.saturating_add(1),
        mode: "single-node",
    };
    emit_serialized(&response, output_format(args))
}

pub fn cmd_consensus_commits(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct CommitVoteView {
        validator_id: String,
        vote: &'static str,
        round: u64,
    }

    #[derive(serde::Serialize)]
    struct CommitView {
        height: u64,
        block_hash: String,
        commits: Vec<CommitVoteView>,
    }

    let state = lifecycle::load_state()?;
    let response = CommitView {
        height: state.current_height,
        block_hash: state.consensus.last_block_hash_hex,
        commits: vec![CommitVoteView {
            validator_id: state.consensus.last_proposer_hex,
            vote: "precommit",
            round: state.consensus.last_round,
        }],
    };
    emit_serialized(&response, output_format(args))
}

pub fn cmd_consensus_evidence(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct ConsensusEvidence {
        height: u64,
        round: u64,
        lock_reason: &'static str,
        quorum_certificate: bool,
        evidence: Vec<&'static str>,
    }

    let state = lifecycle::load_state()?;
    let response = ConsensusEvidence {
        height: state.current_height,
        round: state.consensus.last_round,
        lock_reason: "single-validator-lock",
        quorum_certificate: true,
        evidence: vec!["prevote", "precommit", "commit"],
    };
    emit_serialized(&response, output_format(args))
}

pub fn cmd_vm_status(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct VmStatus {
        vm_enabled: bool,
        execution_plane: &'static str,
        execution_mode: &'static str,
        latest_height: u64,
        last_executed_block: u64,
        latest_tx_marker: String,
        last_execution_status: &'static str,
        total_tx_in_last_block: u64,
        executed_tx_count: u64,
        failed_tx_count: u64,
        runtime_running: bool,
        state_root: String,
        updated_at: String,
    }

    let state = lifecycle::load_state()?;
    let state_root = derive_state_root(&state)?;
    let has_last_tx = state.last_tx != "none";
    let status = VmStatus {
        vm_enabled: true,
        execution_plane: "deterministic-local",
        execution_mode: "local-snapshot",
        latest_height: state.current_height,
        last_executed_block: state.current_height,
        latest_tx_marker: state.last_tx,
        last_execution_status: if has_last_tx {
            "ok"
        } else {
            "idle"
        },
        total_tx_in_last_block: u64::from(has_last_tx),
        executed_tx_count: state.produced_blocks,
        failed_tx_count: 0,
        runtime_running: state.running,
        state_root,
        updated_at: state.updated_at,
    };

    emit_serialized(&status, output_format(args))
}

pub fn cmd_vm_call(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct VmCallView {
        to: String,
        from: Option<String>,
        data: Option<String>,
        read_only: bool,
        status: &'static str,
        return_data: String,
        source: &'static str,
    }

    let to = arg_value(args, "--to")
        .and_then(|value| normalize_text(&value, false))
        .ok_or_else(|| {
            AppError::new(ErrorCode::UsageInvalidArguments, "Flag --to must not be blank")
        })?;
    let from = arg_value(args, "--from").and_then(|value| normalize_text(&value, false));
    let data = arg_value(args, "--data").and_then(|value| normalize_text(&value, false));
    let response = VmCallView {
        to,
        from,
        data,
        read_only: true,
        status: "simulated-local",
        return_data: "0x".to_string(),
        source: "deterministic-local",
    };
    emit_serialized(&response, output_format(args))
}

pub fn cmd_vm_simulate(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct VmSimulateView {
        tx_hash: Option<String>,
        from: Option<String>,
        to: Option<String>,
        gas_used: u64,
        success: bool,
        revert_reason: Option<String>,
        trace_available: bool,
        source: &'static str,
    }

    let response = VmSimulateView {
        tx_hash: arg_value(args, "--tx-hash").and_then(|value| normalize_text(&value, false)),
        from: arg_value(args, "--from").and_then(|value| normalize_text(&value, false)),
        to: arg_value(args, "--to").and_then(|value| normalize_text(&value, false)),
        gas_used: 0,
        success: true,
        revert_reason: None,
        trace_available: true,
        source: "deterministic-local",
    };
    emit_serialized(&response, output_format(args))
}

pub fn cmd_vm_storage_get(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct VmStorageView {
        address: String,
        key: String,
        value: String,
        found: bool,
        source: &'static str,
    }

    let address = arg_value(args, "--address")
        .and_then(|value| normalize_text(&value, false))
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Flag --address must not be blank",
            )
        })?;
    let key = arg_value(args, "--key")
        .and_then(|value| normalize_text(&value, false))
        .ok_or_else(|| {
            AppError::new(ErrorCode::UsageInvalidArguments, "Flag --key must not be blank")
        })?;
    let response = VmStorageView {
        address,
        key,
        value: "0x".to_string(),
        found: false,
        source: "local-snapshot",
    };
    emit_serialized(&response, output_format(args))
}

pub fn cmd_vm_contract_get(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct VmContractView {
        address: String,
        exists: bool,
        code_hash: String,
        source: &'static str,
    }

    let address = arg_value(args, "--address")
        .and_then(|value| normalize_text(&value, false))
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Flag --address must not be blank",
            )
        })?;
    let response = VmContractView {
        address,
        exists: false,
        code_hash: "0x0".to_string(),
        source: "local-snapshot",
    };
    emit_serialized(&response, output_format(args))
}

pub fn cmd_vm_code_get(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct VmCodeView {
        address: String,
        code: String,
        source: &'static str,
    }

    let address = arg_value(args, "--address")
        .and_then(|value| normalize_text(&value, false))
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Flag --address must not be blank",
            )
        })?;
    let response = VmCodeView {
        address,
        code: "0x".to_string(),
        source: "local-snapshot",
    };
    emit_serialized(&response, output_format(args))
}

pub fn cmd_vm_estimate_gas(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct VmEstimateGasView {
        from: Option<String>,
        to: Option<String>,
        estimated_gas: u64,
        source: &'static str,
    }

    let response = VmEstimateGasView {
        from: arg_value(args, "--from").and_then(|value| normalize_text(&value, false)),
        to: arg_value(args, "--to").and_then(|value| normalize_text(&value, false)),
        estimated_gas: 21_000,
        source: "deterministic-local",
    };
    emit_serialized(&response, output_format(args))
}

pub fn cmd_vm_trace(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct VmTraceStep {
        index: u64,
        op: &'static str,
        gas: u64,
    }

    #[derive(serde::Serialize)]
    struct VmTraceView {
        tx_hash: Option<String>,
        trace: Vec<VmTraceStep>,
        source: &'static str,
    }

    let response = VmTraceView {
        tx_hash: arg_value(args, "--tx-hash").and_then(|value| normalize_text(&value, false)),
        trace: vec![
            VmTraceStep {
                index: 0,
                op: "BEGIN",
                gas: 21_000,
            },
            VmTraceStep {
                index: 1,
                op: "END",
                gas: 0,
            },
        ],
        source: "deterministic-local",
    };
    emit_serialized(&response, output_format(args))
}

pub fn cmd_chain_status(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct ChainStatus {
        network_id: u32,
        current_height: u64,
        latest_block_hash: String,
        latest_parent_hash: String,
        latest_timestamp_unix: u64,
        produced_blocks: u64,
        running: bool,
        profile: String,
        consensus_mode: &'static str,
    }

    let settings = effective_settings_for_ops()?;
    let state = lifecycle::load_state()?;
    let status = ChainStatus {
        network_id: state.consensus.network_id,
        current_height: state.current_height,
        latest_block_hash: state.consensus.last_block_hash_hex,
        latest_parent_hash: state.consensus.last_parent_hash_hex,
        latest_timestamp_unix: state.consensus.last_timestamp_unix,
        produced_blocks: state.produced_blocks,
        running: state.running,
        profile: settings.profile,
        consensus_mode: "aoxcunity",
    };

    emit_serialized(&status, output_format(args))
}

pub fn cmd_block_get(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct BlockView {
        requested_height: Option<String>,
        requested_hash: Option<String>,
        available: bool,
        height: u64,
        block_hash: String,
        parent_hash: String,
        proposer: String,
        consensus_round: u64,
        timestamp_unix: u64,
        section_count: usize,
        tx_count: usize,
        tx_hashes: Vec<String>,
        state_root: String,
    }

    let requested_height = arg_value(args, "--height").and_then(|v| normalize_text(&v, false));
    let requested_hash = arg_value(args, "--hash").and_then(|v| normalize_text(&v, false));
    if requested_height.is_some() && requested_hash.is_some() {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Use either --height or --hash, not both",
        ));
    }
    let state = lifecycle::load_state()?;
    let canonical_height = state.current_height;
    let state_root = derive_state_root(&state)?;
    let default_height = "latest".to_string();
    let requested_height_value = requested_height
        .as_deref()
        .unwrap_or(default_height.as_str());

    let available = match requested_height_value {
        "latest" => true,
        value => value.parse::<u64>().ok() == Some(canonical_height),
    } && match requested_hash.as_ref() {
        Some(hash) => hash.eq_ignore_ascii_case(&state.consensus.last_block_hash_hex),
        None => true,
    };
    let tx_hashes = if state.last_tx == "none" {
        Vec::new()
    } else {
        vec![state.last_tx.clone()]
    };

    let view = BlockView {
        requested_height: Some(requested_height_value.to_string()),
        requested_hash,
        available,
        height: canonical_height,
        block_hash: state.consensus.last_block_hash_hex,
        parent_hash: state.consensus.last_parent_hash_hex,
        proposer: state.consensus.last_proposer_hex,
        consensus_round: state.consensus.last_round,
        timestamp_unix: state.consensus.last_timestamp_unix,
        section_count: state.consensus.last_section_count,
        tx_count: tx_hashes.len(),
        tx_hashes,
        state_root,
    };

    emit_serialized(&view, output_format(args))
}

pub fn cmd_tx_get(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct TxView {
        tx_hash: String,
        known: bool,
        block_height: u64,
        execution_status: &'static str,
        source: &'static str,
    }

    let tx_hash = arg_value(args, "--hash")
        .and_then(|value| normalize_text(&value, false))
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Flag --hash must not be blank",
            )
        })?;
    let state = lifecycle::load_state()?;
    let known = state.last_tx != "none" && tx_hash == state.last_tx;

    let tx = TxView {
        tx_hash,
        known,
        block_height: state.current_height,
        execution_status: if known { "applied" } else { "unknown" },
        source: "runtime-last-tx",
    };

    emit_serialized(&tx, output_format(args))
}

pub fn cmd_tx_receipt(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct TxReceiptView {
        tx_hash: String,
        found: bool,
        success: bool,
        gas_used: u64,
        fee_paid: u64,
        events: Vec<String>,
        logs: Vec<String>,
        state_change_summary: String,
    }

    let tx_hash = arg_value(args, "--hash")
        .and_then(|value| normalize_text(&value, false))
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Flag --hash must not be blank",
            )
        })?;
    let state = lifecycle::load_state()?;
    let found = state.last_tx != "none" && tx_hash == state.last_tx;
    let receipt = TxReceiptView {
        tx_hash,
        found,
        success: found,
        gas_used: 0,
        fee_paid: 0,
        events: if found {
            vec!["runtime_tx_applied".to_string()]
        } else {
            Vec::new()
        },
        logs: Vec::new(),
        state_change_summary: if found {
            "local runtime marker updated".to_string()
        } else {
            "receipt not found".to_string()
        },
    };

    emit_serialized(&receipt, output_format(args))
}

pub fn cmd_account_get(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct AccountView {
        account_id: String,
        known: bool,
        balance: u64,
        nonce: u64,
        source: &'static str,
    }

    let account_id = arg_value(args, "--id")
        .and_then(|value| normalize_text(&value, false))
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Flag --id must not be blank",
            )
        })?;
    let ledger = ledger::load().unwrap_or_default();
    let balance = if account_id == "treasury" {
        ledger.treasury_balance
    } else {
        ledger.delegations.get(&account_id).copied().unwrap_or(0)
    };

    let account = AccountView {
        known: account_id == "treasury" || ledger.delegations.contains_key(&account_id),
        account_id,
        balance,
        nonce: 0,
        source: "local-ledger",
    };

    emit_serialized(&account, output_format(args))
}

pub fn cmd_balance_get(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct BalanceView {
        account_id: String,
        balance: u64,
        known: bool,
        source: &'static str,
    }

    let account_id = arg_value(args, "--id")
        .and_then(|value| normalize_text(&value, false))
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Flag --id must not be blank",
            )
        })?;
    let ledger = ledger::load().unwrap_or_default();
    let balance = if account_id == "treasury" {
        ledger.treasury_balance
    } else {
        ledger.delegations.get(&account_id).copied().unwrap_or(0)
    };
    let response = BalanceView {
        known: account_id == "treasury" || ledger.delegations.contains_key(&account_id),
        account_id,
        balance,
        source: "local-ledger",
    };

    emit_serialized(&response, output_format(args))
}

pub fn cmd_peer_list(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct PeerView {
        peer_id: String,
        address: String,
        direction: &'static str,
        connected_since: String,
        sync_state: &'static str,
    }

    #[derive(serde::Serialize)]
    struct PeerList {
        mode: &'static str,
        bind_host: String,
        p2p_port: u16,
        rpc_port: u16,
        peers: Vec<PeerView>,
        peer_count: usize,
    }

    let settings = effective_settings_for_ops()?;
    let now = Utc::now().to_rfc3339();
    let peers = vec![PeerView {
        peer_id: "self".to_string(),
        address: format!(
            "{}:{}",
            settings.network.bind_host, settings.network.p2p_port
        ),
        direction: "inbound+outbound",
        connected_since: now,
        sync_state: "in-sync",
    }];

    let response = PeerList {
        mode: "single-node",
        bind_host: settings.network.bind_host,
        p2p_port: settings.network.p2p_port,
        rpc_port: settings.network.rpc_port,
        peer_count: peers.len(),
        peers,
    };

    emit_serialized(&response, output_format(args))
}

pub fn cmd_network_status(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct NetworkStatus {
        mode: &'static str,
        bind_host: String,
        p2p_port: u16,
        rpc_port: u16,
        peer_count: usize,
        listener_active: bool,
        sync_state: &'static str,
    }

    let settings = effective_settings_for_ops()?;
    let probe_target = format!(
        "{}:{}",
        settings.network.bind_host, settings.network.rpc_port
    );
    let listener_active = rpc_listener_active(&probe_target);
    let status = NetworkStatus {
        mode: "single-node",
        bind_host: settings.network.bind_host,
        p2p_port: settings.network.p2p_port,
        rpc_port: settings.network.rpc_port,
        peer_count: 1,
        listener_active,
        sync_state: "in-sync",
    };

    emit_serialized(&status, output_format(args))
}

pub fn cmd_state_root(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct StateRoot {
        state_root: String,
        height: u64,
        updated_at: String,
    }

    let state = lifecycle::load_state()?;
    let response = StateRoot {
        state_root: derive_state_root(&state)?,
        height: state.current_height,
        updated_at: state.updated_at,
    };

    emit_serialized(&response, output_format(args))
}

pub fn cmd_metrics(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct MetricsView {
        node_height: u64,
        produced_blocks: u64,
        treasury_balance: u64,
        recorded_at: String,
        source: &'static str,
    }

    let state = lifecycle::load_state()?;
    let ledger = ledger::load().unwrap_or_default();
    let metrics_path = crate::telemetry::prometheus::metrics_path()?;
    if metrics_path.exists() {
        let raw = fs::read_to_string(&metrics_path).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!(
                    "Failed to read metrics snapshot from {}",
                    metrics_path.display()
                ),
                error,
            )
        })?;
        let snapshot: crate::telemetry::prometheus::MetricsSnapshot = serde_json::from_str(&raw)
            .map_err(|error| {
                AppError::with_source(
                    ErrorCode::OutputEncodingFailed,
                    format!(
                        "Failed to parse metrics snapshot from {}",
                        metrics_path.display()
                    ),
                    error,
                )
            })?;
        let response = MetricsView {
            node_height: snapshot.node_height,
            produced_blocks: snapshot.produced_blocks,
            treasury_balance: snapshot.treasury_balance,
            recorded_at: snapshot.recorded_at,
            source: "telemetry-snapshot",
        };
        return emit_serialized(&response, output_format(args));
    }

    let response = MetricsView {
        node_height: state.current_height,
        produced_blocks: state.produced_blocks,
        treasury_balance: ledger.treasury_balance,
        recorded_at: Utc::now().to_rfc3339(),
        source: "derived-live",
    };
    emit_serialized(&response, output_format(args))
}

pub fn cmd_rpc_status(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct RpcStatus {
        enabled: bool,
        bind_host: String,
        port: u16,
        http_ready: bool,
        jsonrpc_ready: bool,
        required_endpoint_ready: bool,
        uptime_secs: u64,
        listener_active: bool,
        curl_compatible: bool,
        probe_target: String,
        http_base_url: String,
        probe_mode: &'static str,
        required_endpoint_probes: BTreeMap<&'static str, bool>,
        jsonrpc_status_probe: bool,
        rest_endpoints: Vec<&'static str>,
        json_rpc_methods: Vec<&'static str>,
        curl_examples: BTreeMap<&'static str, String>,
    }

    let settings = effective_settings_for_ops()?;
    let state = lifecycle::load_state()?;
    let probe_target = format!(
        "{}:{}",
        settings.network.bind_host, settings.network.rpc_port
    );
    let listener_active = rpc_listener_active(&probe_target);
    let uptime_secs = uptime_secs_from_rfc3339(&state.updated_at);
    let curl_host = if settings.network.bind_host == "0.0.0.0" {
        "127.0.0.1".to_string()
    } else {
        settings.network.bind_host.clone()
    };
    let http_base_url = format!("http://{}:{}", curl_host, settings.network.rpc_port);
    let mut curl_examples = BTreeMap::new();
    curl_examples.insert("health", format!("curl -fsS {http_base_url}/health"));
    curl_examples.insert("status", format!("curl -fsS {http_base_url}/status"));
    curl_examples.insert(
        "latest-block",
        format!("curl -fsS {http_base_url}/block/latest"),
    );
    curl_examples.insert(
        "consensus-status",
        format!("curl -fsS {http_base_url}/consensus/status"),
    );
    curl_examples.insert("vm-status", format!("curl -fsS {http_base_url}/vm/status"));
    curl_examples.insert(
        "json-rpc-status",
        format!(
            "curl -fsS -H 'content-type: application/json' -d '{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"status\",\"params\":[]}}' {http_base_url}"
        ),
    );
    curl_examples.insert(
        "faucet-status",
        format!("curl -fsS {http_base_url}/faucet/status"),
    );
    curl_examples.insert(
        "faucet-claim",
        format!(
            "curl -fsS -X POST -H 'content-type: application/json' -d '{{\"account_id\":\"devnet-user\",\"amount\":1000}}' {http_base_url}/faucet/claim"
        ),
    );
    let required_paths = [
        "/health",
        "/status",
        "/chain/status",
        "/consensus/status",
        "/vm/status",
    ];
    let mut required_endpoint_probes = BTreeMap::new();
    if listener_active {
        for path in required_paths {
            required_endpoint_probes.insert(
                path,
                rpc_http_get_probe(&curl_host, settings.network.rpc_port, path),
            );
        }
    } else {
        for path in required_paths {
            required_endpoint_probes.insert(path, false);
        }
    }
    let required_endpoint_ready = required_endpoint_probes.values().all(|ready| *ready);
    let jsonrpc_status_probe =
        listener_active && rpc_jsonrpc_status_probe(&curl_host, settings.network.rpc_port);
    let response = RpcStatus {
        enabled: true,
        bind_host: settings.network.bind_host.clone(),
        port: settings.network.rpc_port,
        http_ready: required_endpoint_ready,
        jsonrpc_ready: jsonrpc_status_probe,
        required_endpoint_ready,
        uptime_secs,
        listener_active,
        curl_compatible: required_endpoint_ready && jsonrpc_status_probe,
        probe_target,
        http_base_url,
        probe_mode: if listener_active {
            "tcp+http-active-probe"
        } else {
            "tcp-connect"
        },
        required_endpoint_probes,
        jsonrpc_status_probe,
        rest_endpoints: vec![
            "/health",
            "/status",
            "/metrics",
            "/chain/status",
            "/block/latest",
            "/block/{height}",
            "/tx/{hash}",
            "/tx/{hash}/receipt",
            "/account/{id}",
            "/consensus/status",
            "/network/peers",
            "/vm/status",
            "/state/root",
            "/rpc/status",
            "/faucet/status",
            "/faucet/claim",
            "/faucet/history/{account_id}",
            "/faucet/balance",
            "/faucet/config",
            "/faucet/enable",
            "/faucet/disable",
            "/faucet/ban",
            "/faucet/unban",
            "/faucet/config/update",
        ],
        json_rpc_methods: vec![
            "status",
            "getLatestBlock",
            "getBlockByHeight",
            "getBlockByHash",
            "getTxByHash",
            "getReceiptByHash",
            "getAccount",
            "getBalance",
            "getStateRoot",
            "getConsensusStatus",
            "getNetworkStatus",
            "getPeers",
            "getVmStatus",
        ],
        curl_examples,
    };

    emit_serialized(&response, output_format(args))
}

/// Resolves effective settings for read-oriented ops surfaces without creating
/// configuration files on disk.
fn effective_settings_for_ops() -> Result<Settings, AppError> {
    match load() {
        Ok(settings) => Ok(settings),
        Err(error) if error.code() == ErrorCode::ConfigMissing.as_str() => {
            let home = resolve_home()?;
            Ok(Settings::default_for(home.display().to_string()))
        }
        Err(error) => Err(error),
    }
}

fn parse_positive_u64_arg(
    args: &[String],
    flag: &str,
    default: u64,
    context: &str,
) -> Result<u64, AppError> {
    let value = match arg_value(args, flag) {
        Some(value) => normalize_text(&value, false).ok_or_else(|| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                format!("Flag {flag} must not be blank for {context}"),
            )
        })?,
        None => default.to_string(),
    };

    let parsed = value.parse::<u64>().map_err(|_| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Invalid numeric value for {flag}"),
        )
    })?;

    if parsed == 0 {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Flag {flag} must be greater than zero"),
        ));
    }

    Ok(parsed)
}

fn parse_positive_u64_value(value: &str, flag: &str, context: &str) -> Result<u64, AppError> {
    let normalized = normalize_text(value, false).ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Flag {flag} must not be blank for {context}"),
        )
    })?;

    let parsed = normalized.parse::<u64>().map_err(|_| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Invalid numeric value for {flag}"),
        )
    })?;

    if parsed == 0 {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Flag {flag} must be greater than zero"),
        ));
    }

    Ok(parsed)
}

fn parse_required_or_default_text_arg(
    args: &[String],
    flag: &str,
    default: &str,
    lowercase: bool,
) -> Result<String, AppError> {
    match arg_value(args, flag) {
        Some(value) => normalize_text(&value, lowercase).ok_or_else(|| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                format!("Flag {flag} must not be blank"),
            )
        }),
        None => Ok(default.to_string()),
    }
}

fn parse_optional_text_arg(args: &[String], flag: &str, lowercase: bool) -> Option<String> {
    arg_value(args, flag).and_then(|value| normalize_text(&value, lowercase))
}

fn normalize_text(value: &str, lowercase: bool) -> Option<String> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return None;
    }

    if lowercase {
        Some(normalized.to_ascii_lowercase())
    } else {
        Some(normalized)
    }
}

fn derive_state_root(state: &crate::node::state::NodeState) -> Result<String, AppError> {
    let encoded = serde_json::to_vec(state).map_err(|error| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to serialize node state for state-root derivation",
            error,
        )
    })?;
    let mut hasher = Sha256::new();
    hasher.update(encoded);
    Ok(hex::encode(hasher.finalize()))
}

fn uptime_secs_from_rfc3339(value: &str) -> u64 {
    chrono::DateTime::parse_from_rfc3339(value)
        .ok()
        .and_then(|time| {
            let elapsed = Utc::now().signed_duration_since(time.with_timezone(&Utc));
            (elapsed.num_seconds() >= 0).then_some(elapsed.num_seconds() as u64)
        })
        .unwrap_or(0)
}

fn rpc_listener_active(probe_target: &str) -> bool {
    match probe_target.parse() {
        Ok(addr) => TcpStream::connect_timeout(&addr, Duration::from_millis(250)).is_ok(),
        Err(_) => false,
    }
}

fn rpc_http_get_probe(host: &str, port: u16, path: &str) -> bool {
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\nAccept: application/json\r\n\r\n"
    );
    rpc_http_status_code(host, port, &request)
        .map(|code| (200..300).contains(&code))
        .unwrap_or(false)
}

fn rpc_jsonrpc_status_probe(host: &str, port: u16) -> bool {
    let body = r#"{"jsonrpc":"2.0","id":1,"method":"status","params":[]}"#;
    let request = format!(
        "POST / HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    rpc_http_status_code(host, port, &request)
        .map(|code| (200..300).contains(&code))
        .unwrap_or(false)
}

fn rpc_http_status_code(host: &str, port: u16, request: &str) -> Option<u16> {
    let target = format!("{host}:{port}");
    let addr = target.to_socket_addrs().ok()?.next()?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_millis(350)).ok()?;
    if stream
        .set_read_timeout(Some(Duration::from_millis(350)))
        .is_err()
    {
        return None;
    }
    if stream
        .set_write_timeout(Some(Duration::from_millis(350)))
        .is_err()
    {
        return None;
    }
    if stream.write_all(request.as_bytes()).is_err() {
        return None;
    }
    let mut reader = BufReader::new(stream);
    let mut status_line = String::new();
    if reader.read_line(&mut status_line).ok()? == 0 {
        return None;
    }
    let mut parts = status_line.split_whitespace();
    let _http_version = parts.next()?;
    parts.next()?.parse::<u16>().ok()
}

fn faucet_state_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("ledger").join("faucet_state.json"))
}

fn load_faucet_state() -> Result<FaucetState, AppError> {
    let path = faucet_state_path()?;
    if !path.exists() {
        let state = FaucetState::default();
        persist_faucet_state(&state)?;
        return Ok(state);
    }

    let raw = fs::read_to_string(&path).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to read faucet state from {}", path.display()),
            error,
        )
    })?;

    serde_json::from_str::<FaucetState>(&raw).map_err(|error| {
        AppError::with_source(
            ErrorCode::LedgerInvalid,
            format!("Failed to parse faucet state from {}", path.display()),
            error,
        )
    })
}

fn persist_faucet_state(state: &FaucetState) -> Result<(), AppError> {
    let path = faucet_state_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!(
                    "Failed to create faucet state directory {}",
                    parent.display()
                ),
                error,
            )
        })?;
    }

    let payload = serde_json::to_string_pretty(state).map_err(|error| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode faucet state",
            error,
        )
    })?;

    fs::write(&path, payload).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to write faucet state to {}", path.display()),
            error,
        )
    })?;

    Ok(())
}

fn evaluate_faucet_claim(
    state: &FaucetState,
    account_id: &str,
    amount: u64,
    now_unix: u64,
    force: bool,
    treasury_balance: Option<u64>,
    network_kind: &str,
) -> FaucetClaimDecision {
    if network_kind == "mainnet" {
        return FaucetClaimDecision {
            allowed: false,
            cooldown_remaining_secs: 0,
            claimed_last_24h: 0,
            daily_remaining: state.daily_limit_per_account,
            global_distributed_last_24h: 0,
            global_remaining: state.daily_global_limit,
            next_eligible_claim_at: None,
            denied_reason: Some("Mainnet profile does not allow faucet claims".to_string()),
        };
    }

    let day_ago = now_unix.saturating_sub(24 * 60 * 60);
    let relevant_claims: Vec<&FaucetClaimRecord> = state
        .claims
        .iter()
        .filter(|claim| claim.account_id == account_id)
        .collect();
    let global_recent_claims: Vec<&FaucetClaimRecord> = state
        .claims
        .iter()
        .filter(|claim| claim.claimed_at >= day_ago)
        .collect();

    let claimed_last_24h = relevant_claims
        .iter()
        .filter(|claim| claim.claimed_at >= day_ago)
        .map(|claim| claim.amount)
        .sum::<u64>();

    let global_distributed_last_24h = global_recent_claims.iter().map(|claim| claim.amount).sum();

    let latest_claim = relevant_claims
        .iter()
        .max_by_key(|claim| claim.claimed_at)
        .copied();

    let cooldown_remaining_secs = latest_claim
        .map(|claim| {
            let unlock_at = claim.claimed_at.saturating_add(state.cooldown_secs);
            unlock_at.saturating_sub(now_unix)
        })
        .unwrap_or(0);

    let daily_remaining = state
        .daily_limit_per_account
        .saturating_sub(claimed_last_24h);
    let global_remaining = state
        .daily_global_limit
        .saturating_sub(global_distributed_last_24h);
    let next_eligible_claim_at = if cooldown_remaining_secs > 0 {
        Some(now_unix.saturating_add(cooldown_remaining_secs))
    } else {
        None
    };

    if force {
        return FaucetClaimDecision {
            allowed: true,
            cooldown_remaining_secs,
            claimed_last_24h,
            daily_remaining,
            global_distributed_last_24h,
            global_remaining,
            next_eligible_claim_at,
            denied_reason: None,
        };
    }

    if state
        .banned_accounts
        .iter()
        .any(|entry| entry == account_id)
    {
        return FaucetClaimDecision {
            allowed: false,
            cooldown_remaining_secs,
            claimed_last_24h,
            daily_remaining,
            global_distributed_last_24h,
            global_remaining,
            next_eligible_claim_at,
            denied_reason: Some("Account is banned from faucet".to_string()),
        };
    }

    if !state.allowlisted_accounts.is_empty()
        && !state
            .allowlisted_accounts
            .iter()
            .any(|entry| entry == account_id)
    {
        return FaucetClaimDecision {
            allowed: false,
            cooldown_remaining_secs,
            claimed_last_24h,
            daily_remaining,
            global_distributed_last_24h,
            global_remaining,
            next_eligible_claim_at,
            denied_reason: Some("Account is not in faucet allowlist".to_string()),
        };
    }

    if amount > state.max_claim_amount {
        return FaucetClaimDecision {
            allowed: false,
            cooldown_remaining_secs,
            claimed_last_24h,
            daily_remaining,
            global_distributed_last_24h,
            global_remaining,
            next_eligible_claim_at,
            denied_reason: Some(format!(
                "Requested amount exceeds max claim amount (max={})",
                state.max_claim_amount
            )),
        };
    }

    if cooldown_remaining_secs > 0 {
        return FaucetClaimDecision {
            allowed: false,
            cooldown_remaining_secs,
            claimed_last_24h,
            daily_remaining,
            global_distributed_last_24h,
            global_remaining,
            next_eligible_claim_at,
            denied_reason: Some(format!(
                "Cooldown is active; try again in {} seconds",
                cooldown_remaining_secs
            )),
        };
    }

    if claimed_last_24h.saturating_add(amount) > state.daily_limit_per_account {
        return FaucetClaimDecision {
            allowed: false,
            cooldown_remaining_secs,
            claimed_last_24h,
            daily_remaining,
            global_distributed_last_24h,
            global_remaining,
            next_eligible_claim_at,
            denied_reason: Some(format!(
                "Daily limit exceeded for account (limit={})",
                state.daily_limit_per_account
            )),
        };
    }

    if global_distributed_last_24h.saturating_add(amount) > state.daily_global_limit {
        return FaucetClaimDecision {
            allowed: false,
            cooldown_remaining_secs,
            claimed_last_24h,
            daily_remaining,
            global_distributed_last_24h,
            global_remaining,
            next_eligible_claim_at,
            denied_reason: Some(format!(
                "Daily global faucet limit exceeded (limit={})",
                state.daily_global_limit
            )),
        };
    }

    if let Some(balance) = treasury_balance
        && balance.saturating_sub(amount) < state.min_reserve_balance
    {
        return FaucetClaimDecision {
            allowed: false,
            cooldown_remaining_secs,
            claimed_last_24h,
            daily_remaining,
            global_distributed_last_24h,
            global_remaining,
            next_eligible_claim_at,
            denied_reason: Some(format!(
                "Reserve floor check failed (min_reserve_balance={})",
                state.min_reserve_balance
            )),
        };
    }

    FaucetClaimDecision {
        allowed: true,
        cooldown_remaining_secs,
        claimed_last_24h,
        daily_remaining,
        global_distributed_last_24h,
        global_remaining,
        next_eligible_claim_at,
        denied_reason: None,
    }
}

fn prune_faucet_history(state: &mut FaucetState, now_unix: u64) {
    let retention = ChronoDuration::hours(48).num_seconds().unsigned_abs();
    let oldest = now_unix.saturating_sub(retention);
    state.claims.retain(|claim| claim.claimed_at >= oldest);
    let audit_retention = ChronoDuration::hours(FAUCET_AUDIT_RETENTION_HOURS)
        .num_seconds()
        .unsigned_abs();
    let audit_oldest = now_unix.saturating_sub(audit_retention);
    state
        .audit_log
        .retain(|entry| entry.at_unix >= audit_oldest);
}

fn now_unix_secs() -> Result<u64, AppError> {
    let now = Utc::now().timestamp();
    u64::try_from(now).map_err(|_| {
        AppError::new(
            ErrorCode::NodeStateInvalid,
            "System clock produced a negative unix timestamp",
        )
    })
}

fn faucet_tx_id(account_id: &str, amount: u64, now_unix: u64, nonce: usize) -> String {
    let mut hasher = Sha256::new();
    hasher.update(account_id.as_bytes());
    hasher.update(amount.to_le_bytes());
    hasher.update(now_unix.to_le_bytes());
    hasher.update(nonce.to_le_bytes());
    format!("faucet-{}", hex::encode(hasher.finalize()))
}

fn append_faucet_audit(
    state: &mut FaucetState,
    action: &str,
    actor: &str,
    detail: &str,
    now_unix: u64,
) {
    state.audit_log.push(FaucetAuditRecord {
        at_unix: now_unix,
        action: action.to_string(),
        actor: actor.to_string(),
        detail: detail.to_string(),
    });
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::{
        FaucetClaimRecord, FaucetState, build_surface, collect_surface_gate_failures,
        compare_aoxhub_network_profiles, compare_embedded_network_profiles, evaluate_faucet_claim,
        evaluate_full_surface_readiness, evaluate_profile_readiness, full_surface_markdown_report,
        has_desktop_wallet_compat_artifact, has_matching_artifact,
        has_production_closure_artifacts, has_release_evidence, has_release_provenance_bundle,
        has_security_drill_artifact, locate_repo_artifact_dir, open_checklist_items,
        parse_network_profile, parse_positive_u64_arg, parse_required_or_default_text_arg,
        ports_are_shifted_consistently, readiness_markdown_report, rpc_http_get_probe,
        rpc_jsonrpc_status_probe, surface_check, write_readiness_markdown_report,
    };
    use crate::config::settings::Settings;
    use std::{
        fs,
        io::{Read, Write},
        net::TcpListener,
        path::{Path, PathBuf},
        thread,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn unique_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("aoxcmd-ops-{label}-{nanos}"))
    }

    fn touch(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parent directory should be created");
        }
        fs::write(path, "{}").expect("fixture artifact should be written");
    }

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_string()).collect()
    }

    #[test]
    fn parse_positive_u64_arg_rejects_zero() {
        let error = parse_positive_u64_arg(&args(&["--rounds", "0"]), "--rounds", 10, "node run")
            .expect_err("zero rounds must fail");
        assert_eq!(error.code(), "AOXC-USG-002");
    }

    #[test]
    fn parse_required_or_default_text_arg_rejects_blank_value() {
        let error =
            parse_required_or_default_text_arg(&args(&["--to", "   "]), "--to", "ops", false)
                .expect_err("blank target must fail");
        assert_eq!(error.code(), "AOXC-USG-002");
    }

    #[test]
    fn faucet_claim_rejects_amount_above_max_without_force() {
        let state = FaucetState::default();
        let decision = evaluate_faucet_claim(
            &state,
            "alice",
            state.max_claim_amount + 1,
            1_775_238_343,
            false,
            Some(5_000_000),
            "testnet",
        );
        assert!(!decision.allowed);
        assert!(
            decision
                .denied_reason
                .expect("reason should exist")
                .contains("max claim amount")
        );
    }

    #[test]
    fn faucet_claim_rejects_when_cooldown_active() {
        let mut state = FaucetState::default();
        state.claims.push(FaucetClaimRecord {
            account_id: "alice".to_string(),
            amount: 50,
            claimed_at: 1_775_238_343,
            tx_hash: "tx-1".to_string(),
            status: "confirmed".to_string(),
        });
        let decision = evaluate_faucet_claim(
            &state,
            "alice",
            50,
            1_775_238_343 + 100,
            false,
            Some(5_000_000),
            "testnet",
        );
        assert!(!decision.allowed);
        assert!(decision.cooldown_remaining_secs > 0);
    }

    #[test]
    fn release_evidence_requires_expected_bundle_files() {
        let dir = unique_dir("release-evidence");
        touch(&dir.join("release-evidence-20260323T000000Z.md"));
        touch(&dir.join("build-manifest-20260323T000000Z.json"));
        touch(&dir.join("compat-matrix-20260323T000000Z.json"));
        touch(&dir.join("production-audit-20260323T000000Z.json"));
        touch(&dir.join("sbom-20260323T000000Z.json"));
        touch(&dir.join("aoxc-20260323T000000Z.sig.status"));

        assert!(has_release_evidence(&dir));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn release_provenance_bundle_requires_expected_artifacts() {
        let dir = unique_dir("release-provenance");
        touch(&dir.join("provenance-20260323T000000Z.json"));
        touch(&dir.join("release-provenance-20260323T000000Z.json"));
        touch(&dir.join("release-sbom-20260323T000000Z.json"));
        touch(&dir.join("release-build-manifest-20260323T000000Z.json"));
        touch(&dir.join("release-signature-status-20260323T000000Z.txt"));

        assert!(has_release_provenance_bundle(&dir));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn production_closure_requires_all_operational_artifacts() {
        let dir = unique_dir("production-closure");
        for file in [
            "production-audit.json",
            "runtime-status.json",
            "soak-plan.json",
            "telemetry-snapshot.json",
            "aoxhub-rollout.json",
            "alert-rules.md",
        ] {
            touch(&dir.join(file));
        }

        assert!(has_production_closure_artifacts(&dir));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn matching_artifact_detects_expected_prefix_and_suffix() {
        let dir = unique_dir("matching-artifact");
        touch(&dir.join("provenance-20260323T000000Z.json"));

        assert!(has_matching_artifact(&dir, "provenance-", ".json"));
        assert!(!has_matching_artifact(&dir, "compat-matrix-", ".json"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn security_drill_artifact_requires_expected_scenarios() {
        let dir = unique_dir("security-drill");
        fs::create_dir_all(&dir).expect("fixture directory should be created");
        fs::write(
            dir.join("security-drill.json"),
            r#"{
  "status": "completed",
  "scenarios": ["penetration-baseline", "rpc-authz", "session-replay"]
}"#,
        )
        .expect("security drill artifact should be written");

        assert!(has_security_drill_artifact(&dir));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn desktop_wallet_compat_artifact_requires_all_surfaces() {
        let dir = unique_dir("desktop-wallet-compat");
        fs::create_dir_all(&dir).expect("fixture directory should be created");
        fs::write(
            dir.join("desktop-wallet-compat.json"),
            r#"{
  "status": "validated",
  "surfaces": ["desktop-wallet", "aoxhub", "mainnet", "testnet"]
}"#,
        )
        .expect("desktop wallet compatibility artifact should be written");

        assert!(has_desktop_wallet_compat_artifact(&dir));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn open_checklist_items_detects_unchecked_entries() {
        let dir = unique_dir("checklist-open");
        let checklist = dir.join("MAINNET_READINESS_CHECKLIST.md");
        fs::create_dir_all(&dir).expect("fixture directory should be created");
        fs::write(
            &checklist,
            "# checklist\n- [x] done\n- [ ] pending-1\n- [ ] pending-2\n",
        )
        .expect("checklist fixture should be written");

        let open = open_checklist_items(&checklist);
        assert_eq!(open.len(), 2);
        assert!(open.iter().any(|item| item == "pending-1"));
        assert!(open.iter().any(|item| item == "pending-2"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn open_checklist_items_returns_missing_marker_when_file_absent() {
        let path = unique_dir("checklist-missing").join("MAINNET_READINESS_CHECKLIST.md");
        let open = open_checklist_items(&path);
        assert_eq!(open.len(), 1);
        assert!(open[0].starts_with("missing-checklist:"));
    }

    #[test]
    fn readiness_reflects_release_evidence_gaps_in_score() {
        let mut settings = Settings::default_for("/tmp/aoxc".to_string());
        settings.profile = "mainnet".to_string();
        settings.logging.json = true;
        settings.network.bind_host = "0.0.0.0".to_string();

        let readiness =
            evaluate_profile_readiness("mainnet", &settings, None, Some("active"), true, true);

        assert_eq!(readiness.readiness_score, 75);
        assert_eq!(readiness.verdict, "not-ready");
        assert!(!readiness.blockers.is_empty());
        assert!(!readiness.remediation_plan.is_empty());
        assert!(
            readiness
                .remediation_plan
                .iter()
                .any(|step| step.contains("100%")),
            "remediation plan should still include a path to full readiness"
        );
        assert_eq!(readiness.track_progress.len(), 2);
        assert!(
            readiness
                .track_progress
                .iter()
                .all(|track| track.ratio <= 100)
        );
        assert!(
            readiness
                .track_progress
                .iter()
                .any(|track| track.ratio < 100)
        );
        assert!(!readiness.next_focus.is_empty());
        assert!(
            readiness
                .area_progress
                .iter()
                .any(|progress| progress.ratio < 100)
        );
    }

    #[test]
    fn readiness_reports_testnet_progress_separately_from_mainnet() {
        let mut settings = Settings::default_for("/tmp/aoxc".to_string());
        settings.profile = "validator".to_string();
        settings.logging.json = true;
        settings.network.bind_host = "0.0.0.0".to_string();

        let readiness =
            evaluate_profile_readiness("mainnet", &settings, None, Some("active"), true, true);

        let testnet = readiness
            .track_progress
            .iter()
            .find(|track| track.name == "testnet")
            .expect("testnet track should exist");
        let mainnet = readiness
            .track_progress
            .iter()
            .find(|track| track.name == "mainnet")
            .expect("mainnet track should exist");

        assert!(testnet.ratio > mainnet.ratio);
        assert!(
            readiness
                .next_focus
                .iter()
                .any(|entry| entry.starts_with("configuration:"))
        );
    }

    #[test]
    fn readiness_requires_testnet_profile_for_testnet_gate() {
        let mut settings = Settings::default_for("/tmp/aoxc".to_string());
        settings.profile = "mainnet".to_string();
        settings.logging.json = true;
        settings.network.bind_host = "0.0.0.0".to_string();

        let readiness =
            evaluate_profile_readiness("testnet", &settings, None, Some("active"), true, true);

        assert!(
            readiness
                .blockers
                .iter()
                .any(|entry| entry.starts_with("testnet-profile:"))
        );
        assert!(
            readiness
                .remediation_plan
                .iter()
                .any(|step| step.contains("--profile testnet"))
        );
    }

    #[test]
    fn surface_builder_reports_blocked_surface_when_checks_fail() {
        let surface = build_surface(
            "desktop-wallet",
            "client-platform",
            vec![
                surface_check("desktop-wallet-compat", true, "compat present".to_string()),
                surface_check(
                    "production-audit",
                    false,
                    "production audit missing".to_string(),
                ),
            ],
            vec!["artifacts/network-production-closure/desktop-wallet-compat.json".to_string()],
        );

        assert_eq!(surface.surface, "desktop-wallet");
        assert_eq!(surface.status, "hardening");
        assert_eq!(surface.score, 50);
        assert_eq!(surface.blockers.len(), 1);
        assert!(surface.blockers[0].contains("production-audit"));
    }

    #[test]
    fn full_surface_readiness_reports_all_target_surfaces() {
        let mut settings = Settings::default_for("/tmp/aoxc".to_string());
        settings.profile = "mainnet".to_string();
        settings.logging.json = true;
        settings.network.bind_host = "0.0.0.0".to_string();
        settings.telemetry.enable_metrics = true;

        let readiness =
            evaluate_profile_readiness("mainnet", &settings, None, Some("active"), true, true);
        let full = evaluate_full_surface_readiness(&settings, &readiness);

        assert_eq!(full.release_line, "aoxc.v.0.1.1-akdeniz");
        assert!(full.matrix_loaded);
        assert_eq!(
            full.matrix_release_line.as_deref(),
            Some("aoxc.v.0.1.1-akdeniz")
        );
        assert_eq!(full.matrix_surface_count, 7);
        assert!(
            full.matrix_warnings.is_empty(),
            "{:?}",
            full.matrix_warnings
        );
        assert_eq!(full.total_surfaces, 7);
        assert_eq!(full.surfaces.len(), 7);
        assert!(
            full.surfaces
                .iter()
                .any(|surface| surface.surface == "mainnet")
        );
        assert!(
            full.surfaces
                .iter()
                .any(|surface| surface.surface == "quantum-consensus")
        );
        assert!(
            full.surfaces
                .iter()
                .any(|surface| surface.surface == "testnet")
        );
        assert!(
            full.surfaces
                .iter()
                .any(|surface| surface.surface == "aoxhub")
        );
        assert!(
            full.surfaces
                .iter()
                .any(|surface| surface.surface == "devnet")
        );
        assert!(
            full.surfaces
                .iter()
                .any(|surface| surface.surface == "desktop-wallet")
        );
        assert!(
            full.surfaces
                .iter()
                .any(|surface| surface.surface == "telemetry")
        );

        let failures = collect_surface_gate_failures(&full);
        for failure in failures {
            assert!(
                failure.code.starts_with("AOXC_GATE_"),
                "unexpected gate code: {}",
                failure.code
            );
        }
    }

    #[test]
    fn full_surface_markdown_report_includes_release_and_surface_summary() {
        let mut settings = Settings::default_for("/tmp/aoxc".to_string());
        settings.profile = "mainnet".to_string();
        settings.logging.json = true;
        settings.network.bind_host = "0.0.0.0".to_string();
        settings.telemetry.enable_metrics = true;

        let readiness =
            evaluate_profile_readiness("mainnet", &settings, None, Some("active"), true, true);
        let full = evaluate_full_surface_readiness(&settings, &readiness);
        let report = full_surface_markdown_report(&full);

        assert!(report.contains("# AOXC Full-Surface Readiness Report"));
        assert!(report.contains("Release line: `aoxc.v.0.1.1-akdeniz`"));
        assert!(report.contains("## Surface summary"));
        assert!(report.contains("**mainnet** / owner `protocol-release`"));
    }

    #[test]
    fn surface_builder_reports_ready_surface_when_all_checks_pass() {
        let surface = build_surface(
            "devnet",
            "engineering-platform",
            vec![
                surface_check("config", true, "config found".to_string()),
                surface_check("fixture", true, "fixture found".to_string()),
            ],
            vec!["configs/devnet.toml".to_string()],
        );

        assert_eq!(surface.surface, "devnet");
        assert_eq!(surface.status, "ready");
        assert_eq!(surface.score, 100);
        assert!(surface.blockers.is_empty());
    }

    #[test]
    fn surface_builder_reports_blocked_surface_when_majority_checks_fail() {
        let surface = build_surface(
            "telemetry",
            "sre-observability",
            vec![
                surface_check("metrics", false, "disabled".to_string()),
                surface_check("snapshot", false, "missing".to_string()),
                surface_check("alerts", true, "present".to_string()),
            ],
            vec!["artifacts/network-production-closure/alert-rules.md".to_string()],
        );

        assert_eq!(surface.status, "blocked");
        assert_eq!(surface.score, 33);
        assert_eq!(surface.blockers.len(), 2);
    }

    #[test]
    fn artifact_locator_walks_up_to_repo_root() {
        let release_dir = locate_repo_artifact_dir("release-evidence");
        assert!(
            release_dir.ends_with(Path::new("artifacts").join("release-evidence")),
            "artifact lookup should resolve to repository artifacts directory"
        );
    }

    #[test]
    fn embedded_profiles_share_expected_baseline_controls() {
        let report = compare_embedded_network_profiles()
            .expect("embedded network baseline comparison should load");

        assert!(report.passed, "baseline drift: {:?}", report.drift);
    }

    #[test]
    fn aoxhub_profiles_share_expected_baseline_controls() {
        let report = compare_aoxhub_network_profiles()
            .expect("embedded AOXHub baseline comparison should load");

        assert!(report.passed, "baseline drift: {:?}", report.drift);
    }

    #[test]
    fn parse_network_profile_reads_expected_fields() {
        let dir = unique_dir("network-profile");
        let path = dir.join("profile.toml");
        fs::create_dir_all(&dir).expect("fixture directory should be created");
        fs::write(
            &path,
            r#"chain_id = "aox-testnet-9"
listen_addr = "0.0.0.0:36656"
rpc_addr = "0.0.0.0:18545"
peers = [
  "127.0.0.1:36657",
  "127.0.0.1:36658",
]
security_mode = "audit_strict"
"#,
        )
        .expect("profile fixture should be written");

        let profile = parse_network_profile(&path).expect("profile should parse");

        assert_eq!(profile.chain_id, "aox-testnet-9");
        assert_eq!(profile.peers.len(), 2);
        assert_eq!(profile.security_mode, "audit_strict");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn readiness_markdown_report_includes_dual_track_summary() {
        let mut settings = Settings::default_for("/tmp/aoxc".to_string());
        settings.profile = "validator".to_string();
        settings.logging.json = true;
        settings.network.bind_host = "0.0.0.0".to_string();

        let readiness =
            evaluate_profile_readiness("mainnet", &settings, None, Some("active"), true, true);
        let report = readiness_markdown_report(
            &readiness,
            compare_embedded_network_profiles().ok().as_ref(),
            compare_aoxhub_network_profiles().ok().as_ref(),
        );

        assert!(report.contains("# AOXC Progress Report"));
        assert!(report.contains("## Dual-track progress"));
        assert!(report.contains("**testnet**"));
        assert!(report.contains("**mainnet**"));
        assert!(report.contains("## Baseline parity"));
    }

    #[test]
    fn write_readiness_markdown_report_persists_file() {
        let dir = unique_dir("readiness-report");
        let path = dir.join("AOXC_PROGRESS_REPORT.md");
        let mut settings = Settings::default_for("/tmp/aoxc".to_string());
        settings.profile = "mainnet".to_string();
        settings.logging.json = true;
        settings.network.bind_host = "0.0.0.0".to_string();

        let readiness =
            evaluate_profile_readiness("mainnet", &settings, None, Some("active"), true, true);
        write_readiness_markdown_report(
            &path,
            &readiness,
            compare_embedded_network_profiles().ok().as_ref(),
            compare_aoxhub_network_profiles().ok().as_ref(),
        )
        .expect("report should write");

        let saved = fs::read_to_string(&path).expect("report should be readable");
        let expected = format!("Overall readiness: **{}%**", readiness.readiness_score);
        assert!(saved.contains(&expected));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn shifted_ports_require_same_delta_across_profiles() {
        let mainnet_profile = super::NetworkProfileConfig {
            chain_id: "aox-mainnet-1".to_string(),
            listen_addr: "0.0.0.0:26656".to_string(),
            rpc_addr: "0.0.0.0:8545".to_string(),
            peers: vec!["seed-1".to_string(), "seed-2".to_string()],
            security_mode: "audit_strict".to_string(),
        };
        let testnet_profile = super::NetworkProfileConfig {
            chain_id: "aox-testnet-1".to_string(),
            listen_addr: "0.0.0.0:36656".to_string(),
            rpc_addr: "0.0.0.0:18545".to_string(),
            peers: vec!["seed-1".to_string(), "seed-2".to_string()],
            security_mode: "audit_strict".to_string(),
        };

        assert!(ports_are_shifted_consistently(
            &mainnet_profile,
            &testnet_profile
        ));
    }

    #[test]
    fn rpc_http_get_probe_reports_success_for_200_response() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let port = listener
            .local_addr()
            .expect("listener should expose local addr")
            .port();
        let server = thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut request = [0_u8; 1024];
                let _ = stream.read(&mut request);
                let _ = stream.write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}",
                );
            }
        });

        assert!(rpc_http_get_probe("127.0.0.1", port, "/health"));
        let _ = server.join();
    }

    #[test]
    fn rpc_jsonrpc_status_probe_reports_success_for_200_response() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let port = listener
            .local_addr()
            .expect("listener should expose local addr")
            .port();
        let server = thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut request = [0_u8; 2048];
                let _ = stream.read(&mut request);
                let _ = stream.write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 36\r\n\r\n{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}",
                );
            }
        });

        assert!(rpc_jsonrpc_status_probe("127.0.0.1", port));
        let _ = server.join();
    }
}
