// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! core/state/src/account.rs
//!
//! AOXC Account Model.
//!
//! This module defines the canonical account state stored inside the AOXC
//! world-state database.
//!
//! Design objectives:
//! - deterministic and minimal state representation
//! - explicit invariant enforcement
//! - bounded metadata handling
//! - fail-closed state transitions
//! - audit-friendly semantics for nonce and energy accounting

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::block::Capability;

/// Maximum metadata size allowed for accounts.
///
/// This bound exists to prevent abusive or accidental state bloat at the
/// account-storage layer. Higher-level modules must treat account metadata
/// as a compact auxiliary field rather than an unbounded data container.
pub const MAX_ACCOUNT_METADATA_BYTES: usize = 4096;

/// Canonical account-domain error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountError {
    NonceOverflow,
    EnergyOverflow,
    EnergyUnderflow,
    MetadataTooLarge { size: usize, max: usize },
}

impl fmt::Display for AccountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonceOverflow => write!(f, "account nonce overflow"),
            Self::EnergyOverflow => write!(f, "account energy overflow"),
            Self::EnergyUnderflow => write!(f, "account energy underflow"),
            Self::MetadataTooLarge { size, max } => {
                write!(f, "account metadata size {} exceeds maximum {}", size, max)
            }
        }
    }
}

impl std::error::Error for AccountError {}

/// Canonical AOXC account state.
///
/// Security and correctness notes:
/// - `nonce` is the replay-protection counter and must only move forward.
/// - `energy` is the execution-payment balance and must not underflow.
/// - `metadata` is optional and size-bounded to protect state growth.
/// - `capability` represents the account authorization class and is part of
///   the persisted canonical state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Account {
    /// Replay-protection counter.
    pub nonce: u64,

    /// Authorization class.
    pub capability: Capability,

    /// Energy balance used to pay execution cost.
    pub energy: u64,

    /// Optional metadata blob used by higher-level modules.
    ///
    /// This field must remain bounded under `MAX_ACCOUNT_METADATA_BYTES`.
    pub metadata: Option<Vec<u8>>,
}

impl Account {
    /// Creates a new canonical account with zero nonce, zero energy, and no metadata.
    #[must_use]
    pub fn new(capability: Capability) -> Self {
        Self {
            nonce: 0,
            capability,
            energy: 0,
            metadata: None,
        }
    }

    /// Creates a canonical account with an explicitly supplied initial energy balance.
    ///
    /// This constructor is useful for genesis or controlled state bootstrapping.
    #[must_use]
    pub fn with_energy(capability: Capability, energy: u64) -> Self {
        Self {
            nonce: 0,
            capability,
            energy,
            metadata: None,
        }
    }

    /// Creates a canonical account with bounded metadata.
    pub fn with_metadata(
        capability: Capability,
        energy: u64,
        metadata: Vec<u8>,
    ) -> Result<Self, AccountError> {
        validate_metadata_len(metadata.len())?;

        Ok(Self {
            nonce: 0,
            capability,
            energy,
            metadata: Some(metadata),
        })
    }

    /// Validates the internal account invariants.
    ///
    /// This function should be used by storage import paths, migration logic,
    /// deserialization boundaries, and any code that mutates fields directly.
    pub fn validate(&self) -> Result<(), AccountError> {
        if let Some(metadata) = &self.metadata {
            validate_metadata_len(metadata.len())?;
        }

        Ok(())
    }

    /// Safely increments the replay-protection nonce.
    pub fn increment_nonce(&mut self) -> Result<(), AccountError> {
        self.nonce = self
            .nonce
            .checked_add(1)
            .ok_or(AccountError::NonceOverflow)?;
        Ok(())
    }

    /// Sets the nonce to an explicit value.
    ///
    /// This method is intended for trusted state-sync, snapshot restore,
    /// migration, or genesis-loading flows. Regular execution paths should
    /// use `increment_nonce` in order to preserve monotonic replay semantics.
    pub fn set_nonce(&mut self, nonce: u64) {
        self.nonce = nonce;
    }

    /// Adds energy to the account.
    pub fn add_energy(&mut self, value: u64) -> Result<(), AccountError> {
        self.energy = self
            .energy
            .checked_add(value)
            .ok_or(AccountError::EnergyOverflow)?;
        Ok(())
    }

    /// Consumes energy from the account.
    pub fn consume_energy(&mut self, value: u64) -> Result<(), AccountError> {
        self.energy = self
            .energy
            .checked_sub(value)
            .ok_or(AccountError::EnergyUnderflow)?;
        Ok(())
    }

    /// Replaces the energy balance with an explicit value.
    ///
    /// This function is intended for trusted administrative or state-recovery
    /// flows rather than ordinary execution accounting.
    pub fn set_energy(&mut self, value: u64) {
        self.energy = value;
    }

    /// Sets account metadata after enforcing the canonical size bound.
    pub fn set_metadata(&mut self, data: Vec<u8>) -> Result<(), AccountError> {
        validate_metadata_len(data.len())?;
        self.metadata = Some(data);
        Ok(())
    }

    /// Replaces account metadata using an optional value.
    ///
    /// `None` clears metadata; `Some(..)` is validated against the canonical
    /// maximum size before being accepted.
    pub fn replace_metadata(&mut self, data: Option<Vec<u8>>) -> Result<(), AccountError> {
        match data {
            Some(bytes) => self.set_metadata(bytes),
            None => {
                self.metadata = None;
                Ok(())
            }
        }
    }

    /// Clears account metadata.
    pub fn clear_metadata(&mut self) {
        self.metadata = None;
    }

    /// Returns `true` if metadata is present.
    #[must_use]
    pub fn has_metadata(&self) -> bool {
        self.metadata.is_some()
    }

    /// Returns the metadata length in bytes.
    #[must_use]
    pub fn metadata_len(&self) -> usize {
        self.metadata.as_ref().map_or(0, Vec::len)
    }

    /// Returns the current energy balance.
    #[must_use]
    pub const fn energy(&self) -> u64 {
        self.energy
    }

    /// Returns the current nonce.
    #[must_use]
    pub const fn nonce(&self) -> u64 {
        self.nonce
    }

    /// Returns `true` when the account can cover the requested energy amount.
    #[must_use]
    pub const fn can_afford_energy(&self, value: u64) -> bool {
        self.energy >= value
    }
}

#[inline]
fn validate_metadata_len(size: usize) -> Result<(), AccountError> {
    if size > MAX_ACCOUNT_METADATA_BYTES {
        return Err(AccountError::MetadataTooLarge {
            size,
            max: MAX_ACCOUNT_METADATA_BYTES,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::Capability;

    fn sample_capability() -> Capability {
        Capability::User
    }

    #[test]
    fn new_account_starts_from_canonical_zero_state() {
        let account = Account::new(sample_capability());

        assert_eq!(account.nonce, 0);
        assert_eq!(account.energy, 0);
        assert_eq!(account.metadata, None);
        assert!(account.validate().is_ok());
    }

    #[test]
    fn with_energy_sets_initial_energy() {
        let account = Account::with_energy(sample_capability(), 55);

        assert_eq!(account.energy, 55);
        assert_eq!(account.nonce, 0);
        assert!(account.metadata.is_none());
    }

    #[test]
    fn with_metadata_rejects_oversized_input() {
        let oversized = vec![0u8; MAX_ACCOUNT_METADATA_BYTES + 1];

        let result = Account::with_metadata(sample_capability(), 10, oversized);
        assert_eq!(
            result,
            Err(AccountError::MetadataTooLarge {
                size: MAX_ACCOUNT_METADATA_BYTES + 1,
                max: MAX_ACCOUNT_METADATA_BYTES
            })
        );
    }

    #[test]
    fn with_metadata_accepts_bounded_input() {
        let account =
            Account::with_metadata(sample_capability(), 10, vec![1, 2, 3]).expect("valid account");

        assert_eq!(account.energy, 10);
        assert_eq!(account.metadata, Some(vec![1, 2, 3]));
        assert!(account.validate().is_ok());
    }

    #[test]
    fn increment_nonce_advances_monotonically() {
        let mut account = Account::new(sample_capability());

        account.increment_nonce().expect("nonce increment must succeed");
        account.increment_nonce().expect("nonce increment must succeed");

        assert_eq!(account.nonce(), 2);
    }

    #[test]
    fn increment_nonce_rejects_overflow() {
        let mut account = Account::new(sample_capability());
        account.nonce = u64::MAX;

        let result = account.increment_nonce();
        assert_eq!(result, Err(AccountError::NonceOverflow));
    }

    #[test]
    fn add_energy_rejects_overflow() {
        let mut account = Account::new(sample_capability());
        account.energy = u64::MAX;

        let result = account.add_energy(1);
        assert_eq!(result, Err(AccountError::EnergyOverflow));
    }

    #[test]
    fn consume_energy_rejects_underflow() {
        let mut account = Account::new(sample_capability());

        let result = account.consume_energy(1);
        assert_eq!(result, Err(AccountError::EnergyUnderflow));
    }

    #[test]
    fn consume_energy_uses_checked_subtraction() {
        let mut account = Account::with_energy(sample_capability(), 100);

        account.consume_energy(40).expect("energy consumption must succeed");

        assert_eq!(account.energy(), 60);
    }

    #[test]
    fn set_metadata_rejects_oversized_input() {
        let mut account = Account::new(sample_capability());

        let result = account.set_metadata(vec![0u8; MAX_ACCOUNT_METADATA_BYTES + 1]);
        assert_eq!(
            result,
            Err(AccountError::MetadataTooLarge {
                size: MAX_ACCOUNT_METADATA_BYTES + 1,
                max: MAX_ACCOUNT_METADATA_BYTES
            })
        );
    }

    #[test]
    fn replace_metadata_accepts_none_and_some() {
        let mut account = Account::new(sample_capability());

        account
            .replace_metadata(Some(vec![1, 2, 3]))
            .expect("metadata set must succeed");
        assert!(account.has_metadata());
        assert_eq!(account.metadata_len(), 3);

        account
            .replace_metadata(None)
            .expect("metadata clear must succeed");
        assert!(!account.has_metadata());
        assert_eq!(account.metadata_len(), 0);
    }

    #[test]
    fn validate_rejects_oversized_metadata_on_direct_state_mutation() {
        let mut account = Account::new(sample_capability());
        account.metadata = Some(vec![0u8; MAX_ACCOUNT_METADATA_BYTES + 1]);

        let result = account.validate();
        assert_eq!(
            result,
            Err(AccountError::MetadataTooLarge {
                size: MAX_ACCOUNT_METADATA_BYTES + 1,
                max: MAX_ACCOUNT_METADATA_BYTES
            })
        );
    }

    #[test]
    fn can_afford_energy_matches_balance() {
        let account = Account::with_energy(sample_capability(), 50);

        assert!(account.can_afford_energy(50));
        assert!(account.can_afford_energy(49));
        assert!(!account.can_afford_energy(51));
    }

    #[test]
    fn clear_metadata_removes_existing_value() {
        let mut account =
            Account::with_metadata(sample_capability(), 0, vec![7, 8]).expect("valid account");

        assert!(account.has_metadata());
        account.clear_metadata();
        assert!(!account.has_metadata());
    }
}
