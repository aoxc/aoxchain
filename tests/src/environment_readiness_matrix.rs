// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use sha2::{Digest, Sha256};
use std::{collections::HashSet, fs, path::Path};

use serde_json::Value;

const ENV_ROOT: &str = "../configs/environments";

#[test]
fn canonical_environment_manifests_keep_unique_identity_and_schema_guards() {
    let env_paths = canonical_environment_paths();

    let mut seen_chain_ids = HashSet::new();
    let mut seen_network_ids = HashSet::new();
    let mut seen_network_serials = HashSet::new();

    for path in env_paths {
        let manifest_path = format!("{path}/manifest.v1.json");
        let manifest = read_json(&manifest_path);

        assert_eq!(
            manifest.get("schema_version").and_then(Value::as_u64),
            Some(1)
        );
        assert_eq!(
            manifest.get("manifest_kind").and_then(Value::as_str),
            Some("aoxc-environment-manifest")
        );
        assert_eq!(
            manifest.get("family_id").and_then(Value::as_u64),
            Some(2626)
        );

        let identity = manifest
            .get("identity")
            .and_then(Value::as_object)
            .expect("manifest identity should exist");

        let chain_id = identity
            .get("chain_id")
            .and_then(Value::as_u64)
            .expect("chain_id should exist");
        let network_id = identity
            .get("network_id")
            .and_then(Value::as_str)
            .expect("network_id should exist");
        let network_serial = identity
            .get("network_serial")
            .and_then(Value::as_str)
            .expect("network_serial should exist");

        assert!(seen_chain_ids.insert(chain_id), "chain_id must be unique");
        assert!(
            seen_network_ids.insert(network_id.to_string()),
            "network_id must be unique"
        );
        assert!(
            seen_network_serials.insert(network_serial.to_string()),
            "network_serial must be unique"
        );

        let runtime_policy = manifest
            .get("runtime_policy")
            .and_then(Value::as_object)
            .expect("runtime_policy should exist");
        assert_eq!(
            runtime_policy
                .get("reject_manifest_genesis_hash_mismatch")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            runtime_policy
                .get("reject_missing_profile")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            runtime_policy
                .get("reject_missing_release_policy")
                .and_then(Value::as_bool),
            Some(true)
        );
    }
}

#[test]
fn canonical_environment_bundle_files_and_genesis_hashes_match_manifest() {
    for path in canonical_environment_paths() {
        let manifest = read_json(&format!("{path}/manifest.v1.json"));
        let bundle = manifest
            .get("bundle")
            .and_then(Value::as_object)
            .expect("bundle section should exist");

        let genesis_file = bundle
            .get("genesis_file")
            .and_then(Value::as_str)
            .expect("genesis_file should exist");
        let genesis_hash_file = bundle
            .get("genesis_hash_file")
            .and_then(Value::as_str)
            .expect("genesis_hash_file should exist");
        let validators_file = bundle
            .get("validators_file")
            .and_then(Value::as_str)
            .expect("validators_file should exist");
        let bootnodes_file = bundle
            .get("bootnodes_file")
            .and_then(Value::as_str)
            .expect("bootnodes_file should exist");
        let profile_file = bundle
            .get("profile_file")
            .and_then(Value::as_str)
            .expect("profile_file should exist");
        let release_policy_file = bundle
            .get("release_policy_file")
            .and_then(Value::as_str)
            .expect("release_policy_file should exist");
        let certificate_file = bundle
            .get("certificate_file")
            .and_then(Value::as_str)
            .expect("certificate_file should exist");

        for required in [
            genesis_file,
            genesis_hash_file,
            validators_file,
            bootnodes_file,
            profile_file,
            release_policy_file,
            certificate_file,
        ] {
            let full_path = format!("{path}/{required}");
            assert!(
                Path::new(&full_path).exists(),
                "required file missing: {full_path}"
            );
        }

        let genesis_bytes =
            fs::read(format!("{path}/{genesis_file}")).expect("genesis payload should be readable");
        let computed = hex::encode(Sha256::digest(genesis_bytes));
        let recorded_line = fs::read_to_string(format!("{path}/{genesis_hash_file}"))
            .expect("genesis hash file should be readable");
        let recorded = recorded_line
            .split_whitespace()
            .next()
            .expect("genesis hash entry should include digest token");

        assert_eq!(
            computed, recorded,
            "genesis hash mismatch for environment path: {path}"
        );
    }
}

#[test]
fn registry_and_binary_policy_keep_testnet_and_mainnet_release_guards_enabled() {
    let registry = fs::read_to_string("../configs/registry/network-registry.toml")
        .expect("network registry should be readable");
    let binary = fs::read_to_string("../configs/registry/binary-compatibility.toml")
        .expect("binary compatibility policy should be readable");

    for required in [
        "allow_duplicate_network_serial = false",
        "allow_duplicate_network_id = false",
        "allow_duplicate_chain_id = false",
        "require_manifest_for_all_networks = true",
        "require_genesis_hash_for_all_networks = true",
        "require_release_policy_for_all_networks = true",
        "require_profile_for_all_networks = true",
        "require_certificate_for_all_networks = true",
    ] {
        assert!(
            registry.contains(required),
            "registry policy missing required guard: {required}"
        );
    }

    for required in [
        "single_binary_multi_network = true",
        "compile_time_network_binding_forbidden = true",
        "reject_chain_id_mismatch = true",
        "reject_network_id_mismatch = true",
        "reject_network_serial_mismatch = true",
        "reject_manifest_genesis_hash_mismatch = true",
        "signature_required = true",
    ] {
        assert!(
            binary.contains(required),
            "binary compatibility policy missing required guard: {required}"
        );
    }
}

fn canonical_environment_paths() -> Vec<String> {
    vec![
        format!("{ENV_ROOT}/mainnet"),
        format!("{ENV_ROOT}/testnet"),
        format!("{ENV_ROOT}/devnet"),
        format!("{ENV_ROOT}/validation"),
        format!("{ENV_ROOT}/localnet"),
        format!("{ENV_ROOT}/sovereign/template"),
    ]
}

fn read_json(path: &str) -> Value {
    let raw = fs::read_to_string(path).expect("json should be readable");
    serde_json::from_str(&raw).expect("json should parse")
}
