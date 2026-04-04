use super::*;

pub(in crate::cli::ops) fn remediation_plan(checks: &[ReadinessCheck]) -> Vec<String> {
    let mut plan = Vec::new();
    let total_checks = checks.len();
    let passed_checks = checks.iter().filter(|check| check.passed).count();

    for check in checks.iter().filter(|check| !check.passed) {
        let step = match check.name {
            "config-valid" => {
                "Run `aoxc config-validate` and fix the operator settings file before promotion."
            }
            "mainnet-profile" => {
                "Run `aoxc production-bootstrap --profile mainnet --password <value>` or `aoxc config-init --profile mainnet --json-logs`."
            }
            "testnet-profile" => {
                "Run `aoxc production-bootstrap --profile testnet --password <value>` or `aoxc config-init --profile testnet --json-logs`."
            }
            "official-peers" => {
                "Re-enable curated peer enforcement in the operator settings before joining production."
            }
            "telemetry-metrics" => {
                "Keep Prometheus telemetry enabled so production SLOs and alerts remain actionable."
            }
            "structured-logging" => {
                "Enable JSON logging to preserve audit-quality operator trails and SIEM ingestion."
            }
            "genesis-present" => {
                "Materialize genesis with `aoxc genesis-init` or re-run `aoxc production-bootstrap`."
            }
            "node-state-present" => {
                "Initialize runtime state with `aoxc node-bootstrap` or re-run `aoxc production-bootstrap`."
            }
            "operator-key-active" => {
                "Bootstrap or rotate operator keys with `aoxc key-bootstrap --profile mainnet --password <value>`."
            }
            "profile-baseline-parity" => {
                "Run `aoxc profile-baseline --enforce` and align embedded mainnet/testnet configs before promotion."
            }
            "aoxhub-baseline-parity" => {
                "Align `configs/aoxhub/mainnet.toml` and `configs/aoxhub/testnet.toml` so AOXHub rollout controls match promotion policy."
            }
            "release-evidence" => {
                "Regenerate release evidence under `artifacts/release-evidence/` before promotion."
            }
            "production-closure" => {
                "Refresh production closure artifacts under `artifacts/network-production-closure/`."
            }
            "security-drill-evidence" => {
                "Record a fresh security drill with penetration, RPC hardening, and session replay evidence before promotion."
            }
            "desktop-wallet-hub-compat" => {
                "Publish `desktop-wallet-compat.json` proving the desktop wallet remains compatible with AOXHub and both network tracks."
            }
            "compatibility-matrix" => {
                "Publish a fresh compatibility matrix for the candidate release."
            }
            "signature-evidence" => {
                "Attach signature evidence for the candidate binary before release sign-off."
            }
            "sbom-artifact" => {
                "Generate and archive an SBOM/dependency inventory for the candidate release."
            }
            "provenance-attestation" => {
                "Attach provenance attestation evidence before release sign-off."
            }
            _ => continue,
        };

        if !plan.iter().any(|existing| existing == step) {
            plan.push(step.to_string());
        }
    }

    if plan.is_empty() {
        plan.push(
            "Candidate is at 100%; keep running `aoxc mainnet-readiness --enforce --format json` and `aoxc testnet-readiness --enforce --format json` in CI to prevent regressions."
                .to_string(),
        );
    } else {
        let current_ratio = if total_checks == 0 {
            0
        } else {
            (passed_checks * 100) / total_checks
        };
        plan.push(format!(
            "Close remaining blockers to raise readiness from {}% to 100% before release sign-off.",
            current_ratio
        ));
    }

    plan
}
