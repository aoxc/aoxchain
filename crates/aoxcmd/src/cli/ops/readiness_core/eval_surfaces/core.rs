use crate::cli::ops::AOXC_Q_RELEASE_LINE;

pub(in crate::cli::ops) fn evaluate_full_surface_readiness(
    settings: &crate::config::settings::Settings,
    mainnet_readiness: &Readiness,
) -> FullSurfaceReadiness {
    let repo_root = locate_repo_root();
    let release_line = "AOXC-Q-v0.2.0";
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

pub(in crate::cli::ops) fn locate_repo_artifact_dir(artifact_name: &str) -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for candidate in cwd.ancestors() {
        let artifact_dir = candidate.join("artifacts").join(artifact_name);
        if artifact_dir.exists() {
            return artifact_dir;
        }
    }
    cwd.join("artifacts").join(artifact_name)
}

pub(in crate::cli::ops) fn surface_check(
    name: &'static str,
    passed: bool,
    detail: String,
) -> SurfaceCheck {
    SurfaceCheck {
        name,
        passed,
        detail,
    }
}

pub(in crate::cli::ops) fn build_surface(
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

pub(in crate::cli::ops) fn collect_surface_gate_failures(
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

pub(in crate::cli::ops) fn gate_failure_code(surface: &str, check: &str) -> String {
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

pub(in crate::cli::ops) fn readiness_check(
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

pub(in crate::cli::ops) fn readiness_from_checks(
    profile: String,
    checks: Vec<ReadinessCheck>,
) -> Readiness {
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

pub(in crate::cli::ops) fn area_progress(checks: &[ReadinessCheck]) -> Vec<ReadinessAreaProgress> {
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

pub(in crate::cli::ops) fn track_progress(
    checks: &[ReadinessCheck],
) -> Vec<ReadinessTrackProgress> {
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

pub(in crate::cli::ops) fn next_focus(area_progress: &[ReadinessAreaProgress]) -> Vec<String> {
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

pub(in crate::cli::ops) fn ratio(completed_weight: u8, max_weight: u8) -> u8 {
    if max_weight == 0 {
        0
    } else {
        (completed_weight as u16 * 100 / max_weight as u16) as u8
    }
}

pub(in crate::cli::ops) fn progress_status(ratio: u8) -> &'static str {
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
