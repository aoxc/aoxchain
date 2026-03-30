// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::path::Path;
use std::str::FromStr;

/// RPC subsystem configuration.
#[derive(Debug, Clone)]
pub struct RpcConfig {
    pub http_bind_addr: String,
    pub websocket_bind_addr: String,
    pub grpc_bind_addr: String,
    pub tls_cert_path: String,
    pub tls_key_path: String,
    pub mtls_ca_cert_path: Option<String>,
    pub chain_id: String,
    pub genesis_hash: Option<String>,
    pub max_requests_per_minute: u64,
    pub rate_limiter_window_secs: u64,
    pub rate_limiter_max_tracked_keys: usize,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            http_bind_addr: "127.0.0.1:8080".to_string(),
            websocket_bind_addr: "127.0.0.1:8081".to_string(),
            grpc_bind_addr: "127.0.0.1:50051".to_string(),
            tls_cert_path: "./tls/server.crt".to_string(),
            tls_key_path: "./tls/server.key".to_string(),
            mtls_ca_cert_path: Some("./tls/ca.crt".to_string()),
            chain_id: "AOX-MAIN".to_string(),
            genesis_hash: None,
            max_requests_per_minute: 600,
            rate_limiter_window_secs: 60,
            rate_limiter_max_tracked_keys: 100_000,
        }
    }
}

impl RpcConfig {
    #[must_use]
    pub fn validate(&self) -> ConfigValidation {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        if self.chain_id.trim().is_empty() {
            errors.push("chain_id must not be empty".to_string());
        }

        validate_bind_addr("http_bind_addr", &self.http_bind_addr, &mut errors);
        validate_bind_addr(
            "websocket_bind_addr",
            &self.websocket_bind_addr,
            &mut errors,
        );
        validate_bind_addr("grpc_bind_addr", &self.grpc_bind_addr, &mut errors);

        if self.max_requests_per_minute == 0 {
            errors.push("max_requests_per_minute must be greater than zero".to_string());
        }

        if self.rate_limiter_window_secs == 0 {
            errors.push("rate_limiter_window_secs must be greater than zero".to_string());
        }

        if self.rate_limiter_max_tracked_keys == 0 {
            errors.push("rate_limiter_max_tracked_keys must be greater than zero".to_string());
        }

        if self.genesis_hash.is_none() {
            warnings.push("genesis_hash is not configured".to_string());
        } else if !self.has_valid_genesis_hash() {
            errors.push("genesis_hash is malformed (expected 0x-prefixed 64-byte hex)".to_string());
        }

        if !Path::new(&self.tls_cert_path).exists() {
            warnings.push("tls certificate file is missing".to_string());
        }

        if !Path::new(&self.tls_key_path).exists() {
            warnings.push("tls private key file is missing".to_string());
        }

        if let Some(ca_path) = &self.mtls_ca_cert_path {
            if !Path::new(ca_path).exists() {
                warnings.push("mTLS CA certificate file is missing".to_string());
            }
        } else {
            warnings.push("mTLS is disabled".to_string());
        }

        if self.max_requests_per_minute > 100_000 {
            warnings.push(
                "max_requests_per_minute is very high; verify DDoS protection and upstream shielding"
                    .to_string(),
            );
        }

        if self.tls_cert_path == self.tls_key_path {
            warnings.push(
                "tls_cert_path and tls_key_path point to the same file; verify key material separation"
                    .to_string(),
            );
        }

        ConfigValidation { warnings, errors }
    }

    fn has_valid_genesis_hash(&self) -> bool {
        let Some(hash) = &self.genesis_hash else {
            return false;
        };

        hash.starts_with("0x")
            && hash.len() == 66
            && hash[2..].chars().all(|c| c.is_ascii_hexdigit())
    }
}

fn validate_bind_addr(field: &str, value: &str, errors: &mut Vec<String>) {
    if value.trim().is_empty() {
        errors.push(format!("{field} must not be empty"));
        return;
    }

    if std::net::SocketAddr::from_str(value).is_err() {
        errors.push(format!("{field} is malformed (expected ip:port)"));
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConfigValidation {
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl ConfigValidation {
    #[must_use]
    pub fn readiness_score(&self) -> u8 {
        if !self.errors.is_empty() {
            return 0;
        }

        let warning_penalty = (self.warnings.len() as u8).saturating_mul(15);
        100_u8.saturating_sub(warning_penalty)
    }

    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_bad_genesis_hash() {
        let config = RpcConfig {
            genesis_hash: Some("1234".to_string()),
            ..RpcConfig::default()
        };

        let validation = config.validate();
        assert!(
            validation
                .errors
                .iter()
                .any(|error| error.contains("genesis_hash is malformed"))
        );
        assert_eq!(validation.readiness_score(), 0);
    }

    #[test]
    fn validate_accepts_well_formed_genesis_hash() {
        let config = RpcConfig {
            genesis_hash: Some(format!("0x{}", "ab".repeat(32))),
            ..RpcConfig::default()
        };

        let validation = config.validate();
        assert!(validation.errors.is_empty());
    }

    #[test]
    fn validate_flags_empty_chain_id_and_zero_limits() {
        let config = RpcConfig {
            chain_id: "   ".to_string(),
            max_requests_per_minute: 0,
            rate_limiter_window_secs: 0,
            rate_limiter_max_tracked_keys: 0,
            ..RpcConfig::default()
        };

        let validation = config.validate();

        assert!(
            validation
                .errors
                .iter()
                .any(|error| error.contains("chain_id must not be empty"))
        );
        assert!(
            validation
                .errors
                .iter()
                .any(|error| error.contains("max_requests_per_minute"))
        );
        assert!(
            validation
                .errors
                .iter()
                .any(|error| error.contains("rate_limiter_window_secs"))
        );
        assert!(
            validation
                .errors
                .iter()
                .any(|error| error.contains("rate_limiter_max_tracked_keys"))
        );
        assert_eq!(validation.readiness_score(), 0);
    }

    #[test]
    fn readiness_score_penalizes_warnings_without_errors() {
        let validation = ConfigValidation {
            warnings: vec!["w1".to_string(), "w2".to_string()],
            errors: vec![],
        };

        assert_eq!(validation.readiness_score(), 70);
        assert!(!validation.is_ready());
    }

    #[test]
    fn validate_can_be_fully_ready_with_existing_artifacts() {
        let config = RpcConfig {
            genesis_hash: Some(format!("0x{}", "ab".repeat(32))),
            tls_cert_path: "Cargo.toml".to_string(),
            tls_key_path: "README.md".to_string(),
            mtls_ca_cert_path: Some("Cargo.toml".to_string()),
            ..RpcConfig::default()
        };

        let validation = config.validate();

        assert!(validation.errors.is_empty());
        assert!(validation.warnings.is_empty());
        assert!(validation.is_ready());
        assert_eq!(validation.readiness_score(), 100);
    }

    #[test]
    fn validate_rejects_malformed_bind_addresses() {
        let config = RpcConfig {
            http_bind_addr: "localhost".to_string(),
            websocket_bind_addr: "bad-value".to_string(),
            grpc_bind_addr: "".to_string(),
            ..RpcConfig::default()
        };

        let validation = config.validate();
        assert!(
            validation
                .errors
                .iter()
                .any(|item| item.contains("http_bind_addr is malformed"))
        );
        assert!(
            validation
                .errors
                .iter()
                .any(|item| item.contains("websocket_bind_addr is malformed"))
        );
        assert!(
            validation
                .errors
                .iter()
                .any(|item| item.contains("grpc_bind_addr must not be empty"))
        );
    }

    #[test]
    fn validate_warns_on_excessive_rate_and_shared_tls_file() {
        let config = RpcConfig {
            genesis_hash: Some(format!("0x{}", "ab".repeat(32))),
            tls_cert_path: "Cargo.toml".to_string(),
            tls_key_path: "Cargo.toml".to_string(),
            mtls_ca_cert_path: Some("Cargo.toml".to_string()),
            max_requests_per_minute: 200_000,
            ..RpcConfig::default()
        };

        let validation = config.validate();
        assert!(validation.errors.is_empty());
        assert!(
            validation
                .warnings
                .iter()
                .any(|item| item.contains("very high"))
        );
        assert!(
            validation
                .warnings
                .iter()
                .any(|item| item.contains("same file"))
        );
    }
}

/// Contract RPC API configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractApiExposure {
    pub enable_contracts_api: bool,
    pub enable_register_endpoint: bool,
    pub enable_lifecycle_mutations: bool,
    pub enable_runtime_resolve_endpoint: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractApiLimits {
    pub max_manifest_body_size: usize,
    pub max_list_page_size: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractApiSecurity {
    pub strict_validation_mode: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractApiConfig {
    pub exposure: ContractApiExposure,
    pub limits: ContractApiLimits,
    pub security: ContractApiSecurity,
}

impl Default for ContractApiConfig {
    fn default() -> Self {
        Self {
            exposure: ContractApiExposure {
                enable_contracts_api: true,
                enable_register_endpoint: true,
                enable_lifecycle_mutations: true,
                enable_runtime_resolve_endpoint: true,
            },
            limits: ContractApiLimits {
                max_manifest_body_size: 1024 * 1024,
                max_list_page_size: 100,
            },
            security: ContractApiSecurity {
                strict_validation_mode: true,
            },
        }
    }
}
