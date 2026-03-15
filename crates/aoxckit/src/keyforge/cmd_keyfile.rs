use crate::keyforge::cli::{KeyfileCommand, KeyfileSubcommand};
use crate::keyforge::util::{read_file, read_text_file, write_file, write_text_file};
use aoxcore::identity::keyfile;

pub fn handle(command: KeyfileCommand) -> Result<(), String> {
    match command.command {
        KeyfileSubcommand::Encrypt {
            input,
            output,
            password,
        } => encrypt(&input, &output, &password),
        KeyfileSubcommand::Decrypt {
            input,
            output,
            password,
        } => decrypt(&input, &output, &password),
    }
}

fn encrypt(input: &str, output: &str, password: &str) -> Result<(), String> {
    let plaintext = read_file(input)?;
    let encrypted =
        keyfile::encrypt_key(&plaintext, password).map_err(|error| error.to_string())?;

    write_text_file(output, &encrypted)?;
    println!("encrypted keyfile written to {}", output);

    Ok(())
}

fn decrypt(input: &str, output: &str, password: &str) -> Result<(), String> {
    let serialized = read_text_file(input)?;
    let plaintext =
        keyfile::decrypt_key(&serialized, password).map_err(|error| error.to_string())?;

    write_file(output, &plaintext)?;
    println!("decrypted key material written to {}", output);

    Ok(())
}
