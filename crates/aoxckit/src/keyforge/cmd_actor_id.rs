// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::keyforge::cli::{ActorIdCommand, ActorIdSubcommand};
use aoxcore::identity::actor_id;

pub fn handle(command: ActorIdCommand) -> Result<(), String> {
    match command.command {
        ActorIdSubcommand::Generate { pubkey, role, zone } => generate(&pubkey, &role, &zone),
    }
}

fn generate(pubkey_hex: &str, role: &str, zone: &str) -> Result<(), String> {
    let pubkey = hex::decode(pubkey_hex).map_err(|_| "PUBKEY_HEX_INVALID".to_string())?;

    let actor_id = actor_id::generate_actor_id(&pubkey, role, zone).map_err(|e| e.to_string())?;

    let output = serde_json::json!({
        "actor_id": actor_id,
        "role": role,
        "zone": zone
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|e| format!("JSON_SERIALIZE_ERROR: {}", e))?
    );

    Ok(())
}
