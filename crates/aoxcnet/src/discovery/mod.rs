// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::collections::{BTreeMap, BTreeSet};
use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

/// Public RPC envelope advertised by a node for operator and application access.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RpcSurface {
    /// Canonical HTTP endpoint used for health and status requests.
    pub http_endpoint: String,

    /// Canonical JSON-RPC endpoint used by wallets and automation clients.
    pub jsonrpc_endpoint: String,

    /// Indicates whether the endpoint profile satisfies elevated
    /// post-quantum readiness policy expectations.
    pub quantum_ready: bool,
}

impl RpcSurface {
    /// Validates operator-facing RPC endpoints under strict discovery hygiene.
    pub fn validate(&self) -> Result<(), &'static str> {
        validate_endpoint(&self.http_endpoint, false)?;
        validate_endpoint(&self.jsonrpc_endpoint, true)?;
        Ok(())
    }
}

/// Peer candidate observed by discovery.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerCandidate {
    pub peer_id: String,
    pub advertise_addr: String,
    pub score: i32,
    pub source: String,
    pub last_seen_unix: u64,

    /// Genesis fingerprint claimed by the candidate.
    ///
    /// Nodes with mismatched fingerprints are rejected from auto-formation
    /// groups to keep each mesh bounded to a single canonical genesis.
    pub genesis_fingerprint: String,

    /// RPC surface metadata published by the candidate.
    pub rpc: RpcSurface,
}

impl PeerCandidate {
    /// Validates discovery candidate fields before admission.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.peer_id.trim().is_empty() {
            return Err("PEER_ID_EMPTY");
        }

        if self.source.trim().is_empty() {
            return Err("SOURCE_EMPTY");
        }

        if self.genesis_fingerprint.trim().is_empty() {
            return Err("GENESIS_FINGERPRINT_EMPTY");
        }

        if self.advertise_addr.parse::<SocketAddr>().is_err() {
            return Err("ADVERTISE_ADDR_INVALID");
        }

        self.rpc.validate()?;
        Ok(())
    }
}

/// Deterministic discovery table supporting seed registration, genesis-scoped
/// peer admission, and bootstrap selection.
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

    /// Registers a candidate only when it belongs to the local genesis cohort.
    ///
    /// Returns `true` when the candidate is accepted into the table.
    pub fn add_seed_for_genesis(
        &mut self,
        local_genesis_fingerprint: &str,
        candidate: PeerCandidate,
    ) -> bool {
        if candidate.validate().is_err() {
            return false;
        }

        if self.denylist.contains(&candidate.peer_id) {
            return false;
        }

        if candidate.genesis_fingerprint != local_genesis_fingerprint {
            return false;
        }

        self.seeds.insert(candidate.peer_id.clone(), candidate);
        true
    }

    /// Backward-compatible seed insertion that bypasses genesis filtering.
    pub fn add_seed(&mut self, candidate: PeerCandidate) {
        if candidate.validate().is_ok() && !self.denylist.contains(&candidate.peer_id) {
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
                .then_with(|| b.rpc.quantum_ready.cmp(&a.rpc.quantum_ready))
                .then_with(|| a.peer_id.cmp(&b.peer_id))
        });
        values.truncate(limit);
        values
    }

    /// Returns RPC surfaces of currently selected peers, intended for node
    /// auto-configuration and client bootstrapping.
    #[must_use]
    pub fn select_bootstrap_rpc_surfaces(&self, limit: usize) -> Vec<RpcSurface> {
        self.select_bootstrap_peers(limit)
            .into_iter()
            .map(|candidate| candidate.rpc)
            .collect()
    }
}

fn validate_endpoint(endpoint: &str, require_jsonrpc_suffix: bool) -> Result<(), &'static str> {
    let endpoint = endpoint.trim();
    if endpoint.is_empty() {
        return Err("RPC_ENDPOINT_EMPTY");
    }

    let protocol_ok = endpoint.starts_with("https://")
        || endpoint.starts_with("http://")
        || endpoint.starts_with("wss://")
        || endpoint.starts_with("ws://");
    if !protocol_ok {
        return Err("RPC_ENDPOINT_SCHEME_INVALID");
    }

    if endpoint.chars().any(char::is_whitespace) {
        return Err("RPC_ENDPOINT_WHITESPACE_FORBIDDEN");
    }

    if require_jsonrpc_suffix && !endpoint.ends_with("/jsonrpc") {
        return Err("JSONRPC_ENDPOINT_SUFFIX_INVALID");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{DiscoveryTable, PeerCandidate, RpcSurface};

    fn peer(peer_id: &str, score: i32, genesis: &str, quantum_ready: bool) -> PeerCandidate {
        PeerCandidate {
            peer_id: peer_id.to_string(),
            advertise_addr: format!("10.0.0.{}:2727", peer_id.len()),
            score,
            source: "static".to_string(),
            last_seen_unix: 100,
            genesis_fingerprint: genesis.to_string(),
            rpc: RpcSurface {
                http_endpoint: format!("http://{}.mesh.local:28657", peer_id),
                jsonrpc_endpoint: format!("http://{}.mesh.local:28657/jsonrpc", peer_id),
                quantum_ready,
            },
        }
    }

    #[test]
    fn higher_score_candidates_are_selected_first() {
        let mut table = DiscoveryTable::new();
        table.add_seed(peer("b", 5, "g1", false));
        table.add_seed(peer("a", 7, "g1", true));

        let selected = table.select_bootstrap_peers(1);
        assert_eq!(selected[0].peer_id, "a");
    }

    #[test]
    fn add_seed_for_genesis_rejects_mismatched_genesis() {
        let mut table = DiscoveryTable::new();
        assert!(table.add_seed_for_genesis("g1", peer("a", 4, "g1", true)));
        assert!(!table.add_seed_for_genesis("g1", peer("b", 7, "g2", true)));

        let selected = table.select_bootstrap_peers(4);
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].peer_id, "a");
    }

    #[test]
    fn bootstrap_rpc_surfaces_are_returned_in_selection_order() {
        let mut table = DiscoveryTable::new();
        table.add_seed(peer("a", 10, "g1", false));
        table.add_seed(peer("b", 10, "g1", true));

        let rpc = table.select_bootstrap_rpc_surfaces(2);
        assert_eq!(rpc.len(), 2);
        assert!(rpc[0].quantum_ready);
        assert!(!rpc[1].quantum_ready);
    }

    #[test]
    fn candidate_with_invalid_advertise_addr_is_rejected() {
        let mut table = DiscoveryTable::new();
        let mut bad = peer("a", 5, "g1", true);
        bad.advertise_addr = "bad-address".to_string();
        assert!(!table.add_seed_for_genesis("g1", bad));
    }

    #[test]
    fn candidate_with_invalid_jsonrpc_endpoint_is_rejected() {
        let mut table = DiscoveryTable::new();
        let mut bad = peer("a", 5, "g1", true);
        bad.rpc.jsonrpc_endpoint = "https://node.mesh.local/rpc".to_string();
        assert!(!table.add_seed_for_genesis("g1", bad));
    }
}
