// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};
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

/// Domain errors for the native token ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeTokenError {
    SupplyOverflow,
    BalanceOverflow,
    InsufficientBalance,
    MintDisabledPolicy,
}

impl std::fmt::Display for NativeTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SupplyOverflow => write!(f, "native token supply overflow"),
            Self::BalanceOverflow => write!(f, "native token balance overflow"),
            Self::InsufficientBalance => write!(f, "insufficient native token balance"),
            Self::MintDisabledPolicy => write!(f, "native token mint disabled by supply policy"),
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
}

impl Default for NativeTokenPolicy {
    fn default() -> Self {
        Self {
            symbol: NATIVE_TOKEN_SYMBOL.to_string(),
            decimals: 18,
            supply_model: SupplyModel::GovernedEmission,
        }
    }
}

impl NativeTokenPolicy {
    #[must_use]
    pub const fn allows_mint(&self) -> bool {
        matches!(
            self.supply_model,
            SupplyModel::GovernedEmission
                | SupplyModel::ProgrammaticEmission
                | SupplyModel::TreasuryAuthorizedEmission
        )
    }
}

/// Minimal in-memory native token ledger.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct NativeTokenLedger {
    pub policy: NativeTokenPolicy,
    pub total_supply: u128,
    pub balances: HashMap<[u8; 32], u128>,
}

impl NativeTokenLedger {
    #[must_use]
    pub fn new(policy: NativeTokenPolicy) -> Self {
        Self {
            policy,
            total_supply: 0,
            balances: HashMap::new(),
        }
    }

    #[must_use]
    pub fn balance_of(&self, address: &[u8; 32]) -> u128 {
        self.balances.get(address).copied().unwrap_or(0)
    }

    pub fn mint(&mut self, to: [u8; 32], amount: u128) -> Result<(), NativeTokenError> {
        if !self.policy.allows_mint() {
            return Err(NativeTokenError::MintDisabledPolicy);
        }

        self.total_supply = self
            .total_supply
            .checked_add(amount)
            .ok_or(NativeTokenError::SupplyOverflow)?;

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
        };
        Receipt::failure(tx_hash, 0, error_code)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(byte: u8) -> [u8; 32] {
        [byte; 32]
    }

    #[test]
    fn mint_updates_supply_and_balance() {
        let mut ledger = NativeTokenLedger::default();

        ledger.mint(addr(1), 100).unwrap();

        assert_eq!(ledger.total_supply, 100);
        assert_eq!(ledger.balance_of(&addr(1)), 100);
        assert_eq!(ledger.policy.symbol, NATIVE_TOKEN_SYMBOL);
    }

    #[test]
    fn transfer_moves_balance_without_changing_supply() {
        let mut ledger = NativeTokenLedger::default();
        ledger.mint(addr(1), 100).unwrap();

        ledger.transfer(addr(1), addr(2), 30).unwrap();

        assert_eq!(ledger.total_supply, 100);
        assert_eq!(ledger.balance_of(&addr(1)), 70);
        assert_eq!(ledger.balance_of(&addr(2)), 30);
    }

    #[test]
    fn transfer_fails_when_balance_is_insufficient() {
        let mut ledger = NativeTokenLedger::default();
        ledger.mint(addr(1), 10).unwrap();

        let err = ledger.transfer(addr(1), addr(2), 11).unwrap_err();
        assert_eq!(err, NativeTokenError::InsufficientBalance);
    }

    #[test]
    fn receipts_emit_expected_events_and_codes() {
        let ledger = NativeTokenLedger::default();

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
}
