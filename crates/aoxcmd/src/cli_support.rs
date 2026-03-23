use crate::error::{AppError, ErrorCode};
use chrono::Utc;
use serde::Serialize;
use std::{collections::BTreeMap, env};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
    Yaml,
}

pub fn arg_value(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|window| window[0] == flag)
        .map(|window| window[1].clone())
}

pub fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|item| item == flag)
}

pub fn output_format(args: &[String]) -> OutputFormat {
    match arg_value(args, "--format").as_deref() {
        Some("json") => OutputFormat::Json,
        Some("yaml") => OutputFormat::Yaml,
        _ => OutputFormat::Text,
    }
}

pub fn detect_language(_args: &[String]) -> &'static str {
    let language = env::var("LANG").unwrap_or_default().to_ascii_lowercase();
    if language.starts_with("tr") {
        "tr"
    } else {
        "en"
    }
}

pub fn localized_unknown_command(_lang: &str, command: &str) -> AppError {
    AppError::new(
        ErrorCode::UsageUnknownCommand,
        format!("Unknown command: {command}"),
    )
}

pub fn print_usage(_lang: &str) {
    println!(
        r#"╔══════════════════════════════════════════════════════════════╗
║ AOXCMD • Operator Command Plane                             ║
║ Deterministic bootstrap • audit UX • hardened local flows   ║
╚══════════════════════════════════════════════════════════════╝

Usage:
  aoxc <command> [flags]

Describe:
  version | vision | build-manifest | node-connection-policy
  sovereign-core | module-architecture | compat-matrix | port-map | profile-baseline [--enforce]

Bootstrap:
  key-bootstrap | keys-inspect | keys-show-fingerprint | keys-verify [--password <value>]
  genesis-init | genesis-validate | genesis-inspect | genesis-hash
  config-init [--profile <validator|testnet|mainnet>] [--bind-host <host>] [--json-logs]
  config-validate | config-print
  production-bootstrap --password <value> [--profile <testnet|mainnet>] [--name <validator>] [--bind-host <host>]

Node and economy:
  node-bootstrap | produce-once | node-run | node-health
  economy-init | treasury-transfer | stake-delegate | stake-undelegate
  economy-status | runtime-status

Validation and audit:
  load-benchmark | storage-smoke | network-smoke | real-network
  diagnostics-doctor | diagnostics-bundle
  interop-readiness | interop-gate | production-audit
  mainnet-readiness [--enforce] [--write-report <path>]

Global flags:
  --home <path>        Override AOXC home directory.
  --format <text|json|yaml>
  --redact             Redact secrets in config and bundle output.

Operator style notes:
  • text   => curated operator-facing layout
  • json   => machine-readable automation contract
  • yaml   => human review / runbook snapshots
"#
    );
}

pub fn emit_serialized<T: Serialize>(value: &T, format: OutputFormat) -> Result<(), AppError> {
    match format {
        OutputFormat::Text => {
            let text = serde_json::to_string_pretty(value).map_err(|e| {
                AppError::with_source(
                    ErrorCode::OutputEncodingFailed,
                    "Failed to encode text output",
                    e,
                )
            })?;
            println!("{text}");
        }
        OutputFormat::Json => {
            let text = serde_json::to_string_pretty(value).map_err(|e| {
                AppError::with_source(
                    ErrorCode::OutputEncodingFailed,
                    "Failed to encode JSON output",
                    e,
                )
            })?;
            println!("{text}");
        }
        OutputFormat::Yaml => {
            let text = serde_yaml::to_string(value).map_err(|e| {
                AppError::with_source(
                    ErrorCode::OutputEncodingFailed,
                    "Failed to encode YAML output",
                    e,
                )
            })?;
            print!("{text}");
        }
    }
    Ok(())
}

pub fn text_envelope(
    command: &str,
    status: &str,
    details: BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut envelope = BTreeMap::new();
    envelope.insert("command".to_string(), command.to_string());
    envelope.insert("status".to_string(), status.to_string());
    envelope.insert("timestamp".to_string(), Utc::now().to_rfc3339());
    envelope.extend(details);
    envelope
}
