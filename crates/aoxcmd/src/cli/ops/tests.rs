use super::{
    FaucetClaimRecord, FaucetState, build_surface, collect_surface_gate_failures,
    compare_aoxhub_network_profiles, compare_embedded_network_profiles, evaluate_faucet_claim,
    evaluate_full_surface_readiness, evaluate_profile_readiness, full_surface_markdown_report,
    has_desktop_wallet_compat_artifact, has_matching_artifact, has_production_closure_artifacts,
    has_release_evidence, has_release_provenance_bundle, has_security_drill_artifact,
    historical_tx_hashes, locate_repo_artifact_dir, open_checklist_items, parse_network_profile,
    parse_positive_u64_arg, parse_required_or_default_text_arg, ports_are_shifted_consistently,
    readiness_markdown_report, rpc_http_get_probe, rpc_jsonrpc_status_probe, surface_check,
    tx_hash_hex, write_readiness_markdown_report,
};
use crate::config::settings::Settings;
use aoxcdata::BlockEnvelope;
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
    let error = parse_required_or_default_text_arg(&args(&["--to", "   "]), "--to", "ops", false)
        .expect_err("blank target must fail");
    assert_eq!(error.code(), "AOXC-USG-002");
}

#[test]
fn historical_tx_hashes_extracts_payload_from_block_envelope() {
    let envelope = BlockEnvelope {
        height: 7,
        block_hash_hex: "e3c0fdbff6f570f0449557cb9a9d8bc95eeb5d1f7e5bc8f2a580f7f7f6f7a9a7"
            .to_string(),
        parent_hash_hex: "7f6f7a9ae3c0fdbff6f570f0449557cb9a9d8bc95eeb5d1f7e5bc8f2a580f7f7"
            .to_string(),
        payload: br#"{"body":{"sections":[{"payload":"tx-demo-7"}]}}"#.to_vec(),
    };

    let tx_hashes = historical_tx_hashes(&envelope);
    assert_eq!(tx_hashes, vec!["tx-demo-7".to_string()]);
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
