// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! AOXC canonical genesis configuration model.
//!
//! This module defines the forward-compatible genesis identity surface for AOXC.
//! The design intentionally separates:
//!
//! - governance-facing human-readable identifiers,
//! - protocol-facing numeric identifiers,
//! - canonical machine-readable network identifiers,
//! - deterministic genesis hashing inputs.
//!
//! The same node binary is expected to support multiple network instances.
//! Therefore, network identity must be derived from configuration and signed
//! genesis artifacts rather than compile-time constants.

use core::fmt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// AOXC governance-controlled family namespace root.
///
/// This value is intended to remain stable across the AOXC network family.
/// It is not a network instance discriminator by itself.
pub const AOXC_FAMILY_ID: u32 = 2626;

/// Canonical AOXC network identifier prefix.
pub const AOXC_NETWORK_ID_PREFIX: &str = "aoxc";

/// Maximum allowed governance serial ordinal.
///
/// Example:
/// - `2626-001`
/// - `2626-999`
pub const MAX_NETWORK_SERIAL_ORDINAL: u16 = 999;

/// Maximum allowed instance ordinal inside a network class segment.
///
/// This preserves a deterministic, bounded identifier policy and prevents
/// malformed identifiers from entering the system unchecked.
pub const MAX_NETWORK_INSTANCE_ORDINAL: u32 = 9999;

/// Canonical network class.
///
/// The class code is embedded into the numeric `chain_id`.
/// This allows the protocol layer to distinguish public, test, development,
/// validation, and private deployment families without relying on string parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkClass {
    /// Canonical public mainnet line.
    PublicMainnet,

    /// Canonical public testnet line.
    PublicTestnet,

    /// Development or experimental network line.
    Devnet,

    /// Pre-production validation or staging network line.
    Validation,

    /// Sovereign private Layer 1 deployment line.
    SovereignPrivate,

    /// Consortium or federated enterprise network line.
    Consortium,

    /// Regulated or supervised private deployment line.
    RegulatedPrivate,
}

impl NetworkClass {
    /// Returns the canonical two-digit class code used in numeric `chain_id`
    /// derivation.
    ///
    /// Numeric chain identifier layout:
    ///
    /// `FFFFCCNNNN`
    ///
    /// Where:
    /// - `FFFF` = family id
    /// - `CC`   = class code
    /// - `NNNN` = instance ordinal
    pub const fn class_code(self) -> u16 {
        match self {
            Self::PublicMainnet => 0,
            Self::PublicTestnet => 1,
            Self::Devnet => 2,
            Self::Validation => 3,
            Self::SovereignPrivate => 10,
            Self::Consortium => 20,
            Self::RegulatedPrivate => 30,
        }
    }

    /// Returns the canonical machine-readable slug.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::PublicMainnet => "mainnet",
            Self::PublicTestnet => "testnet",
            Self::Devnet => "devnet",
            Self::Validation => "validation",
            Self::SovereignPrivate => "sovereign-private",
            Self::Consortium => "consortium",
            Self::RegulatedPrivate => "regulated-private",
        }
    }
}

impl fmt::Display for NetworkClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.slug())
    }
}

/// Governance-facing chain identity model.
///
/// This structure intentionally carries both human-readable and machine-readable
/// identity fields. These fields should be treated as immutable after genesis
/// finalization for a given network instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChainIdentity {
    /// AOXC family namespace root.
    pub family_id: u32,

    /// Canonical protocol-facing numeric chain identifier.
    ///
    /// Example:
    /// - `2626000001` for public mainnet instance 1
    /// - `2626010001` for public testnet instance 1
    pub chain_id: u64,

    /// Human-readable governance serial.
    ///
    /// Example:
    /// - `2626-001`
    /// - `2626-101`
    pub network_serial: String,

    /// Canonical machine-readable network identifier.
    ///
    /// Example:
    /// - `aoxc-mainnet-2626-001`
    /// - `aoxc-testnet-2626-002`
    pub network_id: String,

    /// Human-readable chain display name.
    ///
    /// Example:
    /// - `AOXC Mihver`
    /// - `AOXC Pusula`
    pub chain_name: String,

    /// Canonical network class.
    pub network_class: NetworkClass,
}

impl ChainIdentity {
    /// Constructs a canonical chain identity from explicit policy inputs.
    ///
    /// This constructor enforces deterministic derivation rules for:
    /// - `chain_id`
    /// - `network_serial`
    /// - `network_id`
    pub fn new(
        family_id: u32,
        network_class: NetworkClass,
        governance_serial_ordinal: u16,
        class_instance_ordinal: u32,
        chain_name: impl Into<String>,
    ) -> Result<Self, GenesisConfigError> {
        if governance_serial_ordinal == 0 || governance_serial_ordinal > MAX_NETWORK_SERIAL_ORDINAL
        {
            return Err(GenesisConfigError::InvalidNetworkSerialOrdinal {
                value: governance_serial_ordinal,
            });
        }

        if class_instance_ordinal == 0 || class_instance_ordinal > MAX_NETWORK_INSTANCE_ORDINAL {
            return Err(GenesisConfigError::InvalidClassInstanceOrdinal {
                value: class_instance_ordinal,
            });
        }

        let network_serial = build_network_serial(family_id, governance_serial_ordinal);
        let chain_id = build_chain_id(family_id, network_class, class_instance_ordinal)?;
        let network_id = build_network_id(network_class, &network_serial);

        let identity = Self {
            family_id,
            chain_id,
            network_serial,
            network_id,
            chain_name: chain_name.into(),
            network_class,
        };

        identity.validate()?;
        Ok(identity)
    }

    /// Validates canonical identity invariants.
    pub fn validate(&self) -> Result<(), GenesisConfigError> {
        if self.family_id == 0 {
            return Err(GenesisConfigError::InvalidFamilyId);
        }

        if self.chain_name.trim().is_empty() {
            return Err(GenesisConfigError::EmptyChainName);
        }

        validate_network_serial(self.family_id, &self.network_serial)?;
        validate_network_id(self.network_class, &self.network_serial, &self.network_id)?;

        let expected_prefix = expected_chain_id_prefix(self.family_id, self.network_class);
        let actual_prefix = self.chain_id / 10_000;

        if actual_prefix != expected_prefix {
            return Err(GenesisConfigError::ChainIdPrefixMismatch {
                expected_prefix,
                actual_prefix,
            });
        }

        let instance_ordinal = self.chain_id % 10_000;
        if instance_ordinal == 0 || instance_ordinal > u64::from(MAX_NETWORK_INSTANCE_ORDINAL) {
            return Err(GenesisConfigError::InvalidDerivedChainIdOrdinal {
                value: instance_ordinal,
            });
        }

        Ok(())
    }

    /// Returns a deterministic fingerprint for the identity itself.
    ///
    /// This is not a signature and must not be treated as such.
    #[must_use]
    pub fn fingerprint_hex(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.family_id.to_le_bytes());
        hasher.update(self.chain_id.to_le_bytes());
        hasher.update(self.network_serial.as_bytes());
        hasher.update(self.network_id.as_bytes());
        hasher.update(self.chain_name.as_bytes());
        hasher.update(self.network_class.slug().as_bytes());
        hex::encode(hasher.finalize())
    }
}

/// Genesis configuration root.
///
/// This structure is intentionally designed to remain stable and extensible.
/// Additional policy or provenance fields may be appended in future versions
/// without changing the core identity derivation model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenesisConfig {
    /// Canonical chain identity.
    pub identity: ChainIdentity,

    /// Block target time in milliseconds.
    pub block_time: u64,

    /// Genesis validators.
    pub validators: Vec<Validator>,

    /// Genesis accounts and their initial balances / permissions.
    pub accounts: Vec<GenesisAccount>,

    /// Initial treasury allocation.
    pub treasury: u128,

    /// Settlement anchoring configuration.
    pub settlement_link: SettlementLink,

    /// AOXC genesis seal material.
    pub genesis_seal: AOXCANDSeal,

    /// Genesis creation timestamp (unix seconds, UTC).
    #[serde(default = "default_genesis_timestamp")]
    pub genesis_time_unix: u64,

    /// Protocol version for compatibility pinning.
    #[serde(default = "default_protocol_version")]
    pub protocol_version: String,

    /// Quantum-resilience and cryptographic policy profile.
    #[serde(default)]
    pub quantum_policy: QuantumPolicy,

    /// Canonical bootstrap peers for the selected network.
    #[serde(default)]
    pub boot_nodes: Vec<BootNode>,

    /// Human-readable genesis release notes.
    #[serde(default)]
    pub genesis_notes: String,
}

impl GenesisConfig {
    /// Constructs a new genesis configuration.
    pub fn new(
        identity: ChainIdentity,
        block_time: u64,
        validators: Vec<Validator>,
        accounts: Vec<GenesisAccount>,
        treasury: u128,
        settlement_link: SettlementLink,
        genesis_seal: AOXCANDSeal,
    ) -> Result<Self, GenesisConfigError> {
        let cfg = Self {
            quantum_policy: QuantumPolicy::for_network_class(identity.network_class),
            boot_nodes: default_boot_nodes(identity.network_class),
            genesis_notes: default_genesis_notes(identity.network_class).to_string(),
            genesis_time_unix: default_genesis_timestamp(),
            protocol_version: default_protocol_version(),
            identity,
            block_time,
            validators,
            accounts,
            treasury,
            settlement_link,
            genesis_seal,
        };

        cfg.validate()?;
        Ok(cfg)
    }

    /// Returns a canonical public mainnet identity helper.
    pub fn mainnet_identity(
        chain_name: impl Into<String>,
    ) -> Result<ChainIdentity, GenesisConfigError> {
        ChainIdentity::new(
            AOXC_FAMILY_ID,
            NetworkClass::PublicMainnet,
            1,
            1,
            chain_name,
        )
    }

    /// Returns a canonical public testnet identity helper.
    pub fn testnet_identity(
        chain_name: impl Into<String>,
    ) -> Result<ChainIdentity, GenesisConfigError> {
        ChainIdentity::new(
            AOXC_FAMILY_ID,
            NetworkClass::PublicTestnet,
            2,
            1,
            chain_name,
        )
    }

    /// Validates genesis invariants.
    pub fn validate(&self) -> Result<(), GenesisConfigError> {
        self.identity.validate()?;

        if self.block_time == 0 {
            return Err(GenesisConfigError::InvalidBlockTime);
        }

        if self.validators.is_empty() {
            return Err(GenesisConfigError::EmptyValidators);
        }

        if self.accounts.is_empty() {
            return Err(GenesisConfigError::EmptyAccounts);
        }

        self.quantum_policy.validate()?;

        if self.protocol_version.trim().is_empty() {
            return Err(GenesisConfigError::InvalidProtocolVersion);
        }

        if self.boot_nodes.is_empty() {
            return Err(GenesisConfigError::EmptyBootNodes);
        }

        for node in &self.boot_nodes {
            node.validate()?;
        }

        Ok(())
    }

    /// Computes a deterministic state fingerprint for the genesis configuration.
    ///
    /// This function intentionally includes identity material so that network
    /// identity drift cannot silently produce the same genesis fingerprint.
    pub fn try_state_hash(&self) -> Result<[u8; 32], GenesisConfigError> {
        self.validate()?;

        let mut hasher = Sha256::new();

        hasher.update(self.identity.family_id.to_le_bytes());
        hasher.update(self.identity.chain_id.to_le_bytes());
        hasher.update(self.identity.network_serial.as_bytes());
        hasher.update(self.identity.network_id.as_bytes());
        hasher.update(self.identity.chain_name.as_bytes());
        hasher.update(self.identity.network_class.slug().as_bytes());

        hasher.update(self.block_time.to_le_bytes());
        hasher.update(self.treasury.to_le_bytes());

        for validator in &self.validators {
            hasher.update(format!("{validator:?}").as_bytes());
        }

        for account in &self.accounts {
            hasher.update(format!("{account:?}").as_bytes());
        }

        hasher.update(format!("{:?}", self.settlement_link).as_bytes());
        hasher.update(format!("{:?}", self.genesis_seal).as_bytes());
        hasher.update(self.genesis_time_unix.to_le_bytes());
        hasher.update(self.protocol_version.as_bytes());
        hasher.update(format!("{:?}", self.quantum_policy).as_bytes());

        for node in &self.boot_nodes {
            hasher.update(format!("{node:?}").as_bytes());
        }

        hasher.update(self.genesis_notes.as_bytes());

        let digest = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&digest);
        Ok(out)
    }

    /// Returns the hex-encoded deterministic genesis fingerprint.
    pub fn fingerprint(&self) -> Result<String, GenesisConfigError> {
        Ok(hex::encode(self.try_state_hash()?))
    }
}

/// Canonical numeric chain identifier builder.
///
/// Format:
/// `FFFFCCNNNN`
///
/// Example:
/// - family `2626`, class `00`, instance `0001` => `2626000001`
/// - family `2626`, class `01`, instance `0001` => `2626010001`
pub fn build_chain_id(
    family_id: u32,
    network_class: NetworkClass,
    class_instance_ordinal: u32,
) -> Result<u64, GenesisConfigError> {
    if family_id == 0 {
        return Err(GenesisConfigError::InvalidFamilyId);
    }

    if class_instance_ordinal == 0 || class_instance_ordinal > MAX_NETWORK_INSTANCE_ORDINAL {
        return Err(GenesisConfigError::InvalidClassInstanceOrdinal {
            value: class_instance_ordinal,
        });
    }

    let class_code = u64::from(network_class.class_code());
    let family = u64::from(family_id);
    let ordinal = u64::from(class_instance_ordinal);

    Ok((family * 1_000_000) + (class_code * 10_000) + ordinal)
}

/// Canonical governance serial builder.
///
/// Format:
/// `FFFF-NNN`
#[must_use]
pub fn build_network_serial(family_id: u32, governance_serial_ordinal: u16) -> String {
    format!("{family_id}-{governance_serial_ordinal:03}")
}

/// Canonical machine-readable network identifier builder.
///
/// Format:
/// `aoxc-<class>-<family>-<ordinal>`
#[must_use]
pub fn build_network_id(network_class: NetworkClass, network_serial: &str) -> String {
    format!(
        "{}-{}-{}",
        AOXC_NETWORK_ID_PREFIX,
        network_class.slug(),
        network_serial.to_ascii_lowercase()
    )
}

/// Validates governance serial shape.
///
/// Expected format:
/// - `<family_id>-<3 digit ordinal>`
/// - Example: `2626-001`
pub fn validate_network_serial(
    expected_family_id: u32,
    network_serial: &str,
) -> Result<(), GenesisConfigError> {
    let Some((family, ordinal)) = network_serial.split_once('-') else {
        return Err(GenesisConfigError::MalformedNetworkSerial);
    };

    let parsed_family = family
        .parse::<u32>()
        .map_err(|_| GenesisConfigError::MalformedNetworkSerial)?;

    if parsed_family != expected_family_id {
        return Err(GenesisConfigError::NetworkSerialFamilyMismatch {
            expected: expected_family_id,
            actual: parsed_family,
        });
    }

    if ordinal.len() != 3 || !ordinal.chars().all(|c| c.is_ascii_digit()) {
        return Err(GenesisConfigError::MalformedNetworkSerial);
    }

    let parsed_ordinal = ordinal
        .parse::<u16>()
        .map_err(|_| GenesisConfigError::MalformedNetworkSerial)?;

    if parsed_ordinal == 0 || parsed_ordinal > MAX_NETWORK_SERIAL_ORDINAL {
        return Err(GenesisConfigError::InvalidNetworkSerialOrdinal {
            value: parsed_ordinal,
        });
    }

    Ok(())
}

/// Validates canonical machine-readable network identifier shape.
pub fn validate_network_id(
    network_class: NetworkClass,
    network_serial: &str,
    network_id: &str,
) -> Result<(), GenesisConfigError> {
    let expected = build_network_id(network_class, network_serial);
    if network_id != expected {
        return Err(GenesisConfigError::NetworkIdMismatch {
            expected,
            actual: network_id.to_owned(),
        });
    }

    Ok(())
}

/// Returns the expected `FFFFCC` prefix segment of the numeric `chain_id`.
fn expected_chain_id_prefix(family_id: u32, network_class: NetworkClass) -> u64 {
    (u64::from(family_id) * 100) + u64::from(network_class.class_code())
}

/// Genesis configuration validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenesisConfigError {
    InvalidFamilyId,
    EmptyChainName,
    InvalidBlockTime,
    EmptyValidators,
    EmptyAccounts,
    MalformedNetworkSerial,
    InvalidNetworkSerialOrdinal {
        value: u16,
    },
    InvalidClassInstanceOrdinal {
        value: u32,
    },
    InvalidDerivedChainIdOrdinal {
        value: u64,
    },
    NetworkSerialFamilyMismatch {
        expected: u32,
        actual: u32,
    },
    NetworkIdMismatch {
        expected: String,
        actual: String,
    },
    ChainIdPrefixMismatch {
        expected_prefix: u64,
        actual_prefix: u64,
    },
    InvalidProtocolVersion,
    EmptyBootNodes,
    InvalidBootNode {
        node_id: String,
    },
    InvalidQuantumPolicy,
}

impl fmt::Display for GenesisConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFamilyId => {
                f.write_str("genesis configuration validation failed: family_id must be non-zero")
            }
            Self::EmptyChainName => {
                f.write_str("genesis configuration validation failed: chain_name must not be empty")
            }
            Self::InvalidBlockTime => {
                f.write_str("genesis configuration validation failed: block_time must be non-zero")
            }
            Self::EmptyValidators => f.write_str(
                "genesis configuration validation failed: validator set must not be empty",
            ),
            Self::EmptyAccounts => f.write_str(
                "genesis configuration validation failed: account set must not be empty",
            ),
            Self::MalformedNetworkSerial => f.write_str(
                "genesis configuration validation failed: network_serial format is invalid",
            ),
            Self::InvalidNetworkSerialOrdinal { value } => {
                write!(
                    f,
                    "genesis configuration validation failed: network serial ordinal `{value}` is outside policy bounds"
                )
            }
            Self::InvalidClassInstanceOrdinal { value } => {
                write!(
                    f,
                    "genesis configuration validation failed: class instance ordinal `{value}` is outside policy bounds"
                )
            }
            Self::InvalidDerivedChainIdOrdinal { value } => {
                write!(
                    f,
                    "genesis configuration validation failed: derived chain_id ordinal `{value}` is invalid"
                )
            }
            Self::NetworkSerialFamilyMismatch { expected, actual } => {
                write!(
                    f,
                    "genesis configuration validation failed: network_serial family mismatch; expected `{expected}`, got `{actual}`"
                )
            }
            Self::NetworkIdMismatch { expected, actual } => {
                write!(
                    f,
                    "genesis configuration validation failed: network_id mismatch; expected `{expected}`, got `{actual}`"
                )
            }
            Self::ChainIdPrefixMismatch {
                expected_prefix,
                actual_prefix,
            } => {
                write!(
                    f,
                    "genesis configuration validation failed: chain_id prefix mismatch; expected `{expected_prefix}`, got `{actual_prefix}`"
                )
            }
            Self::InvalidProtocolVersion => f.write_str(
                "genesis configuration validation failed: protocol_version must not be empty",
            ),
            Self::EmptyBootNodes => f.write_str(
                "genesis configuration validation failed: boot_nodes must not be empty",
            ),
            Self::InvalidBootNode { node_id } => {
                write!(
                    f,
                    "genesis configuration validation failed: boot node `{node_id}` is invalid"
                )
            }
            Self::InvalidQuantumPolicy => f.write_str(
                "genesis configuration validation failed: quantum policy is invalid",
            ),
        }
    }
}

impl std::error::Error for GenesisConfigError {}

/// The project should replace these placeholder integration types with the
/// real canonical domain types already defined in AOXC.
///
/// They are included here only to make the model structurally complete.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Validator {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenesisAccount {
    pub address: String,
    pub balance: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementLink {
    pub endpoint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AOXCANDSeal {
    pub seal_id: String,
}

/// Canonical bootstrap node descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BootNode {
    pub node_id: String,
    pub endpoint: String,
    pub role: String,
}

impl BootNode {
    fn validate(&self) -> Result<(), GenesisConfigError> {
        if self.node_id.trim().is_empty()
            || self.endpoint.trim().is_empty()
            || self.role.trim().is_empty()
        {
            return Err(GenesisConfigError::InvalidBootNode {
                node_id: self.node_id.clone(),
            });
        }

        if !self.endpoint.contains("://") {
            return Err(GenesisConfigError::InvalidBootNode {
                node_id: self.node_id.clone(),
            });
        }

        Ok(())
    }
}

/// Quantum security policy profile for genesis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantumPolicy {
    pub pq_signature_schemes: Vec<String>,
    pub classical_signature_schemes: Vec<String>,
    pub handshake_kem: String,
    pub state_hash: String,
    pub commitment_hash: String,
    pub min_signature_threshold: u16,
    pub rotation_epoch_blocks: u64,
}

impl Default for QuantumPolicy {
    fn default() -> Self {
        Self::for_network_class(NetworkClass::Devnet)
    }
}

impl QuantumPolicy {
    #[must_use]
    pub fn for_network_class(network_class: NetworkClass) -> Self {
        match network_class {
            NetworkClass::PublicMainnet => Self {
                pq_signature_schemes: vec!["ML-DSA-87".into(), "SLH-DSA-SHA2-192f".into()],
                classical_signature_schemes: vec!["Ed25519".into()],
                handshake_kem: "ML-KEM-1024".into(),
                state_hash: "SHA3-256".into(),
                commitment_hash: "BLAKE3".into(),
                min_signature_threshold: 3,
                rotation_epoch_blocks: 43_200,
            },
            NetworkClass::PublicTestnet | NetworkClass::Validation => Self {
                pq_signature_schemes: vec!["ML-DSA-65".into()],
                classical_signature_schemes: vec!["Ed25519".into()],
                handshake_kem: "ML-KEM-768".into(),
                state_hash: "SHA3-256".into(),
                commitment_hash: "BLAKE3".into(),
                min_signature_threshold: 2,
                rotation_epoch_blocks: 14_400,
            },
            NetworkClass::Devnet
            | NetworkClass::SovereignPrivate
            | NetworkClass::Consortium
            | NetworkClass::RegulatedPrivate => Self {
                pq_signature_schemes: vec!["ML-DSA-44".into()],
                classical_signature_schemes: vec!["Ed25519".into()],
                handshake_kem: "ML-KEM-512".into(),
                state_hash: "SHA3-256".into(),
                commitment_hash: "BLAKE3".into(),
                min_signature_threshold: 1,
                rotation_epoch_blocks: 7_200,
            },
        }
    }

    fn validate(&self) -> Result<(), GenesisConfigError> {
        if self.pq_signature_schemes.is_empty()
            || self.classical_signature_schemes.is_empty()
            || self.handshake_kem.trim().is_empty()
            || self.state_hash.trim().is_empty()
            || self.commitment_hash.trim().is_empty()
            || self.min_signature_threshold == 0
            || self.rotation_epoch_blocks == 0
        {
            return Err(GenesisConfigError::InvalidQuantumPolicy);
        }

        Ok(())
    }
}

fn default_genesis_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn default_protocol_version() -> String {
    "aoxc-genesis/v2".to_string()
}

fn default_genesis_notes(network_class: NetworkClass) -> &'static str {
    match network_class {
        NetworkClass::PublicMainnet => {
            "AOXC mainnet genesis: production consensus, strict PQ threshold policy, audited settlement."
        }
        NetworkClass::PublicTestnet => {
            "AOXC testnet genesis: feature proving, interoperability rehearsal, accelerated validator rotation."
        }
        NetworkClass::Devnet => {
            "AOXC devnet genesis: rapid experimentation profile, short epochs, permissive development policy."
        }
        NetworkClass::Validation => {
            "AOXC validation genesis: staging hardening before public release."
        }
        NetworkClass::SovereignPrivate => "AOXC sovereign private genesis profile.",
        NetworkClass::Consortium => "AOXC consortium genesis profile.",
        NetworkClass::RegulatedPrivate => "AOXC regulated private genesis profile.",
    }
}

fn default_boot_nodes(network_class: NetworkClass) -> Vec<BootNode> {
    let suffix = network_class.slug();
    vec![
        BootNode {
            node_id: format!("aoxc-{suffix}-boot-001"),
            endpoint: format!("aoxc://{suffix}.seed-001.aoxc.net:443"),
            role: "seed".to_string(),
        },
        BootNode {
            node_id: format!("aoxc-{suffix}-boot-002"),
            endpoint: format!("aoxc://{suffix}.seed-002.aoxc.net:443"),
            role: "relay".to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_public_mainnet_chain_id_correctly() {
        let chain_id = build_chain_id(AOXC_FAMILY_ID, NetworkClass::PublicMainnet, 1).unwrap();
        assert_eq!(chain_id, 2626000001);
    }

    #[test]
    fn builds_public_testnet_chain_id_correctly() {
        let chain_id = build_chain_id(AOXC_FAMILY_ID, NetworkClass::PublicTestnet, 1).unwrap();
        assert_eq!(chain_id, 2626010001);
    }

    #[test]
    fn builds_network_serial_correctly() {
        assert_eq!(build_network_serial(2626, 1), "2626-001");
        assert_eq!(build_network_serial(2626, 12), "2626-012");
    }

    #[test]
    fn builds_network_id_correctly() {
        let network_id = build_network_id(NetworkClass::PublicMainnet, "2626-001");
        assert_eq!(network_id, "aoxc-mainnet-2626-001");
    }

    #[test]
    fn validates_chain_identity_successfully() {
        let identity = ChainIdentity::new(
            AOXC_FAMILY_ID,
            NetworkClass::PublicMainnet,
            1,
            1,
            "AOXC Mihver",
        )
        .unwrap();

        assert_eq!(identity.chain_id, 2626000001);
        assert_eq!(identity.network_serial, "2626-001");
        assert_eq!(identity.network_id, "aoxc-mainnet-2626-001");
    }

    #[test]
    fn genesis_fingerprint_is_deterministic() {
        let identity = ChainIdentity::new(
            AOXC_FAMILY_ID,
            NetworkClass::PublicMainnet,
            1,
            1,
            "AOXC Mihver",
        )
        .unwrap();

        let cfg = GenesisConfig::new(
            identity,
            3000,
            vec![Validator { id: "val-1".into() }],
            vec![GenesisAccount {
                address: "aox1test".into(),
                balance: 1_000_000,
            }],
            10_000_000,
            SettlementLink {
                endpoint: "settlement://root".into(),
            },
            AOXCANDSeal {
                seal_id: "seal-001".into(),
            },
        )
        .unwrap();

        let fp1 = cfg.fingerprint().unwrap();
        let fp2 = cfg.fingerprint().unwrap();

        assert_eq!(fp1, fp2);
    }

    #[test]
    fn mainnet_quantum_policy_is_strict() {
        let identity = ChainIdentity::new(
            AOXC_FAMILY_ID,
            NetworkClass::PublicMainnet,
            1,
            1,
            "AOXC Mainnet",
        )
        .unwrap();

        let cfg = GenesisConfig::new(
            identity,
            3000,
            vec![Validator {
                id: "val-mainnet-1".into(),
            }],
            vec![GenesisAccount {
                address: "aox1mainnet".into(),
                balance: 10_000_000,
            }],
            100_000_000,
            SettlementLink {
                endpoint: "aoxc://settlement/mainnet".into(),
            },
            AOXCANDSeal {
                seal_id: "seal-mainnet-001".into(),
            },
        )
        .unwrap();

        assert_eq!(cfg.quantum_policy.handshake_kem, "ML-KEM-1024");
        assert!(cfg.quantum_policy.min_signature_threshold >= 3);
        assert!(!cfg.boot_nodes.is_empty());
    }

    #[test]
    fn testnet_and_devnet_receive_distinct_quantum_profiles() {
        let testnet = QuantumPolicy::for_network_class(NetworkClass::PublicTestnet);
        let devnet = QuantumPolicy::for_network_class(NetworkClass::Devnet);

        assert_ne!(testnet.handshake_kem, devnet.handshake_kem);
        assert!(testnet.rotation_epoch_blocks > devnet.rotation_epoch_blocks);
    }
}
