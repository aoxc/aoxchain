// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::keyforge::cli::{KeyCommand, KeySubcommand, OutputFormat};
use crate::keyforge::util::read_text_file;
use aoxcore::identity::key_bundle::NodeKeyBundleV1;
use aoxcore::identity::pq_keys;
use serde::Serialize;

/// Dispatches supported `keyforge` subcommands.
///
/// Design note:
/// - This dispatcher MUST remain strictly aligned with the canonical
///   `KeySubcommand` definition declared in `cli.rs`.
/// - Any CLI contract change at the enum layer MUST be reflected here
///   immediately to preserve compile-time correctness and operational clarity.
/// - Unsupported variants are rejected explicitly instead of being handled
///   implicitly, which provides a clearer operator-facing failure mode.
pub fn handle(command: KeyCommand) -> Result<(), String> {
    match command.command {
        KeySubcommand::GeneratePublic { format } => generate_public(format),
        KeySubcommand::InspectBundle { file, .. } => inspect_bundle(&file),
        other => Err(format!(
            "UNSUPPORTED_KEY_SUBCOMMAND: {:?}. The command dispatcher is not wired for this variant.",
            other
        )),
    }
}

#[derive(Debug, Serialize)]
struct PublicKeyGenerateOutput {
    algorithm: String,
    fingerprint: String,
    public_key: String,
}

/// Generates a Dilithium3 keypair and emits only the public representation.
///
/// Security boundary:
/// - This operation must never print secret key material.
/// - Output remains public-safe for operator terminals and automation logs.
fn generate_public(format: OutputFormat) -> Result<(), String> {
    let (public_key, _secret_key) = pq_keys::generate_keypair();
    let output = PublicKeyGenerateOutput {
        algorithm: "dilithium3".to_string(),
        fingerprint: pq_keys::fingerprint(&public_key),
        public_key: pq_keys::serialize_public_key_hex(&public_key),
    };

    let body = serialize_output(&output, format)?;
    println!("{}", body);
    Ok(())
}

/// Reads, validates, and reprints a canonical AOXC node key bundle.
///
/// Validation contract:
/// - The input file MUST be readable as UTF-8 text.
/// - The payload MUST deserialize into `NodeKeyBundleV1`.
/// - The decoded bundle MUST satisfy the bundle's semantic validation rules.
/// - Only validated bundle content is emitted back to the operator.
fn inspect_bundle(file: &str) -> Result<(), String> {
    let bundle = load_bundle_from_file(file)?;
    let body = serialize_pretty_json(&bundle)?;
    println!("{}", body);
    Ok(())
}

/// Loads and validates a node key bundle from disk.
///
/// Failure model:
/// - I/O failures are surfaced as string errors from the underlying reader.
/// - Deserialization and semantic validation failures are normalized into
///   deterministic string errors suitable for CLI presentation.
fn load_bundle_from_file(file: &str) -> Result<NodeKeyBundleV1, String> {
    let data = read_text_file(file)?;
    NodeKeyBundleV1::from_json(&data).map_err(|error| error.to_string())
}

/// Serializes a value into canonical pretty JSON for operator-facing output.
///
/// Serialization errors are wrapped with a stable prefix to support
/// downstream log inspection and troubleshooting workflows.
fn serialize_pretty_json<T>(value: &T) -> Result<String, String>
where
    T: Serialize,
{
    serde_json::to_string_pretty(value).map_err(|error| format!("JSON_SERIALIZE_ERROR: {}", error))
}

fn serialize_output<T>(value: &T, format: OutputFormat) -> Result<String, String>
where
    T: Serialize,
{
    match format {
        OutputFormat::Json => {
            serde_json::to_string(value).map_err(|error| format!("JSON_SERIALIZE_ERROR: {}", error))
        }
        OutputFormat::PrettyJson => serde_json::to_string_pretty(value)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {}", error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoxcore::identity::{
        key_bundle::{AOXC_PUBLIC_KEY_ENCODING, CryptoProfile, NodeKeyBundleV1},
        key_engine::{KeyEngine, MASTER_SEED_LEN},
        keyfile::encrypt_key_to_envelope,
    };

    fn make_bundle_json() -> String {
        let engine = KeyEngine::from_seed([0x41; MASTER_SEED_LEN]);
        let envelope = encrypt_key_to_envelope(engine.master_seed(), "Test#2026!")
            .expect("encryption must succeed");

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
    fn load_bundle_from_file_accepts_valid_bundle_json() {
        let path =
            std::env::temp_dir().join(format!("aoxc-valid-key-bundle-{}.json", std::process::id()));

        let body = make_bundle_json();
        std::fs::write(&path, body).expect("temp bundle file write must succeed");

        let bundle = load_bundle_from_file(path.to_str().expect("path must be valid UTF-8"))
            .expect("valid bundle JSON must load successfully");

        assert_eq!(bundle.version, 2);
        assert_eq!(bundle.profile, "testnet");
        assert_eq!(bundle.keys.len(), 6);
        assert!(
            bundle
                .keys
                .iter()
                .all(|record| record.public_key_encoding == AOXC_PUBLIC_KEY_ENCODING)
        );

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
            serde_json::to_string_pretty(&invalid_bundle)
                .expect("fixture serialization must succeed"),
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
    fn serialize_pretty_json_returns_valid_json_document() {
        let body = serialize_pretty_json(&serde_json::json!({
            "status": "ok"
        }))
        .expect("JSON serialization must succeed");

        let parsed: serde_json::Value =
            serde_json::from_str(&body).expect("output must be valid JSON");

        assert_eq!(parsed["status"], "ok");
    }

    #[test]
    fn generate_public_output_has_expected_shape() {
        let (public_key, _secret_key) = pq_keys::generate_keypair();
        let output = PublicKeyGenerateOutput {
            algorithm: "dilithium3".to_string(),
            fingerprint: pq_keys::fingerprint(&public_key),
            public_key: pq_keys::serialize_public_key_hex(&public_key),
        };

        let body = serialize_output(&output, OutputFormat::PrettyJson)
            .expect("public output serialization must succeed");
        let parsed: serde_json::Value =
            serde_json::from_str(&body).expect("serialized output must be valid JSON");

        assert_eq!(parsed["algorithm"], "dilithium3");
        assert!(parsed.get("fingerprint").is_some());
        assert!(parsed.get("public_key").is_some());
        assert!(
            parsed.get("secret_key").is_none(),
            "public output must never expose secret_key"
        );
    }
}
