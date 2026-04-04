pub const NATIVE_TOKEN_SYMBOL: &str = "AOXC";

/// Canonical policy schema version for the native token module.
pub const NATIVE_TOKEN_POLICY_VERSION: u8 = 1;

/// Canonical quantum transfer event payload version.
pub const NATIVE_TOKEN_QUANTUM_EVENT_VERSION: u8 = 1;

/// Canonical digest size used by replay commitments and receipt digests.
pub const NATIVE_TOKEN_COMMITMENT_SIZE: usize = 32;

/// Receipt event emitted for legacy/native token transfers.
pub const EVENT_NATIVE_TRANSFER: u16 = 0x1001;

/// Receipt event emitted for native token minting.
pub const EVENT_NATIVE_MINT: u16 = 0x1002;

/// Receipt event emitted for quantum-aware native token transfers.
///
/// This event type is distinct from `EVENT_NATIVE_TRANSFER` because the payload
/// carries replay-binding metadata and should remain explicitly versioned.
pub const EVENT_NATIVE_TRANSFER_QUANTUM_V1: u16 = 0x1003;

/// Error codes returned inside native token receipts.
pub const ERROR_CODE_SUPPLY_OVERFLOW: u16 = 0x2001;
pub const ERROR_CODE_BALANCE_OVERFLOW: u16 = 0x2002;
pub const ERROR_CODE_INSUFFICIENT_BALANCE: u16 = 0x2003;
pub const ERROR_CODE_MINT_DISABLED: u16 = 0x2004;
pub const ERROR_CODE_INVALID_AMOUNT: u16 = 0x2005;
pub const ERROR_CODE_TRANSFER_LIMIT_EXCEEDED: u16 = 0x2006;
pub const ERROR_CODE_NONCE_REGRESSION: u16 = 0x2007;
pub const ERROR_CODE_REPLAY_DETECTED: u16 = 0x2008;
pub const ERROR_CODE_INVALID_PROOF_TAG: u16 = 0x2009;
pub const ERROR_CODE_PROOF_TAG_TOO_LARGE: u16 = 0x200A;
pub const ERROR_CODE_INVALID_POLICY: u16 = 0x200B;

/// Canonical address type for the native token ledger.
pub type Address = [u8; 32];

/// Domain errors for the native token ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeTokenError {
    SupplyOverflow,
    BalanceOverflow,
    InsufficientBalance,
    MintDisabledPolicy,
    InvalidAmount,
    TransferLimitExceeded,
    NonceRegression,
    ReplayDetected,
    InvalidProofTag,
    ProofTagTooLarge,
    InvalidPolicy,
}

impl NativeTokenError {
    /// Returns a stable symbolic error code suitable for logs and telemetry.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::SupplyOverflow => "NATIVE_TOKEN_SUPPLY_OVERFLOW",
            Self::BalanceOverflow => "NATIVE_TOKEN_BALANCE_OVERFLOW",
            Self::InsufficientBalance => "NATIVE_TOKEN_INSUFFICIENT_BALANCE",
            Self::MintDisabledPolicy => "NATIVE_TOKEN_MINT_DISABLED",
            Self::InvalidAmount => "NATIVE_TOKEN_INVALID_AMOUNT",
            Self::TransferLimitExceeded => "NATIVE_TOKEN_TRANSFER_LIMIT_EXCEEDED",
            Self::NonceRegression => "NATIVE_TOKEN_NONCE_REGRESSION",
            Self::ReplayDetected => "NATIVE_TOKEN_REPLAY_DETECTED",
            Self::InvalidProofTag => "NATIVE_TOKEN_INVALID_PROOF_TAG",
            Self::ProofTagTooLarge => "NATIVE_TOKEN_PROOF_TAG_TOO_LARGE",
            Self::InvalidPolicy => "NATIVE_TOKEN_INVALID_POLICY",
        }
    }

    /// Returns the canonical receipt error code for this domain error.
    #[must_use]
    pub const fn receipt_error_code(self) -> u16 {
        match self {
            Self::SupplyOverflow => ERROR_CODE_SUPPLY_OVERFLOW,
            Self::BalanceOverflow => ERROR_CODE_BALANCE_OVERFLOW,
            Self::InsufficientBalance => ERROR_CODE_INSUFFICIENT_BALANCE,
            Self::MintDisabledPolicy => ERROR_CODE_MINT_DISABLED,
            Self::InvalidAmount => ERROR_CODE_INVALID_AMOUNT,
            Self::TransferLimitExceeded => ERROR_CODE_TRANSFER_LIMIT_EXCEEDED,
            Self::NonceRegression => ERROR_CODE_NONCE_REGRESSION,
            Self::ReplayDetected => ERROR_CODE_REPLAY_DETECTED,
            Self::InvalidProofTag => ERROR_CODE_INVALID_PROOF_TAG,
            Self::ProofTagTooLarge => ERROR_CODE_PROOF_TAG_TOO_LARGE,
            Self::InvalidPolicy => ERROR_CODE_INVALID_POLICY,
        }
    }
}

impl fmt::Display for NativeTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SupplyOverflow => write!(f, "native token supply overflow"),
            Self::BalanceOverflow => write!(f, "native token balance overflow"),
            Self::InsufficientBalance => write!(f, "insufficient native token balance"),
            Self::MintDisabledPolicy => write!(f, "native token mint disabled by supply policy"),
            Self::InvalidAmount => write!(f, "native token amount must be non-zero"),
            Self::TransferLimitExceeded => write!(f, "native token transfer exceeds policy limit"),
            Self::NonceRegression => {
                write!(f, "native token transfer nonce is not strictly increasing")
            }
            Self::ReplayDetected => write!(f, "native token transfer replay detected"),
            Self::InvalidProofTag => {
                write!(f, "native token proof tag must not be empty")
            }
            Self::ProofTagTooLarge => {
                write!(f, "native token proof tag exceeds policy limit")
            }
            Self::InvalidPolicy => {
                write!(f, "native token policy is internally invalid")
            }
        }
    }
}

impl std::error::Error for NativeTokenError {}

/// Canonical AOXC deployment profiles for native token policy.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "kebab-case")]
pub enum NativeTokenNetwork {
    #[default]
    Mainnet,
    Testnet,
    Devnet,
}

/// Post-quantum and anti-replay transfer policy.
///
/// Versioning rationale:
/// this structure is part of the public and persisted policy surface and
/// therefore benefits from an explicit schema generation strategy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativeTokenQuantumPolicyV1 {
    pub signature_suite: String,
    pub anti_replay_domain: String,
    pub max_transfer_amount: u128,
    pub max_total_supply: u128,
    pub max_proof_tag_len: u32,
}

/// Static metadata describing the canonical AOXC native token.
///
/// Versioning rationale:
/// this policy is a long-lived, externally meaningful configuration contract.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativeTokenPolicyV1 {
    pub version: u8,
    pub symbol: String,
    pub decimals: u8,
    pub supply_model: SupplyModel,
    pub network: NativeTokenNetwork,
    pub quantum_policy: NativeTokenQuantumPolicyV1,
}

/// Backward-compatible alias for the current canonical policy schema.
pub type NativeTokenPolicy = NativeTokenPolicyV1;

/// Backward-compatible alias for the current canonical quantum policy schema.
pub type NativeTokenQuantumPolicy = NativeTokenQuantumPolicyV1;

impl Default for NativeTokenPolicyV1 {
    fn default() -> Self {
        Self::for_network(NativeTokenNetwork::Mainnet)
    }
}

impl NativeTokenPolicyV1 {
    /// Returns the canonical policy for the requested AOXC deployment profile.
    #[must_use]
    pub fn for_network(network: NativeTokenNetwork) -> Self {
        let quantum_policy = match network {
            NativeTokenNetwork::Mainnet => NativeTokenQuantumPolicyV1 {
                signature_suite: "ML-DSA-87+Ed25519".to_string(),
                anti_replay_domain: "AOXC/NATIVE_TOKEN/MAINNET/V1".to_string(),
                max_transfer_amount: 10_000_000 * 10_u128.pow(18),
                max_total_supply: 10_000_000_000 * 10_u128.pow(18),
                max_proof_tag_len: 4096,
            },
            NativeTokenNetwork::Testnet => NativeTokenQuantumPolicyV1 {
                signature_suite: "ML-DSA-65+Ed25519".to_string(),
                anti_replay_domain: "AOXC/NATIVE_TOKEN/TESTNET/V1".to_string(),
                max_transfer_amount: 500_000_000 * 10_u128.pow(18),
                max_total_supply: 100_000_000_000 * 10_u128.pow(18),
                max_proof_tag_len: 4096,
            },
            NativeTokenNetwork::Devnet => NativeTokenQuantumPolicyV1 {
                signature_suite: "ML-DSA-44+Ed25519".to_string(),
                anti_replay_domain: "AOXC/NATIVE_TOKEN/DEVNET/V1".to_string(),
                max_transfer_amount: u128::MAX,
                max_total_supply: u128::MAX,
                max_proof_tag_len: 16_384,
            },
        };

        Self {
            version: NATIVE_TOKEN_POLICY_VERSION,
            symbol: NATIVE_TOKEN_SYMBOL.to_string(),
            decimals: 18,
            supply_model: SupplyModel::GovernanceAuthorizedEmission,
            network,
            quantum_policy,
        }
    }

    /// Returns whether the configured supply model allows minting.
    #[must_use]
    pub const fn allows_mint(&self) -> bool {
        matches!(
            self.supply_model,
            SupplyModel::GovernanceAuthorizedEmission
                | SupplyModel::ProgrammaticEmission
                | SupplyModel::TreasuryAuthorizedEmission
        )
    }

    /// Validates the policy as a self-consistent public contract.
    pub fn validate(&self) -> Result<(), NativeTokenError> {
        if self.version != NATIVE_TOKEN_POLICY_VERSION {
            return Err(NativeTokenError::InvalidPolicy);
        }

        if self.symbol.trim() != NATIVE_TOKEN_SYMBOL {
            return Err(NativeTokenError::InvalidPolicy);
        }

        if self.decimals != 18 {
            return Err(NativeTokenError::InvalidPolicy);
        }

        if self.quantum_policy.signature_suite.trim().is_empty()
            || self.quantum_policy.anti_replay_domain.trim().is_empty()
            || self.quantum_policy.max_transfer_amount == 0
            || self.quantum_policy.max_total_supply == 0
            || self.quantum_policy.max_transfer_amount > self.quantum_policy.max_total_supply
            || self.quantum_policy.max_proof_tag_len == 0
        {
            return Err(NativeTokenError::InvalidPolicy);
        }

        Ok(())
    }

    /// Validates a transfer amount against the active token policy.
    fn validate_transfer_amount(&self, amount: u128) -> Result<(), NativeTokenError> {
        if amount == 0 {
            return Err(NativeTokenError::InvalidAmount);
        }

        if amount > self.quantum_policy.max_transfer_amount {
            return Err(NativeTokenError::TransferLimitExceeded);
        }

        Ok(())
    }

    /// Validates a mint amount against the active token policy.
    ///
    /// Current policy:
    /// - non-zero amount,
    /// - cannot exceed the transfer upper bound in order to preserve a single
    ///   issuance safety ceiling for operator and governance flows.
    fn validate_mint_amount(&self, amount: u128) -> Result<(), NativeTokenError> {
        self.validate_transfer_amount(amount)
    }

    /// Validates a proof tag according to the active quantum policy.
    fn validate_proof_tag(&self, proof_tag: &[u8]) -> Result<(), NativeTokenError> {
        if proof_tag.is_empty() {
            return Err(NativeTokenError::InvalidProofTag);
        }

        if proof_tag.len() > self.quantum_policy.max_proof_tag_len as usize {
            return Err(NativeTokenError::ProofTagTooLarge);
        }

        Ok(())
    }
}

/// Versioned digest envelope for quantum transfer anti-replay binding.
///
/// This structure provides a stable, explicitly versioned representation of the
/// commitment used by the anti-replay logic and event payload surface.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativeQuantumTransferDigestV1 {
    pub version: u8,
    pub digest: [u8; NATIVE_TOKEN_COMMITMENT_SIZE],
}

/// Minimal in-memory native token ledger.
///
/// Security notes:
/// - this structure tracks last-seen sender nonces,
/// - it also tracks consumed quantum commitments to harden replay detection
///   beyond strict nonce monotonicity,
/// - it remains intentionally minimal and in-memory, leaving durable state
///   management to higher layers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct NativeTokenLedger {
    pub policy: NativeTokenPolicy,
    pub total_supply: u128,
    pub balances: HashMap<Address, u128>,
    pub latest_nonce: HashMap<Address, u64>,
    pub consumed_quantum_commitments: HashSet<[u8; NATIVE_TOKEN_COMMITMENT_SIZE]>,
}
