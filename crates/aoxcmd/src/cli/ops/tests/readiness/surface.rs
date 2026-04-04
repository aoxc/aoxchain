use super::*;

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
