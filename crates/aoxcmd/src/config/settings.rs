use serde::{Deserialize, Serialize};
use std::path::Path;

/// Canonical AOXC CLI settings document.
///
/// This structure intentionally reflects the AOXC single-binary,
/// multi-network operating model. Profile-specific behavior is derived from
/// canonical environment classes rather than legacy validator-centric defaults.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Settings {
    pub home_dir: String,
    pub profile: String,
    pub logging: LoggingSettings,
    pub network: NetworkSettings,
    pub telemetry: TelemetrySettings,
    pub policy: PolicySettings,
}

/// Logging-related runtime settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoggingSettings {
    pub level: String,
    pub json: bool,
}

/// Network-facing runtime settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkSettings {
    pub bind_host: String,
    pub p2p_port: u16,
    pub rpc_port: u16,
    pub enforce_official_peers: bool,
}

/// Telemetry-related runtime settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TelemetrySettings {
    pub enable_metrics: bool,
    pub prometheus_port: u16,
}

/// Startup and safety policy controls.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicySettings {
    pub require_key_material: bool,
    pub require_genesis: bool,
    pub allow_remote_peers: bool,
}

/// Canonical AOXC environment profile enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CanonicalProfile {
    Mainnet,
    Testnet,
    Validation,
    Devnet,
    Localnet,
}

impl CanonicalProfile {
    fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "mainnet" => Ok(Self::Mainnet),
            "testnet" => Ok(Self::Testnet),
            "validation" => Ok(Self::Validation),
            "validator" => Ok(Self::Validation),
            "devnet" => Ok(Self::Devnet),
            "localnet" => Ok(Self::Localnet),
            other => Err(format!(
                "profile must be one of mainnet, testnet, validation, devnet, or localnet; got `{}`",
                other
            )),
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Testnet => "testnet",
            Self::Validation => "validation",
            Self::Devnet => "devnet",
            Self::Localnet => "localnet",
        }
    }

    const fn default_bind_host(self) -> &'static str {
        match self {
            Self::Mainnet => "0.0.0.0",
            Self::Testnet => "0.0.0.0",
            Self::Validation => "127.0.0.1",
            Self::Devnet => "127.0.0.1",
            Self::Localnet => "127.0.0.1",
        }
    }

    const fn default_p2p_port(self) -> u16 {
        match self {
            Self::Mainnet => 26656,
            Self::Testnet => 27656,
            Self::Validation => 28656,
            Self::Devnet => 29656,
            Self::Localnet => 30656,
        }
    }

    const fn default_rpc_port(self) -> u16 {
        match self {
            Self::Mainnet => 26657,
            Self::Testnet => 27657,
            Self::Validation => 28657,
            Self::Devnet => 29657,
            Self::Localnet => 30657,
        }
    }

    const fn default_prometheus_port(self) -> u16 {
        match self {
            Self::Mainnet => 26660,
            Self::Testnet => 27660,
            Self::Validation => 28660,
            Self::Devnet => 29660,
            Self::Localnet => 30660,
        }
    }

    const fn default_enforce_official_peers(self) -> bool {
        match self {
            Self::Mainnet => true,
            Self::Testnet => true,
            Self::Validation => true,
            Self::Devnet => false,
            Self::Localnet => false,
        }
    }

    const fn default_allow_remote_peers(self) -> bool {
        match self {
            Self::Mainnet => false,
            Self::Testnet => false,
            Self::Validation => false,
            Self::Devnet => true,
            Self::Localnet => true,
        }
    }

    const fn default_require_key_material(self) -> bool {
        match self {
            Self::Mainnet => true,
            Self::Testnet => true,
            Self::Validation => true,
            Self::Devnet => true,
            Self::Localnet => true,
        }
    }

    const fn default_require_genesis(self) -> bool {
        true
    }

    const fn default_enable_metrics(self) -> bool {
        true
    }

    const fn default_logging_json(self) -> bool {
        matches!(self, Self::Mainnet)
    }

    const fn default_logging_level(self) -> &'static str {
        match self {
            Self::Mainnet => "info",
            Self::Testnet => "info",
            Self::Validation => "info",
            Self::Devnet => "debug",
            Self::Localnet => "debug",
        }
    }
}

impl Settings {
    /// Returns canonical default settings for the AOXC validation profile.
    ///
    /// Validation is used as the modern safe default because:
    /// - it is non-production,
    /// - it remains governance-aware,
    /// - it is stricter than a free-form devnet default,
    /// - it replaces the older ambiguous `validator` profile terminology.
    pub fn default_for(home_dir: String) -> Self {
        Self::default_for_profile(home_dir, "validation")
            .expect("default validation profile construction must succeed")
    }

    /// Returns canonical default settings for a requested profile.
    ///
    /// Accepted profile values:
    /// - `mainnet`
    /// - `testnet`
    /// - `validation`
    /// - `validator` (legacy alias => `validation`)
    /// - `devnet`
    /// - `localnet`
    pub fn default_for_profile(home_dir: String, profile: &str) -> Result<Self, String> {
        let profile = CanonicalProfile::parse(profile)?;

        Ok(Self {
            home_dir,
            profile: profile.as_str().to_string(),
            logging: LoggingSettings {
                level: profile.default_logging_level().to_string(),
                json: profile.default_logging_json(),
            },
            network: NetworkSettings {
                bind_host: profile.default_bind_host().to_string(),
                p2p_port: profile.default_p2p_port(),
                rpc_port: profile.default_rpc_port(),
                enforce_official_peers: profile.default_enforce_official_peers(),
            },
            telemetry: TelemetrySettings {
                enable_metrics: profile.default_enable_metrics(),
                prometheus_port: profile.default_prometheus_port(),
            },
            policy: PolicySettings {
                require_key_material: profile.default_require_key_material(),
                require_genesis: profile.default_require_genesis(),
                allow_remote_peers: profile.default_allow_remote_peers(),
            },
        })
    }

    /// Validates the current AOXC settings surface.
    pub fn validate(&self) -> Result<(), String> {
        if self.home_dir.trim().is_empty() {
            return Err("home_dir must not be empty".to_string());
        }

        let profile = CanonicalProfile::parse(&self.profile)?;

        if self.network.p2p_port == 0
            || self.network.rpc_port == 0
            || self.telemetry.prometheus_port == 0
        {
            return Err("ports must be non-zero".to_string());
        }

        if self.network.p2p_port == self.network.rpc_port
            || self.network.p2p_port == self.telemetry.prometheus_port
            || self.network.rpc_port == self.telemetry.prometheus_port
        {
            return Err("p2p, rpc, and prometheus ports must be distinct".to_string());
        }

        if self.network.bind_host.trim().is_empty() {
            return Err("bind_host must not be empty".to_string());
        }

        if self.policy.allow_remote_peers && self.network.enforce_official_peers {
            return Err(
                "allow_remote_peers cannot be enabled while enforce_official_peers is active"
                    .to_string(),
            );
        }

        self.validate_profile_guards(profile)?;
        Ok(())
    }

    /// Applies profile-specific hardening rules.
    fn validate_profile_guards(&self, profile: CanonicalProfile) -> Result<(), String> {
        match profile {
            CanonicalProfile::Mainnet => self.validate_mainnet_guards(),
            CanonicalProfile::Testnet => self.validate_testnet_guards(),
            CanonicalProfile::Validation => self.validate_validation_guards(),
            CanonicalProfile::Devnet => self.validate_devnet_guards(),
            CanonicalProfile::Localnet => self.validate_localnet_guards(),
        }
    }

    /// Validates mainnet-specific security expectations.
    fn validate_mainnet_guards(&self) -> Result<(), String> {
        if !Path::new(&self.home_dir).is_absolute() {
            return Err("mainnet profile requires an absolute home_dir".to_string());
        }

        if !self.network.enforce_official_peers {
            return Err(
                "mainnet profile requires enforce_official_peers to remain enabled".to_string(),
            );
        }

        if self.policy.allow_remote_peers {
            return Err(
                "mainnet profile cannot enable allow_remote_peers; peer admission must stay curated"
                    .to_string(),
            );
        }

        if !self.policy.require_key_material {
            return Err(
                "mainnet profile requires key material verification before startup".to_string(),
            );
        }

        if !self.policy.require_genesis {
            return Err("mainnet profile requires a committed genesis document".to_string());
        }

        if !self.telemetry.enable_metrics {
            return Err("mainnet profile requires telemetry metrics to stay enabled".to_string());
        }

        if !self.logging.json {
            return Err(
                "mainnet profile requires structured JSON logging for auditability".to_string(),
            );
        }

        if matches!(
            self.logging.level.trim().to_ascii_lowercase().as_str(),
            "debug" | "trace"
        ) {
            return Err(
                "mainnet profile cannot use debug or trace logging because it expands attack surface and log volume"
                    .to_string(),
            );
        }

        if matches!(
            self.network.bind_host.trim(),
            "127.0.0.1" | "::1" | "localhost"
        ) {
            return Err(
                "mainnet profile requires a non-loopback bind_host so the node is reachable by the production network"
                    .to_string(),
            );
        }

        Ok(())
    }

    /// Validates public testnet expectations.
    fn validate_testnet_guards(&self) -> Result<(), String> {
        if !self.policy.require_key_material {
            return Err(
                "testnet profile requires key material verification before startup".to_string(),
            );
        }

        if !self.policy.require_genesis {
            return Err("testnet profile requires a committed genesis document".to_string());
        }

        if !self.network.enforce_official_peers {
            return Err(
                "testnet profile requires enforce_official_peers to remain enabled".to_string(),
            );
        }

        if self.policy.allow_remote_peers {
            return Err(
                "testnet profile cannot enable allow_remote_peers while curated peer admission is active"
                    .to_string(),
            );
        }

        Ok(())
    }

    /// Validates validation-environment expectations.
    fn validate_validation_guards(&self) -> Result<(), String> {
        if !self.policy.require_key_material {
            return Err(
                "validation profile requires key material verification before startup".to_string(),
            );
        }

        if !self.policy.require_genesis {
            return Err("validation profile requires a committed genesis document".to_string());
        }

        if self.policy.allow_remote_peers && self.network.bind_host.trim() == "127.0.0.1" {
            return Err(
                "validation profile cannot advertise remote peer admission while bound to loopback"
                    .to_string(),
            );
        }

        Ok(())
    }

    /// Validates development-network expectations.
    fn validate_devnet_guards(&self) -> Result<(), String> {
        if !self.policy.require_key_material {
            return Err("devnet profile still requires key material verification".to_string());
        }

        if !self.policy.require_genesis {
            return Err("devnet profile requires a committed genesis document".to_string());
        }

        Ok(())
    }

    /// Validates local deterministic operator-network expectations.
    fn validate_localnet_guards(&self) -> Result<(), String> {
        if !matches!(
            self.network.bind_host.trim(),
            "127.0.0.1" | "::1" | "localhost"
        ) {
            return Err(
                "localnet profile must remain loopback-bound to preserve local-only operation"
                    .to_string(),
            );
        }

        if !self.policy.require_key_material {
            return Err("localnet profile requires key material verification".to_string());
        }

        if !self.policy.require_genesis {
            return Err("localnet profile requires a committed genesis document".to_string());
        }

        Ok(())
    }

    /// Returns a redacted copy of settings.
    ///
    /// The current settings surface does not contain embedded secrets, so
    /// redaction is structurally identical to cloning. The helper is retained
    /// to preserve a stable UI / CLI contract for future secret-bearing fields.
    pub fn redacted(&self) -> Self {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::{CanonicalProfile, Settings};

    #[test]
    fn default_for_uses_validation_profile() {
        let settings = Settings::default_for("/tmp/aoxc".to_string());
        assert_eq!(settings.profile, "validation");
    }

    #[test]
    fn default_for_profile_accepts_legacy_validator_alias() {
        let settings = Settings::default_for_profile("/tmp/aoxc".to_string(), "validator").unwrap();
        assert_eq!(settings.profile, "validation");
    }

    #[test]
    fn validate_rejects_unknown_profile() {
        let mut settings = Settings::default_for("/tmp/aoxc".to_string());
        settings.profile = "staging".to_string();

        assert!(settings.validate().is_err());
    }

    #[test]
    fn validate_rejects_port_collisions() {
        let mut settings = Settings::default_for("/tmp/aoxc".to_string());
        settings.network.rpc_port = settings.network.p2p_port;

        assert!(settings.validate().is_err());
    }

    #[test]
    fn validate_rejects_conflicting_peer_policy() {
        let mut settings = Settings::default_for("/tmp/aoxc".to_string());
        settings.policy.allow_remote_peers = true;
        settings.network.enforce_official_peers = true;

        assert!(settings.validate().is_err());
    }

    #[test]
    fn validate_rejects_mainnet_without_structured_logging() {
        let mut settings =
            Settings::default_for_profile("/tmp/aoxc".to_string(), "mainnet").unwrap();
        settings.logging.json = false;
        settings.network.bind_host = "0.0.0.0".to_string();

        assert!(settings.validate().is_err());
    }

    #[test]
    fn validate_accepts_hardened_mainnet_profile() {
        let settings = Settings::default_for_profile("/tmp/aoxc".to_string(), "mainnet").unwrap();

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn validate_rejects_mainnet_with_relative_home_dir() {
        let mut settings =
            Settings::default_for_profile("relative/aoxc".to_string(), "mainnet").unwrap();
        settings.logging.json = true;
        settings.network.bind_host = "0.0.0.0".to_string();

        assert!(settings.validate().is_err());
    }

    #[test]
    fn validate_rejects_mainnet_with_debug_logging() {
        let mut settings =
            Settings::default_for_profile("/tmp/aoxc".to_string(), "mainnet").unwrap();
        settings.logging.level = "debug".to_string();

        assert!(settings.validate().is_err());
    }

    #[test]
    fn validate_rejects_mainnet_with_loopback_bind_host() {
        let mut settings =
            Settings::default_for_profile("/tmp/aoxc".to_string(), "mainnet").unwrap();
        settings.network.bind_host = "127.0.0.1".to_string();

        assert!(settings.validate().is_err());
    }

    #[test]
    fn validate_accepts_testnet_defaults() {
        let settings = Settings::default_for_profile("/tmp/aoxc".to_string(), "testnet").unwrap();

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn validate_accepts_validation_defaults() {
        let settings =
            Settings::default_for_profile("/tmp/aoxc".to_string(), "validation").unwrap();

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn validate_accepts_devnet_defaults() {
        let settings = Settings::default_for_profile("/tmp/aoxc".to_string(), "devnet").unwrap();

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn validate_accepts_localnet_defaults() {
        let settings = Settings::default_for_profile("/tmp/aoxc".to_string(), "localnet").unwrap();

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn validate_rejects_localnet_with_non_loopback_bind_host() {
        let mut settings =
            Settings::default_for_profile("/tmp/aoxc".to_string(), "localnet").unwrap();
        settings.network.bind_host = "0.0.0.0".to_string();

        assert!(settings.validate().is_err());
    }

    #[test]
    fn canonical_profile_parser_normalizes_validator_alias() {
        let parsed = CanonicalProfile::parse("validator").unwrap();
        assert_eq!(parsed, CanonicalProfile::Validation);
    }
}
