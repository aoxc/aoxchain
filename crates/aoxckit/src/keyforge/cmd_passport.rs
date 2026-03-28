// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::keyforge::cli::{PassportCommand, PassportSubcommand};
use crate::keyforge::util::read_text_file;
use aoxcore::identity::passport::{PASSPORT_VERSION, Passport};
use serde::Serialize;

/// Canonical operator-facing passport inspection response.
///
/// Security posture:
/// - This payload is public-only.
/// - It contains no secret-bearing material.
/// - It preserves the underlying passport document while adding stable
///   inspection metadata useful for operators and automation.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct PassportInspectionOutput {
    fingerprint: String,
    expired: bool,
    passport: Passport,
}

pub fn handle(command: PassportCommand) -> Result<(), String> {
    match command.command {
        PassportSubcommand::Inspect { file } => inspect(&file),
    }
}

/// Loads, validates, and prints a canonical AOXC passport inspection payload.
///
/// Validation policy:
/// - the file path must not be blank,
/// - the file must be readable as UTF-8 text,
/// - the payload must decode into the canonical `Passport` schema,
/// - the decoded passport must satisfy minimal semantic validation rules
///   before it is emitted back to the operator surface.
fn inspect(file: &str) -> Result<(), String> {
    let output = build_passport_inspection_output(file)?;
    let body = serialize_pretty_json(&output)?;
    println!("{}", body);
    Ok(())
}

/// Builds a stable passport inspection response from a serialized passport file.
fn build_passport_inspection_output(file: &str) -> Result<PassportInspectionOutput, String> {
    let passport = load_passport_from_file(file)?;
    validate_passport(&passport)?;

    Ok(PassportInspectionOutput {
        fingerprint: passport.fingerprint(),
        expired: passport.is_expired(passport.expires_at.saturating_add(1)),
        passport,
    })
}

/// Loads a passport from disk and decodes it from JSON.
fn load_passport_from_file(file: &str) -> Result<Passport, String> {
    let normalized_file = normalize_required_text(file, "file")?;
    let data = read_text_file(&normalized_file)?;
    Passport::from_json(&data)
}

/// Performs handler-level semantic validation for a passport document.
///
/// Rationale:
/// The current `Passport` model does not expose an internal `validate()`
/// function, so the command layer must enforce baseline integrity checks.
///
/// Validation rules:
/// - version must match the canonical passport schema version,
/// - actor_id, role, zone, and certificate must not be blank,
/// - issued_at and expires_at must be non-zero,
/// - expires_at must be strictly greater than issued_at.
fn validate_passport(passport: &Passport) -> Result<(), String> {
    if passport.version != PASSPORT_VERSION {
        return Err("PASSPORT_INVALID_VERSION".to_string());
    }

    normalize_required_text(&passport.actor_id, "actor_id")
        .map_err(|_| "PASSPORT_INVALID_ACTOR_ID".to_string())?;
    normalize_required_text(&passport.role, "role")
        .map_err(|_| "PASSPORT_INVALID_ROLE".to_string())?;
    normalize_required_text(&passport.zone, "zone")
        .map_err(|_| "PASSPORT_INVALID_ZONE".to_string())?;
    normalize_required_text(&passport.certificate, "certificate")
        .map_err(|_| "PASSPORT_INVALID_CERTIFICATE".to_string())?;

    if passport.issued_at == 0 {
        return Err("PASSPORT_INVALID_ISSUED_AT".to_string());
    }

    if passport.expires_at == 0 {
        return Err("PASSPORT_INVALID_EXPIRES_AT".to_string());
    }

    if passport.expires_at <= passport.issued_at {
        return Err("PASSPORT_INVALID_VALIDITY_WINDOW".to_string());
    }

    Ok(())
}

/// Serializes an operator-facing payload into canonical pretty JSON.
fn serialize_pretty_json<T>(value: &T) -> Result<String, String>
where
    T: Serialize,
{
    serde_json::to_string_pretty(value).map_err(|error| format!("JSON_SERIALIZE_ERROR: {}", error))
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
        return Err(format!("INVALID_ARGUMENT: {} must not be blank", field));
    }

    Ok(normalized.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn unique_path(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "aoxc-passport-{}-{}.json",
            label,
            std::process::id()
        ))
    }

    fn sample_passport() -> Passport {
        Passport::new(
            "AOXC-VAL-EU-3F7A9C21D4E8B7AA-K9".to_string(),
            "validator".to_string(),
            "EU".to_string(),
            "CERT_DATA".to_string(),
            1_700_000_000,
            1_800_000_000,
        )
    }

    #[test]
    fn validate_passport_accepts_canonical_sample() {
        let passport = sample_passport();

        let result = validate_passport(&passport);

        assert!(result.is_ok());
    }

    #[test]
    fn validate_passport_rejects_invalid_version() {
        let mut passport = sample_passport();
        passport.version = PASSPORT_VERSION + 1;

        let result = validate_passport(&passport);

        assert_eq!(result, Err("PASSPORT_INVALID_VERSION".to_string()));
    }

    #[test]
    fn validate_passport_rejects_blank_actor_id() {
        let mut passport = sample_passport();
        passport.actor_id = "   ".to_string();

        let result = validate_passport(&passport);

        assert_eq!(result, Err("PASSPORT_INVALID_ACTOR_ID".to_string()));
    }

    #[test]
    fn validate_passport_rejects_blank_certificate() {
        let mut passport = sample_passport();
        passport.certificate = "".to_string();

        let result = validate_passport(&passport);

        assert_eq!(result, Err("PASSPORT_INVALID_CERTIFICATE".to_string()));
    }

    #[test]
    fn validate_passport_rejects_invalid_validity_window() {
        let mut passport = sample_passport();
        passport.issued_at = 200;
        passport.expires_at = 100;

        let result = validate_passport(&passport);

        assert_eq!(result, Err("PASSPORT_INVALID_VALIDITY_WINDOW".to_string()));
    }

    #[test]
    fn load_passport_from_file_accepts_valid_json() {
        let path = unique_path("load-valid");
        let passport = sample_passport();
        let body = passport.to_json().expect("passport JSON must serialize");

        fs::write(&path, body).expect("fixture file must be written");

        let loaded = load_passport_from_file(path.to_str().expect("path must be valid UTF-8"))
            .expect("passport must load successfully");

        assert_eq!(loaded.actor_id, passport.actor_id);
        assert_eq!(loaded.role, passport.role);
        assert_eq!(loaded.zone, passport.zone);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn load_passport_from_file_rejects_invalid_json() {
        let path = unique_path("load-invalid");
        fs::write(&path, "{not-json").expect("fixture file must be written");

        let result = load_passport_from_file(path.to_str().expect("path must be valid UTF-8"));

        assert!(matches!(result, Err(error) if error.starts_with("PASSPORT_PARSE_ERROR:")));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn build_passport_inspection_output_contains_fingerprint_and_payload() {
        let path = unique_path("inspection-output");
        let passport = sample_passport();
        let body = passport.to_json().expect("passport JSON must serialize");

        fs::write(&path, body).expect("fixture file must be written");

        let output =
            build_passport_inspection_output(path.to_str().expect("path must be valid UTF-8"))
                .expect("inspection output must build successfully");

        assert_eq!(output.passport.actor_id, passport.actor_id);
        assert!(!output.fingerprint.is_empty());
        assert!(!output.expired);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn serialize_pretty_json_returns_valid_document() {
        let output = PassportInspectionOutput {
            fingerprint: "ABCDEF0123456789".to_string(),
            expired: false,
            passport: sample_passport(),
        };

        let body = serialize_pretty_json(&output).expect("serialization must succeed");
        let parsed: serde_json::Value =
            serde_json::from_str(&body).expect("output must be valid JSON");

        assert_eq!(parsed["fingerprint"], "ABCDEF0123456789");
        assert_eq!(parsed["expired"], false);
        assert!(parsed["passport"].is_object());
    }

    #[test]
    fn inspect_accepts_valid_passport_document() {
        let path = unique_path("inspect-valid");
        let passport = sample_passport();
        let body = passport.to_json().expect("passport JSON must serialize");

        fs::write(&path, body).expect("fixture file must be written");

        let result = inspect(path.to_str().expect("path must be valid UTF-8"));

        assert!(result.is_ok());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn handle_dispatches_inspect_successfully() {
        let path = unique_path("handle-inspect");
        let passport = sample_passport();
        let body = passport.to_json().expect("passport JSON must serialize");

        fs::write(&path, body).expect("fixture file must be written");

        let command = PassportCommand {
            command: PassportSubcommand::Inspect {
                file: path.to_str().expect("path must be valid UTF-8").to_string(),
            },
        };

        let result = handle(command);

        assert!(result.is_ok());

        let _ = fs::remove_file(path);
    }
}
