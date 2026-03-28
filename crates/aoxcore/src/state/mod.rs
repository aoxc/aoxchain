// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! core/state/src/mod.rs
//!
//! AOXC canonical world-state container.
//!
//! This module provides the minimal deterministic account-state container used
//! by the AOXC runtime. The implementation is intentionally explicit and
//! fail-closed because the state root derived here may become consensus-critical
//! for block execution, snapshot validation, replication, and auditing.

pub mod account;

use blake3::Hasher;
use std::collections::HashMap;
use std::fmt;

use crate::block::Capability;
use crate::state::account::{Account, AccountError};

/// Domain-separated namespace for world-state hashing.
///
/// Any incompatible change to canonical world-state hashing must update this
/// namespace and/or the hashing version constant below.
const WORLD_STATE_HASH_NAMESPACE: &[u8] = b"AOXC/STATE/WORLD_STATE";

/// Canonical hashing version for `WorldState::root_hash`.
const WORLD_STATE_HASH_VERSION: u8 = 1;

/// Canonical 32-byte AOXC address type.
pub type Address = [u8; 32];

/// Errors emitted by the world-state container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldStateError {
    Account(AccountError),
    CapabilityEncodingFailed,
}

impl From<AccountError> for WorldStateError {
    fn from(value: AccountError) -> Self {
        Self::Account(value)
    }
}

impl fmt::Display for WorldStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Account(err) => write!(f, "world-state account error: {}", err),
            Self::CapabilityEncodingFailed => {
                write!(f, "world-state capability encoding failed")
            }
        }
    }
}

impl std::error::Error for WorldStateError {}

/// Canonical AOXC world-state container.
///
/// The world state is modeled as a deterministic mapping from 32-byte address
/// to canonical account state. Root derivation hashes all entries in sorted
/// address order and commits to every persisted account field.
#[derive(Debug, Clone, Default)]
pub struct WorldState {
    /// Address (32-byte public key) -> canonical account state.
    pub accounts: HashMap<Address, Account>,
}

impl WorldState {
    /// Creates an empty canonical world state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }

    /// Returns the number of tracked accounts.
    #[must_use]
    pub fn len(&self) -> usize {
        self.accounts.len()
    }

    /// Returns `true` if the world state contains no accounts.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty()
    }

    /// Returns an immutable reference to an account, if present.
    #[must_use]
    pub fn get(&self, address: &Address) -> Option<&Account> {
        self.accounts.get(address)
    }

    /// Returns a mutable reference to an account, if present.
    #[must_use]
    pub fn get_mut(&mut self, address: &Address) -> Option<&mut Account> {
        self.accounts.get_mut(address)
    }

    /// Returns `true` if an account exists for the supplied address.
    #[must_use]
    pub fn contains(&self, address: &Address) -> bool {
        self.accounts.contains_key(address)
    }

    /// Returns the current account state, creating a new account if it does not exist.
    ///
    /// Existing accounts are never overwritten. The supplied capability is only
    /// used when a fresh account must be initialized.
    pub fn get_or_create(&mut self, address: Address, capability: Capability) -> &mut Account {
        self.accounts
            .entry(address)
            .or_insert_with(|| Account::new(capability))
    }

    /// Inserts a fully-formed canonical account after validation.
    ///
    /// Returns the previous account if one existed at the same address.
    pub fn insert(
        &mut self,
        address: Address,
        account: Account,
    ) -> Result<Option<Account>, WorldStateError> {
        account.validate()?;
        Ok(self.accounts.insert(address, account))
    }

    /// Removes and returns the account stored at the supplied address, if any.
    pub fn remove(&mut self, address: &Address) -> Option<Account> {
        self.accounts.remove(address)
    }

    /// Validates all persisted accounts inside the world state.
    pub fn validate(&self) -> Result<(), WorldStateError> {
        for account in self.accounts.values() {
            account.validate()?;
        }

        Ok(())
    }

    /// Produces the canonical world-state root hash.
    pub fn try_root_hash(&self) -> Result<[u8; 32], WorldStateError> {
        self.validate()?;

        let mut hasher = Hasher::new();
        hasher.update(WORLD_STATE_HASH_NAMESPACE);
        hasher.update(&[0x00, WORLD_STATE_HASH_VERSION]);

        let mut sorted_accounts: Vec<_> = self.accounts.iter().collect();
        sorted_accounts.sort_by(|a, b| a.0.cmp(b.0));

        hash_len_prefixed_bytes(&mut hasher, &(sorted_accounts.len() as u64).to_le_bytes());

        for (address, account) in sorted_accounts {
            hash_len_prefixed_bytes(&mut hasher, address);
            hash_len_prefixed_bytes(&mut hasher, &account.nonce.to_le_bytes());

            let capability_bytes =
                serde_json::to_vec(&account.capability).map_err(|_| WorldStateError::CapabilityEncodingFailed)?;
            hash_len_prefixed_bytes(&mut hasher, &capability_bytes);

            hash_len_prefixed_bytes(&mut hasher, &account.energy.to_le_bytes());

            match &account.metadata {
                Some(metadata) => {
                    hash_len_prefixed_bytes(&mut hasher, &[1u8]);
                    hash_len_prefixed_bytes(&mut hasher, metadata);
                }
                None => {
                    hash_len_prefixed_bytes(&mut hasher, &[0u8]);
                }
            }
        }

        Ok(*hasher.finalize().as_bytes())
    }

    /// Produces the canonical world-state root hash.
    ///
    /// This method assumes all accounts are already valid.
    #[must_use]
    pub fn root_hash(&self) -> [u8; 32] {
        self.try_root_hash()
            .expect("world-state root hashing must operate on valid canonical accounts")
    }

    /// Returns the canonical empty-state root hash.
    #[must_use]
    pub fn empty_root_hash() -> [u8; 32] {
        Self::new().root_hash()
    }
}

/// Hashes a field with an explicit little-endian length prefix and separator.
#[inline]
fn hash_len_prefixed_bytes(hasher: &mut Hasher, bytes: &[u8]) {
    let len = bytes.len() as u64;
    hasher.update(&len.to_le_bytes());
    hasher.update(bytes);
    hasher.update(&[0xFF]);
}
