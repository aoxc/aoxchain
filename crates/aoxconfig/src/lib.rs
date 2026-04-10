// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.
//!
//! Typed runtime configuration model for AOXC nodes and services.
//!
//! # Examples
//! ```rust
//! use aoxconfig::AoxConfig;
//!
//! let cfg = AoxConfig::default();
//! assert!(cfg.validate().is_ok());
//! ```

pub mod blockchain;
pub mod contracts;
pub mod mainnet;
pub mod quantum;

pub use blockchain::ChainConfig;
pub use contracts::ContractsConfig;
pub use mainnet::MainnetProgram;
pub use quantum::QuantumSecurityConfig;

use serde::{Deserialize, Serialize};

/// Top-level AOXC runtime configuration bundle.
///
/// This structure groups chain-level and contract-runtime controls under
/// one typed surface so callers can load, validate and pass config as a
/// single object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AoxConfig {
    pub chain: ChainConfig,
    pub contracts: ContractsConfig,
    pub mainnet: MainnetProgram,
    pub quantum: QuantumSecurityConfig,
}

impl AoxConfig {
    /// Validate nested configuration sections and return all violations.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if let Err(err) = self.chain.validate() {
            errors.push(format!("chain: {err}"));
        }

        errors.extend(
            self.contracts
                .validate()
                .into_iter()
                .map(|e| format!("contracts: {e}")),
        );

        errors.extend(
            self.mainnet
                .validate()
                .into_iter()
                .map(|e| format!("mainnet: {e}")),
        );

        errors.extend(
            self.quantum
                .validate()
                .into_iter()
                .map(|e| format!("quantum: {e}")),
        );

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AoxConfig;

    #[test]
    fn default_config_is_valid() {
        let cfg = AoxConfig::default();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn validate_collects_errors_from_multiple_sections() {
        let mut cfg = AoxConfig::default();
        cfg.chain.block_time_secs = 1;
        cfg.contracts.registry.local_manifest_directory.clear();
        cfg.mainnet.milestones.pop();
        cfg.quantum.min_security_level = 64;
        cfg.quantum.key_policy.allowed_signature_schemes.clear();

        let errs = cfg.validate().expect_err("config should be invalid");
        assert_eq!(errs.len(), 8);
        assert!(errs.iter().any(|e| e.starts_with("chain:")));
        assert!(errs.iter().any(|e| e.starts_with("contracts:")));
        assert!(errs.iter().any(|e| e.starts_with("mainnet:")));
        assert!(errs.iter().any(|e| e.starts_with("quantum:")));
    }
}
