use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub home_dir: String,
    pub profile: String,
    pub logging: LoggingSettings,
    pub network: NetworkSettings,
    pub telemetry: TelemetrySettings,
    pub policy: PolicySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSettings {
    pub level: String,
    pub json: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSettings {
    pub bind_host: String,
    pub p2p_port: u16,
    pub rpc_port: u16,
    pub enforce_official_peers: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySettings {
    pub enable_metrics: bool,
    pub prometheus_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySettings {
    pub require_key_material: bool,
    pub require_genesis: bool,
    pub allow_remote_peers: bool,
}

impl Settings {
    pub fn default_for(home_dir: String) -> Self {
        Self {
            home_dir,
            profile: "validator".to_string(),
            logging: LoggingSettings {
                level: "info".to_string(),
                json: false,
            },
            network: NetworkSettings {
                bind_host: "127.0.0.1".to_string(),
                p2p_port: 39001,
                rpc_port: 2626,
                enforce_official_peers: true,
            },
            telemetry: TelemetrySettings {
                enable_metrics: true,
                prometheus_port: 9100,
            },
            policy: PolicySettings {
                require_key_material: true,
                require_genesis: true,
                allow_remote_peers: false,
            },
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.home_dir.trim().is_empty() {
            return Err("home_dir must not be empty".to_string());
        }
        if !matches!(
            self.profile.trim().to_ascii_lowercase().as_str(),
            "validator" | "testnet" | "mainnet"
        ) {
            return Err("profile must be one of validator, testnet, or mainnet".to_string());
        }
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
        self.validate_mainnet_guards()?;
        Ok(())
    }

    fn validate_mainnet_guards(&self) -> Result<(), String> {
        if !self.profile.eq_ignore_ascii_case("mainnet") {
            return Ok(());
        }

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

    pub fn redacted(&self) -> Self {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::Settings;

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

        assert!(settings.validate().is_err());
    }

    #[test]
    fn validate_rejects_mainnet_without_structured_logging() {
        let mut settings = Settings::default_for("/tmp/aoxc".to_string());
        settings.profile = "mainnet".to_string();

        assert!(settings.validate().is_err());
    }

    #[test]
    fn validate_accepts_hardened_mainnet_profile() {
        let mut settings = Settings::default_for("/tmp/aoxc".to_string());
        settings.profile = "mainnet".to_string();
        settings.logging.json = true;
        settings.network.bind_host = "0.0.0.0".to_string();

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn validate_rejects_mainnet_with_relative_home_dir() {
        let mut settings = Settings::default_for("relative/aoxc".to_string());
        settings.profile = "mainnet".to_string();
        settings.logging.json = true;
        settings.network.bind_host = "0.0.0.0".to_string();

        assert!(settings.validate().is_err());
    }

    #[test]
    fn validate_rejects_mainnet_with_debug_logging() {
        let mut settings = Settings::default_for("/tmp/aoxc".to_string());
        settings.profile = "mainnet".to_string();
        settings.logging.json = true;
        settings.logging.level = "debug".to_string();
        settings.network.bind_host = "0.0.0.0".to_string();

        assert!(settings.validate().is_err());
    }

    #[test]
    fn validate_rejects_mainnet_with_loopback_bind_host() {
        let mut settings = Settings::default_for("/tmp/aoxc".to_string());
        settings.profile = "mainnet".to_string();
        settings.logging.json = true;

        assert!(settings.validate().is_err());
    }
}
