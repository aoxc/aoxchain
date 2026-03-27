pub mod blockchain;
pub mod contracts;

pub use blockchain::ChainConfig;
pub use contracts::ContractsConfig;

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

        let errs = cfg.validate().expect_err("config should be invalid");
        assert_eq!(errs.len(), 2);
        assert!(errs.iter().any(|e| e.starts_with("chain:")));
        assert!(errs.iter().any(|e| e.starts_with("contracts:")));
    }
}
