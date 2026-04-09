//! Nonce state model used by validation-policy-backed accounts.

use std::collections::BTreeMap;

use crate::auth::identity::{AccountId, NonceReplayModel};

/// Per-account nonce state.
#[derive(Debug, Clone, Default)]
pub struct NonceState {
    /// Monotonic global nonce (if used by policy).
    global_highest: Option<u64>,
    /// Monotonic per-domain nonce map (if used by policy).
    by_domain_highest: BTreeMap<String, u64>,
}

impl NonceState {
    /// Admits a nonce under the selected nonce/replay model.
    pub fn admit(&mut self, model: NonceReplayModel, domain: &str, nonce: u64) -> bool {
        match model {
            NonceReplayModel::MonotonicGlobal => match self.global_highest {
                Some(previous) if nonce <= previous => false,
                _ => {
                    self.global_highest = Some(nonce);
                    true
                }
            },
            NonceReplayModel::MonotonicPerDomain => match self.by_domain_highest.get(domain) {
                Some(previous) if nonce <= *previous => false,
                _ => {
                    self.by_domain_highest.insert(domain.to_owned(), nonce);
                    true
                }
            },
        }
    }
}

/// Nonce table keyed by account id.
#[derive(Debug, Clone, Default)]
pub struct NonceTable {
    states: BTreeMap<AccountId, NonceState>,
}

impl NonceTable {
    /// Creates an empty nonce table.
    pub fn new() -> Self {
        Self::default()
    }

    /// Admits a nonce against an account under the selected model.
    pub fn admit(
        &mut self,
        account_id: AccountId,
        model: NonceReplayModel,
        domain: &str,
        nonce: u64,
    ) -> bool {
        self.states
            .entry(account_id)
            .or_default()
            .admit(model, domain, nonce)
    }
}

#[cfg(test)]
mod tests {
    use crate::auth::{
        identity::{NonceReplayModel, derive_account_id},
        nonce::{NonceState, NonceTable},
        scheme::SignatureAlgorithm,
    };

    #[test]
    fn global_nonce_model_is_strictly_monotonic() {
        let mut state = NonceState::default();
        assert!(state.admit(NonceReplayModel::MonotonicGlobal, "AOX/TX/V1", 1));
        assert!(!state.admit(NonceReplayModel::MonotonicGlobal, "AOX/TX/V1", 1));
        assert!(!state.admit(NonceReplayModel::MonotonicGlobal, "AOX/TX/V1", 0));
        assert!(state.admit(NonceReplayModel::MonotonicGlobal, "AOX/TX/V1", 2));
    }

    #[test]
    fn per_domain_nonce_model_tracks_each_domain_independently() {
        let mut state = NonceState::default();
        assert!(state.admit(NonceReplayModel::MonotonicPerDomain, "AOX/TX/V1", 1));
        assert!(state.admit(NonceReplayModel::MonotonicPerDomain, "AOX/GOVERNANCE/V1", 1));
        assert!(!state.admit(NonceReplayModel::MonotonicPerDomain, "AOX/TX/V1", 1));
        assert!(state.admit(NonceReplayModel::MonotonicPerDomain, "AOX/TX/V1", 2));
    }

    #[test]
    fn nonce_table_scopes_by_account_id() {
        let mut table = NonceTable::new();
        let root = crate::crypto::hash::quantum_hardened_digest(b"policy", b"v1");
        let alice = derive_account_id(
            "AOX/ACCOUNT/V1",
            SignatureAlgorithm::MlDsa65,
            b"alice",
            root.to_bytes().as_slice(),
        );
        let bob = derive_account_id(
            "AOX/ACCOUNT/V1",
            SignatureAlgorithm::MlDsa65,
            b"bob",
            root.to_bytes().as_slice(),
        );

        assert!(table.admit(alice, NonceReplayModel::MonotonicPerDomain, "AOX/TX/V1", 1));
        assert!(table.admit(bob, NonceReplayModel::MonotonicPerDomain, "AOX/TX/V1", 1));
        assert!(!table.admit(alice, NonceReplayModel::MonotonicPerDomain, "AOX/TX/V1", 1));
    }
}
