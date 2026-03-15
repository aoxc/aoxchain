//! core/state/src/account.rs
//!
//! AOXC Account Model.
//!
//! This structure represents the canonical account state stored inside the
//! world-state database.

use serde::{Deserialize, Serialize};

use crate::block::Capability;

/// Maximum metadata size allowed for accounts.
pub const MAX_ACCOUNT_METADATA_BYTES: usize = 4096;

/// Canonical account-domain error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountError {
    NonceOverflow,
    EnergyOverflow,
    EnergyUnderflow,
    MetadataTooLarge { size: usize, max: usize },
}

impl std::fmt::Display for AccountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonceOverflow => write!(f, "account nonce overflow"),
            Self::EnergyOverflow => write!(f, "account energy overflow"),
            Self::EnergyUnderflow => write!(f, "account energy underflow"),
            Self::MetadataTooLarge { size, max } => {
                write!(f, "metadata size {} exceeds max {}", size, max)
            }
        }
    }
}

impl std::error::Error for AccountError {}

/// Canonical AOXC account state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Account {
    /// Replay protection counter.
    pub nonce: u64,

    /// Authorization class.
    pub capability: Capability,

    /// Energy balance used to pay execution cost.
    pub energy: u64,

    /// Optional metadata blob used by higher-level modules.
    pub metadata: Option<Vec<u8>>,
}

impl Account {
    /// Creates a new account.
    #[must_use]
    pub fn new(capability: Capability) -> Self {
        Self {
            nonce: 0,
            capability,
            energy: 0,
            metadata: None,
        }
    }

    /// Safely increments nonce.
    pub fn increment_nonce(&mut self) -> Result<(), AccountError> {
        self.nonce = self
            .nonce
            .checked_add(1)
            .ok_or(AccountError::NonceOverflow)?;
        Ok(())
    }

    /// Adds energy to the account.
    pub fn add_energy(&mut self, value: u64) -> Result<(), AccountError> {
        self.energy = self
            .energy
            .checked_add(value)
            .ok_or(AccountError::EnergyOverflow)?;
        Ok(())
    }

    /// Consumes energy.
    pub fn consume_energy(&mut self, value: u64) -> Result<(), AccountError> {
        if self.energy < value {
            return Err(AccountError::EnergyUnderflow);
        }

        self.energy -= value;
        Ok(())
    }

    /// Sets account metadata.
    pub fn set_metadata(&mut self, data: Vec<u8>) -> Result<(), AccountError> {
        if data.len() > MAX_ACCOUNT_METADATA_BYTES {
            return Err(AccountError::MetadataTooLarge {
                size: data.len(),
                max: MAX_ACCOUNT_METADATA_BYTES,
            });
        }

        self.metadata = Some(data);
        Ok(())
    }

    /// Clears account metadata.
    pub fn clear_metadata(&mut self) {
        self.metadata = None;
    }

    /// Returns true if metadata is present.
    #[must_use]
    pub fn has_metadata(&self) -> bool {
        self.metadata.is_some()
    }
}
