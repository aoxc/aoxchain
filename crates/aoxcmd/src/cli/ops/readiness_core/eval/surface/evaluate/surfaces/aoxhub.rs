use super::*;

pub(super) fn build(context: &SurfaceContext) -> SurfaceReadiness {
    build_surface(
        "aoxhub",
        "hub-platform",
        vec![
            surface_check(
                "mainnet-profile",
                context.aoxhub_mainnet.exists(),
                format!(
                    "expected AOXHub mainnet config at {}",
                    context.aoxhub_mainnet.display()
                ),
            ),
            surface_check(
                "testnet-profile",
                context.aoxhub_testnet.exists(),
                format!(
                    "expected AOXHub testnet config at {}",
                    context.aoxhub_testnet.display()
                ),
            ),
            surface_check(
                "rollout-evidence",
                context.closure_dir.join("aoxhub-rollout.json").exists(),
                format!(
                    "expected AOXHub rollout artifact at {}",
                    context.closure_dir.join("aoxhub-rollout.json").display()
                ),
            ),
            surface_check(
                "baseline-parity",
                compare_aoxhub_network_profiles()
                    .map(|report| report.passed)
                    .unwrap_or(false),
                "AOXHub mainnet/testnet baseline parity must hold".to_string(),
            ),
        ],
        vec![
            context.aoxhub_mainnet.display().to_string(),
            context.aoxhub_testnet.display().to_string(),
            context
                .closure_dir
                .join("aoxhub-rollout.json")
                .display()
                .to_string(),
        ],
    )
}
