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
    InvalidNodePolicy,
    DuplicateNodeRolePolicy {
        role: String,
    },
    MissingNodeRolePolicy {
        role: String,
    },
    WeakNodeRolePolicy {
        role: String,
        reason: &'static str,
    },
    InvalidSealLayerPolicy {
        layer_id: String,
    },
    DuplicateSealLayerPolicy {
        layer_id: String,
    },
    InvalidMultisigSigner {
        signer: String,
    },
    DuplicateMultisigSigner {
        signer: String,
    },
    InsufficientMultisigSignatures {
        role: String,
        required: u8,
        actual: u8,
    },
    InsufficientMultisigParticipants {
        role: String,
        required: u8,
        actual: u8,
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
            Self::InvalidNodePolicy => {
                f.write_str("genesis validation failed: node policy is invalid")
            }
            Self::DuplicateNodeRolePolicy { role } => write!(
                f,
                "genesis validation failed: duplicate node role policy `{role}` detected"
            ),
            Self::MissingNodeRolePolicy { role } => write!(
                f,
                "genesis validation failed: missing required node role policy `{role}`"
            ),
            Self::WeakNodeRolePolicy { role, reason } => write!(
                f,
                "genesis validation failed: node role policy `{role}` is too weak; {reason}"
            ),
            Self::InvalidSealLayerPolicy { layer_id } => write!(
                f,
                "genesis validation failed: seal layer policy `{layer_id}` is invalid"
            ),
            Self::DuplicateSealLayerPolicy { layer_id } => write!(
                f,
                "genesis validation failed: duplicate seal layer policy `{layer_id}` detected"
            ),
            Self::InvalidMultisigSigner { signer } => write!(
                f,
                "genesis validation failed: multisig signer `{signer}` is not eligible for the role"
            ),
            Self::DuplicateMultisigSigner { signer } => write!(
                f,
                "genesis validation failed: duplicate multisig signer `{signer}` detected"
            ),
            Self::InsufficientMultisigSignatures {
                role,
                required,
                actual,
            } => write!(
                f,
                "genesis validation failed: role `{role}` has insufficient signatures; required `{required}`, actual `{actual}`"
            ),
            Self::InsufficientMultisigParticipants {
                role,
                required,
                actual,
            } => write!(
                f,
                "genesis validation failed: role `{role}` has insufficient participants; required `{required}`, actual `{actual}`"
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
