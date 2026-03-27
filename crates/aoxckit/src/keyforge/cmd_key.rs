// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::keyforge::cli::{KeyCommand, KeySubcommand};
use crate::keyforge::util::read_text_file;
use aoxcore::identity::{key_bundle::NodeKeyBundleV1, pq_keys};

pub fn handle(command: KeyCommand) -> Result<(), String> {
    match command.command {
        KeySubcommand::Generate => generate(),
        KeySubcommand::InspectBundle { file } => inspect_bundle(&file),
    }
}

fn generate() -> Result<(), String> {
    let (pk, sk) = pq_keys::generate_keypair();

    let public_hex = hex::encode_upper(pq_keys::serialize_public_key(&pk));
    let secret_hex = hex::encode_upper(pq_keys::serialize_secret_key(&sk));
    let fingerprint = pq_keys::fingerprint(&pk);

    let output = serde_json::json!({
        "algorithm": "dilithium3",
        "fingerprint": fingerprint,
        "public_key": public_hex,
        "secret_key": secret_hex
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|e| format!("JSON_SERIALIZE_ERROR: {}", e))?
    );

    Ok(())
}

fn inspect_bundle(file: &str) -> Result<(), String> {
    let data = read_text_file(file)?;
    let bundle = NodeKeyBundleV1::from_json(&data).map_err(|error| error.to_string())?;

    println!(
        "{}",
        serde_json::to_string_pretty(&bundle)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {}", error))?
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_outputs_valid_json() {
        // stdout'u yakalamak birim testlerde zordur, ancak ana fonksiyonun
        // çökmeden (panic olmadan) çalıştığını ve Ok(()) döndüğünü test edebiliriz.
        let result = generate();
        assert!(
            result.is_ok(),
            "Key generation should succeed without errors"
        );
    }

    #[test]
    fn inspect_bundle_rejects_invalid_json() {
        let path = std::env::temp_dir().join("aoxc-invalid-key-bundle.json");
        std::fs::write(&path, "{not-json").expect("temp file write must succeed");

        let result = inspect_bundle(path.to_str().expect("path must be valid utf-8"));

        assert!(result.is_err());

        let _ = std::fs::remove_file(path);
    }
}
