use super::*;

pub(super) fn build(repo_root: &Path) -> SurfaceReadiness {
    let consensus_gate = crate::cli::bootstrap::consensus_profile_gate_status(None, None);

    build_surface(
        "quantum-consensus",
        "protocol-security",
        vec![
            surface_check(
                "consensus-profile-gate",
                consensus_gate
                    .as_ref()
                    .map(|status| status.passed)
                    .unwrap_or(false),
                consensus_gate
                    .as_ref()
                    .map(|status| {
                        if status.passed {
                            status.detail.clone()
                        } else if status.blockers.is_empty() {
                            format!("{}; verdict={}", status.detail, status.verdict)
                        } else {
                            format!(
                                "{}; blockers={}",
                                status.detail,
                                status.blockers.join(" | ")
                            )
                        }
                    })
                    .unwrap_or_else(|error| {
                        format!("consensus profile gate unavailable: {}", error)
                    }),
            ),
            surface_check(
                "consensus-hybrid-or-pq-policy",
                consensus_gate
                    .as_ref()
                    .map(|status| !status.detail.contains("consensus_profile=classical"))
                    .unwrap_or(false),
                "mainnet candidate path must avoid classical-only consensus profile".to_string(),
            ),
        ],
        vec![
            repo_root
                .join("identity")
                .join("genesis.json")
                .display()
                .to_string(),
        ],
    )
}
