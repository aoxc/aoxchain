// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    cli_support::{arg_value, emit_serialized, output_format},
    error::{AppError, ErrorCode},
    storage::redb_chain::{append_chain_log, load_chain_logs},
};
use chrono::{SecondsFormat, Utc};
use serde::Serialize;
use sha2::{Digest, Sha256};

const OPERATOR_EVIDENCE_CATEGORY: &str = "operator_evidence";
const OPERATOR_EVIDENCE_ID_DOMAIN: &str = "AOXC_OPERATOR_EVIDENCE_V1";
const DEFAULT_EVIDENCE_SUBJECT: &str = "operator-plane";
const DEFAULT_EVIDENCE_PROFILE: &str = "unspecified";
const DEFAULT_EVIDENCE_COMMAND: &str = "manual";
const DEFAULT_LIST_LIMIT: usize = 20;
const MAX_LIST_LIMIT: usize = 200;

/// Operator-facing evidence record persisted and emitted by the CLI.
///
/// Design intent:
/// - Preserve a stable, machine-readable envelope suitable for audit trails.
/// - Separate identity (`evidence_id`) from operator context fields.
/// - Keep the surface explicit so downstream evidence pipelines can index,
///   archive, or enrich records without schema guessing.
#[derive(Debug, Serialize, PartialEq, Eq)]
struct OperatorEvidenceRecord {
    evidence_id: String,
    created_at: String,
    action: String,
    reason: String,
    subject: String,
    profile: String,
    command: String,
}

/// Operator-facing success envelope for evidence recording.
#[derive(Debug, Serialize, PartialEq, Eq)]
struct OperatorEvidenceEnvelope {
    status: &'static str,
    record: OperatorEvidenceRecord,
}

/// Structured log payload written into the chain log surface.
///
/// Security rationale:
/// - Use structured JSON rather than free-form string concatenation so
///   operator-supplied values cannot degrade downstream parsing reliability.
/// - Preserve the full evidence record that was emitted to the caller.
#[derive(Debug, Serialize, PartialEq, Eq)]
struct OperatorEvidenceLogDetail<'a> {
    evidence_id: &'a str,
    created_at: &'a str,
    action: &'a str,
    reason: &'a str,
    subject: &'a str,
    profile: &'a str,
    command: &'a str,
}

/// Records an operator evidence event and emits the normalized record.
///
/// Required flags:
/// - `--action <value>`
/// - `--reason <value>`
///
/// Optional flags:
/// - `--subject <value>`
/// - `--profile <value>`
/// - `--command <value>`
pub fn cmd_operator_evidence_record(args: &[String]) -> Result<(), AppError> {
    let action = parse_required_text_arg(args, "--action", true)?;
    let reason = parse_required_text_arg(args, "--reason", false)?;
    let subject = parse_optional_text_arg(args, "--subject", DEFAULT_EVIDENCE_SUBJECT, false);
    let profile = parse_optional_text_arg(args, "--profile", DEFAULT_EVIDENCE_PROFILE, true);
    let command = parse_optional_text_arg(args, "--command", DEFAULT_EVIDENCE_COMMAND, false);

    let created_at = Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true);
    let evidence_id =
        compute_evidence_id(&created_at, &action, &reason, &subject, &profile, &command);

    let record = OperatorEvidenceRecord {
        evidence_id,
        created_at,
        action,
        reason,
        subject,
        profile,
        command,
    };

    let detail = serde_json::to_string(&OperatorEvidenceLogDetail {
        evidence_id: &record.evidence_id,
        created_at: &record.created_at,
        action: &record.action,
        reason: &record.reason,
        subject: &record.subject,
        profile: &record.profile,
        command: &record.command,
    })
    .map_err(|error| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode operator evidence log detail",
            error,
        )
    })?;

    append_chain_log(OPERATOR_EVIDENCE_CATEGORY, &record.action, &detail)?;

    emit_serialized(
        &OperatorEvidenceEnvelope {
            status: "recorded",
            record,
        },
        output_format(args),
    )
}

/// Lists chain-log evidence entries with optional category filtering.
///
/// Optional flags:
/// - `--limit <n>`
/// - `--category <value>`
pub fn cmd_operator_evidence_list(args: &[String]) -> Result<(), AppError> {
    let limit = parse_limit_arg(args)?;
    let category = parse_optional_category_filter(args);

    let logs = load_chain_logs(limit, category.as_deref())?;
    emit_serialized(&logs, output_format(args))
}

/// Parses a required text flag and enforces non-blank normalized content.
///
/// Behavioral policy:
/// - Leading and trailing whitespace are trimmed.
/// - Empty values are rejected.
/// - Optional lowercase canonicalization is supported for identifier-like fields.
fn parse_required_text_arg(
    args: &[String],
    flag: &str,
    canonicalize_lowercase: bool,
) -> Result<String, AppError> {
    let value = arg_value(args, flag).ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Missing required flag {flag} <value>"),
        )
    })?;

    normalize_text_value(&value, canonicalize_lowercase).ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Flag {flag} must not be blank"),
        )
    })
}

/// Parses an optional text flag and returns a normalized value or a normalized default.
fn parse_optional_text_arg(
    args: &[String],
    flag: &str,
    default: &str,
    canonicalize_lowercase: bool,
) -> String {
    arg_value(args, flag)
        .as_deref()
        .and_then(|value| normalize_text_value(value, canonicalize_lowercase))
        .unwrap_or_else(|| {
            normalize_text_value(default, canonicalize_lowercase)
                .expect("default optional CLI text must be non-blank")
        })
}

/// Parses an optional category filter.
///
/// Behavioral policy:
/// - Blank values are treated as absent.
/// - The filter is trimmed before use.
/// - Category filtering remains read-only and is passed through to storage.
fn parse_optional_category_filter(args: &[String]) -> Option<String> {
    arg_value(args, "--category").and_then(|value| normalize_text_value(&value, false))
}

/// Parses the operator-requested list limit.
///
/// Validation behavior:
/// - Missing `--limit` falls back to a stable default.
/// - Non-numeric values are rejected.
/// - Zero is rejected.
/// - Values above the operational ceiling are clamped to `MAX_LIST_LIMIT`.
fn parse_limit_arg(args: &[String]) -> Result<usize, AppError> {
    match arg_value(args, "--limit") {
        None => Ok(DEFAULT_LIST_LIMIT),
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(AppError::new(
                    ErrorCode::UsageInvalidArguments,
                    "Flag --limit must not be blank",
                ));
            }

            let parsed = trimmed.parse::<usize>().map_err(|_| {
                AppError::new(
                    ErrorCode::UsageInvalidArguments,
                    "Invalid numeric value for --limit",
                )
            })?;

            if parsed == 0 {
                return Err(AppError::new(
                    ErrorCode::UsageInvalidArguments,
                    "Flag --limit must be greater than zero",
                ));
            }

            Ok(parsed.min(MAX_LIST_LIMIT))
        }
    }
}

/// Normalizes a text value into a canonical CLI-safe representation.
///
/// Normalization policy:
/// - Trim leading and trailing whitespace.
/// - Collapse internal runs of whitespace into a single ASCII space.
/// - Optionally lowercase identifier-like values.
/// - Reject values that become empty after normalization.
fn normalize_text_value(value: &str, canonicalize_lowercase: bool) -> Option<String> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return None;
    }

    if canonicalize_lowercase {
        Some(normalized.to_ascii_lowercase())
    } else {
        Some(normalized)
    }
}

/// Computes a deterministic evidence identifier over the full normalized record.
///
/// Identity policy:
/// - Includes a domain-separation prefix to prevent accidental cross-surface
///   hash reuse.
/// - Covers timestamp and all operator-visible semantic fields, including
///   `profile`.
/// - Uses explicit `0x00` delimiters between fields to avoid concatenation
///   ambiguity.
fn compute_evidence_id(
    created_at: &str,
    action: &str,
    reason: &str,
    subject: &str,
    profile: &str,
    command: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(OPERATOR_EVIDENCE_ID_DOMAIN.as_bytes());
    hasher.update([0]);
    hasher.update(created_at.as_bytes());
    hasher.update([0]);
    hasher.update(action.as_bytes());
    hasher.update([0]);
    hasher.update(reason.as_bytes());
    hasher.update([0]);
    hasher.update(subject.as_bytes());
    hasher.update([0]);
    hasher.update(profile.as_bytes());
    hasher.update([0]);
    hasher.update(command.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::{
        compute_evidence_id, normalize_text_value, parse_limit_arg, parse_optional_category_filter,
        parse_optional_text_arg, parse_required_text_arg, DEFAULT_EVIDENCE_COMMAND,
        DEFAULT_EVIDENCE_PROFILE, DEFAULT_EVIDENCE_SUBJECT, MAX_LIST_LIMIT,
    };

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_string()).collect()
    }

    #[test]
    fn evidence_id_is_stable_for_same_input() {
        let a = compute_evidence_id(
            "2026-03-28T00:00:00.000000000Z",
            "shutdown_prepare",
            "scheduled-upgrade",
            "node-1",
            "devnet",
            "aoxc node-run",
        );
        let b = compute_evidence_id(
            "2026-03-28T00:00:00.000000000Z",
            "shutdown_prepare",
            "scheduled-upgrade",
            "node-1",
            "devnet",
            "aoxc node-run",
        );

        assert_eq!(a, b);
    }

    #[test]
    fn evidence_id_changes_with_reason() {
        let a = compute_evidence_id(
            "2026-03-28T00:00:00.000000000Z",
            "shutdown_prepare",
            "scheduled-upgrade",
            "node-1",
            "devnet",
            "aoxc node-run",
        );
        let b = compute_evidence_id(
            "2026-03-28T00:00:00.000000000Z",
            "shutdown_prepare",
            "incident-response",
            "node-1",
            "devnet",
            "aoxc node-run",
        );

        assert_ne!(a, b);
    }

    #[test]
    fn evidence_id_changes_with_profile() {
        let a = compute_evidence_id(
            "2026-03-28T00:00:00.000000000Z",
            "key_rotation",
            "scheduled",
            "validator-7",
            "testnet",
            "aoxc key-bootstrap",
        );
        let b = compute_evidence_id(
            "2026-03-28T00:00:00.000000000Z",
            "key_rotation",
            "scheduled",
            "validator-7",
            "mainnet",
            "aoxc key-bootstrap",
        );

        assert_ne!(a, b);
    }

    #[test]
    fn evidence_id_uses_full_input_domain() {
        let base = compute_evidence_id(
            "2026-03-28T00:00:00.000000000Z",
            "key_rotation",
            "scheduled",
            "validator-7",
            "validation",
            "aoxc key-bootstrap",
        );
        let changed_action = compute_evidence_id(
            "2026-03-28T00:00:00.000000000Z",
            "key_rotation_force",
            "scheduled",
            "validator-7",
            "validation",
            "aoxc key-bootstrap",
        );
        let changed_subject = compute_evidence_id(
            "2026-03-28T00:00:00.000000000Z",
            "key_rotation",
            "scheduled",
            "validator-8",
            "validation",
            "aoxc key-bootstrap",
        );
        let changed_command = compute_evidence_id(
            "2026-03-28T00:00:00.000000000Z",
            "key_rotation",
            "scheduled",
            "validator-7",
            "validation",
            "aoxc key-bootstrap --dry-run",
        );

        assert_ne!(base, changed_action);
        assert_ne!(base, changed_subject);
        assert_ne!(base, changed_command);
    }

    #[test]
    fn evidence_id_is_hex_sha256_length() {
        let id = compute_evidence_id("t", "a", "r", "s", "p", "c");

        assert_eq!(id.len(), 64);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn normalize_text_value_trims_and_collapses_whitespace() {
        let normalized = normalize_text_value("  scheduled   upgrade  window ", false)
            .expect("value should normalize");

        assert_eq!(normalized, "scheduled upgrade window");
    }

    #[test]
    fn normalize_text_value_can_canonicalize_lowercase() {
        let normalized = normalize_text_value("  DEVNET  ", true).expect("value should normalize");

        assert_eq!(normalized, "devnet");
    }

    #[test]
    fn normalize_text_value_rejects_blank_input() {
        assert_eq!(normalize_text_value("   \t  \n ", false), None);
    }

    #[test]
    fn parse_required_text_arg_rejects_missing_flag() {
        let error = parse_required_text_arg(&[], "--action", true)
            .expect_err("missing required arg must fail");

        assert_eq!(error.code(), "AOXC-USG-002");
    }

    #[test]
    fn parse_required_text_arg_rejects_blank_flag_value() {
        let error = parse_required_text_arg(&args(&["--action", "   "]), "--action", true)
            .expect_err("blank required arg must fail");

        assert_eq!(error.code(), "AOXC-USG-002");
    }

    #[test]
    fn parse_required_text_arg_normalizes_and_lowercases_identifiers() {
        let value = parse_required_text_arg(
            &args(&["--action", "  ShutDown_Prepare  "]),
            "--action",
            true,
        )
        .expect("required arg should parse");

        assert_eq!(value, "shutdown_prepare");
    }

    #[test]
    fn parse_optional_text_arg_returns_normalized_default() {
        let subject = parse_optional_text_arg(&[], "--subject", DEFAULT_EVIDENCE_SUBJECT, false);
        let profile = parse_optional_text_arg(&[], "--profile", DEFAULT_EVIDENCE_PROFILE, true);
        let command = parse_optional_text_arg(&[], "--command", DEFAULT_EVIDENCE_COMMAND, false);

        assert_eq!(subject, DEFAULT_EVIDENCE_SUBJECT);
        assert_eq!(profile, DEFAULT_EVIDENCE_PROFILE);
        assert_eq!(command, DEFAULT_EVIDENCE_COMMAND);
    }

    #[test]
    fn parse_optional_text_arg_ignores_blank_user_value() {
        let profile = parse_optional_text_arg(
            &args(&["--profile", "   "]),
            "--profile",
            DEFAULT_EVIDENCE_PROFILE,
            true,
        );

        assert_eq!(profile, DEFAULT_EVIDENCE_PROFILE);
    }

    #[test]
    fn parse_optional_category_filter_treats_blank_as_absent() {
        let category = parse_optional_category_filter(&args(&["--category", "   "]));
        assert_eq!(category, None);
    }

    #[test]
    fn parse_optional_category_filter_trims_value() {
        let category =
            parse_optional_category_filter(&args(&["--category", "  operator_evidence  "]));
        assert_eq!(category.as_deref(), Some("operator_evidence"));
    }

    #[test]
    fn parse_limit_arg_defaults_when_absent() {
        let limit = parse_limit_arg(&[]).expect("default limit should parse");
        assert_eq!(limit, 20);
    }

    #[test]
    fn parse_limit_arg_rejects_invalid_numeric_input() {
        let error =
            parse_limit_arg(&args(&["--limit", "abc"])).expect_err("invalid limit must fail");

        assert_eq!(error.code(), "AOXC-USG-002");
    }

    #[test]
    fn parse_limit_arg_rejects_zero() {
        let error = parse_limit_arg(&args(&["--limit", "0"])).expect_err("zero limit must fail");

        assert_eq!(error.code(), "AOXC-USG-002");
    }

    #[test]
    fn parse_limit_arg_clamps_to_operational_ceiling() {
        let limit = parse_limit_arg(&args(&["--limit", "9999"])).expect("large limit should clamp");

        assert_eq!(limit, MAX_LIST_LIMIT);
    }

    #[test]
    fn parse_limit_arg_accepts_valid_value() {
        let limit = parse_limit_arg(&args(&["--limit", "42"])).expect("valid limit should parse");

        assert_eq!(limit, 42);
    }
}
