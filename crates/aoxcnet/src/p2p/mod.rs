use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

use aoxcunity::messages::ConsensusMessage;

use crate::config::SecurityMode;
use crate::error::NetworkError;
use crate::gossip::peer::Peer;

#[derive(Debug, Clone)]
pub struct SessionTicket {
    pub peer_id: String,
    pub cert_fingerprint: String,
    pub established_at_unix: u64,
    pub replay_window_nonce: u64,
}

/// In-memory transport shell for deterministic tests and CLI smoke validation.
///
/// This module enforces secure admission checks and provides a drop-in layer
/// for future socket/quic transport integration.
#[derive(Debug, Clone)]
pub struct P2PNetwork {
    security_mode: SecurityMode,
    peers: HashMap<String, Peer>,
    sessions: HashMap<String, SessionTicket>,
    inbound: VecDeque<ConsensusMessage>,
    max_peers: usize,
}

impl P2PNetwork {
    pub fn new(security_mode: SecurityMode, max_peers: usize) -> Self {
        Self {
            security_mode,
            peers: HashMap::new(),
            sessions: HashMap::new(),
            inbound: VecDeque::new(),
            max_peers,
        }
    }

    pub fn security_mode(&self) -> SecurityMode {
        self.security_mode
    }

    pub fn registered_peers(&self) -> usize {
        self.peers.len()
    }

    pub fn active_sessions(&self) -> usize {
        self.sessions.len()
    }

    pub fn register_peer(&mut self, peer: Peer) -> Result<(), NetworkError> {
        if self.peers.contains_key(&peer.id) {
            return Err(NetworkError::PeerAlreadyRegistered(peer.id));
        }

        if self.peers.len() >= self.max_peers {
            return Err(NetworkError::PeerDisconnected);
        }

        if self.security_mode != SecurityMode::Insecure && !peer.validate_certificate() {
            return Err(NetworkError::CertificateValidationFailed(
                "certificate expired or not yet valid".to_string(),
            ));
        }

        self.peers.insert(peer.id.clone(), peer);
        Ok(())
    }

    pub fn establish_session(&mut self, peer_id: &str) -> Result<SessionTicket, NetworkError> {
        let peer = self
            .peers
            .get(peer_id)
            .ok_or_else(|| NetworkError::UnknownPeer(peer_id.to_string()))?;

        let now = unix_now();
        let ticket = SessionTicket {
            peer_id: peer.id.clone(),
            cert_fingerprint: peer.cert_fingerprint.clone(),
            established_at_unix: now,
            replay_window_nonce: now ^ 0xA0C0u64,
        };

        self.sessions.insert(peer.id.clone(), ticket.clone());
        Ok(ticket)
    }

    pub fn broadcast_secure(
        &mut self,
        from_peer_id: &str,
        msg: ConsensusMessage,
    ) -> Result<(), NetworkError> {
        if self.security_mode != SecurityMode::Insecure && !self.sessions.contains_key(from_peer_id)
        {
            return Err(NetworkError::UnknownPeer(from_peer_id.to_string()));
        }

        self.inbound.push_back(msg);
        Ok(())
    }

    pub fn receive(&mut self) -> Option<ConsensusMessage> {
        self.inbound.pop_front()
    }

    pub fn broadcast_compat(&mut self, msg: ConsensusMessage) {
        self.inbound.push_back(msg);
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::P2PNetwork;
    use crate::config::SecurityMode;
    use crate::gossip::peer::{NodeCertificate, Peer};
    use aoxcunity::messages::ConsensusMessage;
    use aoxcunity::vote::{Vote, VoteKind};

    #[test]
    fn rejects_expired_certificates_in_secure_mode() {
        let mut net = P2PNetwork::new(SecurityMode::AuditStrict, 16);
        let cert = NodeCertificate {
            subject: "node-1".to_string(),
            issuer: "AOXC-ROOT".to_string(),
            valid_from_unix: 1,
            valid_until_unix: 2,
            serial: "s1".to_string(),
        };

        let peer = Peer::new("node-1", "10.0.0.1:2727", cert);
        assert!(net.register_peer(peer).is_err());
    }

    #[test]
    fn secure_broadcast_requires_session() {
        let mut net = P2PNetwork::new(SecurityMode::MutualAuth, 16);
        let cert = NodeCertificate {
            subject: "node-1".to_string(),
            issuer: "AOXC-ROOT".to_string(),
            valid_from_unix: 1,
            valid_until_unix: u64::MAX,
            serial: "s1".to_string(),
        };
        let peer = Peer::new("node-1", "10.0.0.1:2727", cert);
        net.register_peer(peer).expect("peer should register");
        net.establish_session("node-1")
            .expect("session should be established");

        let vote = Vote {
            voter: [1u8; 32],
            block_hash: [2u8; 32],
            height: 1,
            round: 0,
            kind: VoteKind::Prepare,
        };
        net.broadcast_secure("node-1", ConsensusMessage::Vote(vote))
            .expect("broadcast should be accepted");

        assert!(net.receive().is_some());
    }
}
