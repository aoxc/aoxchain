use super::*;
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
    let report =
        compare_aoxhub_network_profiles().expect("embedded AOXHub baseline comparison should load");

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
