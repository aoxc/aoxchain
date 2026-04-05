// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};
use std::path::Path;

const PROFILE_MAINNET: &str = "mainnet";
const PROFILE_TESTNET: &str = "testnet";
const PROFILE_VALIDATION: &str = "validation";
const PROFILE_VALIDATOR_ALIAS: &str = "validator";
const PROFILE_DEVNET: &str = "devnet";
const PROFILE_LOCALNET: &str = "localnet";

const LOG_LEVEL_INFO: &str = "info";
const LOG_LEVEL_DEBUG: &str = "debug";
const LOG_LEVEL_TRACE: &str = "trace";
const LOG_LEVEL_WARN: &str = "warn";
const LOG_LEVEL_ERROR: &str = "error";

const LOOPBACK_IPV4: &str = "127.0.0.1";
const LOOPBACK_IPV6: &str = "::1";
const LOOPBACK_HOSTNAME: &str = "localhost";

/// Canonical AOXC CLI settings document.
///
/// This structure reflects the AOXC single-binary, multi-network operating
/// model. Behavior is derived from canonical environment profiles rather than
/// ad hoc runtime toggles. The intent is to keep the operator surface explicit,
/// deterministic, and auditable.
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
///
/// This enum is intentionally internal so the serialized configuration surface
/// remains string-based and operator-friendly, while validation and defaulting
/// logic can still rely on a strongly typed profile model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CanonicalProfile {
    Mainnet,
    Testnet,
    Validation,
    Devnet,
    Localnet,
}

impl CanonicalProfile {
    /// Parses a canonical AOXC profile string.
    ///
    /// Accepted values:
    /// - `mainnet`
    /// - `testnet`
    /// - `validation`
    /// - `validator` (legacy alias => `validation`)
    /// - `devnet`
    /// - `localnet`
    pub(super) fn parse(value: &str) -> Result<Self, String> {
        match normalize_profile_text(value).as_str() {
            PROFILE_MAINNET => Ok(Self::Mainnet),
            PROFILE_TESTNET => Ok(Self::Testnet),
            PROFILE_VALIDATION => Ok(Self::Validation),
            PROFILE_VALIDATOR_ALIAS => Ok(Self::Validation),
            PROFILE_DEVNET => Ok(Self::Devnet),
            PROFILE_LOCALNET => Ok(Self::Localnet),
            other => Err(format!(
                "profile must be one of mainnet, testnet, validation, devnet, or localnet; got `{}`",
                other
            )),
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Mainnet => PROFILE_MAINNET,
            Self::Testnet => PROFILE_TESTNET,
            Self::Validation => PROFILE_VALIDATION,
            Self::Devnet => PROFILE_DEVNET,
            Self::Localnet => PROFILE_LOCALNET,
        }
    }

    const fn default_bind_host(self) -> &'static str {
        match self {
            Self::Mainnet | Self::Testnet => "0.0.0.0",
            Self::Validation | Self::Devnet | Self::Localnet => LOOPBACK_IPV4,
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
            Self::Mainnet | Self::Testnet | Self::Validation => true,
            Self::Devnet | Self::Localnet => false,
        }
    }

    const fn default_allow_remote_peers(self) -> bool {
        match self {
            Self::Mainnet | Self::Testnet | Self::Validation => false,
            Self::Devnet | Self::Localnet => true,
        }
    }

    const fn default_require_key_material(self) -> bool {
        true
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
            Self::Mainnet | Self::Testnet | Self::Validation => LOG_LEVEL_INFO,
            Self::Devnet | Self::Localnet => LOG_LEVEL_DEBUG,
        }
    }
}

impl Settings {
    /// Returns canonical default settings for the AOXC validation profile.
    ///
    /// Validation is the safe default because it is non-production while still
    /// preserving stronger operator constraints than a relaxed devnet posture.
    pub fn default_for(home_dir: String) -> Self {
        Self::default_for_profile(home_dir, PROFILE_VALIDATION)
            .expect("default validation profile construction must succeed")
    }

    /// Returns canonical default settings for a requested profile.
    ///
    /// The returned document is already normalized to the canonical profile
    /// spelling, which means the legacy alias `validator` becomes `validation`.
    pub fn default_for_profile(home_dir: String, profile: &str) -> Result<Self, String> {
        let canonical_profile = CanonicalProfile::parse(profile)?;
        let normalized_home_dir = normalize_required_text(&home_dir, "home_dir")?;

        Ok(Self {
            home_dir: normalized_home_dir,
            profile: canonical_profile.as_str().to_string(),
            logging: LoggingSettings {
                level: canonical_profile.default_logging_level().to_string(),
                json: canonical_profile.default_logging_json(),
            },
            network: NetworkSettings {
                bind_host: canonical_profile.default_bind_host().to_string(),
                p2p_port: canonical_profile.default_p2p_port(),
                rpc_port: canonical_profile.default_rpc_port(),
                enforce_official_peers: canonical_profile.default_enforce_official_peers(),
            },
            telemetry: TelemetrySettings {
                enable_metrics: canonical_profile.default_enable_metrics(),
                prometheus_port: canonical_profile.default_prometheus_port(),
            },
            policy: PolicySettings {
                require_key_material: canonical_profile.default_require_key_material(),
                require_genesis: canonical_profile.default_require_genesis(),
                allow_remote_peers: canonical_profile.default_allow_remote_peers(),
            },
        })
    }

    /// Validates the current AOXC settings surface.
    ///
    /// Validation policy:
    /// - The document must parse into a canonical AOXC profile.
    /// - Core string fields must not be blank after normalization.
    /// - Core ports must be non-zero and distinct.
    /// - Global policy contradictions are rejected before profile-specific
    ///   guard evaluation.
    /// - Profile-specific hardening rules are enforced afterwards.
    pub fn validate(&self) -> Result<(), String> {
        normalize_required_text(&self.home_dir, "home_dir")?;
        let profile = CanonicalProfile::parse(&self.profile)?;
        normalize_required_text(&self.network.bind_host, "bind_host")?;

        let logging_level = normalize_logging_level(&self.logging.level)?;

        validate_non_zero_ports(
            self.network.p2p_port,
            self.network.rpc_port,
            self.telemetry.prometheus_port,
        )?;
        validate_distinct_ports(
            self.network.p2p_port,
            self.network.rpc_port,
            self.telemetry.prometheus_port,
        )?;

        if self.policy.allow_remote_peers && self.network.enforce_official_peers {
            return Err(
                "allow_remote_peers cannot be enabled while enforce_official_peers is active"
                    .to_string(),
            );
        }

        self.validate_profile_guards(profile, logging_level)?;
        Ok(())
    }

    /// Applies profile-specific hardening rules.
    fn validate_profile_guards(
        &self,
        profile: CanonicalProfile,
        normalized_logging_level: &str,
    ) -> Result<(), String> {
        match profile {
            CanonicalProfile::Mainnet => self.validate_mainnet_guards(normalized_logging_level),
            CanonicalProfile::Testnet => self.validate_testnet_guards(normalized_logging_level),
            CanonicalProfile::Validation => {
                self.validate_validation_guards(normalized_logging_level)
            }
            CanonicalProfile::Devnet => self.validate_devnet_guards(normalized_logging_level),
            CanonicalProfile::Localnet => self.validate_localnet_guards(normalized_logging_level),
        }
    }

    /// Validates mainnet-specific security expectations.
    fn validate_mainnet_guards(&self, logging_level: &str) -> Result<(), String> {
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

        if matches!(logging_level, LOG_LEVEL_DEBUG | LOG_LEVEL_TRACE) {
            return Err(
                "mainnet profile cannot use debug or trace logging because it expands attack surface and log volume"
                    .to_string(),
            );
        }

        if is_loopback_host(&self.network.bind_host) {
            return Err(
                "mainnet profile requires a non-loopback bind_host so the node is reachable by the production network"
                    .to_string(),
            );
        }

        Ok(())
    }

    /// Validates public testnet expectations.
    fn validate_testnet_guards(&self, logging_level: &str) -> Result<(), String> {
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

        if matches!(logging_level, LOG_LEVEL_TRACE) {
            return Err(
                "testnet profile cannot use trace logging because it is excessively verbose for public-network operation"
                    .to_string(),
            );
        }

        Ok(())
    }

    /// Validates validation-environment expectations.
    fn validate_validation_guards(&self, _logging_level: &str) -> Result<(), String> {
        if !self.policy.require_key_material {
            return Err(
                "validation profile requires key material verification before startup".to_string(),
            );
        }

        if !self.policy.require_genesis {
            return Err("validation profile requires a committed genesis document".to_string());
        }

        if self.policy.allow_remote_peers && is_loopback_host(&self.network.bind_host) {
            return Err(
                "validation profile cannot advertise remote peer admission while bound to loopback"
                    .to_string(),
            );
        }

        Ok(())
    }

    /// Validates development-network expectations.
    fn validate_devnet_guards(&self, _logging_level: &str) -> Result<(), String> {
        if !self.policy.require_key_material {
            return Err("devnet profile still requires key material verification".to_string());
        }

        if !self.policy.require_genesis {
            return Err("devnet profile requires a committed genesis document".to_string());
        }

        Ok(())
    }

    /// Validates local deterministic operator-network expectations.
    fn validate_localnet_guards(&self, _logging_level: &str) -> Result<(), String> {
        if !is_loopback_host(&self.network.bind_host) {
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
    /// redaction is structurally identical to cloning. The helper is preserved
    /// to maintain a stable UI and CLI contract for future secret-bearing
    /// fields.
    pub fn redacted(&self) -> Self {
        self.clone()
    }
}

fn normalize_profile_text(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn normalize_required_text(value: &str, field: &str) -> Result<String, String> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    Ok(normalized)
}

fn normalize_logging_level(value: &str) -> Result<&str, String> {
    match normalize_profile_text(value).as_str() {
        LOG_LEVEL_INFO => Ok(LOG_LEVEL_INFO),
        LOG_LEVEL_DEBUG => Ok(LOG_LEVEL_DEBUG),
        LOG_LEVEL_TRACE => Ok(LOG_LEVEL_TRACE),
        LOG_LEVEL_WARN => Ok(LOG_LEVEL_WARN),
        LOG_LEVEL_ERROR => Ok(LOG_LEVEL_ERROR),
        other => Err(format!(
            "logging.level must be one of info, debug, trace, warn, or error; got `{}`",
            other
        )),
    }
}

fn validate_non_zero_ports(
    p2p_port: u16,
    rpc_port: u16,
    prometheus_port: u16,
) -> Result<(), String> {
    if p2p_port == 0 || rpc_port == 0 || prometheus_port == 0 {
        return Err("ports must be non-zero".to_string());
    }
    Ok(())
}

fn validate_distinct_ports(
    p2p_port: u16,
    rpc_port: u16,
    prometheus_port: u16,
) -> Result<(), String> {
    if p2p_port == rpc_port || p2p_port == prometheus_port || rpc_port == prometheus_port {
        return Err("p2p, rpc, and prometheus ports must be distinct".to_string());
    }
    Ok(())
}

fn is_loopback_host(value: &str) -> bool {
    matches!(
        value.trim(),
        LOOPBACK_IPV4 | LOOPBACK_IPV6 | LOOPBACK_HOSTNAME
    )
}
