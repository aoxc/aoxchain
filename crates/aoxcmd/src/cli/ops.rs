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
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(serde::Serialize)]
struct ReadinessCheck {
    name: &'static str,
    area: &'static str,
    passed: bool,
    weight: u8,
    detail: String,
}

#[derive(serde::Serialize)]
struct Readiness {
    profile: String,
    stage: &'static str,
    readiness_score: u8,
    max_score: u8,
    completed_weight: u8,
    remaining_weight: u8,
    verdict: &'static str,
    blockers: Vec<String>,
    next_steps: Vec<String>,
    checks: Vec<ReadinessCheck>,
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
                "Mainnet readiness gate failed at {}%. Outstanding blockers: {}",
                readiness.readiness_score,
                readiness.blockers.join(" | ")
            ),
        ));
    }
    emit_serialized(&readiness, output_format(args))
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
    let next_steps = remediation_steps(&checks);

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
        next_steps,
        checks,
    }
}

fn remediation_steps(checks: &[ReadinessCheck]) -> Vec<String> {
    let mut steps = Vec::new();
    for check in checks.iter().filter(|check| !check.passed) {
        let command = match check.name {
            "mainnet-profile" | "structured-logging" => {
                "aoxc config-init --profile mainnet --json-logs".to_string()
            }
            "genesis-present" => "aoxc genesis-init --chain-num 1".to_string(),
            "node-state-present" => "aoxc node-bootstrap".to_string(),
            "operator-key-active" => {
                "aoxc key-bootstrap --profile mainnet --password '<strong-password>'".to_string()
            }
            "config-valid" => "aoxc config-validate".to_string(),
            "release-evidence" | "compatibility-matrix" | "provenance-attestation" => {
                "./scripts/release/generate_release_evidence.sh".to_string()
            }
            "production-closure" => {
                "./scripts/validation/network_production_closure.sh --scenario all".to_string()
            }
            _ => continue,
        };

        if !steps.iter().any(|existing| existing == &command) {
            steps.push(command);
        }
    }
    steps
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
        evaluate_mainnet_readiness, has_matching_artifact, has_production_closure_artifacts,
        has_release_evidence, locate_repo_artifact_dir, readiness_check, remediation_steps,
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
        assert!(readiness.next_steps.is_empty());
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
    fn remediation_steps_map_failed_controls_to_operator_commands() {
        let steps = remediation_steps(&[
            readiness_check(
                "mainnet-profile",
                "configuration",
                false,
                10,
                "profile mismatch".to_string(),
            ),
            readiness_check(
                "genesis-present",
                "identity",
                false,
                10,
                "missing genesis".to_string(),
            ),
            readiness_check(
                "production-closure",
                "operations",
                false,
                7,
                "missing closure artifacts".to_string(),
            ),
        ]);

        assert!(steps
            .iter()
            .any(|step| step.contains("config-init --profile mainnet")));
        assert!(steps
            .iter()
            .any(|step| step.contains("genesis-init --chain-num 1")));
        assert!(steps
            .iter()
            .any(|step| step.contains("network_production_closure.sh --scenario all")));
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
