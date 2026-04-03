use super::*;

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
