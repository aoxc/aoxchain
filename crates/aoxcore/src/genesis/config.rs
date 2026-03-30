// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! AOXC canonical genesis configuration model.
//!
//! This module defines a production-oriented genesis configuration surface for
//! AOXC-native networks.
//!
//! Design objectives:
//! - deterministic identity derivation,
//! - canonical and reproducible genesis fingerprinting,
//! - strict validation of policy-bearing fields,
//! - future-compatible network profile governance,
//! - quantum-aware configuration posture,
//! - explicit protection against non-canonical hashing inputs.
//!
//! Security rationale:
//! This module must not derive consensus-sensitive fingerprints from debug
//! formatting, unstable serialization order, or runtime-dependent defaults.
//! All fingerprintable data is encoded through explicit canonical framing.

use core::fmt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

/// AOXC governance-controlled family namespace root.
///
/// Security rationale:
/// This value is a family-level namespace anchor rather than a specific network
/// discriminator. It must remain stable across AOXC network lines unless
/// governance explicitly authorizes a family reset.
pub const AOXC_FAMILY_ID: u32 = 2626;

/// Canonical AOXC network identifier prefix.
pub const AOXC_NETWORK_ID_PREFIX: &str = "aoxc";

/// Maximum allowed governance serial ordinal.
///
/// Example:
/// - `2626-001`
/// - `2626-999`
pub const MAX_NETWORK_SERIAL_ORDINAL: u16 = 999;

/// Maximum allowed instance ordinal inside a network-class segment.
///
/// Numeric chain-id layout:
/// `FFFFCCNNNN`
pub const MAX_NETWORK_INSTANCE_ORDINAL: u32 = 9999;

/// Canonical genesis encoding version.
///
/// Governance note:
/// This value must be incremented whenever canonical fingerprint framing or
/// identity preimage layout changes in a backward-incompatible manner.
pub const GENESIS_ENCODING_VERSION: u8 = 1;

/// Canonical default genesis timestamp.
///
/// Production rationale:
/// A deterministic default avoids runtime-dependent fingerprint drift. Real
/// production genesis artifacts may override this explicitly.
pub const DEFAULT_GENESIS_TIMESTAMP_UNIX: u64 = 1_700_000_000;

/// Domain separator for identity fingerprint derivation.
const DOMAIN_CHAIN_IDENTITY_FINGERPRINT: &[u8] = b"AOXC/GENESIS/CHAIN_IDENTITY/V1";

/// Domain separator for full genesis fingerprint derivation.
const DOMAIN_GENESIS_STATE_FINGERPRINT: &[u8] = b"AOXC/GENESIS/STATE/V1";

/// Maximum permitted chain-name length.
const MAX_CHAIN_NAME_LEN: usize = 128;

/// Maximum permitted protocol-version length.
const MAX_PROTOCOL_VERSION_LEN: usize = 64;

/// Maximum permitted genesis-notes length.
const MAX_GENESIS_NOTES_LEN: usize = 4096;

/// Maximum permitted endpoint length.
const MAX_ENDPOINT_LEN: usize = 512;

/// Maximum permitted identifier length for validator, account, node, and seal ids.
const MAX_IDENTIFIER_LEN: usize = 128;

/// Maximum permitted algorithm-name length.
const MAX_ALGORITHM_NAME_LEN: usize = 64;

/// Canonical network class.
///
/// The class code is embedded into the numeric `chain_id`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
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
    /// `FFFFCCNNNN`
    #[must_use]
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
    #[must_use]
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

    /// Returns whether the network class is public-facing.
    #[must_use]
    pub const fn is_public(self) -> bool {
        matches!(self, Self::PublicMainnet | Self::PublicTestnet)
    }
}

impl fmt::Display for NetworkClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.slug())
    }
}

/// Governance-facing chain identity model.
///
/// Security rationale:
/// This structure intentionally carries both human-readable and
/// machine-readable identifiers. These fields should become immutable once
/// genesis is finalized for a given network instance.
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
    pub chain_name: String,

    /// Canonical network class.
    pub network_class: NetworkClass,
}

impl ChainIdentity {
    /// Constructs a canonical chain identity from explicit policy inputs.
    ///
    /// This constructor deterministically derives:
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

        let chain_name = chain_name.into();
        validate_chain_name(&chain_name)?;

        let network_serial = build_network_serial(family_id, governance_serial_ordinal);
        let chain_id = build_chain_id(family_id, network_class, class_instance_ordinal)?;
        let network_id = build_network_id(network_class, &network_serial);

        let identity = Self {
            family_id,
            chain_id,
            network_serial,
            network_id,
            chain_name,
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

        validate_chain_name(&self.chain_name)?;
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

    /// Returns a deterministic fingerprint for the identity surface.
    ///
    /// This is not a signature and must not be treated as such.
    #[must_use]
    pub fn fingerprint_hex(&self) -> String {
        let mut enc = CanonicalEncoder::new(DOMAIN_CHAIN_IDENTITY_FINGERPRINT);
        self.encode_canonical(&mut enc);
        hex::encode(enc.finish())
    }

    fn encode_canonical(&self, enc: &mut CanonicalEncoder) {
        enc.u8(GENESIS_ENCODING_VERSION);
        enc.u32(self.family_id);
        enc.u64(self.chain_id);
        enc.str(&self.network_serial);
        enc.str(&self.network_id);
        enc.str(&self.chain_name);
        enc.str(self.network_class.slug());
    }
}

/// Genesis configuration root.
///
/// Security rationale:
/// This structure is intended to remain fingerprintable under a canonical
/// encoding. No consensus-sensitive fingerprint may depend on debug formatting
/// or unstable serialization behavior.
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
    /// Constructs a validated genesis configuration.
    pub fn new(
        identity: ChainIdentity,
        block_time: u64,
        validators: Vec<Validator>,
        accounts: Vec<GenesisAccount>,
        treasury: u128,
        settlement_link: SettlementLink,
        genesis_seal: AOXCANDSeal,
    ) -> Result<Self, GenesisConfigError> {
        let network_class = identity.network_class;

        let cfg = Self {
            identity,
            block_time,
            validators,
            accounts,
            treasury,
            settlement_link,
            genesis_seal,
            genesis_time_unix: default_genesis_timestamp(),
            protocol_version: default_protocol_version(),
            quantum_policy: QuantumPolicy::for_network_class(network_class),
            boot_nodes: default_boot_nodes(network_class),
            genesis_notes: default_genesis_notes(network_class).to_string(),
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

        if self.protocol_version.trim().is_empty() {
            return Err(GenesisConfigError::InvalidProtocolVersion);
        }

        validate_protocol_version(&self.protocol_version)?;
        validate_genesis_notes(&self.genesis_notes)?;

        self.quantum_policy
            .validate_for_network_class(self.identity.network_class)?;

        self.settlement_link.validate()?;
        self.genesis_seal.validate()?;

        if self.boot_nodes.is_empty() {
            return Err(GenesisConfigError::EmptyBootNodes);
        }

        validate_unique_validator_ids(&self.validators)?;
        validate_unique_account_addresses(&self.accounts)?;
        validate_unique_boot_node_ids(&self.boot_nodes)?;

        for validator in &self.validators {
            validator.validate()?;
        }

        for account in &self.accounts {
            account.validate()?;
        }

        for node in &self.boot_nodes {
            node.validate()?;
        }

        Ok(())
    }

    /// Computes a deterministic state fingerprint for the genesis configuration.
    ///
    /// Security properties:
    /// - identity material is included,
    /// - dynamic collections are canonically encoded in explicit order,
    /// - debug formatting is never used,
    /// - field framing is explicit and versioned.
    pub fn try_state_hash(&self) -> Result<[u8; 32], GenesisConfigError> {
        self.validate()?;

        let mut enc = CanonicalEncoder::new(DOMAIN_GENESIS_STATE_FINGERPRINT);
        enc.u8(GENESIS_ENCODING_VERSION);

        self.identity.encode_canonical(&mut enc);

        enc.u64(self.block_time);
        enc.u128(self.treasury);
        enc.u64(self.genesis_time_unix);
        enc.str(&self.protocol_version);
        enc.str(&self.genesis_notes);

        enc.usize(self.validators.len())?;
        for validator in &self.validators {
            validator.encode_canonical(&mut enc);
        }

        enc.usize(self.accounts.len())?;
        for account in &self.accounts {
            account.encode_canonical(&mut enc);
        }

        self.settlement_link.encode_canonical(&mut enc);
        self.genesis_seal.encode_canonical(&mut enc);
        self.quantum_policy.encode_canonical(&mut enc);

        enc.usize(self.boot_nodes.len())?;
        for node in &self.boot_nodes {
            node.encode_canonical(&mut enc);
        }

        Ok(enc.finish())
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

fn validate_chain_name(chain_name: &str) -> Result<(), GenesisConfigError> {
    let trimmed = chain_name.trim();

    if trimmed.is_empty() {
        return Err(GenesisConfigError::EmptyChainName);
    }

    if trimmed.len() > MAX_CHAIN_NAME_LEN {
        return Err(GenesisConfigError::InvalidChainNameLength {
            length: trimmed.len(),
        });
    }

    Ok(())
}

fn validate_protocol_version(version: &str) -> Result<(), GenesisConfigError> {
    let trimmed = version.trim();

    if trimmed.is_empty() {
        return Err(GenesisConfigError::InvalidProtocolVersion);
    }

    if trimmed.len() > MAX_PROTOCOL_VERSION_LEN {
        return Err(GenesisConfigError::ProtocolVersionTooLong {
            length: trimmed.len(),
        });
    }

    Ok(())
}

fn validate_genesis_notes(notes: &str) -> Result<(), GenesisConfigError> {
    if notes.len() > MAX_GENESIS_NOTES_LEN {
        return Err(GenesisConfigError::GenesisNotesTooLong {
            length: notes.len(),
        });
    }

    Ok(())
}

/// Returns the expected `FFFFCC` prefix segment of the numeric `chain_id`.
fn expected_chain_id_prefix(family_id: u32, network_class: NetworkClass) -> u64 {
    (u64::from(family_id) * 100) + u64::from(network_class.class_code())
}

fn validate_unique_validator_ids(validators: &[Validator]) -> Result<(), GenesisConfigError> {
    let mut seen = HashSet::with_capacity(validators.len());

    for validator in validators {
        if !seen.insert(validator.id.as_str()) {
            return Err(GenesisConfigError::DuplicateValidatorId {
                id: validator.id.clone(),
            });
        }
    }

    Ok(())
}

fn validate_unique_account_addresses(
    accounts: &[GenesisAccount],
) -> Result<(), GenesisConfigError> {
    let mut seen = HashSet::with_capacity(accounts.len());

    for account in accounts {
        if !seen.insert(account.address.as_str()) {
            return Err(GenesisConfigError::DuplicateAccountAddress {
                address: account.address.clone(),
            });
        }
    }

    Ok(())
}

fn validate_unique_boot_node_ids(nodes: &[BootNode]) -> Result<(), GenesisConfigError> {
    let mut seen = HashSet::with_capacity(nodes.len());

    for node in nodes {
        if !seen.insert(node.node_id.as_str()) {
            return Err(GenesisConfigError::DuplicateBootNodeId {
                node_id: node.node_id.clone(),
            });
        }
    }

    Ok(())
}

/// Genesis configuration validation error.
///
/// Audit rationale:
/// Errors are intentionally explicit and structured to support operator
/// diagnosis, audit evidence, and future reporting surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenesisConfigError {
    InvalidFamilyId,
    EmptyChainName,
    InvalidChainNameLength {
        length: usize,
    },
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
    ProtocolVersionTooLong {
        length: usize,
    },
    EmptyBootNodes,
    InvalidBootNode {
        node_id: String,
    },
    DuplicateBootNodeId {
        node_id: String,
    },
    DuplicateValidatorId {
        id: String,
    },
    DuplicateAccountAddress {
        address: String,
    },
    InvalidQuantumPolicy,
    InvalidQuantumAlgorithmName {
        value: String,
    },
    WeakQuantumPolicy {
        reason: &'static str,
    },
    InvalidSettlementLink,
    InvalidGenesisSeal,
    GenesisNotesTooLong {
        length: usize,
    },
    CanonicalEncodingLengthOverflow,
}

impl fmt::Display for GenesisConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFamilyId => {
                f.write_str("genesis validation failed: family_id must be non-zero")
            }
            Self::EmptyChainName => {
                f.write_str("genesis validation failed: chain_name must not be empty")
            }
            Self::InvalidChainNameLength { length } => write!(
                f,
                "genesis validation failed: chain_name length `{length}` exceeds policy bounds"
            ),
            Self::InvalidBlockTime => {
                f.write_str("genesis validation failed: block_time must be non-zero")
            }
            Self::EmptyValidators => {
                f.write_str("genesis validation failed: validator set must not be empty")
            }
            Self::EmptyAccounts => {
                f.write_str("genesis validation failed: account set must not be empty")
            }
            Self::MalformedNetworkSerial => {
                f.write_str("genesis validation failed: network_serial format is invalid")
            }
            Self::InvalidNetworkSerialOrdinal { value } => write!(
                f,
                "genesis validation failed: network serial ordinal `{value}` is outside policy bounds"
            ),
            Self::InvalidClassInstanceOrdinal { value } => write!(
                f,
                "genesis validation failed: class instance ordinal `{value}` is outside policy bounds"
            ),
            Self::InvalidDerivedChainIdOrdinal { value } => write!(
                f,
                "genesis validation failed: derived chain_id ordinal `{value}` is invalid"
            ),
            Self::NetworkSerialFamilyMismatch { expected, actual } => write!(
                f,
                "genesis validation failed: network_serial family mismatch; expected `{expected}`, got `{actual}`"
            ),
            Self::NetworkIdMismatch { expected, actual } => write!(
                f,
                "genesis validation failed: network_id mismatch; expected `{expected}`, got `{actual}`"
            ),
            Self::ChainIdPrefixMismatch {
                expected_prefix,
                actual_prefix,
            } => write!(
                f,
                "genesis validation failed: chain_id prefix mismatch; expected `{expected_prefix}`, got `{actual_prefix}`"
            ),
            Self::InvalidProtocolVersion => {
                f.write_str("genesis validation failed: protocol_version must not be empty")
            }
            Self::ProtocolVersionTooLong { length } => write!(
                f,
                "genesis validation failed: protocol_version length `{length}` exceeds policy bounds"
            ),
            Self::EmptyBootNodes => {
                f.write_str("genesis validation failed: boot_nodes must not be empty")
            }
            Self::InvalidBootNode { node_id } => write!(
                f,
                "genesis validation failed: boot node `{node_id}` is invalid"
            ),
            Self::DuplicateBootNodeId { node_id } => write!(
                f,
                "genesis validation failed: duplicate boot node id `{node_id}` detected"
            ),
            Self::DuplicateValidatorId { id } => write!(
                f,
                "genesis validation failed: duplicate validator id `{id}` detected"
            ),
            Self::DuplicateAccountAddress { address } => write!(
                f,
                "genesis validation failed: duplicate account address `{address}` detected"
            ),
            Self::InvalidQuantumPolicy => {
                f.write_str("genesis validation failed: quantum policy is invalid")
            }
            Self::InvalidQuantumAlgorithmName { value } => write!(
                f,
                "genesis validation failed: quantum algorithm name `{value}` is invalid"
            ),
            Self::WeakQuantumPolicy { reason } => write!(
                f,
                "genesis validation failed: quantum policy is too weak; {reason}"
            ),
            Self::InvalidSettlementLink => {
                f.write_str("genesis validation failed: settlement link is invalid")
            }
            Self::InvalidGenesisSeal => {
                f.write_str("genesis validation failed: genesis seal is invalid")
            }
            Self::GenesisNotesTooLong { length } => write!(
                f,
                "genesis validation failed: genesis notes length `{length}` exceeds policy bounds"
            ),
            Self::CanonicalEncodingLengthOverflow => {
                f.write_str("genesis validation failed: canonical encoding length overflow")
            }
        }
    }
}

impl std::error::Error for GenesisConfigError {}

/// Validator descriptor.
///
/// Integration note:
/// This placeholder-compatible type is intentionally strict enough for
/// production-oriented genesis hardening while remaining lightweight.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Validator {
    pub id: String,
}

impl Validator {
    fn validate(&self) -> Result<(), GenesisConfigError> {
        validate_identifier(&self.id).map_err(|_| GenesisConfigError::DuplicateValidatorId {
            id: self.id.clone(),
        })
    }

    fn encode_canonical(&self, enc: &mut CanonicalEncoder) {
        enc.str(&self.id);
    }
}

/// Genesis account descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenesisAccount {
    pub address: String,
    pub balance: u128,
}

impl GenesisAccount {
    fn validate(&self) -> Result<(), GenesisConfigError> {
        validate_identifier(&self.address).map_err(|_| {
            GenesisConfigError::DuplicateAccountAddress {
                address: self.address.clone(),
            }
        })
    }

    fn encode_canonical(&self, enc: &mut CanonicalEncoder) {
        enc.str(&self.address);
        enc.u128(self.balance);
    }
}

/// Settlement anchoring descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementLink {
    pub endpoint: String,
}

impl SettlementLink {
    fn validate(&self) -> Result<(), GenesisConfigError> {
        validate_endpoint(&self.endpoint).map_err(|_| GenesisConfigError::InvalidSettlementLink)
    }

    fn encode_canonical(&self, enc: &mut CanonicalEncoder) {
        enc.str(&self.endpoint);
    }
}

/// Genesis seal descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AOXCANDSeal {
    pub seal_id: String,
}

impl AOXCANDSeal {
    fn validate(&self) -> Result<(), GenesisConfigError> {
        validate_identifier(&self.seal_id).map_err(|_| GenesisConfigError::InvalidGenesisSeal)
    }

    fn encode_canonical(&self, enc: &mut CanonicalEncoder) {
        enc.str(&self.seal_id);
    }
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
        validate_identifier(&self.node_id).map_err(|_| GenesisConfigError::InvalidBootNode {
            node_id: self.node_id.clone(),
        })?;

        validate_identifier(&self.role).map_err(|_| GenesisConfigError::InvalidBootNode {
            node_id: self.node_id.clone(),
        })?;

        validate_endpoint(&self.endpoint).map_err(|_| GenesisConfigError::InvalidBootNode {
            node_id: self.node_id.clone(),
        })?;

        Ok(())
    }

    fn encode_canonical(&self, enc: &mut CanonicalEncoder) {
        enc.str(&self.node_id);
        enc.str(&self.endpoint);
        enc.str(&self.role);
    }
}

/// Quantum security policy profile for genesis.
///
/// Security rationale:
/// This is a policy-bearing object, not a descriptive note. Validation must
/// reject structurally invalid or cryptographically weak profiles for the
/// target network class.
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

    fn validate_for_network_class(
        &self,
        network_class: NetworkClass,
    ) -> Result<(), GenesisConfigError> {
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

        for alg in &self.pq_signature_schemes {
            validate_algorithm_name(alg)?;
        }

        for alg in &self.classical_signature_schemes {
            validate_algorithm_name(alg)?;
        }

        validate_algorithm_name(&self.handshake_kem)?;
        validate_algorithm_name(&self.state_hash)?;
        validate_algorithm_name(&self.commitment_hash)?;

        if network_class == NetworkClass::PublicMainnet {
            if self.handshake_kem != "ML-KEM-1024" {
                return Err(GenesisConfigError::WeakQuantumPolicy {
                    reason: "public mainnet requires ML-KEM-1024 for handshake policy",
                });
            }

            if self.min_signature_threshold < 3 {
                return Err(GenesisConfigError::WeakQuantumPolicy {
                    reason: "public mainnet requires a minimum signature threshold of 3",
                });
            }

            if !self
                .pq_signature_schemes
                .iter()
                .any(|alg| alg == "ML-DSA-87")
            {
                return Err(GenesisConfigError::WeakQuantumPolicy {
                    reason: "public mainnet requires ML-DSA-87 in the PQ signature policy",
                });
            }
        }

        if self.state_hash != "SHA3-256" {
            return Err(GenesisConfigError::WeakQuantumPolicy {
                reason: "state hash policy must use SHA3-256",
            });
        }

        if self.commitment_hash != "BLAKE3" {
            return Err(GenesisConfigError::WeakQuantumPolicy {
                reason: "commitment hash policy must use BLAKE3",
            });
        }

        Ok(())
    }

    fn encode_canonical(&self, enc: &mut CanonicalEncoder) {
        enc.strs(&self.pq_signature_schemes).ok();
        enc.strs(&self.classical_signature_schemes).ok();
        enc.str(&self.handshake_kem);
        enc.str(&self.state_hash);
        enc.str(&self.commitment_hash);
        enc.u16(self.min_signature_threshold);
        enc.u64(self.rotation_epoch_blocks);
    }
}

fn validate_identifier(value: &str) -> Result<(), ()> {
    let trimmed = value.trim();

    if trimmed.is_empty() || trimmed.len() > MAX_IDENTIFIER_LEN {
        return Err(());
    }

    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':' || c == '.')
    {
        return Err(());
    }

    Ok(())
}

fn validate_endpoint(value: &str) -> Result<(), ()> {
    let trimmed = value.trim();

    if trimmed.is_empty() || trimmed.len() > MAX_ENDPOINT_LEN {
        return Err(());
    }

    if !trimmed.contains("://") {
        return Err(());
    }

    Ok(())
}

fn validate_algorithm_name(value: &str) -> Result<(), GenesisConfigError> {
    let trimmed = value.trim();

    if trimmed.is_empty()
        || trimmed.len() > MAX_ALGORITHM_NAME_LEN
        || !trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(GenesisConfigError::InvalidQuantumAlgorithmName {
            value: value.to_string(),
        });
    }

    Ok(())
}

fn default_genesis_timestamp() -> u64 {
    DEFAULT_GENESIS_TIMESTAMP_UNIX
}

fn default_protocol_version() -> String {
    "aoxc-genesis/v3".to_string()
}

fn default_genesis_notes(network_class: NetworkClass) -> &'static str {
    match network_class {
        NetworkClass::PublicMainnet => {
            "AOXC mainnet genesis: production consensus, strict post-quantum threshold policy, audited settlement profile."
        }
        NetworkClass::PublicTestnet => {
            "AOXC testnet genesis: interoperability rehearsal, PQ migration validation, accelerated validator rotation."
        }
        NetworkClass::Devnet => {
            "AOXC devnet genesis: rapid experimentation profile with bounded cryptographic downgrade allowances."
        }
        NetworkClass::Validation => {
            "AOXC validation genesis: pre-production hardening and release-candidate verification."
        }
        NetworkClass::SovereignPrivate => "AOXC sovereign-private genesis profile.",
        NetworkClass::Consortium => "AOXC consortium genesis profile.",
        NetworkClass::RegulatedPrivate => "AOXC regulated-private genesis profile.",
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

/// Canonical encoder for deterministic genesis fingerprints.
///
/// Security rationale:
/// This encoder provides explicit field framing and stable byte ordering.
/// Consensus-sensitive fingerprints must not depend on debug rendering,
/// map ordering, or serializer implementation details.
struct CanonicalEncoder {
    hasher: Sha256,
}

impl CanonicalEncoder {
    fn new(domain: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(domain);
        hasher.update([0x00]);
        Self { hasher }
    }

    fn u8(&mut self, value: u8) {
        self.hasher.update([value]);
    }

    fn u16(&mut self, value: u16) {
        self.hasher.update(value.to_le_bytes());
    }

    fn u32(&mut self, value: u32) {
        self.hasher.update(value.to_le_bytes());
    }

    fn u64(&mut self, value: u64) {
        self.hasher.update(value.to_le_bytes());
    }

    fn u128(&mut self, value: u128) {
        self.hasher.update(value.to_le_bytes());
    }

    fn usize(&mut self, value: usize) -> Result<(), GenesisConfigError> {
        let casted = u64::try_from(value)
            .map_err(|_| GenesisConfigError::CanonicalEncodingLengthOverflow)?;
        self.u64(casted);
        Ok(())
    }

    fn str(&mut self, value: &str) {
        let len = u64::try_from(value.len()).unwrap_or(0);
        self.hasher.update(len.to_le_bytes());
        self.hasher.update(value.as_bytes());
    }

    fn strs(&mut self, values: &[String]) -> Result<(), GenesisConfigError> {
        self.usize(values.len())?;
        for value in values {
            self.str(value);
        }
        Ok(())
    }

    fn finish(self) -> [u8; 32] {
        let digest = self.hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&digest);
        out
    }
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
    fn rejects_duplicate_validator_ids() {
        let identity =
            ChainIdentity::new(AOXC_FAMILY_ID, NetworkClass::Devnet, 3, 1, "AOXC Kivilcim")
                .unwrap();

        let err = GenesisConfig::new(
            identity,
            3000,
            vec![
                Validator { id: "val-1".into() },
                Validator { id: "val-1".into() },
            ],
            vec![GenesisAccount {
                address: "acct-1".into(),
                balance: 10,
            }],
            10,
            SettlementLink {
                endpoint: "aoxc://settlement/root".into(),
            },
            AOXCANDSeal {
                seal_id: "seal-1".into(),
            },
        )
        .unwrap_err();

        assert!(matches!(
            err,
            GenesisConfigError::DuplicateValidatorId { .. }
        ));
    }

    #[test]
    fn mainnet_quantum_policy_is_strict() {
        let policy = QuantumPolicy::for_network_class(NetworkClass::PublicMainnet);
        assert_eq!(policy.handshake_kem, "ML-KEM-1024");
        assert!(policy.min_signature_threshold >= 3);
        assert!(policy.pq_signature_schemes.iter().any(|v| v == "ML-DSA-87"));
    }

    #[test]
    fn testnet_and_devnet_receive_distinct_quantum_profiles() {
        let testnet = QuantumPolicy::for_network_class(NetworkClass::PublicTestnet);
        let devnet = QuantumPolicy::for_network_class(NetworkClass::Devnet);

        assert_ne!(testnet.handshake_kem, devnet.handshake_kem);
        assert!(testnet.rotation_epoch_blocks > devnet.rotation_epoch_blocks);
    }

    #[test]
    fn default_timestamp_is_deterministic() {
        assert_eq!(default_genesis_timestamp(), DEFAULT_GENESIS_TIMESTAMP_UNIX);
    }
}
