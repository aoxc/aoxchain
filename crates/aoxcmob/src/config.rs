use crate::error::MobError;
use serde::{Deserialize, Serialize};

/// Runtime policy for the mobile secure-connection gateway.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MobileConfig {
    pub relay_origin: String,
    pub app_id: String,
    pub chain_id: String,
    pub request_timeout_ms: u64,
    pub session_ttl_secs: u64,
    pub challenge_max_skew_secs: u64,
    pub task_ack_timeout_secs: u64,
}

impl MobileConfig {
    /// Validates critical runtime policy before use.
    pub fn validate(&self) -> Result<(), MobError> {
        let relay_origin = self.relay_origin.trim();
        if relay_origin.is_empty() {
            return Err(MobError::InvalidConfiguration(
                "relay_origin must not be empty",
            ));
        }
        if !relay_origin.starts_with("https://") {
            return Err(MobError::InvalidConfiguration(
                "relay_origin must use https://",
            ));
        }
        if relay_origin.chars().any(char::is_whitespace) {
            return Err(MobError::InvalidConfiguration(
                "relay_origin must not contain whitespace",
            ));
        }
        if relay_origin.len() > 256 {
            return Err(MobError::InvalidConfiguration("relay_origin is too long"));
        }

        let app_id = self.app_id.trim();
        if app_id.is_empty() {
            return Err(MobError::InvalidConfiguration("app_id must not be empty"));
        }
        if app_id.len() > 64
            || !app_id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
        {
            return Err(MobError::InvalidConfiguration(
                "app_id contains invalid characters",
            ));
        }

        let chain_id = self.chain_id.trim();
        if chain_id.is_empty() {
            return Err(MobError::InvalidConfiguration("chain_id must not be empty"));
        }
        if chain_id.len() > 64
            || !chain_id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
        {
            return Err(MobError::InvalidConfiguration(
                "chain_id contains invalid characters",
            ));
        }
        if self.request_timeout_ms == 0 {
            return Err(MobError::InvalidConfiguration(
                "request_timeout_ms must be greater than zero",
            ));
        }
        if self.request_timeout_ms > 120_000 {
            return Err(MobError::InvalidConfiguration(
                "request_timeout_ms must be less than or equal to 120000",
            ));
        }
        if self.session_ttl_secs == 0 {
            return Err(MobError::InvalidConfiguration(
                "session_ttl_secs must be greater than zero",
            ));
        }
        if self.session_ttl_secs > 86_400 {
            return Err(MobError::InvalidConfiguration(
                "session_ttl_secs must be less than or equal to 86400",
            ));
        }
        if self.challenge_max_skew_secs == 0 {
            return Err(MobError::InvalidConfiguration(
                "challenge_max_skew_secs must be greater than zero",
            ));
        }
        if self.challenge_max_skew_secs > 3_600 {
            return Err(MobError::InvalidConfiguration(
                "challenge_max_skew_secs must be less than or equal to 3600",
            ));
        }
        if self.task_ack_timeout_secs == 0 {
            return Err(MobError::InvalidConfiguration(
                "task_ack_timeout_secs must be greater than zero",
            ));
        }
        if self.task_ack_timeout_secs > 3_600 {
            return Err(MobError::InvalidConfiguration(
                "task_ack_timeout_secs must be less than or equal to 3600",
            ));
        }
        if self.task_ack_timeout_secs > self.session_ttl_secs {
            return Err(MobError::InvalidConfiguration(
                "task_ack_timeout_secs must not exceed session_ttl_secs",
            ));
        }
        Ok(())
    }
}

impl Default for MobileConfig {
    fn default() -> Self {
        Self {
            relay_origin: "https://relay.aoxc.local".to_string(),
            app_id: "AOXC-MOBILE".to_string(),
            chain_id: "AOXC-MAIN".to_string(),
            request_timeout_ms: 5_000,
            session_ttl_secs: 300,
            challenge_max_skew_secs: 30,
            task_ack_timeout_secs: 60,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MobileConfig;

    #[test]
    fn default_config_is_valid() {
        MobileConfig::default().validate().expect("default config");
    }

    #[test]
    fn relay_origin_requires_https() {
        let config = MobileConfig {
            relay_origin: "http://relay.aoxc.local".to_string(),
            ..MobileConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn app_id_rejects_invalid_symbols() {
        let config = MobileConfig {
            app_id: "AOXC MOBILE".to_string(),
            ..MobileConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn ack_timeout_must_not_exceed_session_ttl() {
        let defaults = MobileConfig::default();
        let config = MobileConfig {
            task_ack_timeout_secs: defaults.session_ttl_secs + 1,
            ..defaults
        };
        assert!(config.validate().is_err());
    }
}
