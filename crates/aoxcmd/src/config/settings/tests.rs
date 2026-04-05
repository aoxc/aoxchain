use super::Settings;
use super::core::CanonicalProfile;

#[test]
fn default_for_uses_validation_profile() {
    let settings = Settings::default_for("/tmp/aoxc".to_string());
    assert_eq!(settings.profile, "validation");
}

#[test]
fn default_for_profile_accepts_legacy_validator_alias() {
    let settings = Settings::default_for_profile("/tmp/aoxc".to_string(), "validator").unwrap();
    assert_eq!(settings.profile, "validation");
}

#[test]
fn default_for_profile_rejects_blank_home_dir() {
    let error = Settings::default_for_profile("   ".to_string(), "validation")
        .expect_err("blank home_dir must fail");

    assert!(error.contains("home_dir must not be empty"));
}

#[test]
fn validate_rejects_unknown_profile() {
    let mut settings = Settings::default_for("/tmp/aoxc".to_string());
    settings.profile = "staging".to_string();

    assert!(settings.validate().is_err());
}

#[test]
fn validate_rejects_blank_home_dir() {
    let mut settings = Settings::default_for("/tmp/aoxc".to_string());
    settings.home_dir = "   ".to_string();

    assert!(settings.validate().is_err());
}

#[test]
fn validate_rejects_blank_bind_host() {
    let mut settings = Settings::default_for("/tmp/aoxc".to_string());
    settings.network.bind_host = "   ".to_string();

    assert!(settings.validate().is_err());
}

#[test]
fn validate_rejects_invalid_logging_level() {
    let mut settings = Settings::default_for("/tmp/aoxc".to_string());
    settings.logging.level = "verbose".to_string();

    assert!(settings.validate().is_err());
}

#[test]
fn validate_rejects_port_collisions() {
    let mut settings = Settings::default_for("/tmp/aoxc".to_string());
    settings.network.rpc_port = settings.network.p2p_port;

    assert!(settings.validate().is_err());
}

#[test]
fn validate_rejects_zero_ports() {
    let mut settings = Settings::default_for("/tmp/aoxc".to_string());
    settings.network.p2p_port = 0;

    assert!(settings.validate().is_err());
}

#[test]
fn validate_rejects_conflicting_peer_policy() {
    let mut settings = Settings::default_for("/tmp/aoxc".to_string());
    settings.policy.allow_remote_peers = true;
    settings.network.enforce_official_peers = true;

    assert!(settings.validate().is_err());
}

#[test]
fn validate_rejects_mainnet_without_structured_logging() {
    let mut settings =
        Settings::default_for_profile("/tmp/aoxc".to_string(), "mainnet").unwrap();
    settings.logging.json = false;
    settings.network.bind_host = "0.0.0.0".to_string();

    assert!(settings.validate().is_err());
}

#[test]
fn validate_accepts_hardened_mainnet_profile() {
    let settings = Settings::default_for_profile("/opt/aoxc".to_string(), "mainnet").unwrap();

    assert!(settings.validate().is_ok());
}

#[test]
fn validate_rejects_mainnet_with_relative_home_dir() {
    let mut settings =
        Settings::default_for_profile("relative/aoxc".to_string(), "mainnet").unwrap();
    settings.logging.json = true;
    settings.network.bind_host = "0.0.0.0".to_string();

    assert!(settings.validate().is_err());
}

#[test]
fn validate_rejects_mainnet_with_debug_logging() {
    let mut settings =
        Settings::default_for_profile("/tmp/aoxc".to_string(), "mainnet").unwrap();
    settings.logging.level = "debug".to_string();

    assert!(settings.validate().is_err());
}

#[test]
fn validate_rejects_mainnet_with_trace_logging() {
    let mut settings =
        Settings::default_for_profile("/tmp/aoxc".to_string(), "mainnet").unwrap();
    settings.logging.level = "trace".to_string();

    assert!(settings.validate().is_err());
}

#[test]
fn validate_rejects_mainnet_with_loopback_bind_host() {
    let mut settings =
        Settings::default_for_profile("/tmp/aoxc".to_string(), "mainnet").unwrap();
    settings.network.bind_host = "127.0.0.1".to_string();

    assert!(settings.validate().is_err());
}

#[test]
fn validate_accepts_testnet_defaults() {
    let settings = Settings::default_for_profile("/tmp/aoxc".to_string(), "testnet").unwrap();

    assert!(settings.validate().is_ok());
}

#[test]
fn validate_rejects_testnet_trace_logging() {
    let mut settings =
        Settings::default_for_profile("/tmp/aoxc".to_string(), "testnet").unwrap();
    settings.logging.level = "trace".to_string();

    assert!(settings.validate().is_err());
}

#[test]
fn validate_accepts_validation_defaults() {
    let settings =
        Settings::default_for_profile("/tmp/aoxc".to_string(), "validation").unwrap();

    assert!(settings.validate().is_ok());
}

#[test]
fn validate_rejects_validation_remote_peers_on_loopback() {
    let mut settings =
        Settings::default_for_profile("/tmp/aoxc".to_string(), "validation").unwrap();
    settings.policy.allow_remote_peers = true;
    settings.network.enforce_official_peers = false;
    settings.network.bind_host = "127.0.0.1".to_string();

    assert!(settings.validate().is_err());
}

#[test]
fn validate_accepts_devnet_defaults() {
    let settings = Settings::default_for_profile("/tmp/aoxc".to_string(), "devnet").unwrap();

    assert!(settings.validate().is_ok());
}

#[test]
fn validate_accepts_localnet_defaults() {
    let settings = Settings::default_for_profile("/tmp/aoxc".to_string(), "localnet").unwrap();

    assert!(settings.validate().is_ok());
}

#[test]
fn validate_rejects_localnet_with_non_loopback_bind_host() {
    let mut settings =
        Settings::default_for_profile("/tmp/aoxc".to_string(), "localnet").unwrap();
    settings.network.bind_host = "0.0.0.0".to_string();

    assert!(settings.validate().is_err());
}

#[test]
fn canonical_profile_parser_normalizes_validator_alias() {
    let parsed = CanonicalProfile::parse("validator").unwrap();
    assert_eq!(parsed, CanonicalProfile::Validation);
}

#[test]
fn redacted_returns_structurally_identical_settings() {
    let settings = Settings::default_for("/tmp/aoxc".to_string());
    assert_eq!(settings, settings.redacted());
}
