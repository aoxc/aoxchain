use crate::keyforge::cli::{CertCommand, CertSubcommand};
use crate::keyforge::util::{read_text_file, write_text_file};
use core::identity::ca::CertificateAuthority;
use core::identity::certificate::Certificate;

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
    let certificate = Certificate::new_unsigned(
        chain.to_string(),
        actor_id.to_string(),
        role.to_string(),
        zone.to_string(),
        pubkey.to_string(),
        issued_at,
        expires_at,
    );

    certificate
        .validate_unsigned()
        .map_err(|error| error.to_string())?;

    let ca = CertificateAuthority::new(issuer.to_string());

    let signed = ca
        .sign_certificate(certificate)
        .map_err(|error| format!("CERT_SIGN_ERROR: {}", error))?;

    signed
        .validate_signed()
        .map_err(|error| error.to_string())?;

    let json = serde_json::to_string_pretty(&signed)
        .map_err(|error| format!("JSON_SERIALIZE_ERROR: {}", error))?;

    if let Some(path) = output {
        write_text_file(path, &json)?;
        println!("certificate written to {}", path);
    } else {
        println!("{}", json);
    }

    Ok(())
}

fn verify(file: &str, issuer: &str) -> Result<(), String> {
    let data = read_text_file(file)?;
    let cert: Certificate =
        serde_json::from_str(&data).map_err(|error| format!("CERT_PARSE_ERROR: {}", error))?;

    cert.validate_signed().map_err(|error| error.to_string())?;

    let ca = CertificateAuthority::new(issuer.to_string());

    let verified = ca.verify_certificate(&cert);

    let output = serde_json::json!({
        "actor_id": cert.actor_id,
        "issuer": cert.issuer,
        "verified": verified
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {}", error))?
    );

    Ok(())
}

fn inspect(file: &str) -> Result<(), String> {
    let data = read_text_file(file)?;
    let cert: Certificate =
        serde_json::from_str(&data).map_err(|error| format!("CERT_PARSE_ERROR: {}", error))?;

    println!(
        "{}",
        serde_json::to_string_pretty(&cert)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {}", error))?
    );

    Ok(())
}
