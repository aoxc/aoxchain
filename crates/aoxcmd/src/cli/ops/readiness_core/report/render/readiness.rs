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
