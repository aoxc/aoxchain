use super::*;

pub(super) fn build(
    mainnet_readiness: &Readiness,
    context: &SurfaceContext,
    repo_root: &Path,
) -> SurfaceReadiness {
    build_surface(
        "mainnet",
        "protocol-release",
        vec![
            surface_check(
                "candidate-threshold",
                mainnet_readiness.verdict == "candidate",
                format!(
                    "mainnet-readiness verdict is {} at {}%",
                    mainnet_readiness.verdict, mainnet_readiness.readiness_score
                ),
            ),
            surface_check(
                "mainnet-config-present",
                context.mainnet_config.exists(),
                format!("expected config at {}", context.mainnet_config.display()),
            ),
            surface_check(
                "release-evidence-bundle",
                has_release_evidence(&context.release_dir),
                format!(
                    "release evidence bundle under {}",
                    context.release_dir.display()
                ),
            ),
            surface_check(
                "release-provenance-bundle",
                has_release_provenance_bundle(&context.release_dir),
                format!(
                    "release provenance artifacts must exist under {}",
                    context.release_dir.display()
                ),
            ),
            surface_check(
                "api-admission-controls",
                repo_root
                    .join("crates")
                    .join("aoxcrpc")
                    .join("src")
                    .join("middleware")
                    .join("rate_limiter.rs")
                    .exists()
                    && repo_root
                        .join("crates")
                        .join("aoxcrpc")
                        .join("src")
                        .join("middleware")
                        .join("mtls_auth.rs")
                        .exists()
                    && repo_root.join("NETWORK_SECURITY_ARCHITECTURE.md").exists(),
                "RPC admission controls require rate-limiter, mTLS middleware, and network security architecture baseline".to_string(),
            ),
        ],
        vec![
            context.mainnet_checklist.display().to_string(),
            context.release_dir.display().to_string(),
        ],
    )
}
