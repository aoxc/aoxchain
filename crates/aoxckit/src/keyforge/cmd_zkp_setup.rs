// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::keyforge::cli::{ZkpSetupCommand, ZkpSetupSubcommand};
use crate::keyforge::util::write_text_file;
use serde::Serialize;

/// Canonical AOXC trusted-setup initialization artifact.
///
/// Security posture:
/// - This artifact is metadata-only.
/// - It does not contain proving keys, toxic waste, private keys, or ceremony secrets.
/// - It is safe for persistence and operator-visible output.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct ZkpSetupArtifact {
    circuit: String,
    powers_of_tau: u8,
    status: &'static str,
}

pub fn handle(command: ZkpSetupCommand) -> Result<(), String> {
    match command.command {
        ZkpSetupSubcommand::Init {
            circuit,
            output,
            powers,
        } => init(&circuit, &output, powers),
    }
}

/// Initializes a canonical AOXC trusted-setup artifact and writes it to disk.
///
/// Validation policy:
/// - `circuit` must not be blank,
/// - `output` must not be blank,
/// - `powers` must be non-zero.
///
/// Operational note:
/// This command currently emits an initialization artifact only. It does not
/// execute a full multi-party ceremony or materialize real proving parameters.
fn init(circuit: &str, output: &str, powers: u8) -> Result<(), String> {
    let normalized_output = normalize_required_text(output, "output")?;
    let artifact = build_setup_artifact(circuit, powers)?;
    let body = serialize_pretty_json(&artifact)?;

    write_text_file(&normalized_output, &body)?;
    println!("zkp setup artifact written to {}", normalized_output);

    Ok(())
}

/// Builds the canonical AOXC setup artifact without performing I/O.
fn build_setup_artifact(circuit: &str, powers: u8) -> Result<ZkpSetupArtifact, String> {
    let normalized_circuit = normalize_required_text(circuit, "circuit")?;

    if powers == 0 {
        return Err("ZKP_SETUP_INVALID_ARGUMENT".to_string());
    }

    Ok(ZkpSetupArtifact {
        circuit: normalized_circuit,
        powers_of_tau: powers,
        status: "trusted-setup-initialized",
    })
}

/// Serializes an operator-facing value into canonical pretty JSON.
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
    use serde_json::Value;
    use std::fs;

    fn unique_path(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "aoxc-zkp-setup-{}-{}.json",
            label,
            std::process::id()
        ))
    }

    #[test]
    fn build_setup_artifact_accepts_valid_input() {
        let artifact =
            build_setup_artifact("identity-v1", 18).expect("artifact generation must succeed");

        assert_eq!(
            artifact,
            ZkpSetupArtifact {
                circuit: "identity-v1".to_string(),
                powers_of_tau: 18,
                status: "trusted-setup-initialized",
            }
        );
    }

    #[test]
    fn build_setup_artifact_trims_circuit_name() {
        let artifact =
            build_setup_artifact("  identity-v1  ", 18).expect("artifact generation must succeed");

        assert_eq!(artifact.circuit, "identity-v1");
    }

    #[test]
    fn build_setup_artifact_rejects_blank_circuit() {
        let result = build_setup_artifact("   ", 18);

        assert_eq!(
            result,
            Err("INVALID_ARGUMENT: circuit must not be blank".to_string())
        );
    }

    #[test]
    fn build_setup_artifact_rejects_zero_powers() {
        let result = build_setup_artifact("identity-v1", 0);

        assert_eq!(result, Err("ZKP_SETUP_INVALID_ARGUMENT".to_string()));
    }

    #[test]
    fn serialize_pretty_json_returns_valid_document() {
        let artifact = ZkpSetupArtifact {
            circuit: "identity-v1".to_string(),
            powers_of_tau: 18,
            status: "trusted-setup-initialized",
        };

        let body = serialize_pretty_json(&artifact).expect("serialization must succeed");
        let parsed: Value = serde_json::from_str(&body).expect("output must be valid JSON");

        assert_eq!(parsed["circuit"], "identity-v1");
        assert_eq!(parsed["powers_of_tau"], 18);
        assert_eq!(parsed["status"], "trusted-setup-initialized");
    }

    #[test]
    fn init_writes_artifact_to_output_path() {
        let path = unique_path("init-write");
        let _ = fs::remove_file(&path);

        let result = init(
            "identity-v1",
            path.to_str().expect("path must be valid UTF-8"),
            18,
        );

        assert!(result.is_ok());

        let body = fs::read_to_string(&path).expect("artifact file must be readable");
        let parsed: Value = serde_json::from_str(&body).expect("artifact must be valid JSON");

        assert_eq!(parsed["circuit"], "identity-v1");
        assert_eq!(parsed["powers_of_tau"], 18);
        assert_eq!(parsed["status"], "trusted-setup-initialized");

        let _ = fs::remove_file(path);
    }

    #[test]
    fn init_rejects_blank_output_path() {
        let result = init("identity-v1", "   ", 18);

        assert_eq!(
            result,
            Err("INVALID_ARGUMENT: output must not be blank".to_string())
        );
    }

    #[test]
    fn handle_dispatches_init_successfully() {
        let path = unique_path("handle-init");
        let _ = fs::remove_file(&path);

        let command = ZkpSetupCommand {
            command: ZkpSetupSubcommand::Init {
                circuit: "identity-v1".to_string(),
                output: path.to_str().expect("path must be valid UTF-8").to_string(),
                powers: 18,
            },
        };

        let result = handle(command);
        assert!(result.is_ok());

        let _ = fs::remove_file(path);
    }
}
