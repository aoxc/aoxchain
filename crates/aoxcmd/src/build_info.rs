use serde::Serialize;

/// Canonical build metadata exposed by the AOXC operator command plane.
///
/// This structure intentionally combines compile-time package metadata with
/// runtime identity anchors sourced from the protocol core and covenant kernel.
/// The resulting payload is suitable for operator inspection, diagnostics, and
/// release evidence generation.
#[derive(Debug, Clone, Serialize)]
pub struct BuildInfo {
    /// Cargo package name compiled into the current binary.
    pub package_name: &'static str,

    /// Cargo package version compiled into the current binary.
    pub package_version: &'static str,

    /// Build profile observed at compile time.
    ///
    /// This field is informative only. It must not be treated as an
    /// authorization, policy, or environment trust signal.
    pub build_profile: &'static str,

    /// Embedded genesis digest captured during the build pipeline.
    ///
    /// A sentinel value indicates that the digest was not injected by the
    /// release process and should therefore be treated as absent metadata.
    pub genesis_sha256: &'static str,

    /// Canonical AOXC core identity bound to the compiled binary.
    pub canonical_core: aoxcore::version::CoreIdentity,

    /// Covenant kernel identity bound to the compiled binary.
    pub covenant_kernel: aoxcunity::KernelIdentity,
}

/// Returns the canonical build information payload for the current binary.
///
/// Operational notes:
/// - Package metadata is resolved at compile time.
/// - Genesis metadata is optional and may be absent in local builds.
/// - Core and kernel identities are resolved from the linked protocol crates.
pub fn build_info() -> BuildInfo {
    BuildInfo {
        package_name: env!("CARGO_PKG_NAME"),
        package_version: env!("CARGO_PKG_VERSION"),
        build_profile: option_env!("PROFILE").unwrap_or("unknown"),
        genesis_sha256: option_env!("AOXC_BUILD_GENESIS_SHA256")
            .unwrap_or("build-metadata-missing"),
        canonical_core: aoxcore::version::core_identity(),
        covenant_kernel: aoxcunity::kernel_identity(),
    }
}
