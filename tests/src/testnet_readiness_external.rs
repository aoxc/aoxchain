// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use sha2::{Digest, Sha256};
use std::{collections::HashSet, fs, path::Path};

use serde_json::Value;

const TESTNET_DIR: &str = "../configs/environments/testnet";

#[test]
fn testnet_bundle_identity_is_cross_file_consistent() {
    let manifest = read_json(&format!("{TESTNET_DIR}/manifest.v1.json"));
    let validators = read_json(&format!("{TESTNET_DIR}/validators.json"));
    let bootnodes = read_json(&format!("{TESTNET_DIR}/bootnodes.json"));
    let metadata = read_json(&format!("{TESTNET_DIR}/network-metadata.json"));

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

    assert_eq!(
        metadata.get("chain_id").and_then(Value::as_u64),
        Some(chain_id)
    );
    assert_eq!(
        metadata.get("network_id").and_then(Value::as_str),
        Some(network_id)
    );

    let validators_list = validators
        .get("validators")
        .and_then(Value::as_array)
        .expect("validators list must exist");
    assert!(
        validators_list.len() >= 3,
        "testnet must expose at least three validators"
    );

    let mut seen_validator_ids = HashSet::new();
    let mut seed_role_count = 0usize;
    for entry in validators_list {
        let validator_id = entry
            .get("validator_id")
            .and_then(Value::as_str)
            .expect("validator_id must exist");
        assert!(seen_validator_ids.insert(validator_id.to_string()));

        if entry
            .get("role")
            .and_then(Value::as_str)
            .is_some_and(|role| role.contains("seed"))
        {
            seed_role_count += 1;
        }
    }
    assert!(
        seed_role_count >= 1,
        "testnet validator set must include at least one seed-capable validator"
    );

    let bootnodes_list = bootnodes
        .get("bootnodes")
        .and_then(Value::as_array)
        .expect("bootnodes list must exist");
    assert!(
        bootnodes_list.len() >= 2,
        "testnet should expose at least two bootnodes"
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
        "at least one active bootnode is required"
    );
}

#[test]
fn testnet_genesis_hash_and_policy_flags_match_release_gates() {
    let genesis_path = format!("{TESTNET_DIR}/genesis.v1.json");
    let hash_path = format!("{TESTNET_DIR}/genesis.v1.sha256");
    let policy_path = format!("{TESTNET_DIR}/release-policy.toml");
    let profile_path = format!("{TESTNET_DIR}/profile.toml");

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
    assert!(release_policy.contains("environment = \"testnet\""));
    assert!(release_policy.contains("require_tests = true"));
    assert!(
        release_policy.contains("require_manifest_validation = true"),
        "manifest validation gate must remain enabled"
    );
    assert!(
        release_policy.contains("require_genesis_hash_validation = true"),
        "genesis hash gate must remain enabled"
    );
    assert!(
        release_policy.contains("allow_unsigned_release = false"),
        "unsigned release should remain forbidden"
    );

    let profile = fs::read_to_string(&profile_path).expect("profile should read");
    assert!(profile.contains("environment = \"testnet\""));
    assert!(
        profile.contains("require_genesis_hash_verification = true"),
        "profile must require genesis hash verification"
    );
    assert!(
        profile.contains("reject_identity_mismatch = true"),
        "profile must reject identity mismatch"
    );
}

fn read_json(path: &str) -> Value {
    let raw = fs::read_to_string(path).expect("json file should read");
    serde_json::from_str(&raw).expect("json should parse")
}
