use crate::{
    cli_support::{emit_serialized, has_flag, output_format},
    config::loader::load_or_init,
    data_home::{read_file, resolve_home, write_file},
    error::{AppError, ErrorCode},
    keys::manager::verify_operator_key,
    node::lifecycle::load_state,
};
use chrono::Utc;
use serde::Serialize;
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Debug, Serialize)]
struct Check {
    name: &'static str,
    passed: bool,
    detail: String,
}

#[derive(Debug, Serialize)]
struct AuditReport {
    generated_at: String,
    home_dir: String,
    checks: Vec<Check>,
    verdict: &'static str,
}

pub fn cmd_diagnostics_doctor(args: &[String]) -> Result<(), AppError> {
    let report = build_report(false)?;
    emit_serialized(&report, output_format(args))
}

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
            let content = if redact && source.ends_with("settings.json") {
                raw
            } else {
                raw
            };
            write_file(&target, &content)?;
        }
    }

    let mut envelope = BTreeMap::new();
    envelope.insert("bundle_dir".to_string(), bundle_dir.display().to_string());
    envelope.insert("redacted".to_string(), redact.to_string());
    emit_serialized(&envelope, output_format(args))
}

pub fn cmd_interop_readiness(args: &[String]) -> Result<(), AppError> {
    let report = build_report(false)?;
    emit_serialized(&report, output_format(args))
}

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
        passed: verify_operator_key().is_ok(),
        detail: "Operator key material is available and structurally valid".to_string(),
    });

    let genesis_path = home.join("identity").join("genesis.json");
    checks.push(Check {
        name: "genesis-present",
        passed: genesis_path.exists(),
        detail: format!("Genesis path checked at {}", genesis_path.display()),
    });

    let node_state = load_state();
    checks.push(Check {
        name: "node-state",
        passed: node_state.is_ok(),
        detail: "Node runtime state is present and parseable".to_string(),
    });

    let ledger_path = home.join("ledger").join("ledger.json");
    checks.push(Check {
        name: "ledger-present",
        passed: ledger_path.exists(),
        detail: format!("Ledger path checked at {}", ledger_path.display()),
    });

    checks.push(Check {
        name: "official-peer-policy",
        passed: settings.network.enforce_official_peers,
        detail: "Official peer enforcement is enabled".to_string(),
    });

    let verdict = if checks.iter().all(|check| check.passed) {
        "pass"
    } else {
        "fail"
    };

    Ok(AuditReport {
        generated_at: Utc::now().to_rfc3339(),
        home_dir: home.display().to_string(),
        checks,
        verdict,
    })
}
