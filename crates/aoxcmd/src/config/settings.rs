use serde::{Deserialize, Serialize};

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
        if self.network.p2p_port == 0 || self.network.rpc_port == 0 || self.telemetry.prometheus_port == 0 {
            return Err("ports must be non-zero".to_string());
        }
        if self.network.bind_host.trim().is_empty() {
            return Err("bind_host must not be empty".to_string());
        }
        Ok(())
    }

    pub fn redacted(&self) -> Self {
        self.clone()
    }
}
