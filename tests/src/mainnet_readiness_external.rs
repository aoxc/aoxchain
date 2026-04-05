// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

const MAINNET_DIR: &str = "../configs/environments/mainnet";

#[test]
fn mainnet_bundle_identity_remains_cross_file_consistent() {
    let manifest = read_json(&format!("{MAINNET_DIR}/manifest.v1.json"));
    let validators = read_json(&format!("{MAINNET_DIR}/validators.json"));
    let bootnodes = read_json(&format!("{MAINNET_DIR}/bootnodes.json"));

    let manifest_identity = manifest
        .get("identity")
        .and_then(Value::as_object)
        .expect("manifest identity object must exist");

    let chain_id = manifest_identity
        .get("chain_id")
        .and_then(Value::as_u64)
        .expect("manifest chain_id must exist");
    let network_id = manifest_identity
        .get("network_id")
        .and_then(Value::as_str)
        .expect("manifest network_id must exist");
    let network_serial = manifest_identity
        .get("network_serial")
        .and_then(Value::as_str)
        .expect("manifest network_serial must exist");

    let validators_identity = validators
        .get("identity")
        .and_then(Value::as_object)
        .expect("validators identity object must exist");
    assert_eq!(
        validators_identity.get("chain_id").and_then(Value::as_u64),
        Some(chain_id)
    );
    assert_eq!(
        validators_identity
            .get("network_id")
            .and_then(Value::as_str),
        Some(network_id)
    );
    assert_eq!(
        validators_identity
            .get("network_serial")
            .and_then(Value::as_str),
        Some(network_serial)
    );

    let bootnodes_identity = bootnodes
        .get("identity")
        .and_then(Value::as_object)
        .expect("bootnodes identity object must exist");
    assert_eq!(
        bootnodes_identity.get("chain_id").and_then(Value::as_u64),
        Some(chain_id)
    );
    assert_eq!(
        bootnodes_identity.get("network_id").and_then(Value::as_str),
        Some(network_id)
    );

    let validators_list = validators
        .get("validators")
        .and_then(Value::as_array)
        .expect("validators list must exist");
    assert!(
        !validators_list.is_empty(),
        "mainnet validator set must include at least one validator"
    );

    let bootnodes_list = bootnodes
        .get("bootnodes")
        .and_then(Value::as_array)
        .expect("bootnodes list must exist");
    assert!(
        !bootnodes_list.is_empty(),
        "mainnet must include at least one bootnode"
    );

    let active_bootnodes = bootnodes_list
        .iter()
        .filter(|entry| {
            entry
                .get("status")
                .and_then(Value::as_str)
                .is_some_and(|status| status == "active")
        })
        .count();
    assert!(
        active_bootnodes >= 1,
        "mainnet must expose at least one active bootnode"
    );
}

#[test]
fn mainnet_genesis_and_release_controls_remain_production_hardened() {
    let genesis_path = format!("{MAINNET_DIR}/genesis.v1.json");
    let hash_path = format!("{MAINNET_DIR}/genesis.v1.sha256");
    let policy_path = format!("{MAINNET_DIR}/release-policy.toml");
    let profile_path = format!("{MAINNET_DIR}/profile.toml");

    assert!(Path::new(&genesis_path).exists(), "genesis file must exist");
    assert!(
        Path::new(&hash_path).exists(),
        "genesis hash file must exist"
    );
    assert!(
        Path::new(&policy_path).exists(),
        "release policy file must exist"
    );
    assert!(Path::new(&profile_path).exists(), "profile file must exist");

    let genesis_bytes = fs::read(&genesis_path).expect("genesis file should read");
    let computed_hash = hex::encode(Sha256::digest(genesis_bytes));

    let recorded_hash_line =
        fs::read_to_string(&hash_path).expect("genesis hash file should read as UTF-8");
    let recorded_hash = recorded_hash_line
        .split_whitespace()
        .next()
        .expect("genesis hash line must contain hash token");

    assert_eq!(
        computed_hash, recorded_hash,
        "recorded genesis hash must match current genesis payload"
    );

    let release_policy = fs::read_to_string(&policy_path).expect("release policy should read");
    assert!(release_policy.contains("environment = \"mainnet\""));
    assert!(release_policy.contains("release_tier = \"production\""));
    assert!(release_policy.contains("require_tests = true"));
    assert!(
        release_policy.contains("require_registry_consistency = true"),
        "mainnet release must enforce registry consistency"
    );
    assert!(
        release_policy.contains("require_signed_artifacts = true"),
        "mainnet release must require signed artifacts"
    );
    assert!(
        release_policy.contains("allow_unsigned_release = false"),
        "unsigned release must remain forbidden"
    );

    let profile = fs::read_to_string(&profile_path).expect("profile should read");
    assert!(profile.contains("environment = \"mainnet\""));
    assert!(profile.contains("profile_type = \"production\""));
    assert!(
        profile.contains("require_genesis_hash_verification = true"),
        "profile must require genesis hash verification"
    );
    assert!(
        profile.contains("reject_identity_mismatch = true"),
        "profile must reject identity mismatch"
    );
    assert!(
        profile.contains("require_signed_artifacts = true"),
        "profile must keep signed artifact requirement enabled"
    );
}

#[test]
fn mainnet_topology_and_consensus_policy_keep_fail_closed_defaults() {
    let socket_matrix_path = format!("{MAINNET_DIR}/topology/socket-matrix.toml");
    let role_topology_path = format!("{MAINNET_DIR}/topology/role-topology.toml");
    let consensus_policy_path = format!("{MAINNET_DIR}/topology/consensus-policy.toml");

    let socket_matrix = fs::read_to_string(&socket_matrix_path).expect("socket matrix should read");
    assert!(
        socket_matrix.contains("default_action = \"deny\""),
        "socket policy must be fail-closed"
    );
    assert!(
        socket_matrix.contains("auth = \"mtls_role_cert\""),
        "socket policy must enforce role certificate authentication"
    );

    let role_topology = fs::read_to_string(&role_topology_path).expect("role topology should read");
    assert!(
        role_topology.contains("default_deny = true"),
        "role topology must keep default deny posture"
    );
    assert!(
        role_topology.contains("require_validator_governance = true"),
        "activation should remain governance gated"
    );

    let consensus_policy =
        fs::read_to_string(&consensus_policy_path).expect("consensus policy should read");
    assert!(
        consensus_policy.contains("max_reorg_depth = 0"),
        "mainnet consensus policy must preserve zero reorg depth"
    );
    assert!(
        consensus_policy.contains("dual_certificate_mode = true"),
        "mainnet consensus policy must keep dual-certificate mode active"
    );
}

fn read_json(path: &str) -> Value {
    let raw = fs::read_to_string(path).expect("json should be readable");
    serde_json::from_str(&raw).expect("json should parse")
}
