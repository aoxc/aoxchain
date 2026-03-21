use std::collections::VecDeque;

use aoxcunity::messages::ConsensusMessage;
use blake3::Hasher;

use crate::config::NetworkConfig;
use crate::error::NetworkError;
use crate::gossip::peer::Peer;
use crate::p2p::{P2PNetwork, ProtocolEnvelope};

/// Gossip engine responsible for propagating and receiving consensus messages.
#[derive(Debug, Clone)]
pub struct GossipEngine {
    network: P2PNetwork,
    recent_message_ids: VecDeque<String>,
    max_recent_ids: usize,
}

impl GossipEngine {
    #[must_use]
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            network: P2PNetwork::new(config.clone()),
            recent_message_ids: VecDeque::new(),
            max_recent_ids: config.max_gossip_batch.saturating_mul(4),
        }
    }

    pub fn register_peer(&mut self, peer: Peer) -> Result<(), NetworkError> {
        self.network.register_peer(peer)
    }

    pub fn establish_session(&mut self, peer_id: &str) -> Result<(), NetworkError> {
        self.network.establish_session(peer_id).map(|_| ())
    }

    pub fn broadcast_from_peer(
        &mut self,
        peer_id: &str,
        message: ConsensusMessage,
    ) -> Result<ProtocolEnvelope, NetworkError> {
        let message_id = digest_message(&message)?;
        if self
            .recent_message_ids
            .iter()
            .any(|existing| existing == &message_id)
        {
            return Err(NetworkError::ReplayDetected);
        }
        let envelope = self.network.broadcast_secure(peer_id, message)?;
        self.recent_message_ids.push_back(message_id);
        while self.recent_message_ids.len() > self.max_recent_ids {
            self.recent_message_ids.pop_front();
        }
        Ok(envelope)
    }

    pub fn receive(&mut self) -> Option<ConsensusMessage> {
        self.network.receive()
    }
}

fn digest_message(message: &ConsensusMessage) -> Result<String, NetworkError> {
    let bytes =
        serde_json::to_vec(message).map_err(|e| NetworkError::Serialization(e.to_string()))?;
    let mut hasher = Hasher::new();
    hasher.update(b"AOXC-GOSSIP-MESSAGE-V1");
    hasher.update(&bytes);
    Ok(hasher.finalize().to_hex().to_string())
}
