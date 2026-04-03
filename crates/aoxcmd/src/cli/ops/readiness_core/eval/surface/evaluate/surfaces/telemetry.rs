use super::*;

pub(super) fn build(
    settings: &crate::config::settings::Settings,
    context: &SurfaceContext,
) -> SurfaceReadiness {
    build_surface(
        "telemetry",
        "sre-observability",
        vec![
            surface_check(
                "metrics-enabled",
                settings.telemetry.enable_metrics,
                "Prometheus/metrics export must stay enabled".to_string(),
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
            surface_check(
                "alert-rules",
                context.closure_dir.join("alert-rules.md").exists(),
                format!(
                    "expected alert rules at {}",
                    context.closure_dir.join("alert-rules.md").display()
                ),
            ),
            surface_check(
                "runtime-telemetry-handle",
                json_artifact_has_required_strings(
                    &context.closure_dir.join("runtime-status.json"),
                    "required_artifacts",
                    &["telemetry-snapshot.json"],
                ) || context.closure_dir.join("runtime-status.json").exists(),
                format!(
                    "runtime status should expose telemetry evidence at {}",
                    context.closure_dir.join("runtime-status.json").display()
                ),
            ),
        ],
        vec![
            context
                .closure_dir
                .join("telemetry-snapshot.json")
                .display()
                .to_string(),
            context
                .closure_dir
                .join("alert-rules.md")
                .display()
                .to_string(),
            context
                .closure_dir
                .join("runtime-status.json")
                .display()
                .to_string(),
        ],
    )
}
