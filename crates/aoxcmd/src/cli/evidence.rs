// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    cli_support::{arg_value, emit_serialized, output_format},
    error::{AppError, ErrorCode},
    storage::redb_chain::{append_chain_log, load_chain_logs},
};
use chrono::Utc;
use serde::Serialize;
use sha2::{Digest, Sha256};

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

#[derive(Debug, Serialize, PartialEq, Eq)]
struct OperatorEvidenceEnvelope {
    status: &'static str,
    record: OperatorEvidenceRecord,
}

pub fn cmd_operator_evidence_record(args: &[String]) -> Result<(), AppError> {
    let action = arg_value(args, "--action").ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Missing required flag --action <value>",
        )
    })?;

    let reason = arg_value(args, "--reason").ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Missing required flag --reason <value>",
        )
    })?;

    let subject = arg_value(args, "--subject").unwrap_or_else(|| "operator-plane".to_string());
    let profile = arg_value(args, "--profile").unwrap_or_else(|| "unspecified".to_string());
    let command = arg_value(args, "--command").unwrap_or_else(|| "manual".to_string());

    let created_at = Utc::now().to_rfc3339();
    let evidence_id = compute_evidence_id(&created_at, &action, &reason, &subject, &command);

    let detail = format!(
        "evidence_id={evidence_id} action={action} reason={reason} subject={subject} profile={profile} command={command}"
    );
    append_chain_log("operator_evidence", &action, &detail)?;

    emit_serialized(
        &OperatorEvidenceEnvelope {
            status: "recorded",
            record: OperatorEvidenceRecord {
                evidence_id,
                created_at,
                action,
                reason,
                subject,
                profile,
                command,
            },
        },
        output_format(args),
    )
}

pub fn cmd_operator_evidence_list(args: &[String]) -> Result<(), AppError> {
    let limit = arg_value(args, "--limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(20)
        .min(200);

    let category = arg_value(args, "--category");
    let logs = load_chain_logs(limit, category.as_deref())?;

    emit_serialized(&logs, output_format(args))
}

fn compute_evidence_id(
    created_at: &str,
    action: &str,
    reason: &str,
    subject: &str,
    command: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(created_at.as_bytes());
    hasher.update([0]);
    hasher.update(action.as_bytes());
    hasher.update([0]);
    hasher.update(reason.as_bytes());
    hasher.update([0]);
    hasher.update(subject.as_bytes());
    hasher.update([0]);
    hasher.update(command.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::compute_evidence_id;

    #[test]
    fn evidence_id_is_stable_for_same_input() {
        let a = compute_evidence_id(
            "2026-03-28T00:00:00Z",
            "shutdown_prepare",
            "scheduled-upgrade",
            "node-1",
            "aoxc node-run",
        );
        let b = compute_evidence_id(
            "2026-03-28T00:00:00Z",
            "shutdown_prepare",
            "scheduled-upgrade",
            "node-1",
            "aoxc node-run",
        );
        assert_eq!(a, b);
    }

    #[test]
    fn evidence_id_changes_with_reason() {
        let a = compute_evidence_id(
            "2026-03-28T00:00:00Z",
            "shutdown_prepare",
            "scheduled-upgrade",
            "node-1",
            "aoxc node-run",
        );
        let b = compute_evidence_id(
            "2026-03-28T00:00:00Z",
            "shutdown_prepare",
            "incident-response",
            "node-1",
            "aoxc node-run",
        );
        assert_ne!(a, b);
    }

    #[test]
    fn evidence_id_uses_full_input_domain() {
        let base = compute_evidence_id(
            "2026-03-28T00:00:00Z",
            "key_rotation",
            "scheduled",
            "validator-7",
            "aoxc key-bootstrap",
        );
        let changed_action = compute_evidence_id(
            "2026-03-28T00:00:00Z",
            "key_rotation_force",
            "scheduled",
            "validator-7",
            "aoxc key-bootstrap",
        );
        let changed_subject = compute_evidence_id(
            "2026-03-28T00:00:00Z",
            "key_rotation",
            "scheduled",
            "validator-8",
            "aoxc key-bootstrap",
        );
        let changed_command = compute_evidence_id(
            "2026-03-28T00:00:00Z",
            "key_rotation",
            "scheduled",
            "validator-7",
            "aoxc key-bootstrap --dry-run",
        );

        assert_ne!(base, changed_action);
        assert_ne!(base, changed_subject);
        assert_ne!(base, changed_command);
    }

    #[test]
    fn evidence_id_is_hex_sha256_length() {
        let id = compute_evidence_id("t", "a", "r", "s", "c");
        assert_eq!(id.len(), 64);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
