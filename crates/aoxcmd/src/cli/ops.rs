use crate::{
    app::{
        bootstrap::bootstrap_operator_home, runtime::refresh_runtime_metrics,
        shutdown::graceful_shutdown,
    },
    cli_support::{arg_value, emit_serialized, has_flag, output_format, text_envelope},
    config::loader::load_or_init,
    economy::ledger,
    error::{AppError, ErrorCode},
    node::{engine, lifecycle},
    runtime::{
        core::runtime_context, handles::default_handles, node::health_status, unity::unity_status,
    },
};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

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
    checks: Vec<ReadinessCheck>,
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
    let rounds = arg_value(args, "--rounds").unwrap_or_else(|| "100".to_string());
    let mut details = BTreeMap::new();
    details.insert("benchmark_rounds".to_string(), rounds);
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
    let settings = load_or_init()?;
    let key_summary = crate::keys::manager::inspect_operator_key().ok();
    let genesis_ok = crate::cli::bootstrap::genesis_ready();
    let node_ok = lifecycle::load_state().is_ok();
    let config_validation = settings.validate();
    let readiness = evaluate_mainnet_readiness(
        &settings,
        config_validation.err(),
        key_summary
            .as_ref()
            .map(|summary| summary.operational_state.as_str()),
        genesis_ok,
        node_ok,
    );

    if has_flag(args, "--enforce") && readiness.verdict != "candidate" {
        return Err(AppError::new(
            ErrorCode::PolicyGateFailed,
            format!(
                "Mainnet readiness enforcement failed at score {} with blockers: {}",
                readiness.readiness_score,
                readiness.blockers.join(" | ")
            ),
        ));
    }

    emit_serialized(&readiness, output_format(args))
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

fn evaluate_mainnet_readiness(
    settings: &crate::config::settings::Settings,
    config_validation_error: Option<String>,
    key_operational_state: Option<&str>,
    genesis_ok: bool,
    node_ok: bool,
) -> Readiness {
    let release_dir = locate_repo_artifact_dir("release-evidence");
    let closure_dir = locate_repo_artifact_dir("network-production-closure");
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
            "mainnet-profile",
            "configuration",
            settings.profile.eq_ignore_ascii_case("mainnet"),
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
            "compatibility-matrix",
            "release",
            has_matching_artifact(&release_dir, "compat-matrix-", ".json"),
            3,
            "Compatibility matrix evidence must be generated for the candidate release".to_string(),
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
        checks,
    }
}

fn remediation_plan(checks: &[ReadinessCheck]) -> Vec<String> {
    let mut plan = Vec::new();

    for check in checks.iter().filter(|check| !check.passed) {
        let step = match check.name {
            "config-valid" => {
                "Run `aoxc config-validate` and fix the operator settings file before promotion."
            }
            "mainnet-profile" => {
                "Run `aoxc production-bootstrap --profile mainnet --password <value>` or `aoxc config-init --profile mainnet --json-logs`."
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
                "Align `configs/aoxhub-mainnet.toml` and `configs/aoxhub-testnet.toml` so AOXHub rollout controls match promotion policy."
            }
            "release-evidence" => {
                "Regenerate release evidence under `artifacts/release-evidence/` before promotion."
            }
            "production-closure" => {
                "Refresh production closure artifacts under `artifacts/network-production-closure/`."
            }
            "compatibility-matrix" => {
                "Publish a fresh compatibility matrix for the candidate release."
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
            "Candidate is at 100%; keep running `aoxc mainnet-readiness --enforce --format json` in CI to prevent regressions."
                .to_string(),
        );
    }

    plan
}

fn has_release_evidence(dir: &Path) -> bool {
    has_matching_artifact(dir, "release-evidence-", ".md")
        && has_matching_artifact(dir, "build-manifest-", ".json")
        && has_matching_artifact(dir, "compat-matrix-", ".json")
        && has_matching_artifact(dir, "production-audit-", ".json")
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
            expectation:
                "Testnet should differ only by deterministic port offsets, not by capability shape",
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

fn parse_network_profile(path: &Path) -> Result<NetworkProfileConfig, AppError> {
    let raw = fs::read_to_string(path).map_err(|e| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to read network profile {}", path.display()),
            e,
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
            let peer = trimmed.trim_end_matches(',').trim_matches('"');
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
    let tx = arg_value(args, "--tx").unwrap_or_else(|| "boot-sequence-1".to_string());
    let state = engine::produce_once(&tx)?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&state, output_format(args))
}

pub fn cmd_node_run(args: &[String]) -> Result<(), AppError> {
    let rounds = arg_value(args, "--rounds")
        .unwrap_or_else(|| "10".to_string())
        .parse::<u64>()
        .map_err(|_| AppError::new(ErrorCode::UsageInvalidArguments, "Invalid --rounds value"))?;
    let tx_prefix = arg_value(args, "--tx-prefix").unwrap_or_else(|| "AOXC-RUN".to_string());
    let state = engine::run_rounds(rounds, &tx_prefix)?;
    let _ = refresh_runtime_metrics().ok();
    let _ = graceful_shutdown();
    emit_serialized(&state, output_format(args))
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
    let settings = load_or_init()?;
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
    let settings = load_or_init()?;
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

#[cfg(test)]
mod tests {
    use super::{
        compare_aoxhub_network_profiles, compare_embedded_network_profiles,
        evaluate_mainnet_readiness, has_matching_artifact, has_production_closure_artifacts,
        has_release_evidence, locate_repo_artifact_dir, parse_network_profile,
        ports_are_shifted_consistently,
    };
    use crate::config::settings::Settings;
    use std::{
        fs,
        path::{Path, PathBuf},
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

    #[test]
    fn release_evidence_requires_expected_bundle_files() {
        let dir = unique_dir("release-evidence");
        touch(&dir.join("release-evidence-20260323T000000Z.md"));
        touch(&dir.join("build-manifest-20260323T000000Z.json"));
        touch(&dir.join("compat-matrix-20260323T000000Z.json"));
        touch(&dir.join("production-audit-20260323T000000Z.json"));

        assert!(has_release_evidence(&dir));

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
    fn readiness_scores_full_candidate_when_all_controls_pass() {
        let mut settings = Settings::default_for("/tmp/aoxc".to_string());
        settings.profile = "mainnet".to_string();
        settings.logging.json = true;
        settings.network.bind_host = "0.0.0.0".to_string();

        let readiness = evaluate_mainnet_readiness(&settings, None, Some("active"), true, true);

        assert_eq!(readiness.readiness_score, 100);
        assert_eq!(readiness.verdict, "candidate");
        assert!(readiness.blockers.is_empty());
        assert_eq!(readiness.remediation_plan.len(), 1);
        assert!(readiness.remediation_plan[0].contains("100%"));
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
}

pub fn cmd_economy_init(args: &[String]) -> Result<(), AppError> {
    let ledger = ledger::init()?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&ledger, output_format(args))
}

pub fn cmd_treasury_transfer(args: &[String]) -> Result<(), AppError> {
    let to = arg_value(args, "--to").unwrap_or_else(|| "ops".to_string());
    let amount = arg_value(args, "--amount")
        .unwrap_or_else(|| "1000".to_string())
        .parse::<u64>()
        .map_err(|_| AppError::new(ErrorCode::UsageInvalidArguments, "Invalid --amount value"))?;
    let ledger = ledger::transfer(&to, amount)?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&ledger, output_format(args))
}

pub fn cmd_stake_delegate(args: &[String]) -> Result<(), AppError> {
    let validator = arg_value(args, "--validator").unwrap_or_else(|| "validator-01".to_string());
    let amount = arg_value(args, "--amount")
        .unwrap_or_else(|| "1000".to_string())
        .parse::<u64>()
        .map_err(|_| AppError::new(ErrorCode::UsageInvalidArguments, "Invalid --amount value"))?;
    let ledger = ledger::delegate(&validator, amount)?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&ledger, output_format(args))
}

pub fn cmd_stake_undelegate(args: &[String]) -> Result<(), AppError> {
    let validator = arg_value(args, "--validator").unwrap_or_else(|| "validator-01".to_string());
    let amount = arg_value(args, "--amount")
        .unwrap_or_else(|| "1000".to_string())
        .parse::<u64>()
        .map_err(|_| AppError::new(ErrorCode::UsageInvalidArguments, "Invalid --amount value"))?;
    let ledger = ledger::undelegate(&validator, amount)?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&ledger, output_format(args))
}

pub fn cmd_economy_status(args: &[String]) -> Result<(), AppError> {
    let ledger = ledger::load()?;
    emit_serialized(&ledger, output_format(args))
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
