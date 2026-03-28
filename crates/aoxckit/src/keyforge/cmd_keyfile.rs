// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::keyforge::cli::{KeyfileCommand, KeyfileSubcommand, PasswordSourceArgs};
use crate::keyforge::util::{read_file, read_text_file, write_file, write_text_file};
use aoxcore::identity::keyfile;
use std::path::Path;

/// Handles AOXC keyfile subcommands.
///
/// Security posture:
/// - encryption is safe-by-default,
/// - plaintext decryption output requires explicit operator acknowledgement,
/// - output overwrite is blocked unless the operator opts in with `--force`.
pub fn handle(command: KeyfileCommand) -> Result<(), String> {
    match command.command {
        KeyfileSubcommand::Encrypt {
            input,
            output,
            password,
            force,
        } => encrypt(&input, &output, &password, force),
        KeyfileSubcommand::Decrypt {
            input,
            output,
            password,
            allow_plaintext_output,
            force,
        } => decrypt(
            &input,
            &output,
            &password,
            allow_plaintext_output,
            force,
        ),
    }
}

/// Encrypts plaintext key material into a canonical AOXC keyfile envelope.
///
/// Security and operational policy:
/// - input must exist and be readable,
/// - password must be supplied from exactly one supported source,
/// - output overwrite is rejected unless `force == true`,
/// - the resulting serialized keyfile must pass structural validation before persistence.
fn encrypt(
    input: &str,
    output: &str,
    password_source: &PasswordSourceArgs,
    force: bool,
) -> Result<(), String> {
    let normalized_input = normalize_required_text(input, "input")?;
    let normalized_output = normalize_required_text(output, "output")?;
    reject_existing_output_path(&normalized_output, force)?;

    let password = resolve_password(password_source)?;
    let plaintext = read_file(&normalized_input)?;

    if plaintext.is_empty() {
        return Err("KEYFILE_INPUT_EMPTY".to_string());
    }

    let encrypted = keyfile::encrypt_key(&plaintext, &password).map_err(map_keyfile_error)?;

    if !keyfile::is_valid_keyfile(&encrypted) {
        return Err("KEYFILE_ENCRYPT_OUTPUT_INVALID".to_string());
    }

    write_text_file(&normalized_output, &encrypted)?;
    println!("encrypted keyfile written to {}", normalized_output);

    Ok(())
}

/// Decrypts a canonical AOXC keyfile into plaintext output.
///
/// Danger policy:
/// - plaintext secret output is blocked unless `allow_plaintext_output == true`,
/// - output overwrite is rejected unless `force == true`,
/// - callers are required to opt into disk re-materialization explicitly.
fn decrypt(
    input: &str,
    output: &str,
    password_source: &PasswordSourceArgs,
    allow_plaintext_output: bool,
    force: bool,
) -> Result<(), String> {
    if !allow_plaintext_output {
        return Err("KEYFILE_PLAINTEXT_OUTPUT_NOT_ACKNOWLEDGED".to_string());
    }

    let normalized_input = normalize_required_text(input, "input")?;
    let normalized_output = normalize_required_text(output, "output")?;
    reject_existing_output_path(&normalized_output, force)?;

    let password = resolve_password(password_source)?;
    let serialized = read_text_file(&normalized_input)?;

    if !keyfile::is_valid_keyfile(&serialized) {
        return Err("KEYFILE_INPUT_INVALID".to_string());
    }

    let plaintext = keyfile::decrypt_key(&serialized, &password).map_err(map_keyfile_error)?;

    if plaintext.is_empty() {
        return Err("KEYFILE_DECRYPT_OUTPUT_EMPTY".to_string());
    }

    write_file(&normalized_output, &plaintext)?;
    println!("decrypted key material written to {}", normalized_output);

    Ok(())
}

/// Resolves the password from the canonical CLI password source contract.
///
/// Contract:
/// - exactly one password source must be populated,
/// - inline passwords must not be blank,
/// - password-file contents are trimmed for surrounding whitespace only,
/// - blank file-backed passwords are rejected.
fn resolve_password(password_source: &PasswordSourceArgs) -> Result<String, String> {
    match (
        password_source.password.as_deref(),
        password_source.password_file.as_deref(),
    ) {
        (Some(password), None) => normalize_required_text(password, "password"),
        (None, Some(path)) => {
            let normalized_path = normalize_required_text(path, "password_file")?;
            let content = read_text_file(&normalized_path)?;
            normalize_required_text(&content, "password_file_content")
        }
        (Some(_), Some(_)) => Err("KEYFILE_PASSWORD_SOURCE_CONFLICT".to_string()),
        (None, None) => Err("KEYFILE_PASSWORD_SOURCE_MISSING".to_string()),
    }
}

/// Rejects writes to an existing output path unless force mode is enabled.
fn reject_existing_output_path(path: &str, force: bool) -> Result<(), String> {
    if Path::new(path).exists() && !force {
        return Err("OUTPUT_FILE_EXISTS_USE_FORCE".to_string());
    }

    Ok(())
}

/// Enforces non-blank operator-facing text input.
///
/// Policy:
/// - trims leading and trailing whitespace,
/// - rejects whitespace-only values,
/// - returns normalized content.
fn normalize_required_text(value: &str, field: &str) -> Result<String, String> {
    let normalized = value.trim();

    if normalized.is_empty() {
        return Err(format!(
            "INVALID_ARGUMENT: {} must not be blank",
            field
        ));
    }

    Ok(normalized.to_string())
}

/// Maps keyfile-domain errors into stable symbolic CLI-facing error codes.
fn map_keyfile_error(error: keyfile::KeyfileError) -> String {
    error.code().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn unique_path(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("aoxc-{}-{}.tmp", label, std::process::id()))
    }

    fn inline_password_source(password: &str) -> PasswordSourceArgs {
        PasswordSourceArgs {
            password: Some(password.to_string()),
            password_file: None,
        }
    }

    fn file_password_source(path: &str) -> PasswordSourceArgs {
        PasswordSourceArgs {
            password: None,
            password_file: Some(path.to_string()),
        }
    }

    #[test]
    fn resolve_password_accepts_inline_password() {
        let source = inline_password_source("Test#2026!");
        let password = resolve_password(&source).expect("inline password must resolve");

        assert_eq!(password, "Test#2026!");
    }

    #[test]
    fn resolve_password_accepts_password_file() {
        let password_file = unique_path("keyfile-password-source");
        fs::write(&password_file, "File#2026!\n").expect("password file must be written");

        let source = file_password_source(
            password_file
                .to_str()
                .expect("password file path must be valid UTF-8"),
        );

        let password = resolve_password(&source).expect("password file must resolve");
        assert_eq!(password, "File#2026!");

        let _ = fs::remove_file(password_file);
    }

    #[test]
    fn resolve_password_rejects_missing_source() {
        let source = PasswordSourceArgs {
            password: None,
            password_file: None,
        };

        let result = resolve_password(&source);
        assert_eq!(result, Err("KEYFILE_PASSWORD_SOURCE_MISSING".to_string()));
    }

    #[test]
    fn resolve_password_rejects_conflicting_sources() {
        let source = PasswordSourceArgs {
            password: Some("Inline#2026!".to_string()),
            password_file: Some("pw.txt".to_string()),
        };

        let result = resolve_password(&source);
        assert_eq!(result, Err("KEYFILE_PASSWORD_SOURCE_CONFLICT".to_string()));
    }

    #[test]
    fn reject_existing_output_path_blocks_overwrite_without_force() {
        let path = unique_path("keyfile-overwrite-block");
        fs::write(&path, "existing").expect("fixture file must be written");

        let result = reject_existing_output_path(
            path.to_str().expect("path must be valid UTF-8"),
            false,
        );

        assert_eq!(result, Err("OUTPUT_FILE_EXISTS_USE_FORCE".to_string()));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn reject_existing_output_path_allows_overwrite_with_force() {
        let path = unique_path("keyfile-overwrite-force");
        fs::write(&path, "existing").expect("fixture file must be written");

        let result = reject_existing_output_path(
            path.to_str().expect("path must be valid UTF-8"),
            true,
        );

        assert!(result.is_ok());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn encrypt_and_decrypt_roundtrip_succeeds() {
        let input_path = unique_path("keyfile-roundtrip-input");
        let encrypted_path = unique_path("keyfile-roundtrip-encrypted");
        let decrypted_path = unique_path("keyfile-roundtrip-decrypted");

        let plaintext = b"super-secret-key-material";
        fs::write(&input_path, plaintext).expect("plaintext input must be written");

        let password = inline_password_source("Correct#2026!");

        encrypt(
            input_path.to_str().expect("input path must be valid UTF-8"),
            encrypted_path
                .to_str()
                .expect("encrypted path must be valid UTF-8"),
            &password,
            false,
        )
        .expect("encryption must succeed");

        decrypt(
            encrypted_path
                .to_str()
                .expect("encrypted path must be valid UTF-8"),
            decrypted_path
                .to_str()
                .expect("decrypted path must be valid UTF-8"),
            &password,
            true,
            false,
        )
        .expect("decryption must succeed");

        let recovered = fs::read(&decrypted_path).expect("decrypted output must be readable");
        assert_eq!(recovered, plaintext);

        let _ = fs::remove_file(input_path);
        let _ = fs::remove_file(encrypted_path);
        let _ = fs::remove_file(decrypted_path);
    }

    #[test]
    fn encrypt_rejects_empty_plaintext_input() {
        let input_path = unique_path("keyfile-empty-input");
        let encrypted_path = unique_path("keyfile-empty-encrypted");

        fs::write(&input_path, []).expect("empty fixture file must be written");

        let result = encrypt(
            input_path.to_str().expect("input path must be valid UTF-8"),
            encrypted_path
                .to_str()
                .expect("encrypted path must be valid UTF-8"),
            &inline_password_source("Correct#2026!"),
            false,
        );

        assert_eq!(result, Err("KEYFILE_INPUT_EMPTY".to_string()));

        let _ = fs::remove_file(input_path);
        let _ = fs::remove_file(encrypted_path);
    }

    #[test]
    fn decrypt_requires_explicit_plaintext_output_acknowledgement() {
        let encrypted_path = unique_path("keyfile-decrypt-no-ack");

        let encrypted = keyfile::encrypt_key(b"secret-material", "Correct#2026!")
            .expect("encrypted fixture must be created");
        fs::write(&encrypted_path, encrypted).expect("encrypted fixture file must be written");

        let result = decrypt(
            encrypted_path
                .to_str()
                .expect("encrypted path must be valid UTF-8"),
            unique_path("keyfile-decrypt-no-ack-output")
                .to_str()
                .expect("output path must be valid UTF-8"),
            &inline_password_source("Correct#2026!"),
            false,
            false,
        );

        assert_eq!(
            result,
            Err("KEYFILE_PLAINTEXT_OUTPUT_NOT_ACKNOWLEDGED".to_string())
        );

        let _ = fs::remove_file(encrypted_path);
    }

    #[test]
    fn decrypt_rejects_invalid_keyfile_input() {
        let input_path = unique_path("keyfile-invalid-input");
        let output_path = unique_path("keyfile-invalid-output");

        fs::write(&input_path, "not-a-keyfile").expect("invalid fixture file must be written");

        let result = decrypt(
            input_path.to_str().expect("input path must be valid UTF-8"),
            output_path.to_str().expect("output path must be valid UTF-8"),
            &inline_password_source("Correct#2026!"),
            true,
            false,
        );

        assert_eq!(result, Err("KEYFILE_INPUT_INVALID".to_string()));

        let _ = fs::remove_file(input_path);
        let _ = fs::remove_file(output_path);
    }

    #[test]
    fn handle_dispatches_encrypt_successfully() {
        let input_path = unique_path("keyfile-handle-encrypt-input");
        let output_path = unique_path("keyfile-handle-encrypt-output");

        fs::write(&input_path, b"secret-material").expect("fixture file must be written");

        let command = KeyfileCommand {
            command: KeyfileSubcommand::Encrypt {
                input: input_path
                    .to_str()
                    .expect("input path must be valid UTF-8")
                    .to_string(),
                output: output_path
                    .to_str()
                    .expect("output path must be valid UTF-8")
                    .to_string(),
                password: inline_password_source("Correct#2026!"),
                force: false,
            },
        };

        let result = handle(command);
        assert!(result.is_ok());

        let _ = fs::remove_file(input_path);
        let _ = fs::remove_file(output_path);
    }

    #[test]
    fn handle_dispatches_decrypt_successfully() {
        let encrypted_path = unique_path("keyfile-handle-decrypt-input");
        let output_path = unique_path("keyfile-handle-decrypt-output");

        let encrypted = keyfile::encrypt_key(b"secret-material", "Correct#2026!")
            .expect("encrypted fixture must be created");
        fs::write(&encrypted_path, encrypted).expect("encrypted fixture file must be written");

        let command = KeyfileCommand {
            command: KeyfileSubcommand::Decrypt {
                input: encrypted_path
                    .to_str()
                    .expect("input path must be valid UTF-8")
                    .to_string(),
                output: output_path
                    .to_str()
                    .expect("output path must be valid UTF-8")
                    .to_string(),
                password: inline_password_source("Correct#2026!"),
                allow_plaintext_output: true,
                force: false,
            },
        };

        let result = handle(command);
        assert!(result.is_ok());

        let _ = fs::remove_file(encrypted_path);
        let _ = fs::remove_file(output_path);
    }
}
