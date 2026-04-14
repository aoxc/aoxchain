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

    assert_eq!(read_chain_id(&metadata), Some(chain_id));
    assert_eq!(read_network_id(&metadata), Some(network_id));

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

        let role = read_validator_role(entry).expect("validator role must exist");
        if role.contains("seed") {
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

#[test]
fn testnet_public_endpoints_and_topology_remain_local_operator_compatible() {
    let metadata = read_json(&format!("{TESTNET_DIR}/network-metadata.json"));

    let rpc = metadata
        .get("rpc")
        .and_then(Value::as_object)
        .expect("rpc object must exist");
    let primary = rpc
        .get("primary")
        .and_then(Value::as_str)
        .expect("primary rpc must exist");
    let secondary = rpc
        .get("secondary")
        .and_then(Value::as_str)
        .expect("secondary rpc must exist");
    let ws = rpc
        .get("ws")
        .and_then(Value::as_str)
        .expect("websocket rpc must exist");

    assert_local_endpoint(primary, "https://", "primary rpc");
    assert_local_endpoint(secondary, "https://", "secondary rpc");
    assert_local_endpoint(ws, "wss://", "websocket rpc");
    assert_ne!(
        primary, secondary,
        "primary and secondary RPC endpoints must not be identical"
    );

    let public_endpoints = metadata
        .get("public_endpoints")
        .and_then(Value::as_object)
        .expect("public_endpoints object must exist");

    for key in ["faucet", "explorer", "status"] {
        let endpoint = public_endpoints
            .get(key)
            .and_then(Value::as_str)
            .expect("public endpoint must exist");
        assert_local_endpoint(endpoint, "http://", key);
    }

    let topology = metadata
        .get("topology")
        .and_then(Value::as_array)
        .expect("topology must exist");
    assert!(
        topology.len() >= 3,
        "testnet topology must include at least three declared nodes"
    );

    let mut rpc_public_nodes = 0usize;
    for entry in topology {
        let roles = entry
            .get("roles")
            .and_then(Value::as_array)
            .expect("roles list must exist");
        let has_rpc_role = roles
            .iter()
            .filter_map(Value::as_str)
            .any(|role| role == "rpc");
        let rpc_public = entry
            .get("rpc_public")
            .and_then(Value::as_bool)
            .expect("rpc_public flag must exist");

        if rpc_public {
            rpc_public_nodes += 1;
        }

        assert_eq!(
            has_rpc_role, rpc_public,
            "rpc_public should match explicit rpc role assignment"
        );
    }

    assert!(
        rpc_public_nodes >= 1,
        "at least one rpc node must be marked public"
    );
}

fn read_json(path: &str) -> Value {
    let raw = fs::read_to_string(path).expect("json file should read");
    serde_json::from_str(&raw).expect("json should parse")
}

fn read_chain_id(metadata: &Value) -> Option<u64> {
    metadata
        .get("chain_id")
        .and_then(Value::as_u64)
        .or_else(|| {
            metadata
                .get("identity")
                .and_then(Value::as_object)
                .and_then(|identity| identity.get("chain_id"))
                .and_then(Value::as_u64)
        })
}

fn read_network_id(metadata: &Value) -> Option<&str> {
    metadata
        .get("network_id")
        .and_then(Value::as_str)
        .or_else(|| {
            metadata
                .get("identity")
                .and_then(Value::as_object)
                .and_then(|identity| identity.get("network_id"))
                .and_then(Value::as_str)
        })
}

fn read_validator_role(entry: &Value) -> Option<&str> {
    let explicit = entry.get("role").and_then(Value::as_str);
    let operational = entry.get("operational_role").and_then(Value::as_str);

    if let (Some(role), Some(operational_role)) = (explicit, operational) {
        assert_eq!(
            role, operational_role,
            "validator role and operational_role must remain consistent"
        );
    }

    explicit.or(operational)
}

fn assert_local_endpoint(endpoint: &str, expected_scheme: &str, endpoint_name: &str) {
    assert!(
        endpoint.starts_with(expected_scheme),
        "{endpoint_name} endpoint must use {expected_scheme}"
    );

    let normalized = endpoint.to_ascii_lowercase();
    assert!(
        normalized.contains("127.0.0.1") || normalized.contains("localhost"),
        "{endpoint_name} endpoint must target a local loopback host"
    );
}
