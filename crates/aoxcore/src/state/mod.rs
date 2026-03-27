// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! core/state/src/mod.rs

pub mod account;

use crate::state::account::Account;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct WorldState {
    /// Address (32-byte public key) -> account data.
    pub accounts: HashMap<[u8; 32], Account>,
}

impl WorldState {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }

    /// Returns the current account state, creating a new account if it does not exist.
    pub fn get_or_create(
        &mut self,
        address: [u8; 32],
        cap: crate::block::Capability,
    ) -> &mut Account {
        self.accounts.entry(address).or_insert(Account::new(cap))
    }

    /// Produces a unique State Root hash (fingerprint of the current chain state).
    pub fn root_hash(&self) -> [u8; 32] {
        // AOXC minimalist approach: hash all accounts in sorted order.
        // A Merkle Patricia Trie can be introduced here in the future.
        let mut hasher = blake3::Hasher::new();
        let mut sorted_accounts: Vec<_> = self.accounts.iter().collect();
        sorted_accounts.sort_by(|a, b| a.0.cmp(b.0));

        for (addr, acc) in sorted_accounts {
            hasher.update(addr);
            hasher.update(&acc.nonce.to_le_bytes());
            hasher.update(&acc.energy.to_le_bytes());
        }
        *hasher.finalize().as_bytes()
    }
}
