use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct BuildInfo {
    pub package_name: &'static str,
    pub package_version: &'static str,
    pub profile: &'static str,
    pub genesis_sha256: &'static str,
    pub canonical_core: aoxcore::version::CoreIdentity,
    pub covenant_kernel: aoxcunity::KernelIdentity,
}

pub fn build_info() -> BuildInfo {
    BuildInfo {
        package_name: env!("CARGO_PKG_NAME"),
        package_version: env!("CARGO_PKG_VERSION"),
        profile: option_env!("PROFILE").unwrap_or("unknown"),
        genesis_sha256: option_env!("AOXC_BUILD_GENESIS_SHA256").unwrap_or("unavailable"),
        canonical_core: aoxcore::version::core_identity(),
        covenant_kernel: aoxcunity::kernel_identity(),
    }
}
