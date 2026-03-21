use crate::keyforge::cli::{KeyCommand, KeySubcommand};
use aoxcore::identity::pq_keys;

pub fn handle(command: KeyCommand) -> Result<(), String> {
    match command.command {
        KeySubcommand::Generate => generate(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_outputs_valid_json() {
        // stdout'u yakalamak birim testlerde zordur, ancak ana fonksiyonun
        // çökmeden (panic olmadan) çalıştığını ve Ok(()) döndüğünü test edebiliriz.
        let result = generate();
        assert!(result.is_ok(), "Key generation should succeed without errors");
    }
}
