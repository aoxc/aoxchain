// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::keyforge::cli::{KeyCommand, KeySubcommand};
use crate::keyforge::util::read_text_file;
use aoxcore::identity::{key_bundle::NodeKeyBundleV1, pq_keys};
use serde::Serialize;

/// Canonical public-only key generation response emitted by the AOXC keyforge CLI.
///
/// Security posture:
/// - This response is intentionally restricted to non-sensitive material.
/// - Private key material MUST NOT be emitted to stdout, logs, or shell-visible surfaces.
/// - Secret custody is expected to occur through dedicated encrypted persistence flows,
///   not through operator terminal output.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct GeneratedKeyOutput {
    algorithm: &'static str,
    fingerprint: String,
    public_key: String,
}

pub fn handle(command: KeyCommand) -> Result<(), String> {
    match command.command {
        KeySubcommand::Generate => generate(),
        KeySubcommand::InspectBundle { file } => inspect_bundle(&file),
    }
}

/// Generates a fresh post-quantum keypair and emits only the public operational view.
///
/// Security rationale:
/// - The generated secret key is intentionally not serialized or printed.
/// - Emitting plaintext private material through stdout would create a high-risk
///   leakage channel across shells, CI logs, remote sessions, and log shippers.
fn generate() -> Result<(), String> {
    let output = build_generate_output();
    let body = serialize_pretty_json(&output)?;
    println!("{}", body);
    Ok(())
}

/// Builds the canonical public-only generation response.
///
/// This helper is intentionally pure from the caller perspective and improves
/// testability by separating cryptographic generation from stdout emission.
fn build_generate_output() -> GeneratedKeyOutput {
    let (public_key, _secret_key) = pq_keys::generate_keypair();

    GeneratedKeyOutput {
        algorithm: "dilithium3",
        fingerprint: pq_keys::fingerprint(&public_key),
        public_key: hex::encode_upper(pq_keys::serialize_public_key(&public_key)),
    }
}

/// Reads, validates, and reprints a canonical AOXC node key bundle.
///
/// Validation contract:
/// - The input file must be readable as UTF-8 text.
/// - The payload must decode into `NodeKeyBundleV1`.
/// - The decoded bundle must satisfy the bundle's semantic validation rules.
fn inspect_bundle(file: &str) -> Result<(), String> {
    let bundle = load_bundle_from_file(file)?;
    let body = serialize_pretty_json(&bundle)?;
    println!("{}", body);
    Ok(())
}

/// Loads and validates a node key bundle from disk.
fn load_bundle_from_file(file: &str) -> Result<NodeKeyBundleV1, String> {
    let data = read_text_file(file)?;
    NodeKeyBundleV1::from_json(&data).map_err(|error| error.to_string())
}

/// Serializes a value into canonical pretty JSON for operator-facing output.
fn serialize_pretty_json<T>(value: &T) -> Result<String, String>
where
    T: Serialize,
{
    serde_json::to_string_pretty(value).map_err(|error| format!("JSON_SERIALIZE_ERROR: {}", error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoxcore::identity::{
        key_bundle::{CryptoProfile, NodeKeyBundleV1, AOXC_PUBLIC_KEY_ENCODING},
        key_engine::{KeyEngine, MASTER_SEED_LEN},
        keyfile::encrypt_key_to_envelope,
    };
    use serde_json::Value;

    fn make_bundle_json() -> String {
        let engine = KeyEngine::from_seed([0x41; MASTER_SEED_LEN]);
        let envelope =
            encrypt_key_to_envelope(engine.master_seed(), "Test#2026!").expect("encryption must succeed");

        let bundle = NodeKeyBundleV1::generate(
            "validator-01",
            "testnet",
            "2026-01-01T00:00:00Z".to_string(),
            CryptoProfile::HybridEd25519Dilithium3,
            &engine,
            envelope,
        )
        .expect("bundle generation must succeed");

        bundle.to_json().expect("bundle JSON encoding must succeed")
    }

    #[test]
    fn build_generate_output_returns_public_only_payload() {
        let output = build_generate_output();

        assert_eq!(output.algorithm, "dilithium3");
        assert!(!output.fingerprint.is_empty(), "fingerprint must not be empty");
        assert!(!output.public_key.is_empty(), "public key must not be empty");
    }

    #[test]
    fn build_generate_output_public_key_is_valid_uppercase_hex() {
        let output = build_generate_output();

        assert_eq!(
            output.public_key,
            output.public_key.to_ascii_uppercase(),
            "public key must be encoded as uppercase hex"
        );

        let decoded = hex::decode(&output.public_key).expect("public key must decode from hex");
        assert!(
            !decoded.is_empty(),
            "decoded public key bytes must not be empty"
        );
    }

    #[test]
    fn serialize_pretty_json_for_generated_output_contains_no_secret_key_field() {
        let output = build_generate_output();
        let body = serialize_pretty_json(&output).expect("JSON serialization must succeed");

        let parsed: Value = serde_json::from_str(&body).expect("serialized output must be valid JSON");

        assert_eq!(parsed["algorithm"], "dilithium3");
        assert!(parsed.get("fingerprint").is_some());
        assert!(parsed.get("public_key").is_some());
        assert!(
            parsed.get("secret_key").is_none(),
            "public CLI output must not contain secret_key"
        );
    }

    #[test]
    fn serialize_pretty_json_returns_valid_json_document() {
        let output = build_generate_output();
        let body = serialize_pretty_json(&output).expect("JSON serialization must succeed");

        let parsed: Value = serde_json::from_str(&body).expect("output must be valid JSON");

        assert_eq!(parsed["algorithm"], "dilithium3");
        assert!(parsed["fingerprint"].as_str().is_some());
        assert!(parsed["public_key"].as_str().is_some());
    }

    #[test]
    fn load_bundle_from_file_accepts_valid_bundle_json() {
        let path = std::env::temp_dir().join(format!(
            "aoxc-valid-key-bundle-{}.json",
            std::process::id()
        ));

        let body = make_bundle_json();
        std::fs::write(&path, body).expect("temp bundle file write must succeed");

        let bundle = load_bundle_from_file(path.to_str().expect("path must be valid UTF-8"))
            .expect("valid bundle JSON must load successfully");

        assert_eq!(bundle.version, 2);
        assert_eq!(bundle.profile, "testnet");
        assert_eq!(bundle.keys.len(), 6);
        assert!(bundle
            .keys
            .iter()
            .all(|record| record.public_key_encoding == AOXC_PUBLIC_KEY_ENCODING));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn inspect_bundle_rejects_invalid_json() {
        let path = std::env::temp_dir().join(format!(
            "aoxc-invalid-key-bundle-{}.json",
            std::process::id()
        ));

        std::fs::write(&path, "{not-json").expect("temp file write must succeed");

        let result = inspect_bundle(path.to_str().expect("path must be valid UTF-8"));

        assert!(result.is_err(), "invalid JSON input must be rejected");

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn load_bundle_from_file_rejects_semantically_invalid_bundle() {
        let path = std::env::temp_dir().join(format!(
            "aoxc-semantic-invalid-key-bundle-{}.json",
            std::process::id()
        ));

        let invalid_bundle = serde_json::json!({
            "version": 2,
            "node_name": "validator-02",
            "profile": "testnet",
            "created_at": "2026-01-01T00:00:00Z",
            "crypto_profile": "classic-ed25519",
            "custody_model": "encrypted-root-seed-envelope",
            "engine_fingerprint": "ABCDEF0123456789ABCDEF0123456789",
            "bundle_fingerprint": "ABCDEF0123456789ABCDEF0123456789",
            "encrypted_root_seed": {
                "version": 1,
                "kdf": {
                    "algorithm": "argon2id",
                    "memory_cost_kib": 65536,
                    "time_cost": 3,
                    "parallelism": 1,
                    "output_len": 32
                },
                "salt_b64": "AAAAAAAAAAAAAAAAAAAAAA==",
                "nonce_b64": "AAAAAAAAAAAAAAAA",
                "ciphertext_b64": "AQ=="
            },
            "keys": []
        });

        std::fs::write(
            &path,
            serde_json::to_string_pretty(&invalid_bundle).expect("fixture serialization must succeed"),
        )
        .expect("temp file write must succeed");

        let result = load_bundle_from_file(path.to_str().expect("path must be valid UTF-8"));

        assert!(
            result.is_err(),
            "semantically invalid bundle must be rejected"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn generate_executes_successfully() {
        let result = generate();
        assert!(result.is_ok(), "generate must complete successfully");
    }
}
