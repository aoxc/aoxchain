use super::*;
use std::path::Path;

pub(super) fn build_surface_readiness_set(
    settings: &crate::config::settings::Settings,
    mainnet_readiness: &Readiness,
    repo_root: &Path,
) -> Vec<SurfaceReadiness> {
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

    vec![
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
    ]
}
