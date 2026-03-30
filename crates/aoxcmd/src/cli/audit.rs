// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    cli_support::{arg_value, emit_serialized, has_flag, output_format},
    config::{loader::load, settings::Settings},
    data_home::{file_permissions_are_hardened, read_file, resolve_home, write_file},
    error::{AppError, ErrorCode},
    keys::manager::verify_operator_key,
    node::lifecycle::load_state,
};
use chrono::{SecondsFormat, Utc};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
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
#[derive(Debug, Serialize)]
struct AiAssistView {
    authoritative: bool,
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
#[derive(Debug, Serialize)]
struct AuditReport {
    generated_at: String,
    home_dir: String,
    checks: Vec<Check>,
    verdict: &'static str,
    ai_assist: AiAssistView,
}

/// Executes the diagnostics doctor command and emits the resulting report.
pub fn cmd_diagnostics_doctor(args: &[String]) -> Result<(), AppError> {
    let report = build_report(false)?;
    emit_serialized(&report, output_format(args))
}

/// Builds a support bundle containing the diagnostics report and selected
/// operator-relevant local artifacts.
pub fn cmd_diagnostics_bundle(args: &[String]) -> Result<(), AppError> {
    let redact = has_flag(args, "--redact");
    let report = build_report(redact)?;
    let home = resolve_home()?;
    let bundle_dir = home.join("support").join(unique_bundle_name());

    fs::create_dir_all(&bundle_dir).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!(
                "Failed to create support bundle directory {}",
                bundle_dir.display()
            ),
            error,
        )
    })?;

    let report_content = serde_json::to_string_pretty(&report).map_err(|error| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode diagnostics report",
            error,
        )
    })?;
    write_file(&bundle_dir.join("doctor.json"), &report_content)?;

    copy_bundle_artifact(
        &home.join("config").join("settings.json"),
        &bundle_dir.join("settings.json"),
        redact,
        false,
    )?;
    copy_bundle_artifact(
        &home.join("identity").join("genesis.json"),
        &bundle_dir.join("genesis.json"),
        redact,
        false,
    )?;
    copy_bundle_artifact(
        &home.join("runtime").join("db").join("main.redb"),
        &bundle_dir.join("main.redb"),
        false,
        true,
    )?;
    copy_bundle_artifact(
        &home.join("ledger").join("ledger.json"),
        &bundle_dir.join("ledger.json"),
        redact,
        false,
    )?;

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
    let out = arg_value(args, "--out");

    if let Some(path) = out {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            return Err(AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Flag --out must not be blank",
            ));
        }

        let content = serde_json::to_string_pretty(&report).map_err(|error| {
            AppError::with_source(
                ErrorCode::OutputEncodingFailed,
                "Failed to encode production audit report",
                error,
            )
        })?;
        write_file(&PathBuf::from(trimmed), &content)?;
    }

    emit_serialized(&report, output_format(args))
}

/// Builds the operator diagnostics report.
///
/// # Authority Model
/// Native checks determine the final `verdict`. AI output is attached only as
/// optional assistance and does not alter native correctness, readiness truth,
/// or enforcement semantics.
fn build_report(redact: bool) -> Result<AuditReport, AppError> {
    let home = resolve_home()?;
    let settings = effective_settings_for_audit()?;

    let mut checks = Vec::new();

    let config_valid = settings.validate().is_ok();
    checks.push(Check {
        name: "config-valid",
        passed: config_valid,
        detail: if config_valid {
            "Operator configuration passed validation".to_string()
        } else {
            "Operator configuration failed validation".to_string()
        },
    });

    let key_material_ok = verify_operator_key(None).is_ok();
    checks.push(Check {
        name: "key-material",
        passed: key_material_ok,
        detail: if key_material_ok {
            "Operator key material is available and structurally valid".to_string()
        } else {
            "Operator key material is missing or structurally invalid".to_string()
        },
    });

    let genesis_path = home.join("identity").join("genesis.json");
    let genesis_present = genesis_path.exists();
    checks.push(Check {
        name: "genesis-present",
        passed: genesis_present,
        detail: if genesis_present {
            format!("Genesis file is present at {}", genesis_path.display())
        } else {
            format!("Genesis file is missing at {}", genesis_path.display())
        },
    });

    let node_state = load_state();
    checks.push(Check {
        name: "node-state",
        passed: node_state.is_ok(),
        detail: match node_state.as_ref() {
            Ok(_) => "Node runtime state is present and parseable".to_string(),
            Err(error) => error.to_string(),
        },
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
        detail: if settings.network.enforce_official_peers {
            "Official peer enforcement is enabled".to_string()
        } else {
            "Official peer enforcement is disabled".to_string()
        },
    });

    checks.push(Check {
        name: "mainnet-profile",
        passed: settings.profile.eq_ignore_ascii_case("mainnet"),
        detail: format!("Active operator profile is {}", settings.profile),
    });

    checks.push(Check {
        name: "structured-logging",
        passed: settings.logging.json,
        detail: if settings.logging.json {
            "Structured JSON logging is enabled for audit trails".to_string()
        } else {
            "Structured JSON logging is disabled".to_string()
        },
    });

    checks.push(Check {
        name: "telemetry-metrics",
        passed: settings.telemetry.enable_metrics,
        detail: if settings.telemetry.enable_metrics {
            "Telemetry metrics export is enabled".to_string()
        } else {
            "Telemetry metrics export is disabled".to_string()
        },
    });

    let settings_path = home.join("config").join("settings.json");
    checks.push(permission_check(
        "settings-file-permissions",
        &settings_path,
        redact,
    )?);

    let operator_key_path = home.join("keys").join("operator_key.json");
    checks.push(permission_check(
        "operator-key-permissions",
        &operator_key_path,
        redact,
    )?);

    checks.push(permission_check(
        "genesis-file-permissions",
        &genesis_path,
        redact,
    )?);

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
            failed_checks,
        },
    );

    Ok(AuditReport {
        generated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        home_dir: if redact {
            redact_path(&home)
        } else {
            home.display().to_string()
        },
        checks,
        verdict,
        ai_assist: AiAssistView {
            authoritative: false,
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

/// Resolves effective settings for audit surfaces without mutating operator state.
///
/// Behavioral policy:
/// - Existing validated settings are used when present.
/// - Missing configuration falls back to deterministic in-memory defaults.
/// - Invalid configuration remains an explicit failure.
fn effective_settings_for_audit() -> Result<Settings, AppError> {
    match load() {
        Ok(settings) => Ok(settings),
        Err(error) if error.code() == ErrorCode::ConfigMissing.as_str() => {
            let home = resolve_home()?;
            Ok(Settings::default_for(home.display().to_string()))
        }
        Err(error) => Err(error),
    }
}

fn permission_check(name: &'static str, path: &Path, redact: bool) -> Result<Check, AppError> {
    let rendered_path = if redact {
        redact_path(path)
    } else {
        path.display().to_string()
    };

    if !path.exists() {
        return Ok(Check {
            name,
            passed: false,
            detail: format!("Required sensitive file is missing at {}", rendered_path),
        });
    }

    let hardened = file_permissions_are_hardened(path)?;
    Ok(Check {
        name,
        passed: hardened,
        detail: if hardened {
            format!(
                "Verified hardened sensitive file permissions for {}",
                rendered_path
            )
        } else {
            format!(
                "Sensitive file permissions are not hardened for {}",
                rendered_path
            )
        },
    })
}

fn native_verdict(checks: &[Check]) -> &'static str {
    if checks.iter().all(|check| check.passed) {
        "pass"
    } else {
        "fail"
    }
}

/// Copies a support-bundle artifact to the requested target path.
///
/// Behavior:
/// - Missing source files are ignored to keep bundle creation best-effort.
/// - Binary artifacts are copied byte-for-byte.
/// - Text artifacts can optionally be redacted before writing.
fn copy_bundle_artifact(
    source: &Path,
    target: &Path,
    redact: bool,
    binary: bool,
) -> Result<(), AppError> {
    if !source.exists() {
        return Ok(());
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!(
                    "Failed to create bundle artifact directory {}",
                    parent.display()
                ),
                error,
            )
        })?;
    }

    if binary {
        fs::copy(source, target).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!(
                    "Failed to copy binary artifact from {} to {}",
                    source.display(),
                    target.display()
                ),
                error,
            )
        })?;
        return Ok(());
    }

    let raw = read_file(source)?;
    let content = if redact { redact_text(&raw) } else { raw };
    write_file(target, &content)
}

fn redact_text(input: &str) -> String {
    input
        .replace("operator_key", "[REDACTED_KEY_REF]")
        .replace("private_key", "[REDACTED_PRIVATE_KEY]")
        .replace("mnemonic", "[REDACTED_MNEMONIC]")
        .replace("password", "[REDACTED_PASSWORD]")
}

fn redact_path(path: &Path) -> String {
    path.file_name()
        .map(|name| format!("[REDACTED_PATH]/{}", name.to_string_lossy()))
        .unwrap_or_else(|| "[REDACTED_PATH]".to_string())
}

fn unique_bundle_name() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();

    format!("bundle-{}-pid{}", nanos, process::id())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{data_home::write_file, test_support::TestHome};

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

    #[test]
    fn native_verdict_is_determined_only_by_native_checks() {
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
        assert_eq!(native_verdict(&checks), "fail");
    }

    #[test]
    fn permission_check_requires_existing_hardened_sensitive_files() {
        let home = TestHome::new("permission-check");
        let target = home.path().join("keys").join("operator_key.json");
        write_file(&target, "{\"test\":true}").expect("fixture file should be written");

        let check = permission_check("operator-key-permissions", &target, false)
            .expect("check should succeed");

        assert!(check.passed);
        assert!(
            check
                .detail
                .contains("Verified hardened sensitive file permissions")
        );
    }

    #[test]
    fn permission_check_fails_when_sensitive_file_is_missing() {
        let home = TestHome::new("permission-check-missing");
        let target = home.path().join("keys").join("operator_key.json");

        let check = permission_check("operator-key-permissions", &target, false)
            .expect("check should succeed");

        assert!(!check.passed);
        assert!(check.detail.contains("missing"));
    }

    #[test]
    fn redact_path_hides_directory_components() {
        let redacted = redact_path(Path::new("/home/orcun/.AOXCData/keys/operator_key.json"));
        assert_eq!(redacted, "[REDACTED_PATH]/operator_key.json");
    }

    #[test]
    fn redact_text_masks_known_sensitive_terms() {
        let redacted = redact_text("operator_key private_key mnemonic password");
        assert!(redacted.contains("[REDACTED_KEY_REF]"));
        assert!(redacted.contains("[REDACTED_PRIVATE_KEY]"));
        assert!(redacted.contains("[REDACTED_MNEMONIC]"));
        assert!(redacted.contains("[REDACTED_PASSWORD]"));
    }

    #[test]
    fn unique_bundle_name_contains_bundle_prefix() {
        let name = unique_bundle_name();
        assert!(name.starts_with("bundle-"));
        assert!(name.contains("-pid"));
    }
}
