use super::core::DEFAULT_TREASURY;
use super::*;
use crate::genesis::{AOXC_FAMILY_ID, NetworkClass};
use std::fs;
use std::path::PathBuf;

#[test]
fn load_default_builds_aoxc_mainnet_identity() {
    let config = GenesisLoader::load_default().expect("default mainnet genesis must build");

    assert_eq!(config.identity.family_id, AOXC_FAMILY_ID);
    assert_eq!(config.identity.chain_id, 2626000001);
    assert_eq!(config.identity.network_serial, "2626-001");
    assert_eq!(config.identity.network_id, "aoxc-mainnet-2626-001");
    assert_eq!(config.identity.network_class, NetworkClass::PublicMainnet);
    assert_eq!(config.treasury, DEFAULT_TREASURY);
    assert_eq!(config.accounts.len(), 1);
    assert_eq!(config.accounts[0].address, TREASURY_ACCOUNT);
    assert_eq!(config.accounts[0].balance, DEFAULT_TREASURY);
}

#[test]
fn load_default_testnet_builds_aoxc_testnet_identity() {
    let config = GenesisLoader::load_default_testnet().expect("default testnet genesis must build");

    assert_eq!(config.identity.family_id, AOXC_FAMILY_ID);
    assert_eq!(config.identity.chain_id, 2626010001);
    assert_eq!(config.identity.network_serial, "2626-002");
    assert_eq!(config.identity.network_id, "aoxc-testnet-2626-002");
    assert_eq!(config.identity.network_class, NetworkClass::PublicTestnet);
}

#[test]
fn resolve_sidecar_paths_are_stable() {
    let path = PathBuf::from("/tmp/genesis.json");

    assert_eq!(
        GenesisLoader::resolve_fingerprint_sidecar_path(&path),
        PathBuf::from("/tmp/genesis.json.fingerprint")
    );

    assert_eq!(
        GenesisLoader::resolve_signature_sidecar_path(&path),
        PathBuf::from("/tmp/genesis.json.sig")
    );
}

#[test]
fn load_returns_validation_error_for_invalid_genesis_file() {
    let temp_dir =
        std::env::temp_dir().join(format!("aoxc-genesis-loader-test-{}", std::process::id()));
    fs::create_dir_all(&temp_dir).expect("temp dir must be created");

    let path = temp_dir.join("genesis.json");

    fs::write(
        &path,
        r#"{
            "identity": {
                "family_id": 2626,
                "chain_id": 1,
                "network_serial": "2626-001",
                "network_id": "aoxc-mainnet-2626-001",
                "chain_name": "AOXC AKDENIZ",
                "network_class": "public-mainnet"
            },
            "block_time": 0,
            "validators": [],
            "accounts": [],
            "treasury": 0,
            "settlement_link": { "endpoint": "aoxc://settlement/root" },
            "genesis_seal": { "seal_id": "aoxc-seal-mainnet-001" }
        }"#,
    )
    .expect("invalid genesis fixture must write");

    let err = GenesisLoader::load(&path).expect_err("invalid genesis must be rejected");
    assert!(matches!(err, GenesisError::ValidationError(_)));

    let _ = fs::remove_file(&path);
    let _ = fs::remove_dir(&temp_dir);
}

#[test]
fn save_writes_fingerprint_sidecar() {
    let temp_dir =
        std::env::temp_dir().join(format!("aoxc-genesis-save-test-{}", std::process::id()));
    fs::create_dir_all(&temp_dir).expect("temp dir must be created");

    let path = temp_dir.join("genesis.json");
    let genesis = GenesisLoader::load_default().expect("default genesis must build");

    GenesisLoader::save(&genesis, &path).expect("save must succeed");

    let fingerprint_path = GenesisLoader::resolve_fingerprint_sidecar_path(&path);
    assert!(path.is_file());
    assert!(fingerprint_path.is_file());

    let sidecar = fs::read_to_string(&fingerprint_path).expect("sidecar must be readable");
    assert_eq!(sidecar.trim(), genesis.fingerprint().unwrap());

    let _ = fs::remove_file(&path);
    let _ = fs::remove_file(&fingerprint_path);
    let _ = fs::remove_dir(&temp_dir);
}

#[test]
fn load_artifact_verifies_matching_fingerprint_sidecar() {
    let temp_dir = std::env::temp_dir().join(format!(
        "aoxc-genesis-load-artifact-test-{}",
        std::process::id()
    ));
    fs::create_dir_all(&temp_dir).expect("temp dir must be created");

    let path = temp_dir.join("genesis.json");
    let genesis = GenesisLoader::load_default().expect("default genesis must build");
    GenesisLoader::save(&genesis, &path).expect("save must succeed");

    let artifact = GenesisLoader::load_artifact(&path).expect("artifact load must succeed");
    assert!(artifact.fingerprint_sidecar_present);
    assert!(!artifact.detached_signature_sidecar_present);

    let _ = fs::remove_file(GenesisLoader::resolve_fingerprint_sidecar_path(&path));
    let _ = fs::remove_file(&path);
    let _ = fs::remove_dir(&temp_dir);
}
