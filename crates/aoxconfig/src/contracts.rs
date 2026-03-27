use serde::{Deserialize, Serialize};

use aoxcontract::{ArtifactFormat, SourceTrustLevel, VmTarget};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub enabled: bool,
    pub local_manifest_directory: String,
    pub activation_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactPolicyConfig {
    pub max_artifact_size: u64,
    pub allowed_vm_targets: Vec<VmTarget>,
    pub allowed_artifact_formats: Vec<ArtifactFormat>,
    pub review_required: bool,
    pub source_trust_level: SourceTrustLevel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractLimitsConfig {
    pub max_entrypoints: usize,
    pub max_name_len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractsConfig {
    pub registry: RegistryConfig,
    pub artifact_policy: ArtifactPolicyConfig,
    pub limits: ContractLimitsConfig,
}

impl ContractsConfig {
    /// Validate contract subsystem configuration and return all violations.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.registry.local_manifest_directory.trim().is_empty() {
            errors.push("registry.local_manifest_directory must not be empty".to_string());
        }

        if !matches!(
            self.registry.activation_policy.as_str(),
            "manual" | "auto" | "governance"
        ) {
            errors.push(
                "registry.activation_policy must be one of: manual, auto, governance".to_string(),
            );
        }

        if self.artifact_policy.max_artifact_size == 0 {
            errors.push("artifact_policy.max_artifact_size must be greater than zero".to_string());
        }

        if self.artifact_policy.allowed_vm_targets.is_empty() {
            errors.push("artifact_policy.allowed_vm_targets must not be empty".to_string());
        }

        if self.artifact_policy.allowed_artifact_formats.is_empty() {
            errors.push("artifact_policy.allowed_artifact_formats must not be empty".to_string());
        }

        if self.limits.max_entrypoints == 0 {
            errors.push("limits.max_entrypoints must be greater than zero".to_string());
        }

        if self.limits.max_name_len == 0 {
            errors.push("limits.max_name_len must be greater than zero".to_string());
        }

        errors
    }
}

impl Default for ContractsConfig {
    fn default() -> Self {
        Self {
            registry: RegistryConfig {
                enabled: true,
                local_manifest_directory: ".aox/contracts".into(),
                activation_policy: "manual".into(),
            },
            artifact_policy: ArtifactPolicyConfig {
                max_artifact_size: 10 * 1024 * 1024,
                allowed_vm_targets: vec![VmTarget::Wasm, VmTarget::Evm],
                allowed_artifact_formats: vec![
                    ArtifactFormat::WasmModule,
                    ArtifactFormat::EvmBytecode,
                ],
                review_required: true,
                source_trust_level: SourceTrustLevel::ReviewRequired,
            },
            limits: ContractLimitsConfig {
                max_entrypoints: 32,
                max_name_len: 128,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ContractsConfig;

    #[test]
    fn default_contract_config_is_valid() {
        let cfg = ContractsConfig::default();
        assert!(cfg.validate().is_empty());
    }

    #[test]
    fn invalid_values_are_reported() {
        let mut cfg = ContractsConfig::default();
        cfg.registry.activation_policy = "invalid".to_string();
        cfg.registry.local_manifest_directory.clear();
        cfg.limits.max_name_len = 0;

        let errs = cfg.validate();
        assert!(errs.len() >= 3);
        assert!(errs.iter().any(|e| e.contains("local_manifest_directory")));
        assert!(errs.iter().any(|e| e.contains("activation_policy")));
        assert!(errs.iter().any(|e| e.contains("max_name_len")));
    }
}
