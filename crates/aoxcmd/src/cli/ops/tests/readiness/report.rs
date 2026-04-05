use super::*;
use crate::cli::ops::AOXC_Q_RELEASE_LINE;

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
    assert!(report.contains(&format!("Release line: `{AOXC_Q_RELEASE_LINE}`")));
    assert!(report.contains("## Surface summary"));
    assert!(report.contains("**mainnet** / owner `protocol-release`"));
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
