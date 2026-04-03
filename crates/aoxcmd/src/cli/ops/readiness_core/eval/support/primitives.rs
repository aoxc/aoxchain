use super::*;

pub(in crate::cli::ops) fn locate_repo_artifact_dir(artifact_name: &str) -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for candidate in cwd.ancestors() {
        let artifact_dir = candidate.join("artifacts").join(artifact_name);
        if artifact_dir.exists() {
            return artifact_dir;
        }
    }
    cwd.join("artifacts").join(artifact_name)
}

pub(in crate::cli::ops) fn surface_check(
    name: &'static str,
    passed: bool,
    detail: String,
) -> SurfaceCheck {
    SurfaceCheck {
        name,
        passed,
        detail,
    }
}

pub(in crate::cli::ops) fn build_surface(
    surface: &'static str,
    owner: &'static str,
    checks: Vec<SurfaceCheck>,
    evidence: Vec<String>,
) -> SurfaceReadiness {
    let blockers = checks
        .iter()
        .filter(|check| !check.passed)
        .map(|check| format!("{}: {}", check.name, check.detail))
        .collect::<Vec<_>>();
    let passed = checks.iter().filter(|check| check.passed).count() as u16;
    let score = if checks.is_empty() {
        0
    } else {
        (passed * 100 / checks.len() as u16) as u8
    };

    SurfaceReadiness {
        surface,
        owner,
        status: if blockers.is_empty() {
            "ready"
        } else if score >= 50 {
            "hardening"
        } else {
            "blocked"
        },
        score,
        blockers,
        evidence,
        checks,
    }
}

pub(in crate::cli::ops) fn collect_surface_gate_failures(
    readiness: &FullSurfaceReadiness,
) -> Vec<SurfaceGateFailure> {
    let mut failures = Vec::new();

    for surface in &readiness.surfaces {
        for check in &surface.checks {
            if check.passed {
                continue;
            }
            failures.push(SurfaceGateFailure {
                surface: surface.surface.to_string(),
                check: check.name.to_string(),
                code: gate_failure_code(surface.surface, check.name),
                detail: check.detail.clone(),
            });
        }
    }

    failures
}

pub(in crate::cli::ops) fn gate_failure_code(surface: &str, check: &str) -> String {
    let surface_token = surface
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>()
        .to_ascii_uppercase();
    let check_token = check
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>()
        .to_ascii_uppercase();
    format!("AOXC_GATE_{}_{}", surface_token, check_token)
}

pub(in crate::cli::ops) fn readiness_check(
    name: &'static str,
    area: &'static str,
    passed: bool,
    weight: u8,
    detail: String,
) -> ReadinessCheck {
    ReadinessCheck {
        name,
        area,
        passed,
        weight,
        detail,
    }
}
