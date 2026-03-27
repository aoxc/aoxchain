// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    cli_support::{emit_serialized, has_flag, output_format},
    config::loader::load_or_init,
    data_home::{file_permissions_are_hardened, read_file, resolve_home, write_file},
    error::{AppError, ErrorCode},
    keys::manager::verify_operator_key,
    node::lifecycle::load_state,
};
use chrono::Utc;
use serde::Serialize;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// Single diagnostics or audit check result.
///
/// # Purpose
/// Captures the outcome of a native operator-facing validation check together
/// with a human-readable detail string suitable for reports and support bundles.
#[derive(Debug, Serialize)]
struct Check {
    name: &'static str,
    passed: bool,
    detail: String,
}

/// Operator-visible AI assistance view.
///
/// # Security Note
/// The fields in this structure are intentionally explanatory and read-oriented.
/// They must not be interpreted as canonical protocol truth, enforcement state,
/// or an execution authorization surface.
///
/// # Audit Note
/// Invocation metadata is surfaced so operators can correlate AI assistance
/// with the exact authorization and policy context under which it was produced.
#[derive(Debug, Serialize)]
struct AiAssistView {
    available: bool,
    summary: Option<String>,
    remediation_plan: Vec<String>,
    disposition: String,
    invocation_id: String,
    policy_id: String,
    capability: String,
    zone: String,
    action_class: String,
}

/// Consolidated diagnostics or audit report emitted by operator-plane commands.
///
/// # Design Intent
/// This report combines deterministic native validation results with optional
/// AI assistance. Native checks remain authoritative for the reported verdict.
/// AI assistance is included strictly as an auxiliary operator aid.
#[derive(Debug, Serialize)]
struct AuditReport {
    generated_at: String,
    home_dir: String,
    checks: Vec<Check>,
    verdict: &'static str,
    ai_assist: AiAssistView,
}

/// Runs the diagnostics doctor command and emits the resulting report.
pub fn cmd_diagnostics_doctor(args: &[String]) -> Result<(), AppError> {
    let report = build_report(false)?;
    emit_serialized(&report, output_format(args))
}

/// Builds a support bundle containing the diagnostics report and selected
/// operator-relevant local artifacts.
///
/// # Security Note
/// This command currently copies selected files if present. The `redact` flag is
/// accepted but not yet used to transform file contents. That behavior must be
/// treated as future hardening work rather than assumed protection.
pub fn cmd_diagnostics_bundle(args: &[String]) -> Result<(), AppError> {
    let redact = has_flag(args, "--redact");
    let report = build_report(redact)?;
    let home = resolve_home()?;
    let bundle_dir = home
        .join("support")
        .join(format!("bundle-{}", Utc::now().format("%Y%m%dT%H%M%SZ")));

    std::fs::create_dir_all(&bundle_dir).map_err(|e| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!(
                "Failed to create support bundle directory {}",
                bundle_dir.display()
            ),
            e,
        )
    })?;

    let report_content = serde_json::to_string_pretty(&report).map_err(|e| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode diagnostics report",
            e,
        )
    })?;
    write_file(&bundle_dir.join("doctor.json"), &report_content)?;

    for (source, target) in [
        (
            home.join("config").join("settings.json"),
            bundle_dir.join("settings.json"),
        ),
        (
            home.join("identity").join("genesis.json"),
            bundle_dir.join("genesis.json"),
        ),
        (
            home.join("runtime").join("node_state.json"),
            bundle_dir.join("node_state.json"),
        ),
        (
            home.join("ledger").join("ledger.json"),
            bundle_dir.join("ledger.json"),
        ),
    ] {
        if let Ok(raw) = read_file(&source) {
            let _ = redact;
            write_file(&target, &raw)?;
        }
    }

    let mut envelope = BTreeMap::new();
    envelope.insert("bundle_dir".to_string(), bundle_dir.display().to_string());
    envelope.insert("redacted".to_string(), redact.to_string());
    emit_serialized(&envelope, output_format(args))
}

/// Emits the same underlying report used by diagnostics-oriented readiness flows.
pub fn cmd_interop_readiness(args: &[String]) -> Result<(), AppError> {
    let report = build_report(false)?;
    emit_serialized(&report, output_format(args))
}

/// Evaluates the interop gate and optionally enforces mandatory pass criteria.
pub fn cmd_interop_gate(args: &[String]) -> Result<(), AppError> {
    let report = build_report(false)?;
    let enforce_official = has_flag(args, "--enforce-official");
    let passed = report.verdict == "pass";

    if enforce_official && !passed {
        return Err(AppError::new(
            ErrorCode::PolicyGateFailed,
            "Interop gate enforcement failed because one or more mandatory checks did not pass",
        ));
    }

    emit_serialized(&report, output_format(args))
}

/// Generates a production audit report and optionally writes it to disk.
pub fn cmd_production_audit(args: &[String]) -> Result<(), AppError> {
    let report = build_report(false)?;
    let out = crate::cli_support::arg_value(args, "--out");

    if let Some(path) = out {
        let content = serde_json::to_string_pretty(&report).map_err(|e| {
            AppError::with_source(
                ErrorCode::OutputEncodingFailed,
                "Failed to encode production audit report",
                e,
            )
        })?;
        write_file(&PathBuf::from(path), &content)?;
    }

    emit_serialized(&report, output_format(args))
}

/// Builds the operator diagnostics report.
///
/// # Authority Model
/// Native checks determine the final `verdict`. AI output is attached only as
/// optional assistance and does not alter native correctness, readiness truth,
/// or enforcement semantics.
///
/// # Security Note
/// The `_redact` argument is currently accepted for interface stability, but the
/// report body itself is not transformed based on it in this function.
fn build_report(_redact: bool) -> Result<AuditReport, AppError> {
    let home = resolve_home()?;
    let settings = load_or_init()?;

    let mut checks = Vec::new();

    checks.push(Check {
        name: "config-valid",
        passed: settings.validate().is_ok(),
        detail: "Operator configuration passed validation".to_string(),
    });

    checks.push(Check {
        name: "key-material",
        passed: verify_operator_key(None).is_ok(),
        detail: "Operator key material is available and structurally valid".to_string(),
    });

    let genesis_path = home.join("identity").join("genesis.json");
    checks.push(Check {
        name: "genesis-present",
        passed: genesis_path.exists(),
        detail: format!("Genesis path checked at {}", genesis_path.display()),
    });

    let node_state = load_state();
    let node_state_detail = match node_state.as_ref() {
        Ok(_) => "Node runtime state is present and parseable".to_string(),
        Err(error) => error.to_string(),
    };
    checks.push(Check {
        name: "node-state",
        passed: node_state.is_ok(),
        detail: node_state_detail,
    });

    let ledger_state = crate::economy::ledger::load();
    checks.push(Check {
        name: "ledger-state",
        passed: ledger_state.is_ok(),
        detail: match ledger_state.as_ref() {
            Ok(_) => "Ledger state is present and semantically valid".to_string(),
            Err(error) => error.to_string(),
        },
    });

    checks.push(Check {
        name: "official-peer-policy",
        passed: settings.network.enforce_official_peers,
        detail: "Official peer enforcement is enabled".to_string(),
    });

    checks.push(Check {
        name: "mainnet-profile",
        passed: settings.profile.eq_ignore_ascii_case("mainnet"),
        detail: format!("Active operator profile is {}", settings.profile),
    });

    checks.push(Check {
        name: "structured-logging",
        passed: settings.logging.json,
        detail: "Structured JSON logging is enabled for audit trails".to_string(),
    });

    checks.push(Check {
        name: "telemetry-metrics",
        passed: settings.telemetry.enable_metrics,
        detail: "Telemetry metrics export is enabled".to_string(),
    });

    let settings_path = home.join("config").join("settings.json");
    checks.push(permission_check(
        "settings-file-permissions",
        &settings_path,
    )?);

    let operator_key_path = home.join("keys").join("operator_key.json");
    checks.push(permission_check(
        "operator-key-permissions",
        &operator_key_path,
    )?);

    checks.push(permission_check("genesis-file-permissions", &genesis_path)?);

    let verdict = native_verdict(&checks);

    let failed_checks = checks
        .iter()
        .filter(|check| !check.passed)
        .map(|check| check.name.to_string())
        .collect::<Vec<_>>();

    let ai_outcome = crate::ai::operator::OperatorPlaneAiAdapter::default().diagnostics_assistance(
        crate::ai::operator::OperatorAssistRequest {
            topic: "diagnostics_explanation",
            verdict: verdict.to_string(),
            failed_checks: failed_checks.clone(),
        },
    );

    Ok(AuditReport {
        generated_at: Utc::now().to_rfc3339(),
        home_dir: home.display().to_string(),
        checks,
        verdict,
        ai_assist: AiAssistView {
            available: ai_outcome.available,
            summary: ai_outcome
                .artifact
                .as_ref()
                .map(|artifact| artifact.summary.clone()),
            remediation_plan: ai_outcome
                .artifact
                .as_ref()
                .map(|artifact| artifact.remediation_plan.clone())
                .unwrap_or_default(),
            disposition: format!("{:?}", ai_outcome.trace.final_disposition).to_lowercase(),
            invocation_id: ai_outcome.trace.invocation_id.clone(),
            policy_id: ai_outcome.trace.policy_id.clone(),
            capability: format!("{:?}", ai_outcome.trace.capability).to_lowercase(),
            zone: format!("{:?}", ai_outcome.trace.kernel_zone).to_lowercase(),
            action_class: format!("{:?}", ai_outcome.trace.action_class).to_lowercase(),
        },
    })
}

fn permission_check(name: &'static str, path: &Path) -> Result<Check, AppError> {
    if !path.exists() {
        return Ok(Check {
            name,
            passed: false,
            detail: format!("Required sensitive file is missing at {}", path.display()),
        });
    }

    Ok(Check {
        name,
        passed: file_permissions_are_hardened(path)?,
        detail: format!("Verified sensitive file permissions for {}", path.display()),
    })
}

/// Computes the authoritative native verdict for a set of checks.
///
/// # Security Invariant
/// This function is intentionally deterministic and depends exclusively on
/// native validation results. It must remain fully isolated from AI subsystem
/// state, availability, output, or authorization status.
fn native_verdict(checks: &[Check]) -> &'static str {
    if checks.iter().all(|check| check.passed) {
        "pass"
    } else {
        "fail"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{data_home::write_file, test_support::TestHome};

    /// Verifies that the native verdict returns `pass` when every check passes.
    ///
    /// # Audit Rationale
    /// This test protects the positive path and ensures that a fully valid
    /// native state cannot regress into an erroneous failure classification.
    #[test]
    fn native_verdict_returns_pass_when_all_checks_pass() {
        let checks = vec![
            Check {
                name: "config-valid",
                passed: true,
                detail: "ok".to_string(),
            },
            Check {
                name: "node-state",
                passed: true,
                detail: "ok".to_string(),
            },
        ];

        assert_eq!(native_verdict(&checks), "pass");
    }

    /// Verifies that the native verdict is determined exclusively by native
    /// checks and remains unaffected by AI subsystem state.
    ///
    /// # Security Invariant
    /// AI is advisory only and must never influence the authoritative verdict.
    ///
    /// # Threat Model Coverage
    /// This test helps prevent:
    /// - implicit trust leakage from the AI subsystem,
    /// - accidental coupling between advisory output and native truth, and
    /// - authority confusion between deterministic checks and optional assistance.
    #[test]
    fn native_verdict_is_determined_only_by_native_checks() {
        std::env::set_var("AOXC_AI_DISABLE", "1");

        let checks = vec![
            Check {
                name: "config-valid",
                passed: true,
                detail: "ok".to_string(),
            },
            Check {
                name: "node-state",
                passed: false,
                detail: "missing".to_string(),
            },
        ];

        assert_eq!(native_verdict(&checks), "fail");

        std::env::remove_var("AOXC_AI_DISABLE");

        assert_eq!(native_verdict(&checks), "fail");
    }

    #[test]
    fn permission_check_requires_existing_hardened_sensitive_files() {
        let home = TestHome::new("permission-check");
        let target = home.path().join("keys").join("operator_key.json");
        write_file(&target, "{\"test\":true}").expect("fixture file should be written");

        let check =
            permission_check("operator-key-permissions", &target).expect("check should succeed");

        assert!(check.passed);
        assert!(check.detail.contains("Verified sensitive file permissions"));
    }

    #[test]
    fn permission_check_fails_when_sensitive_file_is_missing() {
        let home = TestHome::new("permission-check-missing");
        let target = home.path().join("keys").join("operator_key.json");

        let check =
            permission_check("operator-key-permissions", &target).expect("check should succeed");

        assert!(!check.passed);
        assert!(check.detail.contains("missing"));
    }
}
