// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

/// Peer candidate observed by discovery.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerCandidate {
    pub peer_id: String,
    pub advertise_addr: String,
    pub score: i32,
    pub source: String,
    pub last_seen_unix: u64,
}

/// Deterministic discovery table supporting seed registration and bootstrap selection.
#[derive(Debug, Clone, Default)]
pub struct DiscoveryTable {
    seeds: BTreeMap<String, PeerCandidate>,
    denylist: BTreeSet<String>,
}

impl DiscoveryTable {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_seed(&mut self, candidate: PeerCandidate) {
        if !self.denylist.contains(&candidate.peer_id) {
            self.seeds.insert(candidate.peer_id.clone(), candidate);
        }
    }

    pub fn observe(&mut self, peer_id: &str, score_delta: i32, last_seen_unix: u64) {
        if let Some(candidate) = self.seeds.get_mut(peer_id) {
            candidate.score = candidate.score.saturating_add(score_delta);
            candidate.last_seen_unix = last_seen_unix;
        }
    }

    pub fn deny(&mut self, peer_id: impl Into<String>) {
        let peer_id = peer_id.into();
        self.denylist.insert(peer_id.clone());
        self.seeds.remove(&peer_id);
    }

    #[must_use]
    pub fn select_bootstrap_peers(&self, limit: usize) -> Vec<PeerCandidate> {
        let mut values = self.seeds.values().cloned().collect::<Vec<_>>();
        values.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| b.last_seen_unix.cmp(&a.last_seen_unix))
                .then_with(|| a.peer_id.cmp(&b.peer_id))
        });
        values.truncate(limit);
        values
    }
}

#[cfg(test)]
mod tests {
    use super::{DiscoveryTable, PeerCandidate};

    #[test]
    fn higher_score_candidates_are_selected_first() {
        let mut table = DiscoveryTable::new();
        table.add_seed(PeerCandidate {
            peer_id: "b".to_string(),
            advertise_addr: "10.0.0.2:2727".to_string(),
            score: 5,
            source: "static".to_string(),
            last_seen_unix: 10,
        });
        table.add_seed(PeerCandidate {
            peer_id: "a".to_string(),
            advertise_addr: "10.0.0.1:2727".to_string(),
            score: 7,
            source: "static".to_string(),
            last_seen_unix: 20,
        });

        let selected = table.select_bootstrap_peers(1);
        assert_eq!(selected[0].peer_id, "a");
    }
}
