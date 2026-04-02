//! Deterministic replay-admission helpers for AOXCVM auth envelopes.

use std::collections::BTreeMap;

use crate::auth::envelope::AuthEnvelope;

/// Per-domain replay guard that enforces strictly increasing nonces per sender key.
#[derive(Debug, Clone, Default)]
pub struct ReplayGuard {
    highest_nonce_by_sender: BTreeMap<String, u64>,
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
}

#[cfg(test)]
mod tests {
    use crate::auth::envelope::AuthEnvelope;

    use super::ReplayGuard;

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
}
