use super::*;

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
