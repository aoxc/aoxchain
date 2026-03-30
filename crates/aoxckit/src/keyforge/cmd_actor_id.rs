// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::keyforge::cli::ActorIdSubcommand;
use crate::keyforge::cli::{ActorIdCommand, OutputFormat};
use aoxcore::identity::actor_id;
use serde::Serialize;

/// Canonical public response emitted by the AOXC actor-id CLI surface.
///
/// Security posture:
/// - This payload is intentionally public-only.
/// - It contains no secret-bearing material.
/// - It is suitable for stdout emission, CI consumption, and operator review.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct ActorIdOutput {
    actor_id: String,
    role: String,
    zone: String,
}

/// Handles AOXC actor-id subcommands.
///
/// Dispatch policy:
/// - Preserve a minimal and explicit command routing surface.
/// - Keep each subcommand path independently testable.
/// - Avoid embedding business logic directly inside the match expression.
pub fn handle(command: ActorIdCommand) -> Result<(), String> {
    match command.command {
        ActorIdSubcommand::Generate { pubkey, role, zone } => {
            generate_and_print(&pubkey, &role, &zone, OutputFormat::PrettyJson)
        }
    }
}

/// Generates a canonical AOXC actor identifier and emits it to stdout.
///
/// Security rationale:
/// - The input public key is validated as uppercase/lowercase hexadecimal text.
/// - Only the derived public actor identifier and normalized metadata are emitted.
/// - Secret material is never accepted or produced by this command surface.
fn generate_and_print(
    pubkey_hex: &str,
    role: &str,
    zone: &str,
    format: OutputFormat,
) -> Result<(), String> {
    let output = build_actor_id_output(pubkey_hex, role, zone)?;
    let body = serialize_actor_id_output(&output, format)?;
    println!("{}", body);
    Ok(())
}

/// Builds the canonical public AOXC actor-id output payload.
///
/// Validation contract:
/// - `pubkey_hex` must decode as non-empty hexadecimal text.
/// - `role` and `zone` must be accepted by the underlying actor-id derivation layer.
/// - The returned actor id is revalidated as a hard postcondition.
fn build_actor_id_output(
    pubkey_hex: &str,
    role: &str,
    zone: &str,
) -> Result<ActorIdOutput, String> {
    let normalized_pubkey_hex = normalize_non_empty_text(pubkey_hex, "pubkey")?;
    let normalized_role_input = normalize_non_empty_text(role, "role")?;
    let normalized_zone_input = normalize_non_empty_text(zone, "zone")?;

    let pubkey = decode_public_key_hex(&normalized_pubkey_hex)?;

    let actor_id = actor_id::generate_and_validate_actor_id(
        &pubkey,
        &normalized_role_input,
        &normalized_zone_input,
    )
    .map_err(map_actor_id_error)?;

    let parsed = actor_id::parse_actor_id(&actor_id).map_err(map_actor_id_error)?;

    Ok(ActorIdOutput {
        actor_id,
        role: parsed.role,
        zone: parsed.zone,
    })
}

/// Decodes operator-supplied public-key hex into raw bytes.
///
/// Error policy:
/// - blank input is rejected,
/// - invalid hex encoding is rejected,
/// - empty decoded material is rejected.
fn decode_public_key_hex(pubkey_hex: &str) -> Result<Vec<u8>, String> {
    let decoded = hex::decode(pubkey_hex).map_err(|_| "PUBKEY_HEX_INVALID".to_string())?;

    if decoded.is_empty() {
        return Err("PUBKEY_EMPTY".to_string());
    }

    Ok(decoded)
}

/// Serializes the canonical actor-id output according to the requested format.
fn serialize_actor_id_output(
    output: &ActorIdOutput,
    format: OutputFormat,
) -> Result<String, String> {
    match format {
        OutputFormat::Json => serde_json::to_string(output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {}", error)),
        OutputFormat::PrettyJson => serde_json::to_string_pretty(output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {}", error)),
    }
}

/// Normalizes and validates required operator-facing text input.
///
/// Policy:
/// - leading and trailing whitespace are removed,
/// - whitespace-only values are rejected,
/// - internal content is otherwise preserved.
fn normalize_non_empty_text(value: &str, field: &str) -> Result<String, String> {
    let normalized = value.trim();

    if normalized.is_empty() {
        return Err(format!("INVALID_ARGUMENT: {} must not be blank", field));
    }

    Ok(normalized.to_string())
}

/// Converts actor-id library errors into stable CLI-facing error strings.
fn map_actor_id_error(error: actor_id::ActorIdError) -> String {
    error.code().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn sample_pubkey_hex() -> String {
        "11".repeat(32)
    }

    #[test]
    fn build_actor_id_output_generates_canonical_public_payload() {
        let output = build_actor_id_output(&sample_pubkey_hex(), "validator", "europe")
            .expect("actor-id generation must succeed");

        assert!(output.actor_id.starts_with("AOXC-"));
        assert_eq!(output.role.len(), actor_id::ROLE_LEN);
        assert_eq!(output.zone.len(), actor_id::ZONE_LEN);
        assert_eq!(output.role, "VAL");
        assert_eq!(output.zone, "EU");
    }

    #[test]
    fn build_actor_id_output_is_deterministic_for_same_inputs() {
        let a = build_actor_id_output(&sample_pubkey_hex(), "validator", "europe")
            .expect("first derivation must succeed");
        let b = build_actor_id_output(&sample_pubkey_hex(), "validator", "europe")
            .expect("second derivation must succeed");

        assert_eq!(a, b);
    }

    #[test]
    fn build_actor_id_output_rejects_invalid_hex() {
        let result = build_actor_id_output("not-hex", "validator", "europe");

        assert_eq!(result, Err("PUBKEY_HEX_INVALID".to_string()));
    }

    #[test]
    fn build_actor_id_output_rejects_empty_decoded_public_key() {
        let result = build_actor_id_output("", "validator", "europe");

        assert_eq!(
            result,
            Err("INVALID_ARGUMENT: pubkey must not be blank".to_string())
        );
    }

    #[test]
    fn build_actor_id_output_rejects_blank_role() {
        let result = build_actor_id_output(&sample_pubkey_hex(), "   ", "europe");

        assert_eq!(
            result,
            Err("INVALID_ARGUMENT: role must not be blank".to_string())
        );
    }

    #[test]
    fn build_actor_id_output_rejects_blank_zone() {
        let result = build_actor_id_output(&sample_pubkey_hex(), "validator", "   ");

        assert_eq!(
            result,
            Err("INVALID_ARGUMENT: zone must not be blank".to_string())
        );
    }

    #[test]
    fn serialize_actor_id_output_pretty_json_returns_valid_document() {
        let output = build_actor_id_output(&sample_pubkey_hex(), "validator", "europe")
            .expect("actor-id generation must succeed");

        let body = serialize_actor_id_output(&output, OutputFormat::PrettyJson)
            .expect("pretty JSON serialization must succeed");

        let parsed: Value = serde_json::from_str(&body).expect("output must be valid JSON");

        assert_eq!(parsed["actor_id"], output.actor_id);
        assert_eq!(parsed["role"], "VAL");
        assert_eq!(parsed["zone"], "EU");
    }

    #[test]
    fn serialize_actor_id_output_json_returns_compact_document() {
        let output = build_actor_id_output(&sample_pubkey_hex(), "validator", "europe")
            .expect("actor-id generation must succeed");

        let body = serialize_actor_id_output(&output, OutputFormat::Json)
            .expect("compact JSON serialization must succeed");

        assert!(!body.contains('\n'));

        let parsed: Value = serde_json::from_str(&body).expect("output must be valid JSON");
        assert_eq!(parsed["actor_id"], output.actor_id);
    }

    #[test]
    fn generated_actor_id_roundtrips_through_library_validation() {
        let output = build_actor_id_output(&sample_pubkey_hex(), "validator", "europe")
            .expect("actor-id generation must succeed");

        actor_id::validate_actor_id(&output.actor_id)
            .expect("generated actor id must validate successfully");
    }

    #[test]
    fn map_actor_id_error_returns_stable_symbolic_code() {
        let code = map_actor_id_error(actor_id::ActorIdError::InvalidRole);

        assert_eq!(code, "ACTOR_ID_INVALID_ROLE");
    }

    #[test]
    fn handle_accepts_generate_command() {
        let command = ActorIdCommand {
            command: ActorIdSubcommand::Generate {
                pubkey: sample_pubkey_hex(),
                role: "validator".to_string(),
                zone: "europe".to_string(),
            },
        };

        let result = handle(command);

        assert!(result.is_ok(), "generate command must succeed");
    }
}
