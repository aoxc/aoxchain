// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.
//
// Production-oriented native token ledger with replay-hardened transfer support.
// This implementation is designed to remain deterministic, auditable, and
// compatible with a hardened receipt primitive.
//
// Security objectives:
// - strict policy validation
// - bounded replay metadata validation
// - deterministic anti-replay commitment derivation
// - safe arithmetic discipline
// - receipt construction compatible with fail-closed receipt APIs
// - no dead code and no placeholder branches

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::asset::SupplyModel;
use crate::receipts::{Event, HASH_SIZE, Receipt, ReceiptError};

/// Native AOXC token symbol.
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

impl NativeTokenLedger {
    /// Constructs a new ledger from the supplied policy.
    pub fn new(policy: NativeTokenPolicy) -> Result<Self, NativeTokenError> {
        policy.validate()?;

        Ok(Self {
            policy,
            total_supply: 0,
            balances: HashMap::new(),
            latest_nonce: HashMap::new(),
            consumed_quantum_commitments: HashSet::new(),
        })
    }

    /// Constructs a new ledger using the canonical policy for the selected network.
    pub fn new_for_network(network: NativeTokenNetwork) -> Result<Self, NativeTokenError> {
        Self::new(NativeTokenPolicy::for_network(network))
    }

    /// Returns the balance for the requested address.
    #[must_use]
    pub fn balance_of(&self, address: &Address) -> u128 {
        self.balances.get(address).copied().unwrap_or(0)
    }

    /// Returns the last accepted sender nonce, when present.
    #[must_use]
    pub fn latest_nonce_of(&self, address: &Address) -> Option<u64> {
        self.latest_nonce.get(address).copied()
    }

    /// Returns whether the supplied quantum commitment was already consumed.
    #[must_use]
    pub fn has_consumed_quantum_commitment(
        &self,
        digest: &[u8; NATIVE_TOKEN_COMMITMENT_SIZE],
    ) -> bool {
        self.consumed_quantum_commitments.contains(digest)
    }

    /// Mints native tokens into the supplied destination account.
    pub fn mint(&mut self, to: Address, amount: u128) -> Result<(), NativeTokenError> {
        self.policy.validate()?;

        if !self.policy.allows_mint() {
            return Err(NativeTokenError::MintDisabledPolicy);
        }

        self.policy.validate_mint_amount(amount)?;

        let updated_supply = self
            .total_supply
            .checked_add(amount)
            .ok_or(NativeTokenError::SupplyOverflow)?;

        if updated_supply > self.policy.quantum_policy.max_total_supply {
            return Err(NativeTokenError::SupplyOverflow);
        }

        let current_balance = self.balance_of(&to);
        let updated_balance = current_balance
            .checked_add(amount)
            .ok_or(NativeTokenError::BalanceOverflow)?;

        self.total_supply = updated_supply;
        self.balances.insert(to, updated_balance);

        Ok(())
    }

    /// Transfers native tokens between two accounts.
    pub fn transfer(
        &mut self,
        from: Address,
        to: Address,
        amount: u128,
    ) -> Result<(), NativeTokenError> {
        self.policy.validate()?;
        self.policy.validate_transfer_amount(amount)?;

        let current_from_balance = self.balance_of(&from);
        if current_from_balance < amount {
            return Err(NativeTokenError::InsufficientBalance);
        }

        let remaining_from_balance = current_from_balance - amount;
        if remaining_from_balance == 0 {
            self.balances.remove(&from);
        } else {
            self.balances.insert(from, remaining_from_balance);
        }

        let current_to_balance = self.balance_of(&to);
        let updated_to_balance = current_to_balance
            .checked_add(amount)
            .ok_or(NativeTokenError::BalanceOverflow)?;
        self.balances.insert(to, updated_to_balance);

        Ok(())
    }

    /// Executes a replay-hardened transfer with explicit quantum-proof binding.
    ///
    /// Replay protection currently includes:
    /// - strict sender nonce monotonicity,
    /// - non-empty proof tag validation,
    /// - proof-tag size enforcement,
    /// - full commitment digest tracking under the configured replay domain.
    pub fn transfer_quantum(
        &mut self,
        from: Address,
        to: Address,
        amount: u128,
        nonce: u64,
        proof_tag: &[u8],
    ) -> Result<(), NativeTokenError> {
        self.policy.validate()?;
        self.policy.validate_transfer_amount(amount)?;
        self.policy.validate_proof_tag(proof_tag)?;

        match self.latest_nonce.get(&from).copied() {
            Some(last_nonce) if nonce < last_nonce => {
                return Err(NativeTokenError::NonceRegression);
            }
            Some(last_nonce) if nonce == last_nonce => {
                return Err(NativeTokenError::ReplayDetected);
            }
            _ => {}
        }

        let commitment = self.quantum_transfer_digest(from, to, amount, nonce, proof_tag);

        if self
            .consumed_quantum_commitments
            .contains(&commitment.digest)
        {
            return Err(NativeTokenError::ReplayDetected);
        }

        self.transfer(from, to, amount)?;

        self.latest_nonce.insert(from, nonce);
        self.consumed_quantum_commitments.insert(commitment.digest);

        Ok(())
    }

    /// Computes the canonical quantum transfer digest under the active policy domain.
    #[must_use]
    pub fn quantum_transfer_digest(
        &self,
        from: Address,
        to: Address,
        amount: u128,
        nonce: u64,
        proof_tag: &[u8],
    ) -> NativeQuantumTransferDigestV1 {
        NativeQuantumTransferDigestV1 {
            version: NATIVE_TOKEN_QUANTUM_EVENT_VERSION,
            digest: compute_quantum_transfer_digest(
                &self.policy.quantum_policy.anti_replay_domain,
                from,
                to,
                amount,
                nonce,
                proof_tag,
            ),
        }
    }

    /// Builds a receipt for a successful mint operation.
    ///
    /// This method is compatible with the hardened receipt API and therefore
    /// returns `Result` instead of constructing invalid states implicitly.
    pub fn mint_receipt(
        &self,
        tx_hash: [u8; HASH_SIZE],
        to: Address,
        amount: u128,
        energy_used: u64,
    ) -> Result<Receipt, ReceiptError> {
        let mut receipt = Receipt::success(tx_hash, energy_used)?;

        let event = Event::new(
            EVENT_NATIVE_MINT,
            encode_transfer_like_event([0u8; 32], to, amount),
        )?;
        receipt.push_event(event)?;

        Ok(receipt)
    }

    /// Builds a receipt for a successful classic native transfer operation.
    pub fn transfer_receipt(
        &self,
        tx_hash: [u8; HASH_SIZE],
        from: Address,
        to: Address,
        amount: u128,
        energy_used: u64,
    ) -> Result<Receipt, ReceiptError> {
        let mut receipt = Receipt::success(tx_hash, energy_used)?;

        let event = Event::new(
            EVENT_NATIVE_TRANSFER,
            encode_transfer_like_event(from, to, amount),
        )?;
        receipt.push_event(event)?;

        Ok(receipt)
    }

    /// Builds a receipt for a failed native token operation.
    pub fn error_receipt(
        &self,
        tx_hash: [u8; HASH_SIZE],
        energy_used: u64,
        error: NativeTokenError,
    ) -> Result<Receipt, ReceiptError> {
        Receipt::failure(tx_hash, energy_used, error.receipt_error_code())
    }

    /// Builds a receipt for a successful replay-hardened quantum transfer.
    pub fn transfer_quantum_receipt(
        &self,
        tx_hash: [u8; HASH_SIZE],
        from: Address,
        to: Address,
        amount: u128,
        nonce: u64,
        proof_tag: &[u8],
        energy_used: u64,
    ) -> Result<Receipt, ReceiptError> {
        let mut receipt = Receipt::success(tx_hash, energy_used)?;

        let event = Event::new(
            EVENT_NATIVE_TRANSFER_QUANTUM_V1,
            encode_quantum_transfer_event_v1(
                &self.policy.quantum_policy.anti_replay_domain,
                from,
                to,
                amount,
                nonce,
                proof_tag,
            ),
        )?;
        receipt.push_event(event)?;

        Ok(receipt)
    }
}

/// Encodes a classic transfer-like event payload.
///
/// Layout:
/// - from: 32 bytes
/// - to: 32 bytes
/// - amount: 16 bytes little-endian
#[must_use]
pub fn encode_transfer_like_event(from: Address, to: Address, amount: u128) -> Vec<u8> {
    let mut payload = Vec::with_capacity(80);
    payload.extend_from_slice(&from);
    payload.extend_from_slice(&to);
    payload.extend_from_slice(&amount.to_le_bytes());
    payload
}

/// Encodes a versioned quantum transfer event payload.
///
/// Layout:
/// - version: 1 byte
/// - from: 32 bytes
/// - to: 32 bytes
/// - amount: 16 bytes little-endian
/// - nonce: 8 bytes little-endian
/// - digest: 32 bytes
#[must_use]
pub fn encode_quantum_transfer_event_v1(
    domain: &str,
    from: Address,
    to: Address,
    amount: u128,
    nonce: u64,
    proof_tag: &[u8],
) -> Vec<u8> {
    let digest = compute_quantum_transfer_digest(domain, from, to, amount, nonce, proof_tag);

    let mut payload = Vec::with_capacity(89);
    payload.push(NATIVE_TOKEN_QUANTUM_EVENT_VERSION);
    payload.extend_from_slice(&from);
    payload.extend_from_slice(&to);
    payload.extend_from_slice(&amount.to_le_bytes());
    payload.extend_from_slice(&nonce.to_le_bytes());
    payload.extend_from_slice(&digest);
    payload
}

/// Backward-compatible alias for callers still using the previous helper name.
///
/// The implementation emits the versioned V1 quantum event layout.
#[must_use]
pub fn encode_quantum_transfer_event(
    domain: &str,
    from: Address,
    to: Address,
    amount: u128,
    nonce: u64,
    proof_tag: &[u8],
) -> Vec<u8> {
    encode_quantum_transfer_event_v1(domain, from, to, amount, nonce, proof_tag)
}

/// Computes the canonical replay-binding digest for a quantum transfer.
#[must_use]
pub fn compute_quantum_transfer_digest(
    domain: &str,
    from: Address,
    to: Address,
    amount: u128,
    nonce: u64,
    proof_tag: &[u8],
) -> [u8; NATIVE_TOKEN_COMMITMENT_SIZE] {
    let mut hasher = Sha3_256::new();
    hasher.update(domain.as_bytes());
    hasher.update([0x00]);
    hasher.update(from);
    hasher.update([0x00]);
    hasher.update(to);
    hasher.update([0x00]);
    hasher.update(amount.to_le_bytes());
    hasher.update([0x00]);
    hasher.update(nonce.to_le_bytes());
    hasher.update([0x00]);
    hasher.update(proof_tag);

    let digest = hasher.finalize();

    let mut out = [0u8; NATIVE_TOKEN_COMMITMENT_SIZE];
    out.copy_from_slice(&digest[..NATIVE_TOKEN_COMMITMENT_SIZE]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    fn addr(byte: u8) -> Address {
        [byte; 32]
    }

    #[test]
    fn policy_profiles_validate_successfully() {
        assert!(
            NativeTokenPolicy::for_network(NativeTokenNetwork::Mainnet)
                .validate()
                .is_ok()
        );
        assert!(
            NativeTokenPolicy::for_network(NativeTokenNetwork::Testnet)
                .validate()
                .is_ok()
        );
        assert!(
            NativeTokenPolicy::for_network(NativeTokenNetwork::Devnet)
                .validate()
                .is_ok()
        );
    }

    #[test]
    fn new_ledger_rejects_invalid_policy() {
        let mut policy = NativeTokenPolicy::default();
        policy.version = 99;

        let err = NativeTokenLedger::new(policy).unwrap_err();
        assert_eq!(err, NativeTokenError::InvalidPolicy);
    }

    #[test]
    fn mint_updates_supply_and_balance() {
        let mut ledger = NativeTokenLedger::new_for_network(NativeTokenNetwork::Mainnet).unwrap();

        ledger.mint(addr(1), 100).unwrap();

        assert_eq!(ledger.total_supply, 100);
        assert_eq!(ledger.balance_of(&addr(1)), 100);
        assert_eq!(ledger.policy.symbol, NATIVE_TOKEN_SYMBOL);
    }

    #[test]
    fn transfer_moves_balance_without_changing_supply() {
        let mut ledger = NativeTokenLedger::new_for_network(NativeTokenNetwork::Mainnet).unwrap();
        ledger.mint(addr(1), 100).unwrap();

        ledger.transfer(addr(1), addr(2), 30).unwrap();

        assert_eq!(ledger.total_supply, 100);
        assert_eq!(ledger.balance_of(&addr(1)), 70);
        assert_eq!(ledger.balance_of(&addr(2)), 30);
    }

    #[test]
    fn transfer_fails_when_balance_is_insufficient() {
        let mut ledger = NativeTokenLedger::new_for_network(NativeTokenNetwork::Mainnet).unwrap();
        ledger.mint(addr(1), 10).unwrap();

        let err = ledger.transfer(addr(1), addr(2), 11).unwrap_err();
        assert_eq!(err, NativeTokenError::InsufficientBalance);
    }

    #[test]
    fn receipts_emit_expected_events_and_codes() {
        let ledger = NativeTokenLedger::new_for_network(NativeTokenNetwork::Mainnet).unwrap();

        let mint_receipt = ledger
            .mint_receipt([7; HASH_SIZE], addr(9), 42, 21)
            .unwrap();
        assert!(mint_receipt.success);
        assert_eq!(mint_receipt.events.len(), 1);
        assert_eq!(mint_receipt.events[0].event_type, EVENT_NATIVE_MINT);
        assert_eq!(mint_receipt.events[0].data.len(), 80);

        let error_receipt = ledger
            .error_receipt([8; HASH_SIZE], 17, NativeTokenError::InsufficientBalance)
            .unwrap();
        assert!(!error_receipt.success);
        assert_eq!(
            error_receipt.error_code,
            Some(ERROR_CODE_INSUFFICIENT_BALANCE)
        );
    }

    #[test]
    fn mint_is_rejected_when_supply_model_disables_mint() {
        let mut ledger = NativeTokenLedger::new(NativeTokenPolicy {
            supply_model: SupplyModel::FixedGenesis,
            ..NativeTokenPolicy::default()
        })
        .unwrap();

        let err = ledger.mint(addr(1), 10).unwrap_err();
        assert_eq!(err, NativeTokenError::MintDisabledPolicy);
    }

    #[test]
    fn network_profiles_are_distinct_and_quantum_domains_do_not_overlap() {
        let mainnet = NativeTokenPolicy::for_network(NativeTokenNetwork::Mainnet);
        let testnet = NativeTokenPolicy::for_network(NativeTokenNetwork::Testnet);
        let devnet = NativeTokenPolicy::for_network(NativeTokenNetwork::Devnet);

        assert_eq!(mainnet.decimals, 18);
        assert_eq!(testnet.decimals, 18);
        assert_eq!(devnet.decimals, 18);
        assert_ne!(
            mainnet.quantum_policy.anti_replay_domain,
            testnet.quantum_policy.anti_replay_domain
        );
        assert_ne!(
            testnet.quantum_policy.anti_replay_domain,
            devnet.quantum_policy.anti_replay_domain
        );
    }

    #[test]
    fn quantum_transfer_rejects_empty_proof_tag() {
        let mut ledger = NativeTokenLedger::new_for_network(NativeTokenNetwork::Mainnet).unwrap();
        ledger.mint(addr(1), 1_000).unwrap();

        let error = ledger
            .transfer_quantum(addr(1), addr(2), 100, 1, b"")
            .unwrap_err();

        assert_eq!(error, NativeTokenError::InvalidProofTag);
    }

    #[test]
    fn quantum_transfer_rejects_replay_and_nonce_regression() {
        let mut ledger = NativeTokenLedger::new_for_network(NativeTokenNetwork::Mainnet).unwrap();
        ledger.mint(addr(1), 1_000).unwrap();

        ledger
            .transfer_quantum(addr(1), addr(2), 100, 1, b"sig-proof")
            .unwrap();

        let replay_err = ledger
            .transfer_quantum(addr(1), addr(2), 100, 1, b"sig-proof")
            .unwrap_err();
        assert_eq!(replay_err, NativeTokenError::ReplayDetected);

        let regression_err = ledger
            .transfer_quantum(addr(1), addr(2), 100, 0, b"other-proof")
            .unwrap_err();
        assert_eq!(regression_err, NativeTokenError::NonceRegression);
    }

    #[test]
    fn quantum_transfer_rejects_duplicate_commitment_even_if_nonce_path_is_bypassed() {
        let mut ledger = NativeTokenLedger::new_for_network(NativeTokenNetwork::Mainnet).unwrap();
        ledger.mint(addr(1), 1_000).unwrap();

        let digest = ledger.quantum_transfer_digest(addr(1), addr(2), 100, 9, b"proof");
        ledger.consumed_quantum_commitments.insert(digest.digest);

        let error = ledger
            .transfer_quantum(addr(1), addr(2), 100, 9, b"proof")
            .unwrap_err();

        assert_eq!(error, NativeTokenError::ReplayDetected);
    }

    #[test]
    fn quantum_transfer_updates_nonce_and_commitment_store() {
        let mut ledger = NativeTokenLedger::new_for_network(NativeTokenNetwork::Mainnet).unwrap();
        ledger.mint(addr(1), 500).unwrap();

        let digest = ledger.quantum_transfer_digest(addr(1), addr(2), 50, 3, b"proof");

        ledger
            .transfer_quantum(addr(1), addr(2), 50, 3, b"proof")
            .unwrap();

        assert_eq!(ledger.latest_nonce_of(&addr(1)), Some(3));
        assert!(ledger.has_consumed_quantum_commitment(&digest.digest));
    }

    #[test]
    fn quantum_transfer_event_encoding_contains_expected_layout() {
        let from = addr(1);
        let to = addr(2);
        let amount = 77u128;
        let nonce = 9u64;
        let payload = encode_quantum_transfer_event_v1(
            "AOXC/NATIVE_TOKEN/TESTNET/V1",
            from,
            to,
            amount,
            nonce,
            b"proof",
        );

        let expected_len = 1
            + size_of::<Address>()
            + size_of::<Address>()
            + size_of::<u128>()
            + size_of::<u64>()
            + HASH_SIZE;
        assert_eq!(payload[0], NATIVE_TOKEN_QUANTUM_EVENT_VERSION);
        assert_eq!(payload.len(), expected_len);
        assert_eq!(&payload[1..33], &from);
        assert_eq!(&payload[33..65], &to);
        assert_eq!(&payload[65..81], &amount.to_le_bytes());
        assert_eq!(&payload[81..89], &nonce.to_le_bytes());
    }

    #[test]
    fn computed_quantum_transfer_digest_is_deterministic() {
        let a = compute_quantum_transfer_digest(
            "AOXC/NATIVE_TOKEN/MAINNET/V1",
            addr(1),
            addr(2),
            10,
            7,
            b"proof",
        );
        let b = compute_quantum_transfer_digest(
            "AOXC/NATIVE_TOKEN/MAINNET/V1",
            addr(1),
            addr(2),
            10,
            7,
            b"proof",
        );

        assert_eq!(a, b);
    }

    #[test]
    fn quantum_receipt_is_constructed_successfully() {
        let ledger = NativeTokenLedger::new_for_network(NativeTokenNetwork::Mainnet).unwrap();

        let receipt = ledger
            .transfer_quantum_receipt([9; HASH_SIZE], addr(1), addr(2), 55, 4, b"proof", 88)
            .unwrap();

        assert!(receipt.success);
        assert_eq!(receipt.events.len(), 1);
        assert_eq!(
            receipt.events[0].event_type,
            EVENT_NATIVE_TRANSFER_QUANTUM_V1
        );
        assert_eq!(
            receipt.events[0].data.len(),
            1 + size_of::<Address>()
                + size_of::<Address>()
                + size_of::<u128>()
                + size_of::<u64>()
                + HASH_SIZE
        );
    }
}
