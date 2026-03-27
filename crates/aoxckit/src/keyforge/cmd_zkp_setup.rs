// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::keyforge::cli::{ZkpSetupCommand, ZkpSetupSubcommand};
use crate::keyforge::util::write_text_file;

pub fn handle(command: ZkpSetupCommand) -> Result<(), String> {
    match command.command {
        ZkpSetupSubcommand::Init {
            circuit,
            output,
            powers,
        } => init(&circuit, &output, powers),
    }
}

fn init(circuit: &str, output: &str, powers: u8) -> Result<(), String> {
    if circuit.trim().is_empty() || output.trim().is_empty() || powers == 0 {
        return Err("ZKP_SETUP_INVALID_ARGUMENT".to_string());
    }

    let artifact = serde_json::json!({
        "circuit": circuit,
        "powers_of_tau": powers,
        "status": "trusted-setup-initialized"
    });

    let body = serde_json::to_string_pretty(&artifact)
        .map_err(|error| format!("JSON_SERIALIZE_ERROR: {}", error))?;

    write_text_file(output, &body)?;
    println!("zkp setup artifact written to {}", output);

    Ok(())
}
