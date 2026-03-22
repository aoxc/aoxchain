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
