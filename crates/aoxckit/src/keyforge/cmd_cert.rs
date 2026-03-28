// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::keyforge::cli::{CertCommand, CertSubcommand};
use crate::keyforge::util::{read_text_file, write_text_file};
use aoxcore::identity::ca::CertificateAuthority;
use aoxcore::identity::certificate::Certificate;
use serde::Serialize;

/// Canonical verification response emitted by the AOXC certificate CLI surface.
///
/// Security posture:
/// - This payload is public-only.
/// - It contains no private key material.
/// - It is suitable for stdout emission and machine parsing.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct CertificateVerificationOutput {
    actor_id: String,
    issuer: String,
    verified: bool,
}

/// Canonical mTLS template emitted by the AOXC certificate CLI surface.
///
/// This object is intentionally metadata-only and does not represent a signed
/// X.509 certificate artifact.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct MtlsTemplateOutput {
    subject_cn: String,
    usage: Vec<&'static str>,
    mtls: bool,
    issuer: &'static str,
}

pub fn handle(command: CertCommand) -> Result<(), String> {
    match command.command {
        CertSubcommand::Issue {
            chain,
            actor_id,
            role,
            zone,
            pubkey,
            issued_at,
            expires_at,
            issuer,
            output,
        } => issue(
            &chain,
            &actor_id,
            &role,
            &zone,
            &pubkey,
            issued_at,
            expires_at,
            &issuer,
            output.as_deref(),
        ),
        CertSubcommand::Verify { file, issuer } => verify(&file, &issuer),
        CertSubcommand::Inspect { file } => inspect(&file),
        CertSubcommand::GenerateMtls {
            common_name,
            output,
        } => generate_mtls(&common_name, &output),
    }
}

#[allow(clippy::too_many_arguments)]
fn issue(
    chain: &str,
    actor_id: &str,
    role: &str,
    zone: &str,
    pubkey: &str,
    issued_at: u64,
    expires_at: u64,
    issuer: &str,
    output: Option<&str>,
) -> Result<(), String> {
    let signed = build_signed_certificate(
        chain,
        actor_id,
        role,
        zone,
        pubkey,
        issued_at,
        expires_at,
        issuer,
    )?;

    let body = serialize_pretty_json(&signed)?;

    match output {
        Some(path) => {
            let normalized_path = normalize_required_text(path, "output")?;
            write_text_file(&normalized_path, &body)?;
            println!("certificate written to {}", normalized_path);
        }
        None => {
            println!("{}", body);
        }
    }

    Ok(())
}

fn verify(file: &str, issuer: &str) -> Result<(), String> {
    let cert = load_certificate_from_file(file)?;
    cert.validate_signed().map_err(map_certificate_error)?;

    let normalized_issuer = normalize_required_text(issuer, "issuer")?;
    let ca = CertificateAuthority::new(normalized_issuer);

    let output = CertificateVerificationOutput {
        actor_id: cert.actor_id.clone(),
        issuer: cert.issuer.clone(),
        verified: ca.verify_certificate(&cert),
    };

    println!("{}", serialize_pretty_json(&output)?);

    Ok(())
}

fn inspect(file: &str) -> Result<(), String> {
    let cert = load_certificate_from_file(file)?;
    println!("{}", serialize_pretty_json(&cert)?);
    Ok(())
}

fn generate_mtls(common_name: &str, output: &str) -> Result<(), String> {
    let template = build_mtls_template(common_name)?;
    let body = serialize_pretty_json(&template)?;
    let normalized_output = normalize_required_text(output, "output")?;

    write_text_file(&normalized_output, &body)?;
    println!(
        "mTLS certificate template written to {}",
        normalized_output
    );

    Ok(())
}

/// Builds, signs, and validates a canonical AOXC certificate.
///
/// Validation contract:
/// - all operator-facing textual inputs must be non-blank,
/// - the unsigned certificate must pass semantic validation,
/// - the signed certificate must pass semantic validation after issuance.
#[allow(clippy::too_many_arguments)]
fn build_signed_certificate(
    chain: &str,
    actor_id: &str,
    role: &str,
    zone: &str,
    pubkey: &str,
    issued_at: u64,
    expires_at: u64,
    issuer: &str,
) -> Result<Certificate, String> {
    let normalized_chain = normalize_required_text(chain, "chain")?;
    let normalized_actor_id = normalize_required_text(actor_id, "actor_id")?;
    let normalized_role = normalize_required_text(role, "role")?;
    let normalized_zone = normalize_required_text(zone, "zone")?;
    let normalized_pubkey = normalize_required_text(pubkey, "pubkey")?;
    let normalized_issuer = normalize_required_text(issuer, "issuer")?;

    let certificate = Certificate::new_unsigned(
        normalized_chain,
        normalized_actor_id,
        normalized_role,
        normalized_zone,
        normalized_pubkey,
        issued_at,
        expires_at,
    );

    certificate
        .validate_unsigned()
        .map_err(map_certificate_error)?;

    let ca = CertificateAuthority::new(normalized_issuer);

    let signed = ca
        .sign_certificate(certificate)
        .map_err(|error| format!("CERT_SIGN_ERROR: {}", error))?;

    signed.validate_signed().map_err(map_certificate_error)?;

    Ok(signed)
}

/// Loads and validates a serialized certificate from disk.
///
/// Validation policy:
/// - the file must be readable as UTF-8,
/// - the payload must decode as `Certificate`.
fn load_certificate_from_file(file: &str) -> Result<Certificate, String> {
    let normalized_file = normalize_required_text(file, "file")?;
    let data = read_text_file(&normalized_file)?;
    serde_json::from_str::<Certificate>(&data)
        .map_err(|error| format!("CERT_PARSE_ERROR: {}", error))
}

/// Builds a canonical AOXC mTLS template response.
///
/// The template is intentionally metadata-only and does not imply a signed
/// certificate chain or materialized private key.
fn build_mtls_template(common_name: &str) -> Result<MtlsTemplateOutput, String> {
    let normalized_common_name = normalize_required_text(common_name, "common_name")?;

    Ok(MtlsTemplateOutput {
        subject_cn: normalized_common_name,
        usage: vec!["serverAuth", "clientAuth"],
        mtls: true,
        issuer: "AOXC-LOCAL-CA",
    })
}

/// Serializes any operator-facing value into canonical pretty JSON.
fn serialize_pretty_json<T>(value: &T) -> Result<String, String>
where
    T: Serialize,
{
    serde_json::to_string_pretty(value).map_err(|error| format!("JSON_SERIALIZE_ERROR: {}", error))
}

/// Enforces non-blank operator-facing textual input.
///
/// Policy:
/// - trims leading/trailing whitespace,
/// - rejects whitespace-only values,
/// - preserves normalized content.
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

/// Maps certificate-domain errors into stable symbolic CLI error codes.
fn map_certificate_error(error: aoxcore::identity::certificate::CertificateError) -> String {
    error.code().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn sample_unsigned_certificate_json() -> String {
        serde_json::json!({
            "version": 1,
            "chain": "AOXC-0001-MAIN",
            "actor_id": "AOXC-VAL-EU-3F7A9C21D4E8B7AA-K9",
            "role": "VAL",
            "zone": "EU",
            "pubkey": "A1B2C3D4",
            "issued_at": 1700000000_u64,
            "expires_at": 1800000000_u64,
            "issuer": "",
            "signature": ""
        })
        .to_string()
    }

    #[test]
    fn build_signed_certificate_returns_semantically_valid_signed_certificate() {
        let signed = build_signed_certificate(
            "AOXC-0001-MAIN",
            "AOXC-VAL-EU-3F7A9C21D4E8B7AA-K9",
            "VAL",
            "EU",
            "A1B2C3D4",
            1_700_000_000,
            1_800_000_000,
            "AOXC-ROOT-CA",
        )
        .expect("certificate issuance must succeed");

        assert_eq!(signed.issuer, "AOXC-ROOT-CA");
        assert!(!signed.signature.is_empty());
        assert!(signed.validate_signed().is_ok());
    }

    #[test]
    fn build_signed_certificate_rejects_blank_chain() {
        let result = build_signed_certificate(
            "   ",
            "AOXC-VAL-EU-3F7A9C21D4E8B7AA-K9",
            "VAL",
            "EU",
            "A1B2C3D4",
            1_700_000_000,
            1_800_000_000,
            "AOXC-ROOT-CA",
        );

        assert_eq!(
            result,
            Err("INVALID_ARGUMENT: chain must not be blank".to_string())
        );
    }

    #[test]
    fn build_signed_certificate_rejects_invalid_public_key_hex() {
        let result = build_signed_certificate(
            "AOXC-0001-MAIN",
            "AOXC-VAL-EU-3F7A9C21D4E8B7AA-K9",
            "VAL",
            "EU",
            "NOT_HEX",
            1_700_000_000,
            1_800_000_000,
            "AOXC-ROOT-CA",
        );

        assert_eq!(result, Err("CERT_INVALID_PUBLIC_KEY_HEX".to_string()));
    }

    #[test]
    fn build_signed_certificate_rejects_invalid_validity_window() {
        let result = build_signed_certificate(
            "AOXC-0001-MAIN",
            "AOXC-VAL-EU-3F7A9C21D4E8B7AA-K9",
            "VAL",
            "EU",
            "A1B2C3D4",
            200,
            100,
            "AOXC-ROOT-CA",
        );

        assert_eq!(result, Err("CERT_INVALID_VALIDITY_WINDOW".to_string()));
    }

    #[test]
    fn serialize_pretty_json_produces_valid_json_document() {
        let template = build_mtls_template("node-01.aoxc.internal")
            .expect("template generation must succeed");

        let body = serialize_pretty_json(&template).expect("serialization must succeed");
        let parsed: Value = serde_json::from_str(&body).expect("output must be valid JSON");

        assert_eq!(parsed["subject_cn"], "node-01.aoxc.internal");
        assert_eq!(parsed["mtls"], true);
    }

    #[test]
    fn build_mtls_template_rejects_blank_common_name() {
        let result = build_mtls_template("   ");

        assert_eq!(
            result,
            Err("INVALID_ARGUMENT: common_name must not be blank".to_string())
        );
    }

    #[test]
    fn load_certificate_from_file_accepts_valid_json_shape() {
        let path = std::env::temp_dir().join(format!(
            "aoxc-cert-load-valid-{}.json",
            std::process::id()
        ));

        std::fs::write(&path, sample_unsigned_certificate_json())
            .expect("fixture file must be written");

        let cert = load_certificate_from_file(path.to_str().expect("path must be valid UTF-8"))
            .expect("certificate JSON must parse");

        assert_eq!(cert.version, 1);
        assert_eq!(cert.role, "VAL");
        assert_eq!(cert.zone, "EU");

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn load_certificate_from_file_rejects_invalid_json() {
        let path = std::env::temp_dir().join(format!(
            "aoxc-cert-load-invalid-{}.json",
            std::process::id()
        ));

        std::fs::write(&path, "{not-json").expect("fixture file must be written");

        let result = load_certificate_from_file(path.to_str().expect("path must be valid UTF-8"));

        assert!(result.is_err());

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn verify_returns_false_for_unsigned_certificate_payload() {
        let path = std::env::temp_dir().join(format!(
            "aoxc-cert-verify-unsigned-{}.json",
            std::process::id()
        ));

        std::fs::write(&path, sample_unsigned_certificate_json())
            .expect("fixture file must be written");

        let result = verify(
            path.to_str().expect("path must be valid UTF-8"),
            "AOXC-ROOT-CA",
        );

        assert_eq!(result, Err("CERT_EMPTY_ISSUER".to_string()));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn inspect_accepts_valid_certificate_document() {
        let path = std::env::temp_dir().join(format!(
            "aoxc-cert-inspect-valid-{}.json",
            std::process::id()
        ));

        std::fs::write(&path, sample_unsigned_certificate_json())
            .expect("fixture file must be written");

        let result = inspect(path.to_str().expect("path must be valid UTF-8"));

        assert!(result.is_ok());

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn handle_dispatches_generate_mtls_successfully() {
        let path = std::env::temp_dir().join(format!(
            "aoxc-cert-mtls-template-{}.json",
            std::process::id()
        ));

        let command = CertCommand {
            command: CertSubcommand::GenerateMtls {
                common_name: "node-01.aoxc.internal".to_string(),
                output: path.to_str().expect("path must be valid UTF-8").to_string(),
            },
        };

        let result = handle(command);
        assert!(result.is_ok());

        let _ = std::fs::remove_file(path);
    }
}
