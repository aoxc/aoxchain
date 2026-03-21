use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct BuildInfo {
    pub semver: &'static str,
    pub git_commit: &'static str,
    pub git_dirty: &'static str,
    pub source_date_epoch: &'static str,
    pub build_profile: &'static str,
    pub release_channel: &'static str,
    pub attestation_hash: &'static str,
    pub cert_path: &'static str,
    pub cert_sha256: &'static str,
    pub cert_error: &'static str,
}

impl BuildInfo {
    pub fn collect() -> Self {
        Self {
            semver: env_or("AOXC_BUILD_SEMVER", env!("CARGO_PKG_VERSION")),
            git_commit: env_or("AOXC_BUILD_GIT_COMMIT", "unknown"),
            git_dirty: env_or("AOXC_BUILD_GIT_DIRTY", "unknown"),
            source_date_epoch: env_or("AOXC_BUILD_SOURCE_DATE_EPOCH", "not-set"),
            build_profile: env_or("AOXC_BUILD_PROFILE", "unknown"),
            release_channel: env_or("AOXC_BUILD_RELEASE_CHANNEL", "dev"),
            attestation_hash: env_or("AOXC_BUILD_ATTESTATION_HASH", "not-configured"),
            cert_path: env_or("AOXC_BUILD_CERT_PATH", "not-configured"),
            cert_sha256: env_or("AOXC_BUILD_CERT_SHA256", "not-configured"),
            cert_error: env_or("AOXC_BUILD_CERT_ERROR", "none"),
        }
    }
}

fn env_or(key: &str, default: &'static str) -> &'static str {
    option_env_any(key).unwrap_or(default)
}

fn option_env_any(key: &str) -> Option<&'static str> {
    match key {
        "AOXC_BUILD_SEMVER" => option_env!("AOXC_BUILD_SEMVER"),
        "AOXC_BUILD_GIT_COMMIT" => option_env!("AOXC_BUILD_GIT_COMMIT"),
        "AOXC_BUILD_GIT_DIRTY" => option_env!("AOXC_BUILD_GIT_DIRTY"),
        "AOXC_BUILD_SOURCE_DATE_EPOCH" => option_env!("AOXC_BUILD_SOURCE_DATE_EPOCH"),
        "AOXC_BUILD_PROFILE" => option_env!("AOXC_BUILD_PROFILE"),
        "AOXC_BUILD_RELEASE_CHANNEL" => option_env!("AOXC_BUILD_RELEASE_CHANNEL"),
        "AOXC_BUILD_ATTESTATION_HASH" => option_env!("AOXC_BUILD_ATTESTATION_HASH"),
        "AOXC_BUILD_CERT_PATH" => option_env!("AOXC_BUILD_CERT_PATH"),
        "AOXC_BUILD_CERT_SHA256" => option_env!("AOXC_BUILD_CERT_SHA256"),
        "AOXC_BUILD_CERT_ERROR" => option_env!("AOXC_BUILD_CERT_ERROR"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::BuildInfo;

    #[test]
    fn build_info_has_non_empty_version() {
        let info = BuildInfo::collect();
        assert!(!info.semver.trim().is_empty());
        assert!(!info.release_channel.trim().is_empty());
    }
}
