// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::error::{AppError, ErrorCode};
use serde::{Deserialize, Serialize};

pub(super) fn default_consensus_identity_profile() -> String {
    "hybrid".to_string()
}

/// Canonical AOXC environment identity description used by bootstrap flows.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct CanonicalIdentity {
    pub(super) family_id: u32,
    pub(super) chain_name: String,
    pub(super) network_class: String,
    pub(super) network_serial: String,
    pub(super) chain_id: u64,
    pub(super) network_id: String,
}

/// Canonical bootstrap genesis document.
///
/// This structure intentionally mirrors the AOXC environment-level genesis
/// schema used under `configs/environments/*/genesis.v1.json` rather than the
/// older `chain_num`-based bootstrap format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct BootstrapGenesisDocument {
    pub(super) schema_version: u8,
    pub(super) genesis_kind: String,
    pub(super) environment: String,
    pub(super) family_name: String,
    pub(super) family_code: String,
    pub(super) identity: CanonicalIdentity,
    pub(super) consensus: BootstrapConsensusConfig,
    #[serde(default)]
    pub(super) vm: BootstrapVmConfig,
    pub(super) economics: BootstrapEconomicsConfig,
    pub(super) state: BootstrapStateConfig,
    pub(super) bindings: BootstrapBindingsConfig,
    pub(super) integrity: BootstrapIntegrityConfig,
    pub(super) metadata: BootstrapMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct BootstrapConsensusConfig {
    pub(super) engine: String,
    pub(super) mode: String,
    pub(super) genesis_epoch: u64,
    pub(super) block_time_ms: u64,
    pub(super) validator_quorum_policy: String,
    #[serde(default = "default_consensus_identity_profile")]
    pub(super) consensus_identity_profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct BootstrapVmConfig {
    pub(super) vm_engine: String,
    pub(super) gas_model: String,
    pub(super) block_gas_limit: u64,
    pub(super) tx_gas_limit: u64,
    pub(super) min_gas_price: String,
    pub(super) max_contract_size_bytes: u64,
    pub(super) max_call_depth: u16,
    pub(super) enable_parallel_execution: bool,
}

impl Default for BootstrapVmConfig {
    fn default() -> Self {
        Self {
            vm_engine: "aoxcvm".to_string(),
            gas_model: "aoxc-gas-v1".to_string(),
            block_gas_limit: 120_000_000,
            tx_gas_limit: 15_000_000,
            min_gas_price: "1".to_string(),
            max_contract_size_bytes: 256 * 1024,
            max_call_depth: 64,
            enable_parallel_execution: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct BootstrapEconomicsConfig {
    pub(super) native_symbol: String,
    pub(super) native_decimals: u8,
    pub(super) initial_treasury: BootstrapTreasuryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct BootstrapTreasuryConfig {
    pub(super) account_id: String,
    pub(super) amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct BootstrapStateConfig {
    pub(super) accounts: Vec<BootstrapAccountRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct BootstrapAccountRecord {
    pub(super) account_id: String,
    pub(super) balance: String,
    pub(super) role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct BootstrapBindingsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) accounts_file: Option<String>,
    pub(super) validators_file: String,
    pub(super) bootnodes_file: String,
    pub(super) certificate_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct BootstrapIntegrityConfig {
    pub(super) hash_algorithm: String,
    pub(super) deterministic_serialization_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct BootstrapMetadata {
    pub(super) description: String,
    pub(super) status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct BootstrapValidatorBindingsDocument {
    pub(super) schema_version: u8,
    pub(super) environment: String,
    pub(super) identity: CanonicalIdentity,
    pub(super) validators: Vec<BootstrapValidatorBindingRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct BootstrapValidatorBindingRecord {
    pub(super) validator_id: String,
    pub(super) display_name: String,
    pub(super) role: String,
    pub(super) consensus_key_algorithm: String,
    pub(super) consensus_public_key_encoding: String,
    pub(super) consensus_public_key: String,
    pub(super) consensus_key_fingerprint: String,
    pub(super) network_key_algorithm: String,
    pub(super) network_public_key_encoding: String,
    pub(super) network_public_key: String,
    pub(super) network_key_fingerprint: String,
    pub(super) weight: u64,
    pub(super) status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct BootstrapBootnodesDocument {
    pub(super) schema_version: u8,
    pub(super) environment: String,
    pub(super) identity: CanonicalIdentity,
    pub(super) bootnodes: Vec<BootstrapBootnodeRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct BootstrapBootnodeRecord {
    pub(super) node_id: String,
    pub(super) display_name: String,
    pub(super) transport_key_algorithm: String,
    pub(super) transport_public_key_encoding: String,
    pub(super) transport_public_key: String,
    pub(super) transport_key_fingerprint: String,
    pub(super) address: String,
    pub(super) transport: String,
    pub(super) status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct BootstrapCertificateDocument {
    pub(super) schema_version: u8,
    pub(super) certificate_kind: String,
    pub(super) environment: String,
    pub(super) identity: CanonicalIdentity,
    pub(super) certificate: BootstrapCertificateBody,
    pub(super) metadata: BootstrapMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct BootstrapCertificateBody {
    pub(super) status: String,
    pub(super) issuer: String,
    pub(super) subject: String,
    pub(super) certificate_serial: String,
    pub(super) issued_at: String,
    pub(super) expires_at: Option<String>,
    pub(super) fingerprint_sha256: String,
    pub(super) signature_algorithm: String,
    pub(super) signature: String,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct ProfileBootstrapSummary {
    pub(super) profile: String,
    pub(super) home_dir: String,
    pub(super) bind_host: String,
    pub(super) p2p_port: u16,
    pub(super) rpc_port: u16,
    pub(super) prometheus_port: u16,
    pub(super) chain_id: u64,
    pub(super) network_id: String,
    pub(super) operator_fingerprint: String,
    pub(super) consensus_public_key: String,
    pub(super) node_height: u64,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct DualProfileBootstrapResult {
    pub(super) output_dir: String,
    pub(super) profiles: Vec<ProfileBootstrapSummary>,
    pub(super) launch_hint: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct TopologyBootstrapNodeSummary {
    pub(super) topology_role: String,
    pub(super) bootstrap: ProfileBootstrapSummary,
    pub(super) rpc_url: String,
    pub(super) metrics_url: String,
    pub(super) start_command: String,
    pub(super) query_commands: Vec<String>,
    pub(super) allocation_preset: String,
    pub(super) genesis_accounts_total: usize,
    pub(super) genesis_accounts_preview: Vec<BootstrapAccountRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct TopologyBootstrapResult {
    pub(super) topology_mode: String,
    pub(super) output_dir: String,
    pub(super) profile: String,
    pub(super) node_count: usize,
    pub(super) nodes: Vec<TopologyBootstrapNodeSummary>,
    pub(super) genesis_consistency: Vec<String>,
    pub(super) rpc_api_runbook: Vec<String>,
    pub(super) economics_summary: Vec<String>,
    pub(super) launch_hint: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct AddressCreateOutput {
    pub(super) profile: String,
    pub(super) validator_name: String,
    pub(super) validator_account_id: String,
    pub(super) bundle_fingerprint: String,
    pub(super) consensus_public_key: String,
    pub(super) transport_public_key: String,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct GenesisTemplateOutput {
    pub(super) profile: String,
    pub(super) output_path: String,
    pub(super) chain_name: String,
    pub(super) network_id: String,
    pub(super) validator_quorum_policy: String,
    pub(super) deterministic_serialization_required: bool,
    pub(super) notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct GenesisSecurityAuditReport {
    pub(super) genesis_path: String,
    pub(super) profile: String,
    pub(super) score: u8,
    pub(super) verdict: &'static str,
    pub(super) passed: Vec<String>,
    pub(super) warnings: Vec<String>,
    pub(super) blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct ConsensusProfileAuditReport {
    pub(super) genesis_path: String,
    pub(super) profile: String,
    pub(super) consensus_identity_profile: String,
    pub(super) score: u8,
    pub(super) verdict: &'static str,
    pub(super) passed: Vec<String>,
    pub(super) warnings: Vec<String>,
    pub(super) blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConsensusProfileGateStatus {
    pub passed: bool,
    pub detail: String,
    pub verdict: String,
    pub blockers: Vec<String>,
    pub profile: String,
    pub genesis_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EnvironmentProfile {
    Mainnet,
    Testnet,
    Validation,
    Devnet,
    Localnet,
}

impl EnvironmentProfile {
    pub(super) fn parse(value: &str) -> Result<Self, AppError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "mainnet" => Ok(Self::Mainnet),
            "testnet" => Ok(Self::Testnet),
            "validation" => Ok(Self::Validation),
            "validator" => Ok(Self::Validation),
            "devnet" => Ok(Self::Devnet),
            "localnet" => Ok(Self::Localnet),
            other => Err(AppError::new(
                ErrorCode::UsageInvalidArguments,
                format!("Unsupported AOXC profile `{}`", other),
            )),
        }
    }

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Testnet => "testnet",
            Self::Validation => "validation",
            Self::Devnet => "devnet",
            Self::Localnet => "localnet",
        }
    }

    pub(super) fn identity(self) -> CanonicalIdentity {
        match self {
            Self::Mainnet => CanonicalIdentity {
                family_id: 2626,
                chain_name: "AOXC AKDENIZ".to_string(),
                network_class: "public_mainnet".to_string(),
                network_serial: "2626-001".to_string(),
                chain_id: 2_626_000_001,
                network_id: "aoxc-mainnet-2626-001".to_string(),
            },
            Self::Testnet => CanonicalIdentity {
                family_id: 2626,
                chain_name: "AOXC PUSULA".to_string(),
                network_class: "public_testnet".to_string(),
                network_serial: "2626-002".to_string(),
                chain_id: 2_626_010_001,
                network_id: "aoxc-testnet-2626-002".to_string(),
            },
            Self::Validation => CanonicalIdentity {
                family_id: 2626,
                chain_name: "AOXC MIZAN".to_string(),
                network_class: "validation".to_string(),
                network_serial: "2626-004".to_string(),
                chain_id: 2_626_030_001,
                network_id: "aoxc-validation-2626-004".to_string(),
            },
            Self::Devnet => CanonicalIdentity {
                family_id: 2626,
                chain_name: "AOXC KIVILCIM".to_string(),
                network_class: "devnet".to_string(),
                network_serial: "2626-003".to_string(),
                chain_id: 2_626_020_001,
                network_id: "aoxc-devnet-2626-003".to_string(),
            },
            Self::Localnet => CanonicalIdentity {
                family_id: 2626,
                chain_name: "AOXC LOCALNET ATLAS".to_string(),
                network_class: "localnet".to_string(),
                network_serial: "2626-900".to_string(),
                chain_id: 2_626_900_001,
                network_id: "aoxc-localnet-2626-900".to_string(),
            },
        }
    }

    pub(super) fn genesis_document(self) -> BootstrapGenesisDocument {
        let identity = self.identity();
        let validator_quorum_policy = match self {
            Self::Mainnet | Self::Testnet => "pq-hybrid-threshold-2of3".to_string(),
            _ => "strict-majority".to_string(),
        };

        BootstrapGenesisDocument {
            schema_version: 1,
            genesis_kind: "aoxc-genesis-config".to_string(),
            environment: self.as_str().to_string(),
            family_name: "AOXC".to_string(),
            family_code: "aoxc".to_string(),
            identity,
            consensus: BootstrapConsensusConfig {
                engine: "aoxcunity".to_string(),
                mode: "bft".to_string(),
                genesis_epoch: 0,
                block_time_ms: 6_000,
                validator_quorum_policy,
                consensus_identity_profile: default_consensus_identity_profile(),
            },
            vm: BootstrapVmConfig::default(),
            economics: BootstrapEconomicsConfig {
                native_symbol: "AOXC".to_string(),
                native_decimals: 18,
                initial_treasury: BootstrapTreasuryConfig {
                    account_id: "AOXC_TREASURY_GENESIS".to_string(),
                    amount: "1000000000".to_string(),
                },
            },
            state: BootstrapStateConfig {
                accounts: vec![BootstrapAccountRecord {
                    account_id: "AOXC_TREASURY_GENESIS".to_string(),
                    balance: "1000000000".to_string(),
                    role: "treasury".to_string(),
                }],
            },
            bindings: BootstrapBindingsConfig {
                accounts_file: if self == Self::Localnet {
                    Some("accounts.json".to_string())
                } else {
                    None
                },
                validators_file: "validators.json".to_string(),
                bootnodes_file: "bootnodes.json".to_string(),
                certificate_file: "certificate.json".to_string(),
            },
            integrity: BootstrapIntegrityConfig {
                hash_algorithm: "sha256".to_string(),
                deterministic_serialization_required: true,
            },
            metadata: BootstrapMetadata {
                description: format!("Canonical AOXC {} genesis configuration.", self.as_str()),
                status: "active".to_string(),
            },
        }
    }
}
