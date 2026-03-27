// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

// crates/aoxckit/tests/cli_tests.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_key_generate_command_produces_json() {
    // "aoxckit" derlenmiş binary'sini çağır
    let mut cmd = Command::cargo_bin("aoxckit").unwrap();

    // "key generate" argümanlarını yolla
    cmd.arg("key").arg("generate");

    // Beklenen sonuçlar:
    // 1. Komut başarıyla (0 koduyla) çalışmalı
    // 2. Çıktı "dilithium3" kelimesini içermeli
    // 3. Çıktı "public_key" ve "secret_key" alanlarını içermeli
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"algorithm\": \"dilithium3\""))
        .stdout(predicate::str::contains("\"public_key\":"))
        .stdout(predicate::str::contains("\"secret_key\":"));
}

#[test]
fn test_quorum_evaluate_command_outputs_passed_status() {
    let mut cmd = Command::cargo_bin("aoxckit").unwrap();
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
    let pid = std::process::id();
    let registry_path = std::env::temp_dir()
        .join(format!("aoxckit-cli-registry-{pid}.json"))
        .to_string_lossy()
        .into_owned();
    let _ = std::fs::remove_file(&registry_path);

    let mut upsert = Command::cargo_bin("aoxckit").unwrap();
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

    let mut list = Command::cargo_bin("aoxckit").unwrap();
    list.arg("registry")
        .arg("list")
        .arg("--registry")
        .arg(&registry_path);
    list.assert()
        .success()
        .stdout(predicate::str::contains("\"actor_id\": \"actor-1\""))
        .stdout(predicate::str::contains("\"status\": \"active\""));

    let _ = std::fs::remove_file(&registry_path);
}
