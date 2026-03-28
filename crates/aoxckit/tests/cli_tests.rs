// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

fn unique_path(label: &str) -> String {
    std::env::temp_dir()
        .join(format!("aoxckit-cli-{label}-{}.json", std::process::id()))
        .to_string_lossy()
        .into_owned()
}

#[test]
fn test_key_generate_command_produces_public_only_json() {
    let mut cmd = Command::cargo_bin("aoxckit").expect("aoxckit binary must build");

    cmd.arg("key").arg("generate");

    let assert = cmd.assert().success();

    let stdout =
        String::from_utf8(assert.get_output().stdout.clone()).expect("stdout must be valid UTF-8");

    let parsed: Value =
        serde_json::from_str(&stdout).expect("key generate output must be valid JSON");

    assert_eq!(parsed["algorithm"], "dilithium3");
    assert!(parsed.get("fingerprint").is_some());
    assert!(parsed.get("public_key").is_some());
    assert!(
        parsed.get("secret_key").is_none(),
        "public CLI output must not expose secret_key"
    );
}

#[test]
fn test_quorum_evaluate_command_outputs_passed_status() {
    let mut cmd = Command::cargo_bin("aoxckit").expect("aoxckit binary must build");

    cmd.arg("quorum")
        .arg("evaluate")
        .arg("--total")
        .arg("10")
        .arg("--approvals")
        .arg("7")
        .arg("--threshold-bps")
        .arg("6667");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"required_approvals\": 7"))
        .stdout(predicate::str::contains("\"passed\": true"));
}

#[test]
fn test_registry_upsert_then_list_round_trip() {
    let registry_path = unique_path("registry-roundtrip");
    let _ = std::fs::remove_file(&registry_path);

    let mut upsert = Command::cargo_bin("aoxckit").expect("aoxckit binary must build");
    upsert
        .arg("registry")
        .arg("upsert-entry")
        .arg("--registry")
        .arg(&registry_path)
        .arg("--actor-id")
        .arg("actor-1")
        .arg("--status")
        .arg("active")
        .arg("--reason")
        .arg("bootstrap");

    upsert.assert().success();

    let mut list = Command::cargo_bin("aoxckit").expect("aoxckit binary must build");
    list.arg("registry")
        .arg("list")
        .arg("--registry")
        .arg(&registry_path);

    list.assert()
        .success()
        .stdout(predicate::str::contains("\"actor_id\": \"actor-1\""))
        .stdout(predicate::str::contains("\"status\": \"active\""))
        .stdout(predicate::str::contains("\"reason\": \"bootstrap\""));

    let _ = std::fs::remove_file(&registry_path);
}

#[test]
fn test_key_generate_keyfile_then_decrypt_round_trip() {
    let plaintext_path = unique_path("keyfile-plain-input");
    let encrypted_path = unique_path("keyfile-encrypted-output");
    let decrypted_path = unique_path("keyfile-decrypted-output");

    let _ = std::fs::remove_file(&plaintext_path);
    let _ = std::fs::remove_file(&encrypted_path);
    let _ = std::fs::remove_file(&decrypted_path);

    std::fs::write(&plaintext_path, b"super-secret-key-material")
        .expect("plaintext fixture must be written");

    let mut encrypt = Command::cargo_bin("aoxckit").expect("aoxckit binary must build");
    encrypt
        .arg("keyfile")
        .arg("encrypt")
        .arg("--input")
        .arg(&plaintext_path)
        .arg("--output")
        .arg(&encrypted_path)
        .arg("--password")
        .arg("Correct#2026!");

    encrypt
        .assert()
        .success()
        .stdout(predicate::str::contains("encrypted keyfile written to"));

    let mut decrypt = Command::cargo_bin("aoxckit").expect("aoxckit binary must build");
    decrypt
        .arg("keyfile")
        .arg("decrypt")
        .arg("--input")
        .arg(&encrypted_path)
        .arg("--output")
        .arg(&decrypted_path)
        .arg("--password")
        .arg("Correct#2026!")
        .arg("--allow-plaintext-output");

    decrypt.assert().success().stdout(predicate::str::contains(
        "decrypted key material written to",
    ));

    let recovered = std::fs::read(&decrypted_path).expect("decrypted output file must be readable");
    assert_eq!(recovered, b"super-secret-key-material");

    let _ = std::fs::remove_file(&plaintext_path);
    let _ = std::fs::remove_file(&encrypted_path);
    let _ = std::fs::remove_file(&decrypted_path);
}

#[test]
fn test_keyfile_decrypt_requires_explicit_plaintext_acknowledgement() {
    let plaintext_path = unique_path("keyfile-no-ack-plain");
    let encrypted_path = unique_path("keyfile-no-ack-encrypted");
    let decrypted_path = unique_path("keyfile-no-ack-output");

    let _ = std::fs::remove_file(&plaintext_path);
    let _ = std::fs::remove_file(&encrypted_path);
    let _ = std::fs::remove_file(&decrypted_path);

    std::fs::write(&plaintext_path, b"secret-material").expect("plaintext fixture must be written");

    let mut encrypt = Command::cargo_bin("aoxckit").expect("aoxckit binary must build");
    encrypt
        .arg("keyfile")
        .arg("encrypt")
        .arg("--input")
        .arg(&plaintext_path)
        .arg("--output")
        .arg(&encrypted_path)
        .arg("--password")
        .arg("Correct#2026!");

    encrypt.assert().success();

    let mut decrypt = Command::cargo_bin("aoxckit").expect("aoxckit binary must build");
    decrypt
        .arg("keyfile")
        .arg("decrypt")
        .arg("--input")
        .arg(&encrypted_path)
        .arg("--output")
        .arg(&decrypted_path)
        .arg("--password")
        .arg("Correct#2026!");

    decrypt.assert().failure().stderr(predicate::str::contains(
        "KEYFILE_PLAINTEXT_OUTPUT_NOT_ACKNOWLEDGED",
    ));

    let _ = std::fs::remove_file(&plaintext_path);
    let _ = std::fs::remove_file(&encrypted_path);
    let _ = std::fs::remove_file(&decrypted_path);
}

#[test]
fn test_registry_upsert_rejects_invalid_status() {
    let registry_path = unique_path("registry-invalid-status");
    let _ = std::fs::remove_file(&registry_path);

    let mut cmd = Command::cargo_bin("aoxckit").expect("aoxckit binary must build");
    cmd.arg("registry")
        .arg("upsert-entry")
        .arg("--registry")
        .arg(&registry_path)
        .arg("--actor-id")
        .arg("actor-1")
        .arg("--status")
        .arg("pending");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("REGISTRY_STATUS_INVALID"));

    let _ = std::fs::remove_file(&registry_path);
}

#[test]
fn test_quorum_evaluate_rejects_invalid_threshold() {
    let mut cmd = Command::cargo_bin("aoxckit").expect("aoxckit binary must build");

    cmd.arg("quorum")
        .arg("evaluate")
        .arg("--total")
        .arg("10")
        .arg("--approvals")
        .arg("7")
        .arg("--threshold-bps")
        .arg("0");

    cmd.assert().failure();
}
