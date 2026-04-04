use super::*;

pub(super) fn evaluate_profile_readiness(
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

pub(super) fn load_full_surface_matrix(
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

pub(super) fn validate_full_surface_matrix(
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

pub(super) fn evaluate_full_surface_readiness(
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

pub(super) fn locate_repo_artifact_dir(artifact_name: &str) -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for candidate in cwd.ancestors() {
        let artifact_dir = candidate.join("artifacts").join(artifact_name);
        if artifact_dir.exists() {
            return artifact_dir;
        }
    }
    cwd.join("artifacts").join(artifact_name)
}

pub(super) fn surface_check(name: &'static str, passed: bool, detail: String) -> SurfaceCheck {
    SurfaceCheck {
        name,
        passed,
        detail,
    }
}

pub(super) fn build_surface(
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

pub(super) fn collect_surface_gate_failures(
    readiness: &FullSurfaceReadiness,
) -> Vec<SurfaceGateFailure> {
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

pub(super) fn gate_failure_code(surface: &str, check: &str) -> String {
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

pub(super) fn readiness_check(
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

pub(super) fn readiness_from_checks(profile: String, checks: Vec<ReadinessCheck>) -> Readiness {
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

pub(super) fn area_progress(checks: &[ReadinessCheck]) -> Vec<ReadinessAreaProgress> {
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

pub(super) fn track_progress(checks: &[ReadinessCheck]) -> Vec<ReadinessTrackProgress> {
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

pub(super) fn next_focus(area_progress: &[ReadinessAreaProgress]) -> Vec<String> {
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

pub(super) fn ratio(completed_weight: u8, max_weight: u8) -> u8 {
    if max_weight == 0 {
        0
    } else {
        (completed_weight as u16 * 100 / max_weight as u16) as u8
    }
}

pub(super) fn progress_status(ratio: u8) -> &'static str {
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

pub(super) fn write_readiness_markdown_report(
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

pub(super) fn write_full_surface_markdown_report(
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

pub(super) fn readiness_markdown_report(
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

pub(super) fn full_surface_markdown_report(readiness: &FullSurfaceReadiness) -> String {
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

pub(super) fn append_baseline_section(
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

pub(super) fn remediation_plan(checks: &[ReadinessCheck]) -> Vec<String> {
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

pub(super) fn has_release_evidence(dir: &Path) -> bool {
    has_matching_artifact(dir, "release-evidence-", ".md")
        && has_matching_artifact(dir, "build-manifest-", ".json")
        && has_matching_artifact(dir, "compat-matrix-", ".json")
        && has_matching_artifact(dir, "production-audit-", ".json")
        && has_matching_artifact(dir, "sbom-", ".json")
        && (has_matching_artifact(dir, "aoxc-", ".sig")
            || has_matching_artifact(dir, "aoxc-", ".sig.status"))
}

pub(super) fn has_release_provenance_bundle(dir: &Path) -> bool {
    has_matching_artifact(dir, "provenance-", ".json")
        && has_matching_artifact(dir, "release-provenance-", ".json")
        && has_matching_artifact(dir, "release-sbom-", ".json")
        && has_matching_artifact(dir, "release-build-manifest-", ".json")
        && has_matching_artifact(dir, "release-signature-status-", ".txt")
}

pub(super) fn has_production_closure_artifacts(dir: &Path) -> bool {
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

pub(super) fn has_security_drill_artifact(dir: &Path) -> bool {
    json_artifact_has_required_strings(
        &dir.join("security-drill.json"),
        "scenarios",
        &["penetration-baseline", "rpc-authz", "session-replay"],
    )
}

pub(super) fn has_desktop_wallet_compat_artifact(dir: &Path) -> bool {
    json_artifact_has_required_strings(
        &dir.join("desktop-wallet-compat.json"),
        "surfaces",
        &["desktop-wallet", "aoxhub", "mainnet", "testnet"],
    )
}

pub(super) fn json_artifact_has_required_strings(
    path: &Path,
    key: &str,
    required: &[&str],
) -> bool {
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

pub(super) fn has_matching_artifact(dir: &Path, prefix: &str, suffix: &str) -> bool {
    std::fs::read_dir(dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .any(|name| name.starts_with(prefix) && name.ends_with(suffix))
}

pub(super) fn compare_embedded_network_profiles() -> Result<ProfileBaselineReport, AppError> {
    let repo_root = locate_repo_root();
    compare_network_profile_pair(
        repo_root.join("configs").join("mainnet.toml"),
        repo_root.join("configs").join("testnet.toml"),
    )
}

pub(super) fn compare_aoxhub_network_profiles() -> Result<ProfileBaselineReport, AppError> {
    let repo_root = locate_repo_root();
    compare_network_profile_pair(
        repo_root.join("configs").join("aoxhub-mainnet.toml"),
        repo_root.join("configs").join("aoxhub-testnet.toml"),
    )
}

pub(super) fn compare_network_profile_pair(
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

pub(super) fn locate_repo_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for candidate in cwd.ancestors() {
        if candidate.join("Cargo.toml").exists() && candidate.join("configs").exists() {
            return candidate.to_path_buf();
        }
    }
    cwd
}

pub(super) fn open_checklist_items(path: &Path) -> Vec<String> {
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

pub(super) fn parse_network_profile(path: &Path) -> Result<NetworkProfileConfig, AppError> {
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

pub(super) fn unquote(value: &str) -> String {
    value.trim().trim_matches('"').to_string()
}

pub(super) fn normalized_port_pair(listen_addr: &str, rpc_addr: &str) -> String {
    format!(
        "{}/{}",
        extract_port(listen_addr).map_or_else(|| "?".to_string(), |p| p.to_string()),
        extract_port(rpc_addr).map_or_else(|| "?".to_string(), |p| p.to_string())
    )
}

pub(super) fn ports_are_shifted_consistently(
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

pub(super) fn extract_port(addr: &str) -> Option<u16> {
    addr.rsplit(':').next()?.parse::<u16>().ok()
}
