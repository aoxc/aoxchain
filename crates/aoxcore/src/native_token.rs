// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;

use crate::asset::SupplyModel;
use crate::receipts::{Event, HASH_SIZE, Receipt};

/// Native AOXC token symbol.
pub const NATIVE_TOKEN_SYMBOL: &str = "AOXC";

/// Receipt event emitted for native token transfers.
pub const EVENT_NATIVE_TRANSFER: u16 = 0x1001;

/// Receipt event emitted for native token minting.
pub const EVENT_NATIVE_MINT: u16 = 0x1002;

/// Error codes returned inside native token receipts.
pub const ERROR_CODE_SUPPLY_OVERFLOW: u16 = 0x2001;
pub const ERROR_CODE_BALANCE_OVERFLOW: u16 = 0x2002;
pub const ERROR_CODE_INSUFFICIENT_BALANCE: u16 = 0x2003;
pub const ERROR_CODE_MINT_DISABLED: u16 = 0x2004;
pub const ERROR_CODE_INVALID_AMOUNT: u16 = 0x2005;
pub const ERROR_CODE_TRANSFER_LIMIT_EXCEEDED: u16 = 0x2006;
pub const ERROR_CODE_NONCE_REGRESSION: u16 = 0x2007;
pub const ERROR_CODE_REPLAY_DETECTED: u16 = 0x2008;

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
}

impl std::fmt::Display for NativeTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
        }
    }
}

impl std::error::Error for NativeTokenError {}

/// Static metadata describing the canonical AOXC native token.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativeTokenPolicy {
    pub symbol: String,
    pub decimals: u8,
    pub supply_model: SupplyModel,
    pub network: NativeTokenNetwork,
    pub quantum_policy: NativeTokenQuantumPolicy,
}

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativeTokenQuantumPolicy {
    pub signature_suite: String,
    pub anti_replay_domain: String,
    pub max_transfer_amount: u128,
    pub max_total_supply: u128,
}

impl Default for NativeTokenPolicy {
    fn default() -> Self {
        Self::for_network(NativeTokenNetwork::Mainnet)
    }
}

impl NativeTokenPolicy {
    #[must_use]
    pub fn for_network(network: NativeTokenNetwork) -> Self {
        let quantum_policy = match network {
            NativeTokenNetwork::Mainnet => NativeTokenQuantumPolicy {
                signature_suite: "ML-DSA-87+Ed25519".to_string(),
                anti_replay_domain: "AOXC/NATIVE_TOKEN/MAINNET/V1".to_string(),
                max_transfer_amount: 10_000_000 * 10_u128.pow(18),
                max_total_supply: 10_000_000_000 * 10_u128.pow(18),
            },
            NativeTokenNetwork::Testnet => NativeTokenQuantumPolicy {
                signature_suite: "ML-DSA-65+Ed25519".to_string(),
                anti_replay_domain: "AOXC/NATIVE_TOKEN/TESTNET/V1".to_string(),
                max_transfer_amount: 500_000_000 * 10_u128.pow(18),
                max_total_supply: 100_000_000_000 * 10_u128.pow(18),
            },
            NativeTokenNetwork::Devnet => NativeTokenQuantumPolicy {
                signature_suite: "ML-DSA-44+Ed25519".to_string(),
                anti_replay_domain: "AOXC/NATIVE_TOKEN/DEVNET/V1".to_string(),
                max_transfer_amount: u128::MAX,
                max_total_supply: u128::MAX,
            },
        };

        Self {
            symbol: NATIVE_TOKEN_SYMBOL.to_string(),
            decimals: 18,
            supply_model: SupplyModel::GovernedEmission,
            network,
            quantum_policy,
        }
    }

    #[must_use]
    pub const fn allows_mint(&self) -> bool {
        matches!(
            self.supply_model,
            SupplyModel::GovernedEmission
                | SupplyModel::ProgrammaticEmission
                | SupplyModel::TreasuryAuthorizedEmission
        )
    }

    fn validate_amount(&self, amount: u128) -> Result<(), NativeTokenError> {
        if amount == 0 {
            return Err(NativeTokenError::InvalidAmount);
        }

        if amount > self.quantum_policy.max_transfer_amount {
            return Err(NativeTokenError::TransferLimitExceeded);
        }

        Ok(())
    }
}

/// Minimal in-memory native token ledger.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct NativeTokenLedger {
    pub policy: NativeTokenPolicy,
    pub total_supply: u128,
    pub balances: HashMap<[u8; 32], u128>,
    pub latest_nonce: HashMap<[u8; 32], u64>,
}

impl NativeTokenLedger {
    #[must_use]
    pub fn new(policy: NativeTokenPolicy) -> Self {
        Self {
            policy,
            total_supply: 0,
            balances: HashMap::new(),
            latest_nonce: HashMap::new(),
        }
    }

    #[must_use]
    pub fn new_for_network(network: NativeTokenNetwork) -> Self {
        Self::new(NativeTokenPolicy::for_network(network))
    }

    #[must_use]
    pub fn balance_of(&self, address: &[u8; 32]) -> u128 {
        self.balances.get(address).copied().unwrap_or(0)
    }

    pub fn mint(&mut self, to: [u8; 32], amount: u128) -> Result<(), NativeTokenError> {
        if !self.policy.allows_mint() {
            return Err(NativeTokenError::MintDisabledPolicy);
        }
        self.policy.validate_amount(amount)?;

        self.total_supply = self
            .total_supply
            .checked_add(amount)
            .ok_or(NativeTokenError::SupplyOverflow)?;

        if self.total_supply > self.policy.quantum_policy.max_total_supply {
            return Err(NativeTokenError::SupplyOverflow);
        }

        let balance = self.balances.entry(to).or_insert(0);
        *balance = balance
            .checked_add(amount)
            .ok_or(NativeTokenError::BalanceOverflow)?;

        Ok(())
    }

    pub fn transfer(
        &mut self,
        from: [u8; 32],
        to: [u8; 32],
        amount: u128,
    ) -> Result<(), NativeTokenError> {
        self.policy.validate_amount(amount)?;

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

    /// Transfer with nonce and proof tag, designed for anti-replay
    /// and future post-quantum signature attestation integration.
    pub fn transfer_quantum(
        &mut self,
        from: [u8; 32],
        to: [u8; 32],
        amount: u128,
        nonce: u64,
        proof_tag: &[u8],
    ) -> Result<(), NativeTokenError> {
        if proof_tag.is_empty() {
            return Err(NativeTokenError::ReplayDetected);
        }

        match self.latest_nonce.get(&from).copied() {
            Some(last_nonce) if nonce < last_nonce => {
                return Err(NativeTokenError::NonceRegression);
            }
            Some(last_nonce) if nonce == last_nonce => {
                return Err(NativeTokenError::ReplayDetected);
            }
            _ => {}
        }

        self.transfer(from, to, amount)?;
        self.latest_nonce.insert(from, nonce);
        Ok(())
    }

    #[must_use]
    pub fn mint_receipt(&self, tx_hash: [u8; HASH_SIZE], to: [u8; 32], amount: u128) -> Receipt {
        let mut receipt = Receipt::success(tx_hash, 0);
        receipt.push_event(Event {
            event_type: EVENT_NATIVE_MINT,
            data: encode_transfer_like_event([0; 32], to, amount),
        });
        receipt
    }

    #[must_use]
    pub fn transfer_receipt(
        &self,
        tx_hash: [u8; HASH_SIZE],
        from: [u8; 32],
        to: [u8; 32],
        amount: u128,
    ) -> Receipt {
        let mut receipt = Receipt::success(tx_hash, 0);
        receipt.push_event(Event {
            event_type: EVENT_NATIVE_TRANSFER,
            data: encode_transfer_like_event(from, to, amount),
        });
        receipt
    }

    #[must_use]
    pub fn error_receipt(&self, tx_hash: [u8; HASH_SIZE], error: NativeTokenError) -> Receipt {
        let error_code = match error {
            NativeTokenError::SupplyOverflow => ERROR_CODE_SUPPLY_OVERFLOW,
            NativeTokenError::BalanceOverflow => ERROR_CODE_BALANCE_OVERFLOW,
            NativeTokenError::InsufficientBalance => ERROR_CODE_INSUFFICIENT_BALANCE,
            NativeTokenError::MintDisabledPolicy => ERROR_CODE_MINT_DISABLED,
            NativeTokenError::InvalidAmount => ERROR_CODE_INVALID_AMOUNT,
            NativeTokenError::TransferLimitExceeded => ERROR_CODE_TRANSFER_LIMIT_EXCEEDED,
            NativeTokenError::NonceRegression => ERROR_CODE_NONCE_REGRESSION,
            NativeTokenError::ReplayDetected => ERROR_CODE_REPLAY_DETECTED,
        };
        Receipt::failure(tx_hash, 0, error_code)
    }

    #[must_use]
    pub fn transfer_quantum_receipt(
        &self,
        tx_hash: [u8; HASH_SIZE],
        from: [u8; 32],
        to: [u8; 32],
        amount: u128,
        nonce: u64,
        proof_tag: &[u8],
    ) -> Receipt {
        let mut receipt = Receipt::success(tx_hash, 0);
        receipt.push_event(Event {
            event_type: EVENT_NATIVE_TRANSFER,
            data: encode_quantum_transfer_event(
                &self.policy.quantum_policy.anti_replay_domain,
                from,
                to,
                amount,
                nonce,
                proof_tag,
            ),
        });
        receipt
    }
}

#[must_use]
pub fn encode_transfer_like_event(from: [u8; 32], to: [u8; 32], amount: u128) -> Vec<u8> {
    let mut payload = Vec::with_capacity(80);
    payload.extend_from_slice(&from);
    payload.extend_from_slice(&to);
    payload.extend_from_slice(&amount.to_le_bytes());
    payload
}

#[must_use]
pub fn encode_quantum_transfer_event(
    domain: &str,
    from: [u8; 32],
    to: [u8; 32],
    amount: u128,
    nonce: u64,
    proof_tag: &[u8],
) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(domain.as_bytes());
    hasher.update([0x00]);
    hasher.update(from);
    hasher.update(to);
    hasher.update(amount.to_le_bytes());
    hasher.update(nonce.to_le_bytes());
    hasher.update(proof_tag);
    let digest = hasher.finalize();

    let mut payload = Vec::with_capacity(120);
    payload.extend_from_slice(&from);
    payload.extend_from_slice(&to);
    payload.extend_from_slice(&amount.to_le_bytes());
    payload.extend_from_slice(&nonce.to_le_bytes());
    payload.extend_from_slice(&digest);
    payload
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(byte: u8) -> [u8; 32] {
        [byte; 32]
    }

    #[test]
    fn mint_updates_supply_and_balance() {
        let mut ledger = NativeTokenLedger::new_for_network(NativeTokenNetwork::Mainnet);

        ledger.mint(addr(1), 100).unwrap();

        assert_eq!(ledger.total_supply, 100);
        assert_eq!(ledger.balance_of(&addr(1)), 100);
        assert_eq!(ledger.policy.symbol, NATIVE_TOKEN_SYMBOL);
    }

    #[test]
    fn transfer_moves_balance_without_changing_supply() {
        let mut ledger = NativeTokenLedger::new_for_network(NativeTokenNetwork::Mainnet);
        ledger.mint(addr(1), 100).unwrap();

        ledger.transfer(addr(1), addr(2), 30).unwrap();

        assert_eq!(ledger.total_supply, 100);
        assert_eq!(ledger.balance_of(&addr(1)), 70);
        assert_eq!(ledger.balance_of(&addr(2)), 30);
    }

    #[test]
    fn transfer_fails_when_balance_is_insufficient() {
        let mut ledger = NativeTokenLedger::new_for_network(NativeTokenNetwork::Mainnet);
        ledger.mint(addr(1), 10).unwrap();

        let err = ledger.transfer(addr(1), addr(2), 11).unwrap_err();
        assert_eq!(err, NativeTokenError::InsufficientBalance);
    }

    #[test]
    fn receipts_emit_expected_events_and_codes() {
        let ledger = NativeTokenLedger::new_for_network(NativeTokenNetwork::Mainnet);

        let mint_receipt = ledger.mint_receipt([7; HASH_SIZE], addr(9), 42);
        assert!(mint_receipt.success);
        assert_eq!(mint_receipt.events.len(), 1);
        assert_eq!(mint_receipt.events[0].event_type, EVENT_NATIVE_MINT);
        assert_eq!(mint_receipt.events[0].data.len(), 80);

        let error_receipt =
            ledger.error_receipt([8; HASH_SIZE], NativeTokenError::InsufficientBalance);
        assert!(!error_receipt.success);
        assert_eq!(
            error_receipt.error_code,
            Some(ERROR_CODE_INSUFFICIENT_BALANCE)
        );
    }

    #[test]
    fn mint_is_rejected_when_supply_model_disables_mint() {
        let mut ledger = NativeTokenLedger::new(NativeTokenPolicy {
            supply_model: SupplyModel::MintDisabled,
            ..NativeTokenPolicy::default()
        });

        let err = ledger.mint(addr(1), 10).unwrap_err();
        assert_eq!(err, NativeTokenError::MintDisabledPolicy);
    }

    #[test]
    fn network_profiles_are_mainnet_testnet_devnet_compatible() {
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
    fn quantum_transfer_rejects_replay_and_nonce_regression() {
        let mut ledger = NativeTokenLedger::new_for_network(NativeTokenNetwork::Mainnet);
        ledger.mint(addr(1), 1_000).unwrap();

        ledger
            .transfer_quantum(addr(1), addr(2), 100, 1, b"sig-proof")
            .unwrap();

        let replay_err = ledger
            .transfer_quantum(addr(1), addr(2), 100, 1, b"sig-proof")
            .unwrap_err();
        assert_eq!(replay_err, NativeTokenError::ReplayDetected);

        let regression_err = ledger
            .transfer_quantum(addr(1), addr(2), 100, 0, b"sig-proof")
            .unwrap_err();
        assert_eq!(regression_err, NativeTokenError::NonceRegression);
    }

    #[test]
    fn quantum_transfer_event_encoding_contains_expected_layout() {
        let payload = encode_quantum_transfer_event(
            "AOXC/NATIVE_TOKEN/TESTNET/V1",
            addr(1),
            addr(2),
            77,
            9,
            b"proof",
        );

        assert_eq!(payload.len(), 120);
    }
}
