use consensus::messages::ConsensusMessage;

/// Gossip engine responsible for propagating and receiving consensus messages
/// across peers.
///
/// This compatibility-preserving implementation keeps the node layer stable
/// while the networking subsystem is being reorganized.
#[derive(Debug, Default, Clone)]
pub struct GossipEngine;

impl GossipEngine {
    /// Creates a new gossip engine instance.
    pub fn new() -> Self {
        Self
    }

    /// Broadcasts a consensus message to connected peers.
    ///
    /// Transport integration is intentionally deferred until the p2p layer is
    /// finalized.
    pub fn broadcast(&self, _msg: ConsensusMessage) {
        // TODO: integrate with p2p transport and peer routing.
    }

    /// Receives the next available consensus message from the gossip layer.
    ///
    /// The compatibility stub returns no message until the inbound transport
    /// pipeline is implemented.
    pub fn receive(&mut self) -> Option<ConsensusMessage> {
        None
    }
}
