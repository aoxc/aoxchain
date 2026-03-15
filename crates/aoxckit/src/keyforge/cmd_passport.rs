use crate::keyforge::cli::{PassportCommand, PassportSubcommand};
use crate::keyforge::util::read_text_file;
use core::identity::passport::Passport;

pub fn handle(command: PassportCommand) -> Result<(), String> {
    match command.command {
        PassportSubcommand::Inspect { file } => inspect(&file),
    }
}

fn inspect(file: &str) -> Result<(), String> {
    let data = read_text_file(file)?;
    let passport: Passport =
        serde_json::from_str(&data).map_err(|e| format!("PASSPORT_PARSE_ERROR: {}", e))?;

    println!(
        "{}",
        serde_json::to_string_pretty(&passport)
            .map_err(|e| format!("JSON_SERIALIZE_ERROR: {}", e))?
    );

    Ok(())
}
