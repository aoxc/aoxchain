use super::*;

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
