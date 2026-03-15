use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

/// Revocation reason codes.
///
/// These values allow operators and auditors to understand why an identity
/// was revoked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RevocationReason {
    KeyCompromise,
    OperatorAction,
    GovernanceDecision,
    ExpiredCertificate,
    Other(String),
}

/// Represents a single revocation record.
#[derive(Debug, Clone)]
pub struct RevocationEntry {
    pub actor_id: String,
    pub reason: RevocationReason,
    pub revoked_at: u64,
}

/// In-memory revocation list.
///
/// This structure functions similarly to a CRL but is optimized for fast
/// lookup and deterministic export.
pub struct RevocationList {
    revoked: HashMap<String, RevocationEntry>,
}

impl Default for RevocationList {
    fn default() -> Self {
        Self::new()
    }
}

impl RevocationList {
    /// Creates an empty revocation list.
    pub fn new() -> Self {
        Self {
            revoked: HashMap::new(),
        }
    }

    /// Revokes an actor identity.
    ///
    /// If the actor is already revoked, the call is ignored.
    pub fn revoke(&mut self, actor_id: &str, reason: RevocationReason) {
        if self.revoked.contains_key(actor_id) {
            return;
        }

        let entry = RevocationEntry {
            actor_id: actor_id.to_string(),
            reason,
            revoked_at: current_time(),
        };

        self.revoked.insert(actor_id.to_string(), entry);
    }

    /// Returns true if an actor identity has been revoked.
    pub fn is_revoked(&self, actor_id: &str) -> bool {
        self.revoked.contains_key(actor_id)
    }

    /// Returns revocation metadata if present.
    pub fn get(&self, actor_id: &str) -> Option<&RevocationEntry> {
        self.revoked.get(actor_id)
    }

    /// Returns the number of revoked identities.
    pub fn len(&self) -> usize {
        self.revoked.len()
    }

    /// Returns true if the revocation list is empty.
    pub fn is_empty(&self) -> bool {
        self.revoked.is_empty()
    }

    /// Deterministically exports the revoked actor IDs.
    ///
    /// Useful for hashing or gossip synchronization.
    pub fn export_actor_ids(&self) -> Vec<String> {
        let mut list: Vec<String> = self.revoked.keys().cloned().collect();
        list.sort();
        list
    }

    /// Returns a deterministic set of revoked actors.
    pub fn export_set(&self) -> HashSet<String> {
        self.revoked.keys().cloned().collect()
    }
}

/// Returns current UNIX timestamp in seconds.
///
/// If system time is invalid, zero is returned.
fn current_time() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(v) => v.as_secs(),
        Err(_) => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revoke_and_query_actor() {
        let mut crl = RevocationList::new();

        crl.revoke("AOXC-VAL-EU-1234", RevocationReason::KeyCompromise);

        assert!(crl.is_revoked("AOXC-VAL-EU-1234"));
        assert_eq!(crl.len(), 1);
    }

    #[test]
    fn duplicate_revoke_is_ignored() {
        let mut crl = RevocationList::new();

        crl.revoke("node-1", RevocationReason::OperatorAction);
        crl.revoke("node-1", RevocationReason::OperatorAction);

        assert_eq!(crl.len(), 1);
    }

    #[test]
    fn deterministic_export() {
        let mut crl = RevocationList::new();

        crl.revoke("b", RevocationReason::OperatorAction);
        crl.revoke("a", RevocationReason::OperatorAction);

        let list = crl.export_actor_ids();

        assert_eq!(list, vec!["a".to_string(), "b".to_string()]);
    }
}
