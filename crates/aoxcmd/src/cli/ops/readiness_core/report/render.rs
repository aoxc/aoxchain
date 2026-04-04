use super::*;

pub(in crate::cli::ops) fn readiness_markdown_report(
    readiness: &Readiness,
    embedded_baseline: Option<&ProfileBaselineReport>,
    aoxhub_baseline: Option<&ProfileBaselineReport>,
) -> String {
    let mut out = String::new();
    out.push_str("# AOXC Progress Report\n\n");
    out.push_str(&format!(
        "- Profile: `{}`\n- Stage: `{}`\n- Overall readiness: **{}%** ({}/{})\n- Verdict: `{}`\n\n",
        readiness.profile,
        readiness.stage,
        readiness.readiness_score,
        readiness.completed_weight,
        readiness.max_score,
        readiness.verdict,
    ));

    out.push_str("## Dual-track progress\n\n");
    for track in &readiness.track_progress {
        out.push_str(&format!(
            "- **{}**: {}% ({}/{}) — {}\n  - Objective: {}\n",
            track.name,
            track.ratio,
            track.completed_weight,
            track.max_weight,
            track.status,
            track.objective
        ));
    }

    out.push_str("\n## Area progress\n\n");
    for area in &readiness.area_progress {
        out.push_str(&format!(
            "- **{}**: {}% ({}/{} checks, weight {}/{}) — {}\n",
            area.area,
            area.ratio,
            area.passed_checks,
            area.total_checks,
            area.completed_weight,
            area.max_weight,
            area.status
        ));
    }

    out.push_str("\n## Remaining blockers\n\n");
    if readiness.blockers.is_empty() {
        out.push_str("- No active blockers.\n");
    } else {
        for blocker in &readiness.blockers {
            out.push_str(&format!("- {}\n", blocker));
        }
    }

    out.push_str("\n## Recommended next focus\n\n");
    if readiness.next_focus.is_empty() {
        out.push_str("- Keep CI enforcement active and preserve current closure state.\n");
    } else {
        for focus in &readiness.next_focus {
            out.push_str(&format!("- {}\n", focus));
        }
    }

    out.push_str("\n## Remediation plan\n\n");
    for step in &readiness.remediation_plan {
        out.push_str(&format!("- {}\n", step));
    }

    out.push_str("\n## Baseline parity\n\n");
    append_baseline_section(&mut out, "Embedded network profiles", embedded_baseline);
    append_baseline_section(&mut out, "AOXHub network profiles", aoxhub_baseline);

    out.push_str("\n## Check matrix\n\n");
    for check in &readiness.checks {
        let marker = if check.passed { "PASS" } else { "FAIL" };
        out.push_str(&format!(
            "- [{}] **{}** / {} / weight {} — {}\n",
            marker, check.name, check.area, check.weight, check.detail
        ));
    }

    out
}

pub(in crate::cli::ops) fn full_surface_markdown_report(
    readiness: &FullSurfaceReadiness,
) -> String {
    let mut out = String::new();
    out.push_str("# AOXC Full-Surface Readiness Report\n\n");
    out.push_str(&format!(
        "- Release line: `{}`\n- Matrix path: `{}`\n- Matrix loaded: `{}`\n- Matrix release line: `{}`\n- Matrix surface count: `{}`\n- Overall status: `{}`\n- Overall score: **{}%**\n- Candidate surfaces: **{}/{}**\n\n",
        readiness.release_line,
        readiness.matrix_path,
        readiness.matrix_loaded,
        readiness
            .matrix_release_line
            .as_deref()
            .unwrap_or("unavailable"),
        readiness.matrix_surface_count,
        readiness.overall_status,
        readiness.overall_score,
        readiness.candidate_surfaces,
        readiness.total_surfaces,
    ));

    out.push_str("## Matrix validation\n\n");
    if readiness.matrix_warnings.is_empty() {
        out.push_str("- Canonical matrix matches the runtime readiness surface map.\n");
    } else {
        for warning in &readiness.matrix_warnings {
            out.push_str(&format!("- {}\n", warning));
        }
    }

    out.push_str("## Surface summary\n\n");
    for surface in &readiness.surfaces {
        let passed_checks = surface.checks.iter().filter(|check| check.passed).count();
        let total_checks = surface.checks.len();
        out.push_str(&format!(
            "- **{}** / owner `{}` — status `{}` — score **{}%** ({}/{})\n",
            surface.surface,
            surface.owner,
            surface.status,
            surface.score,
            passed_checks,
            total_checks
        ));
    }

    out.push_str("\n## Global blockers\n\n");
    if readiness.blockers.is_empty() {
        out.push_str("- No active blockers.\n");
    } else {
        for blocker in &readiness.blockers {
            out.push_str(&format!("- {}\n", blocker));
        }
    }

    out.push_str("\n## Next focus\n\n");
    if readiness.next_focus.is_empty() {
        out.push_str("- Preserve current candidate state and keep evidence fresh.\n");
    } else {
        for focus in &readiness.next_focus {
            out.push_str(&format!("- {}\n", focus));
        }
    }

    out.push_str("\n## Surface details\n\n");
    for surface in &readiness.surfaces {
        let passed_checks = surface.checks.iter().filter(|check| check.passed).count();
        let total_checks = surface.checks.len();
        out.push_str(&format!(
            "### {} ({})\n\n- Owner: `{}`\n- Status: `{}`\n- Score: **{}%** ({}/{})\n",
            surface.surface,
            surface.surface.to_uppercase(),
            surface.owner,
            surface.status,
            surface.score,
            passed_checks,
            total_checks
        ));

        out.push_str("- Evidence:\n");
        for item in &surface.evidence {
            out.push_str(&format!("  - `{}`\n", item));
        }

        out.push_str("- Checks:\n");
        for check in &surface.checks {
            out.push_str(&format!(
                "  - [{}] {} — {}\n",
                if check.passed { "PASS" } else { "FAIL" },
                check.name,
                check.detail
            ));
        }

        out.push_str("- Next actions:\n");
        if surface.blockers.is_empty() {
            out.push_str("  - Keep evidence current and preserve candidate posture.\n");
        } else {
            for blocker in &surface.blockers {
                out.push_str(&format!("  - Close blocker: {}\n", blocker));
            }
        }

        if !surface.blockers.is_empty() {
            out.push_str("- Blockers:\n");
            for blocker in &surface.blockers {
                out.push_str(&format!("  - {}\n", blocker));
            }
        }

        out.push('\n');
    }

    out
}

pub(in crate::cli::ops) fn append_baseline_section(
    out: &mut String,
    title: &str,
    baseline: Option<&ProfileBaselineReport>,
) {
    out.push_str(&format!("### {}\n\n", title));
    match baseline {
        Some(report) => {
            out.push_str(&format!(
                "- Status: **{}**\n",
                if report.passed {
                    "aligned"
                } else {
                    "drift-detected"
                }
            ));
            out.push_str(&format!("- Mainnet file: `{}`\n", report.mainnet_path));
            out.push_str(&format!("- Testnet file: `{}`\n", report.testnet_path));
            for control in &report.shared_controls {
                out.push_str(&format!(
                    "- {}: {} (mainnet=`{}`, testnet=`{}`)\n",
                    control.name,
                    if control.passed { "ok" } else { "drift" },
                    control.mainnet,
                    control.testnet
                ));
            }
            if !report.drift.is_empty() {
                out.push_str("- Drift summary:\n");
                for drift in &report.drift {
                    out.push_str(&format!("  - {}\n", drift));
                }
            }
        }
        None => out.push_str("- Status: unavailable\n"),
    }
    out.push('\n');
}

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
