use crate::error::{AppError, ErrorCode};
use chrono::Utc;
use serde::Serialize;
use serde_json::Value;
use std::{collections::BTreeMap, env};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
    Yaml,
}

/// Returns the value immediately following a flag, if present.
///
/// Example:
/// - `--format json` => `Some("json")`
/// - `--format` without a following value => `None`
pub fn arg_value(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|window| window[0] == flag)
        .map(|window| window[1].clone())
}

/// Returns true when the provided flag is present in the argument vector.
pub fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|item| item == flag)
}

/// Resolves the requested operator output format.
///
/// The function defaults to text output because text is the safest interactive
/// operator mode for manual terminal use. Structured machine-facing consumers
/// should explicitly request `json` or `yaml`.
pub fn output_format(args: &[String]) -> OutputFormat {
    match arg_value(args, "--format").as_deref() {
        Some("json") => OutputFormat::Json,
        Some("yaml") => OutputFormat::Yaml,
        _ => OutputFormat::Text,
    }
}

/// Detects the preferred language from the ambient process locale.
///
/// The current CLI surface supports Turkish and English selection semantics.
/// Any non-Turkish locale currently falls back to English.
pub fn detect_language(_args: &[String]) -> &'static str {
    let language = env::var("LANG").unwrap_or_default().to_ascii_lowercase();
    if language.starts_with("tr") {
        "tr"
    } else {
        "en"
    }
}

/// Returns a stable unknown-command application error.
pub fn localized_unknown_command(_lang: &str, command: &str) -> AppError {
    AppError::new(
        ErrorCode::UsageUnknownCommand,
        format!("Unknown command: {command}"),
    )
}

/// Prints the top-level AOXC operator help surface.
///
/// This output is intentionally ASCII-safe and icon-free in order to preserve
/// consistent rendering across SSH sessions, CI terminals, container logs,
/// and restricted console environments.
pub fn print_usage(_lang: &str) {
    println!(
        "\
AOXCMD - Operator Command Plane
Deterministic bootstrap, audit-oriented UX, hardened local flows

USAGE
  aoxc <command> [flags]
  aoxc <group> <subcommand> [flags]

GUIDED GROUPS
  chain init|create|start|status|doctor|demo
  genesis init|add-validator|add-account|build|verify|inspect|fingerprint|production-gate
  validator create|inspect|status|rotate-key
  wallet create|balance
  account fund
  node init|start|status|doctor
  network create|start|status|verify|identity-gate|doctor
  role list|status|activate-core7
  api [status|contract|smoke|metrics|health|full|chain|consensus|vm|block|tx|receipt|account|balance|state-root|network]
  query chain|consensus|vm|full|block|tx|receipt|account|balance|network|state-root|rpc
  tx transfer|stake delegate|stake undelegate
  stake delegate|undelegate|validators|rewards
  doctor [node|network|runtime]
  audit [chain|genesis|validator-set]

DESCRIBE
  version
  vision
  build-manifest
  node-connection-policy
  sovereign-core
  module-architecture
  compat-matrix
  quantum-blueprint
  quantum-posture
  port-map
  api-contract
  profile-baseline [--enforce]

BOOTSTRAP
  key-bootstrap
  keys-inspect
  keys-show-fingerprint
  keys-verify [--password <value>]
  address-create --name <validator> --profile <validation|testnet|mainnet|devnet|localnet> --password <value>
  genesis-init
  genesis-add-account --account-id <id> --balance <amount> [--role <treasury|validator|system|user|governance|forge|quorum|seal|archive|sentinel|relay|pocket>]
  genesis-add-validator --validator-id <id> --consensus-public-key <hex> --network-public-key <hex> [--consensus-fingerprint <hex>] [--network-fingerprint <hex>] [--bootnode-address <host:port>] [--balance <amount>] [--display-name <label>]
  genesis-validate [--strict]
  genesis-inspect
  genesis-hash
  config-init [--profile <validation|testnet|mainnet>] [--bind-host <host>] [--json-logs]
  config-validate
  config-print
  production-bootstrap --password <value> [--profile <testnet|mainnet>] [--name <validator>] [--bind-host <host>] [--produce-once-tx <value>] [--skip-produce-once]
  dual-profile-bootstrap --password <value> [--output-dir <path>] [--name-prefix <validator>]

NODE AND ECONOMY
  node-bootstrap
  produce-once
  node-run [--rounds <n>] [--tx-prefix <value>] [--log-level <info|debug>] [--no-live-log]
  node-health
  economy-init
  treasury-transfer
  stake-delegate
  stake-undelegate
  economy-status
  faucet-status [--account-id <id>]
  faucet-history --account-id <id>
  faucet-balance
  faucet-enable
  faucet-disable
  faucet-config-show
  faucet-audit
  faucet-config [--enable|--disable] [--max-claim-amount <n>] [--cooldown-secs <n>] [--daily-limit-per-account <n>] [--daily-global-limit <n>] [--min-reserve-balance <n>] [--ban-account <id>] [--unban-account <id>] [--allow-account <id>] [--disallow-account <id>]
  faucet-claim --account-id <id> [--amount <n>] [--auto-init] [--force]
  faucet-reset [--keep-config]
  runtime-status
  chain-status
  consensus-status
  consensus-validators
  consensus-proposer
  consensus-round
  consensus-finality
  consensus-commits
  consensus-evidence
  vm-status
  query chain status
  query block --height <latest|n>
  query tx --hash <tx-hash>
  query receipt --hash <tx-hash>
  query account --id <account-id>
  query balance --id <account-id>
  query consensus status
  query consensus validators
  query consensus proposer
  query consensus round
  query consensus finality
  query consensus commits
  query consensus evidence
  query vm status
  query vm call
  query vm simulate
  query vm storage
  query vm contract
  query vm code
  query vm estimate-gas
  query vm trace
  query full [--account-id <id>] [--tx-hash <hash>]
  query network status
  query network peers
  query network full
  query state-root
  query rpc
  api status
  api contract
  api smoke
  api metrics
  api health
  api full [--account-id <id>] [--tx-hash <hash>]
  api network full
  block-get --height <latest|n>
  tx-get --hash <tx-hash>
  tx-receipt --hash <tx-hash>
  account-get --id <account-id>
  balance-get --id <account-id>
  peer-list
  network-status
  state-root [--height <n>]
  metrics
  rpc-status
  rpc-curl-smoke

VALIDATION AND AUDIT
  load-benchmark
  storage-smoke
  network-smoke
  real-network
  db-init [--backend <sqlite|redb>]
  db-status [--backend <sqlite|redb>]
  db-put-block --block-file <path> [--backend <sqlite|redb>]
  db-get-height --height <n> [--backend <sqlite|redb>]
  db-get-hash --hash <hex64> [--backend <sqlite|redb>]
  db-compact [--backend <sqlite|redb>]
  diagnostics-doctor
  diagnostics-bundle
  interop-readiness
  interop-gate
  production-audit
  mainnet-readiness [--enforce] [--write-report <path>]
  testnet-readiness [--enforce] [--write-report <path>]
  full-surface-readiness [--enforce] [--write-report <path>]
  network-identity-gate [--full] [--env <name>] [--enforce]
  level-score [--enforce]
  operator-evidence-record --action <name> --reason <text> [--subject <id>] [--profile <name>] [--command <text>]
  operator-evidence-list [--limit <n>] [--category <name>]

GLOBAL FLAGS
  --home <path>               Override AOXC home directory.
  --format <text|json|yaml>   Select operator output format.
  --redact                    Redact sensitive fields where supported.

OUTPUT MODES
  text  - operator-facing curated terminal layout
  json  - machine-readable automation contract
  yaml  - human review and runbook snapshot format"
    );
}

/// Emits a serializable value according to the requested output format.
///
/// Output contract:
/// - Text: curated terminal layout intended for direct operator use.
/// - JSON: strict machine-facing structured output.
/// - YAML: review-oriented snapshot output.
///
/// The text renderer intentionally differs from JSON output. This preserves
/// a clean separation between human-facing terminal ergonomics and automation
/// contracts consumed by tooling.
pub fn emit_serialized<T: Serialize>(value: &T, format: OutputFormat) -> Result<(), AppError> {
    match format {
        OutputFormat::Text => {
            let value = serde_json::to_value(value).map_err(|error| {
                AppError::with_source(
                    ErrorCode::OutputEncodingFailed,
                    "Failed to convert text output into an intermediate value",
                    error,
                )
            })?;

            let rendered = render_text_value(&value);
            println!("{rendered}");
        }
        OutputFormat::Json => {
            let text = serde_json::to_string_pretty(value).map_err(|error| {
                AppError::with_source(
                    ErrorCode::OutputEncodingFailed,
                    "Failed to encode JSON output",
                    error,
                )
            })?;
            println!("{text}");
        }
        OutputFormat::Yaml => {
            let text = serde_yaml::to_string(value).map_err(|error| {
                AppError::with_source(
                    ErrorCode::OutputEncodingFailed,
                    "Failed to encode YAML output",
                    error,
                )
            })?;
            print!("{text}");
        }
    }

    Ok(())
}

/// Builds a stable text envelope for compact operator status commands.
///
/// The timestamp is emitted in RFC 3339 UTC format to preserve deterministic
/// interoperability across shells, CI collectors, and audit pipelines.
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

/// Renders a serializable value into a deterministic operator-facing text view.
fn render_text_value(value: &Value) -> String {
    let mut output = String::new();
    render_text_value_into(value, 0, None, &mut output);

    while output.ends_with('\n') {
        output.pop();
    }

    output
}

/// Recursively renders a JSON value into a compact, readable text layout.
///
/// Formatting policy:
/// - Objects are printed as `key: value`.
/// - Arrays are printed as list items.
/// - Nested structures are indented with spaces only.
/// - The renderer remains ASCII-safe and icon-free.
fn render_text_value_into(value: &Value, indent: usize, key: Option<&str>, output: &mut String) {
    let padding = " ".repeat(indent);

    match value {
        Value::Object(map) => {
            if let Some(key) = key {
                output.push_str(&padding);
                output.push_str(key);
                output.push_str(":\n");
            }

            for (child_key, child_value) in map {
                match child_value {
                    Value::Object(_) | Value::Array(_) => {
                        render_text_value_into(child_value, indent + 2, Some(child_key), output);
                    }
                    _ => {
                        output.push_str(&" ".repeat(indent + 2));
                        output.push_str(child_key);
                        output.push_str(": ");
                        output.push_str(&scalar_to_text(child_value));
                        output.push('\n');
                    }
                }
            }
        }
        Value::Array(items) => {
            if let Some(key) = key {
                output.push_str(&padding);
                output.push_str(key);
                output.push_str(":\n");
            }

            for item in items {
                match item {
                    Value::Object(_) | Value::Array(_) => {
                        output.push_str(&" ".repeat(indent + 2));
                        output.push_str("-\n");
                        render_text_value_into(item, indent + 4, None, output);
                    }
                    _ => {
                        output.push_str(&" ".repeat(indent + 2));
                        output.push_str("- ");
                        output.push_str(&scalar_to_text(item));
                        output.push('\n');
                    }
                }
            }
        }
        _ => {
            if let Some(key) = key {
                output.push_str(&padding);
                output.push_str(key);
                output.push_str(": ");
                output.push_str(&scalar_to_text(value));
                output.push('\n');
            } else {
                output.push_str(&padding);
                output.push_str(&scalar_to_text(value));
                output.push('\n');
            }
        }
    }
}

/// Converts a scalar JSON value into a terminal-safe textual representation.
fn scalar_to_text(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(text) => text.clone(),
        Value::Array(_) | Value::Object(_) => String::new(),
    }
}
