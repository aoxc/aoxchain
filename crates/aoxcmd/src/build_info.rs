use serde::Serialize;

/// Canonical build metadata exposed by the AOXC operator command plane.
///
/// Design goals:
/// - Provide a stable, operator-readable identity envelope for the current binary.
/// - Distinguish mandatory compile-time package metadata from optional release metadata.
/// - Preserve strong semantic typing for fields that may legitimately be absent in
///   local, development, or non-release builds.
///
/// Serialization notes:
/// - Optional fields are omitted when unavailable in order to keep the exported
///   payload semantically precise.
/// - Identity anchors are resolved from linked protocol crates and therefore
///   represent the effective binary composition rather than untrusted external input.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct BuildInfo {
    /// Cargo package name embedded into the compiled binary.
    pub package_name: &'static str,

    /// Cargo package version embedded into the compiled binary.
    pub package_version: &'static str,

    /// Build profile observed at compile time.
    ///
    /// This field is informational only. It must not be interpreted as an
    /// authorization control, trust boundary, or runtime policy signal.
    pub build_profile: BuildProfile,

    /// Optional genesis digest injected by the release pipeline.
    ///
    /// Presence indicates that the build pipeline supplied a canonical genesis
    /// fingerprint. Absence typically indicates a local, ad hoc, or otherwise
    /// non-release build.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genesis_sha256: Option<&'static str>,

    /// Canonical AOXC core identity linked into the current binary.
    pub canonical_core: aoxcore::version::CoreIdentity,

    /// Covenant kernel identity linked into the current binary.
    pub covenant_kernel: aoxcunity::KernelIdentity,
}

/// Canonicalized build profile classification.
///
/// Rationale:
/// - Using an enum instead of a free-form string improves payload consistency.
/// - Unknown values are preserved explicitly rather than silently collapsed into
///   misleading assumptions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildProfile {
    /// Standard Cargo development profile.
    Dev,

    /// Standard Cargo release profile.
    Release,

    /// Any non-standard or unavailable build profile.
    Unknown,
}

impl BuildProfile {
    /// Resolves the canonical build profile from compile-time metadata.
    fn current() -> Self {
        match option_env!("PROFILE") {
            Some("dev") => Self::Dev,
            Some("release") => Self::Release,
            Some(_) | None => Self::Unknown,
        }
    }
}

/// Returns the canonical build information payload for the current binary.
///
/// Operational notes:
/// - Package metadata is resolved entirely at compile time.
/// - Optional release metadata is represented explicitly using `Option`.
/// - Core and kernel identities are resolved from linked protocol crates and
///   therefore reflect the effective protocol composition of the binary.
pub fn build_info() -> BuildInfo {
    BuildInfo {
        package_name: env!("CARGO_PKG_NAME"),
        package_version: env!("CARGO_PKG_VERSION"),
        build_profile: BuildProfile::current(),
        genesis_sha256: option_env!("AOXC_BUILD_GENESIS_SHA256"),
        canonical_core: aoxcore::version::core_identity(),
        covenant_kernel: aoxcunity::kernel_identity(),
    }
}
