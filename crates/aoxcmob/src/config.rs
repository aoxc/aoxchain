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
        if self.relay_origin.trim().is_empty() {
            return Err(MobError::InvalidConfiguration(
                "relay_origin must not be empty",
            ));
        }
        if self.app_id.trim().is_empty() {
            return Err(MobError::InvalidConfiguration("app_id must not be empty"));
        }
        if self.chain_id.trim().is_empty() {
            return Err(MobError::InvalidConfiguration("chain_id must not be empty"));
        }
        if self.request_timeout_ms == 0 {
            return Err(MobError::InvalidConfiguration(
                "request_timeout_ms must be greater than zero",
            ));
        }
        if self.session_ttl_secs == 0 {
            return Err(MobError::InvalidConfiguration(
                "session_ttl_secs must be greater than zero",
            ));
        }
        if self.challenge_max_skew_secs == 0 {
            return Err(MobError::InvalidConfiguration(
                "challenge_max_skew_secs must be greater than zero",
            ));
        }
        if self.task_ack_timeout_secs == 0 {
            return Err(MobError::InvalidConfiguration(
                "task_ack_timeout_secs must be greater than zero",
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
