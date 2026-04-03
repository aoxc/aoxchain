use super::*;

pub(super) fn build(context: &SurfaceContext) -> SurfaceReadiness {
    build_surface(
        "devnet",
        "engineering-platform",
        vec![
            surface_check(
                "devnet-config-present",
                context.devnet_config.exists(),
                format!("expected config at {}", context.devnet_config.display()),
            ),
            surface_check(
                "devnet-fixture-present",
                context.devnet_fixture.exists(),
                format!(
                    "expected deterministic devnet fixture at {}",
                    context.devnet_fixture.display()
                ),
            ),
            surface_check(
                "telemetry-snapshot",
                context.closure_dir.join("telemetry-snapshot.json").exists(),
                format!(
                    "expected telemetry snapshot at {}",
                    context
                        .closure_dir
                        .join("telemetry-snapshot.json")
                        .display()
                ),
            ),
        ],
        vec![
            context.devnet_config.display().to_string(),
            context.devnet_fixture.display().to_string(),
            context
                .closure_dir
                .join("telemetry-snapshot.json")
                .display()
                .to_string(),
        ],
    )
}
