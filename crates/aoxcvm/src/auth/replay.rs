//! Deterministic replay-admission helpers for AOXCVM auth envelopes.

use std::collections::BTreeMap;

use crate::auth::nonce::NonceTable;
use crate::auth::{
    envelope::AuthEnvelope,
    identity::{AccountId, NonceReplayModel},
};

/// Per-domain replay guard that enforces strictly increasing nonces per sender key.
#[derive(Debug, Clone, Default)]
pub struct ReplayGuard {
    highest_nonce_by_sender: BTreeMap<String, u64>,
    nonce_table: NonceTable,
}

impl ReplayGuard {
    /// Creates a fresh replay guard state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true when the envelope nonce is strictly newer than the sender's last accepted
    /// envelope nonce and records it.
    pub fn admit(&mut self, sender_key: &str, envelope: &AuthEnvelope) -> bool {
        match self.highest_nonce_by_sender.get(sender_key).copied() {
            Some(previous) if envelope.nonce <= previous => false,
            _ => {
                self.highest_nonce_by_sender
                    .insert(sender_key.to_owned(), envelope.nonce);
                true
            }
        }
    }

    /// Reads the highest admitted nonce for a sender key.
    pub fn highest_nonce(&self, sender_key: &str) -> Option<u64> {
        self.highest_nonce_by_sender.get(sender_key).copied()
    }

    /// Constitution-aware replay admission for validation-policy account objects.
    pub fn admit_account(
        &mut self,
        account_id: AccountId,
        model: NonceReplayModel,
        envelope: &AuthEnvelope,
    ) -> bool {
        self.nonce_table
            .admit(account_id, model, envelope.domain.as_str(), envelope.nonce)
    }
}

#[cfg(test)]
mod tests {
    use super::ReplayGuard;
    use crate::auth::{
        envelope::AuthEnvelope,
        identity::{NonceReplayModel, derive_account_id},
        scheme::SignatureAlgorithm,
    };

    fn envelope(nonce: u64) -> AuthEnvelope {
        AuthEnvelope {
            domain: "tx".to_owned(),
            nonce,
            signers: vec![],
        }
    }

    #[test]
    fn replay_guard_rejects_reused_or_older_nonce() {
        let mut guard = ReplayGuard::new();

        assert!(guard.admit("alice", &envelope(10)));
        assert!(!guard.admit("alice", &envelope(10)));
        assert!(!guard.admit("alice", &envelope(9)));
        assert_eq!(guard.highest_nonce("alice"), Some(10));
    }

    #[test]
    fn replay_guard_tracks_senders_independently() {
        let mut guard = ReplayGuard::new();

        assert!(guard.admit("alice", &envelope(3)));
        assert!(guard.admit("bob", &envelope(1)));
        assert!(guard.admit("alice", &envelope(4)));

        assert_eq!(guard.highest_nonce("alice"), Some(4));
        assert_eq!(guard.highest_nonce("bob"), Some(1));
    }

    #[test]
    fn account_replay_uses_nonce_model_and_domain_scope() {
        let mut guard = ReplayGuard::new();
        let policy_root = crate::crypto::hash::quantum_hardened_digest(b"policy", b"v1");
        let account_id = derive_account_id(
            "AOX/ACCOUNT/V1",
            SignatureAlgorithm::MlDsa65,
            b"alice-key",
            policy_root.to_bytes().as_slice(),
        );

        let tx1 = AuthEnvelope {
            domain: "AOX/TX/V1".to_owned(),
            nonce: 1,
            signers: vec![],
        };
        let gov1 = AuthEnvelope {
            domain: "AOX/GOVERNANCE/V1".to_owned(),
            nonce: 1,
            signers: vec![],
        };

        assert!(guard.admit_account(account_id, NonceReplayModel::MonotonicPerDomain, &tx1));
        assert!(guard.admit_account(account_id, NonceReplayModel::MonotonicPerDomain, &gov1));
        assert!(!guard.admit_account(account_id, NonceReplayModel::MonotonicPerDomain, &tx1));
    }
}
