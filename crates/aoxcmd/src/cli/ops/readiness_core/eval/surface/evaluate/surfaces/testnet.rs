use super::*;

pub(super) fn build(context: &SurfaceContext) -> SurfaceReadiness {
    build_surface(
        "testnet",
        "network-operations",
        vec![
            surface_check(
                "testnet-config-present",
                context.testnet_config.exists(),
                format!("expected config at {}", context.testnet_config.display()),
            ),
            surface_check(
                "deterministic-fixture",
                context.testnet_fixture_exists,
                format!(
                    "expected canonical testnet genesis fixture at {}",
                    context.testnet_fixture_v1.display()
                ),
            ),
            surface_check(
                "launch-script",
                context.testnet_launch.exists(),
                format!(
                    "expected launch script at {}",
                    context.testnet_launch.display()
                ),
            ),
            surface_check(
                "multi-host-validation-entrypoint",
                context.multi_host.exists(),
                format!(
                    "expected validation script at {}",
                    context.multi_host.display()
                ),
            ),
        ],
        vec![
            context.testnet_fixture_v1.display().to_string(),
            context.multi_host.display().to_string(),
        ],
    )
}
