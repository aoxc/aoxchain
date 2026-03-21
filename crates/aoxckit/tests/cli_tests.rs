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
