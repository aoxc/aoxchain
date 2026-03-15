//! core/state/src/mod.rs

pub mod account;

use crate::state::account::Account;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct WorldState {
    /// Adres (32 byte public key) -> Hesap verisi.
    pub accounts: HashMap<[u8; 32], Account>,
}

impl WorldState {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }

    /// Bir hesabın güncel durumunu getirir, yoksa yeni oluşturur.
    pub fn get_or_create(
        &mut self,
        address: [u8; 32],
        cap: crate::block::Capability,
    ) -> &mut Account {
        self.accounts.entry(address).or_insert(Account::new(cap))
    }

    /// Eşsiz bir State Root (hash) üretir.
    /// (Blockchain'in o andaki halinin parmak izi).
    pub fn root_hash(&self) -> [u8; 32] {
        // AOXC Minimalist Yaklaşım: Tüm hesapları sıralı hashle.
        // İleride buraya Merkle Patricia Trie eklenebilir.
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
