// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    cli_support::{arg_value, emit_serialized, output_format},
    error::{AppError, ErrorCode},
    storage::redb_chain::{append_chain_log, load_chain_logs, ChainLogEntry},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const PQ_MODE_HYBRID: &str = "hybrid";
const PQ_MODE_FULL: &str = "full";
const OPERATOR_EVIDENCE_VERSION: &str = "aoxc.operator.evidence.v2";
const EVIDENCE_COMMITMENT_DOMAIN_V2: &[u8] = b"AOXC_OPERATOR_EVIDENCE_COMMITMENT_V2";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct PostQuantumContext {
    mode: String,
    signature_scheme: String,
    attestation_root: String,
    signer_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct OperatorEvidenceRecord {
    version: String,
    evidence_id: String,
    evidence_commitment: String,
    created_at: String,
    action: String,
    reason: String,
    subject: String,
    profile: String,
    command: String,
    pq: PostQuantumContext,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct OperatorEvidenceEnvelope {
    status: &'static str,
    record: OperatorEvidenceRecord,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct OperatorEvidenceListItem {
    log_id: String,
    logged_at: String,
    action: String,
    category: String,
    record: Option<OperatorEvidenceRecord>,
    raw_detail: String,
}

pub fn cmd_operator_evidence_record(args: &[String]) -> Result<(), AppError> {
    let action = required_arg(args, "--action")?;
    let reason = required_arg(args, "--reason")?;

    let subject = arg_value(args, "--subject").unwrap_or_else(|| "operator-plane".to_string());
    let profile = arg_value(args, "--profile").unwrap_or_else(|| "unspecified".to_string());
    let command = arg_value(args, "--command").unwrap_or_else(|| "manual".to_string());

    let pq_mode = arg_value(args, "--pq-mode").unwrap_or_else(|| PQ_MODE_HYBRID.to_string());
    ensure_valid_pq_mode(&pq_mode)?;

    let attestation_root = required_arg(args, "--pq-attestation-root")?;
    ensure_hex_len(&attestation_root, 64, "--pq-attestation-root")?;

    let signer_fingerprint = required_arg(args, "--pq-signer-fingerprint")?;
    ensure_hex_len(&signer_fingerprint, 64, "--pq-signer-fingerprint")?;

    let signature_scheme = arg_value(args, "--signature-scheme")
        .unwrap_or_else(|| "dilithium3+ed25519-hybrid".to_string());

    let created_at = Utc::now().to_rfc3339();
    let evidence_id = compute_evidence_id(&created_at, &action, &reason, &subject, &command);

    let pq = PostQuantumContext {
        mode: pq_mode,
        signature_scheme,
        attestation_root,
        signer_fingerprint,
    };

    let evidence_commitment = compute_evidence_commitment(
        &created_at,
        &action,
        &reason,
        &subject,
        &profile,
        &command,
        &pq,
    );

    let record = OperatorEvidenceRecord {
        version: OPERATOR_EVIDENCE_VERSION.to_string(),
        evidence_id,
        evidence_commitment,
        created_at,
        action: action.clone(),
        reason,
        subject,
        profile,
        command,
        pq,
    };

    let detail = serde_json::to_string(&record).map_err(|e| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode operator evidence record",
            e,
        )
    })?;

    append_chain_log("operator_evidence", &action, &detail)?;

    emit_serialized(
        &OperatorEvidenceEnvelope {
            status: "recorded",
            record,
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

    let result = logs
        .into_iter()
        .map(|entry| {
            let record = try_parse_record(&entry);
            OperatorEvidenceListItem {
                log_id: entry.id,
                logged_at: entry.ts,
                action: entry.action,
                category: entry.category,
                record,
                raw_detail: entry.detail,
            }
        })
        .collect::<Vec<_>>();

    emit_serialized(&result, output_format(args))
}

fn required_arg(args: &[String], flag: &str) -> Result<String, AppError> {
    arg_value(args, flag).ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Missing required flag {flag} <value>"),
        )
    })
}

fn ensure_valid_pq_mode(mode: &str) -> Result<(), AppError> {
    match mode.trim().to_ascii_lowercase().as_str() {
        PQ_MODE_HYBRID | PQ_MODE_FULL => Ok(()),
        _ => Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!(
                "Invalid --pq-mode value `{mode}`. Supported values: {PQ_MODE_HYBRID}, {PQ_MODE_FULL}"
            ),
        )),
    }
}

fn ensure_hex_len(value: &str, expected_len: usize, flag: &str) -> Result<(), AppError> {
    if value.len() != expected_len || !value.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!(
                "Invalid {flag} value: expected {expected_len} hex characters, received `{value}`"
            ),
        ));
    }
    Ok(())
}

fn try_parse_record(entry: &ChainLogEntry) -> Option<OperatorEvidenceRecord> {
    if entry.category != "operator_evidence" {
        return None;
    }

    serde_json::from_str::<OperatorEvidenceRecord>(&entry.detail).ok()
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

fn compute_evidence_commitment(
    created_at: &str,
    action: &str,
    reason: &str,
    subject: &str,
    profile: &str,
    command: &str,
    pq: &PostQuantumContext,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(EVIDENCE_COMMITMENT_DOMAIN_V2);
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
    hasher.update([0]);
    hasher.update(pq.mode.as_bytes());
    hasher.update([0]);
    hasher.update(pq.signature_scheme.as_bytes());
    hasher.update([0]);
    hasher.update(pq.attestation_root.as_bytes());
    hasher.update([0]);
    hasher.update(pq.signer_fingerprint.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::{
        compute_evidence_commitment, compute_evidence_id, ensure_hex_len, ensure_valid_pq_mode,
        PostQuantumContext,
    };

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
    fn evidence_commitment_changes_when_pq_context_changes() {
        let pq_a = PostQuantumContext {
            mode: "hybrid".to_string(),
            signature_scheme: "dilithium3+ed25519-hybrid".to_string(),
            attestation_root: "A".repeat(64),
            signer_fingerprint: "B".repeat(64),
        };
        let pq_b = PostQuantumContext {
            mode: "full".to_string(),
            signature_scheme: "dilithium3".to_string(),
            attestation_root: "A".repeat(64),
            signer_fingerprint: "B".repeat(64),
        };

        let a = compute_evidence_commitment(
            "2026-03-28T00:00:00Z",
            "emergency_freeze",
            "safety-threshold",
            "validator-set",
            "mainnet",
            "aoxc node-run --safe",
            &pq_a,
        );
        let b = compute_evidence_commitment(
            "2026-03-28T00:00:00Z",
            "emergency_freeze",
            "safety-threshold",
            "validator-set",
            "mainnet",
            "aoxc node-run --safe",
            &pq_b,
        );

        assert_ne!(a, b);
    }

    #[test]
    fn pq_mode_validation_accepts_hybrid_and_full() {
        assert!(ensure_valid_pq_mode("hybrid").is_ok());
        assert!(ensure_valid_pq_mode("full").is_ok());
        assert!(ensure_valid_pq_mode("FULL").is_ok());
    }

    #[test]
    fn pq_mode_validation_rejects_unsupported_values() {
        assert!(ensure_valid_pq_mode("legacy").is_err());
        assert!(ensure_valid_pq_mode(" ").is_err());
    }

    #[test]
    fn hex_len_validation_enforces_hex_shape() {
        assert!(ensure_hex_len(&"A".repeat(64), 64, "--x").is_ok());
        assert!(ensure_hex_len("xyz", 64, "--x").is_err());
        assert!(ensure_hex_len(&"A".repeat(63), 64, "--x").is_err());
    }
}
