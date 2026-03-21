use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

use aoxcunity::messages::ConsensusMessage;
use blake3::Hasher;
use serde::{Deserialize, Serialize};

use crate::config::{NetworkConfig, SecurityMode};
use crate::error::NetworkError;
use crate::gossip::peer::Peer;
use crate::metrics::{NetworkMetrics, NetworkMetricsSnapshot};

/// Canonical protocol framing version enforced by the in-memory AOXC transport.
const PROTOCOL_ENVELOPE_VERSION: u16 = 1;

/// Domain-separated BLAKE3 namespace for session identifiers.
const SESSION_HASH_DOMAIN: &[u8] = b"AOXC-NET-SESSION-V2";

/// Domain-separated BLAKE3 namespace for payload integrity.
const PAYLOAD_HASH_DOMAIN: &[u8] = b"AOXC-NET-PAYLOAD-V2";

/// Domain-separated BLAKE3 namespace for frame integrity.
const FRAME_HASH_DOMAIN: &[u8] = b"AOXC-NET-FRAME-V2";

/// Represents an authenticated peer session established by the in-memory P2P
/// runtime.
///
/// The session ticket binds a peer identity to a certificate fingerprint,
/// deterministic session identifier, replay nonce stream, and finite trust
/// window. The structure is intentionally serializable so it can be surfaced
/// in diagnostics, test fixtures, and future persistence adapters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionTicket {
    /// Canonical peer identifier recognized by the runtime.
    pub peer_id: String,

    /// Fingerprint of the certificate accepted during session establishment.
    pub cert_fingerprint: String,

    /// UNIX timestamp at which the session was established.
    pub established_at_unix: u64,

    /// Session-scoped replay-protection nonce.
    pub replay_window_nonce: u64,

    /// Deterministically derived session identifier.
    pub session_id: String,

    /// UNIX timestamp after which the session must no longer be trusted.
    pub expires_at_unix: u64,
}

/// Canonical in-memory protocol frame used by the AOXC network shell.
///
/// The envelope binds the payload to a chain-domain label, protocol serial,
/// session identity, replay nonce, issuance metadata, and deterministic
/// integrity hashes. This structure is suitable for future transport-adapter
/// hardening and audit-oriented diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolEnvelope {
    /// Protocol framing version for forward compatibility.
    pub protocol_version: u16,

    /// Canonical AOXC chain identifier bound to the frame.
    pub chain_id: String,

    /// Canonical AOXC numeric protocol serial bound to the frame.
    pub protocol_serial: u64,

    /// Peer identifier from which the frame originated.
    pub peer_id: String,

    /// Session identifier under which the frame was emitted.
    pub session_id: String,

    /// Session-scoped replay nonce used for duplicate rejection.
    pub nonce: u64,

    /// UNIX timestamp at which the frame was issued.
    pub issued_at_unix: u64,

    /// UNIX timestamp after which the frame must no longer be trusted.
    pub expires_at_unix: u64,

    /// Deterministic payload integrity hash.
    pub payload_hash_hex: String,

    /// Deterministic frame integrity hash.
    pub frame_hash_hex: String,

    /// Consensus payload transported by this envelope.
    pub payload: ConsensusMessage,
}

impl ProtocolEnvelope {
    /// Creates a new canonical protocol envelope from a trusted session ticket.
    pub fn new(
        chain_id: &str,
        protocol_serial: u64,
        ticket: &SessionTicket,
        payload: ConsensusMessage,
        issued_at_unix: u64,
    ) -> Result<Self, NetworkError> {
        if chain_id.trim().is_empty() {
            return Err(NetworkError::ProtocolMismatch(
                "canonical chain identifier must not be empty".to_string(),
            ));
        }

        if protocol_serial == 0 {
            return Err(NetworkError::ProtocolMismatch(
                "canonical protocol serial must not be zero".to_string(),
            ));
        }

        if issued_at_unix > ticket.expires_at_unix {
            return Err(NetworkError::HandshakeTimeout);
        }

        let payload_hash_hex = digest_payload(&payload)?;
        let frame_hash_hex = derive_frame_hash(
            chain_id,
            protocol_serial,
            &ticket.peer_id,
            &ticket.session_id,
            ticket.replay_window_nonce,
            issued_at_unix,
            ticket.expires_at_unix,
            &payload_hash_hex,
        );

        Ok(Self {
            protocol_version: PROTOCOL_ENVELOPE_VERSION,
            chain_id: chain_id.to_string(),
            protocol_serial,
            peer_id: ticket.peer_id.clone(),
            session_id: ticket.session_id.clone(),
            nonce: ticket.replay_window_nonce,
            issued_at_unix,
            expires_at_unix: ticket.expires_at_unix,
            payload_hash_hex,
            frame_hash_hex,
            payload,
        })
    }

    /// Verifies the envelope against canonical protocol identity and integrity
    /// expectations.
    pub fn verify_against(
        &self,
        canonical_chain_id: &str,
        canonical_protocol_serial: u64,
    ) -> Result<(), NetworkError> {
        if self.protocol_version != PROTOCOL_ENVELOPE_VERSION {
            return Err(NetworkError::ProtocolMismatch(
                "unexpected protocol envelope version".to_string(),
            ));
        }

        if self.chain_id != canonical_chain_id {
            return Err(NetworkError::ProtocolMismatch(
                "envelope chain identifier does not match local canonical chain identity"
                    .to_string(),
            ));
        }

        if self.protocol_serial != canonical_protocol_serial {
            return Err(NetworkError::ProtocolMismatch(
                "envelope protocol serial does not match local canonical protocol serial"
                    .to_string(),
            ));
        }

        if self.issued_at_unix > self.expires_at_unix {
            return Err(NetworkError::ProtocolMismatch(
                "envelope issuance timestamp exceeds expiry".to_string(),
            ));
        }

        let expected_payload_hash = digest_payload(&self.payload)?;
        if self.payload_hash_hex != expected_payload_hash {
            return Err(NetworkError::ProtocolMismatch(
                "payload integrity hash mismatch".to_string(),
            ));
        }

        let expected_frame_hash = derive_frame_hash(
            &self.chain_id,
            self.protocol_serial,
            &self.peer_id,
            &self.session_id,
            self.nonce,
            self.issued_at_unix,
            self.expires_at_unix,
            &self.payload_hash_hex,
        );

        if self.frame_hash_hex != expected_frame_hash {
            return Err(NetworkError::ProtocolMismatch(
                "frame integrity hash mismatch".to_string(),
            ));
        }

        Ok(())
    }
}

/// In-memory secure transport shell for deterministic tests, smoke validation,
/// and future transport-adapter integration.
///
/// This implementation prioritizes predictable security behavior over transport
/// sophistication. It enforces peer admission checks, authenticated session
/// establishment, replay detection, bounded session lifetime, explicit ban
/// handling, deterministic cache eviction, and frame-size enforcement.
#[derive(Debug, Clone)]
pub struct P2PNetwork {
    config: NetworkConfig,
    peers: HashMap<String, Peer>,
    sessions: HashMap<String, SessionTicket>,
    replay_cache: HashSet<String>,
    replay_order: VecDeque<String>,
    inbound: VecDeque<ProtocolEnvelope>,
    banned_until_unix: HashMap<String, u64>,
    metrics: NetworkMetrics,
}

impl P2PNetwork {
    /// Creates a new in-memory AOXC network runtime bound to the supplied
    /// network configuration.
    #[must_use]
    pub fn new(config: NetworkConfig) -> Self {
        debug_assert!(
            config.validate().is_ok(),
            "P2PNetwork::new received an invalid NetworkConfig"
        );

        Self {
            config,
            peers: HashMap::new(),
            sessions: HashMap::new(),
            replay_cache: HashSet::new(),
            replay_order: VecDeque::new(),
            inbound: VecDeque::new(),
            banned_until_unix: HashMap::new(),
            metrics: NetworkMetrics::default(),
        }
    }

    /// Creates a new network runtime after validating the supplied config.
    pub fn new_checked(config: NetworkConfig) -> Result<Self, NetworkError> {
        config
            .validate()
            .map_err(|code| NetworkError::InvalidConfig(code.to_string()))?;
        Ok(Self::new(config))
    }

    /// Returns the active security mode.
    #[must_use]
    pub fn security_mode(&self) -> SecurityMode {
        self.config.security_mode
    }

    /// Returns the number of currently registered peers.
    #[must_use]
    pub fn registered_peers(&self) -> usize {
        self.peers.len()
    }

    /// Returns the number of currently active authenticated sessions.
    #[must_use]
    pub fn active_sessions(&self) -> usize {
        self.sessions.len()
    }

    /// Returns immutable access to current metrics.
    #[must_use]
    pub fn metrics(&self) -> &NetworkMetrics {
        &self.metrics
    }

    /// Returns a stable snapshot of current metrics.
    #[must_use]
    pub fn metrics_snapshot(&self) -> NetworkMetricsSnapshot {
        self.metrics.snapshot()
    }

    /// Returns immutable access to the runtime configuration.
    #[must_use]
    pub fn config(&self) -> &NetworkConfig {
        &self.config
    }

    /// Registers a peer after capacity, certificate, policy, and ban-state
    /// checks.
    pub fn register_peer(&mut self, peer: Peer) -> Result<(), NetworkError> {
        self.evict_expired_bans();

        if self.peers.contains_key(&peer.id) {
            self.metrics.rejected_peers = self.metrics.rejected_peers.saturating_add(1);
            return Err(NetworkError::PeerAlreadyRegistered(peer.id));
        }

        if self.is_banned(&peer.id) {
            self.metrics.rejected_peers = self.metrics.rejected_peers.saturating_add(1);
            return Err(NetworkError::PeerBanned(peer.id));
        }

        if self.peers.len() >= self.config.max_peers_total() {
            self.metrics.rejected_peers = self.metrics.rejected_peers.saturating_add(1);
            return Err(NetworkError::PeerAdmissionDenied(
                "peer capacity exceeded".to_string(),
            ));
        }

        peer.validate_certificate(&self.config)?;

        self.peers.insert(peer.id.clone(), peer);
        self.metrics.accepted_peers = self.metrics.accepted_peers.saturating_add(1);

        Ok(())
    }

    /// Establishes an authenticated session for a previously registered peer.
    pub fn establish_session(&mut self, peer_id: &str) -> Result<SessionTicket, NetworkError> {
        self.evict_expired_bans();

        if self.is_banned(peer_id) {
            self.metrics.failed_handshakes = self.metrics.failed_handshakes.saturating_add(1);
            return Err(NetworkError::PeerBanned(peer_id.to_string()));
        }

        let peer = self
            .peers
            .get(peer_id)
            .ok_or_else(|| NetworkError::UnknownPeer(peer_id.to_string()))?;

        peer.validate_certificate(&self.config)?;

        let now = unix_now();
        let ticket = SessionTicket {
            peer_id: peer.id.clone(),
            cert_fingerprint: peer.cert_fingerprint.clone(),
            established_at_unix: now,
            replay_window_nonce: initial_nonce(now),
            session_id: derive_session_id(peer_id, &peer.cert_fingerprint, now),
            expires_at_unix: now.saturating_add(session_lifetime_secs(&self.config)),
        };

        self.sessions.insert(peer.id.clone(), ticket.clone());
        self.metrics.active_sessions = self.sessions.len() as u64;

        Ok(ticket)
    }

    /// Securely broadcasts a payload from an authenticated peer session.
    ///
    /// The method rejects unknown peers, banned peers, expired sessions, frame
    /// oversize conditions, protocol mismatches, and replayed session-nonce
    /// pairs. It returns the canonical protocol envelope pushed into the
    /// inbound queue.
    pub fn broadcast_secure(
        &mut self,
        from_peer_id: &str,
        payload: ConsensusMessage,
    ) -> Result<ProtocolEnvelope, NetworkError> {
        self.evict_expired_bans();

        if self.is_banned(from_peer_id) {
            self.metrics.rejected_peers = self.metrics.rejected_peers.saturating_add(1);
            return Err(NetworkError::PeerBanned(from_peer_id.to_string()));
        }

        let now = unix_now();
        let security_mode = self.config.security_mode;
        let canonical_chain_id = self.config.interop.canonical_chain_id().to_string();
        let canonical_protocol_serial = self.config.interop.canonical_protocol_serial();

        let envelope = {
            let ticket = self
                .sessions
                .get_mut(from_peer_id)
                .ok_or_else(|| NetworkError::UnknownPeer(from_peer_id.to_string()))?;

            if security_mode != SecurityMode::Insecure && now > ticket.expires_at_unix {
                self.metrics.failed_handshakes = self.metrics.failed_handshakes.saturating_add(1);
                return Err(NetworkError::HandshakeTimeout);
            }

            let envelope = ProtocolEnvelope::new(
                &canonical_chain_id,
                canonical_protocol_serial,
                ticket,
                payload,
                now,
            )?;

            ticket.replay_window_nonce = ticket.replay_window_nonce.saturating_add(1);
            envelope
        };

        envelope.verify_against(&canonical_chain_id, canonical_protocol_serial)?;

        let replay_key = replay_key(&envelope.session_id, envelope.nonce);
        if self.replay_cache.contains(&replay_key) {
            self.metrics.replay_detections = self.metrics.replay_detections.saturating_add(1);
            return Err(NetworkError::ReplayDetected);
        }

        let encoded = serde_json::to_vec(&envelope)
            .map_err(|error| NetworkError::Serialization(error.to_string()))?;

        if encoded.len() > self.config.max_frame_bytes {
            return Err(NetworkError::FrameTooLarge);
        }

        self.replay_cache.insert(replay_key.clone());
        self.replay_order.push_back(replay_key);
        self.trim_replay_cache();

        self.metrics.frames_out = self.metrics.frames_out.saturating_add(1);
        self.metrics.bytes_out = self
            .metrics
            .bytes_out
            .saturating_add(encoded.len() as u64);
        self.metrics.gossip_messages = self.metrics.gossip_messages.saturating_add(1);

        self.inbound.push_back(envelope.clone());
        Ok(envelope)
    }

    /// Receives the next consensus payload from the inbound queue, if any.
    ///
    /// Incoming frames are re-verified against canonical chain identity and
    /// integrity expectations before the payload is released to callers.
    pub fn receive(&mut self) -> Option<ConsensusMessage> {
        let envelope = self.inbound.pop_front()?;
        let canonical_chain_id = self.config.interop.canonical_chain_id().to_string();
        let canonical_protocol_serial = self.config.interop.canonical_protocol_serial();

        if envelope
            .verify_against(&canonical_chain_id, canonical_protocol_serial)
            .is_err()
        {
            return None;
        }

        if let Ok(encoded) = serde_json::to_vec(&envelope) {
            self.metrics.frames_in = self.metrics.frames_in.saturating_add(1);
            self.metrics.bytes_in = self
                .metrics
                .bytes_in
                .saturating_add(encoded.len() as u64);
        }

        Some(envelope.payload)
    }

    /// Bans the peer for the configured ban window and tears down any active
    /// session associated with it.
    pub fn ban_peer(&mut self, peer_id: &str) {
        let until = unix_now().saturating_add(self.config.peer_ban_secs);
        self.banned_until_unix.insert(peer_id.to_string(), until);
        self.sessions.remove(peer_id);
        self.metrics.banned_peers = self.metrics.banned_peers.saturating_add(1);
        self.metrics.active_sessions = self.sessions.len() as u64;
    }

    /// Removes a peer and any active session bound to it.
    pub fn remove_peer(&mut self, peer_id: &str) {
        self.peers.remove(peer_id);
        self.sessions.remove(peer_id);
        self.metrics.active_sessions = self.sessions.len() as u64;
    }

    /// Returns `true` when the peer is presently inside an active ban window.
    #[must_use]
    fn is_banned(&self, peer_id: &str) -> bool {
        self.banned_until_unix
            .get(peer_id)
            .map(|until| unix_now() <= *until)
            .unwrap_or(false)
    }

    /// Evicts expired ban records so the deny map cannot grow indefinitely.
    fn evict_expired_bans(&mut self) {
        let now = unix_now();
        self.banned_until_unix.retain(|_, until| *until >= now);
    }

    /// Trims the replay cache deterministically by evicting the oldest replay
    /// keys first.
    fn trim_replay_cache(&mut self) {
        while self.replay_cache.len() > self.config.replay_window_size {
            let Some(oldest_key) = self.replay_order.pop_front() else {
                break;
            };
            self.replay_cache.remove(&oldest_key);
        }
    }
}

/// Returns the session lifetime in seconds derived from the configured idle
/// timeout. A minimum lifetime of one second is enforced.
#[must_use]
fn session_lifetime_secs(config: &NetworkConfig) -> u64 {
    (config.idle_timeout_ms / 1_000).max(1)
}

/// Returns the initial replay nonce derived from current time.
///
/// The nonce derivation intentionally avoids fixed zero initialization so that
/// independent sessions started at different times do not share the same
/// opening nonce.
#[must_use]
fn initial_nonce(now: u64) -> u64 {
    now ^ 0xA0C0_A0C0_A0C0_A0C0_u64
}

/// Derives a deterministic session identifier from peer identity,
/// certificate fingerprint, and establishment timestamp.
#[must_use]
fn derive_session_id(peer_id: &str, cert_fingerprint: &str, unix_ts: u64) -> String {
    let mut hasher = Hasher::new();
    hasher.update(SESSION_HASH_DOMAIN);
    hasher.update(peer_id.as_bytes());
    hasher.update(cert_fingerprint.as_bytes());
    hasher.update(&unix_ts.to_be_bytes());
    hasher.finalize().to_hex().to_string()
}

/// Returns a deterministic payload integrity hash.
///
/// BLAKE3 is used here because it provides strong modern cryptographic
/// properties, excellent performance, and deterministic output suitable for
/// protocol binding and replay-safe framing.
fn digest_payload(payload: &ConsensusMessage) -> Result<String, NetworkError> {
    let encoded = serde_json::to_vec(payload)
        .map_err(|error| NetworkError::Serialization(error.to_string()))?;

    let mut hasher = Hasher::new();
    hasher.update(PAYLOAD_HASH_DOMAIN);
    hasher.update(&encoded);
    Ok(hasher.finalize().to_hex().to_string())
}

/// Derives a deterministic frame integrity hash from envelope metadata and
/// payload integrity hash.
#[must_use]
fn derive_frame_hash(
    chain_id: &str,
    protocol_serial: u64,
    peer_id: &str,
    session_id: &str,
    nonce: u64,
    issued_at_unix: u64,
    expires_at_unix: u64,
    payload_hash_hex: &str,
) -> String {
    let mut hasher = Hasher::new();
    hasher.update(FRAME_HASH_DOMAIN);
    hasher.update(chain_id.as_bytes());
    hasher.update(&protocol_serial.to_be_bytes());
    hasher.update(peer_id.as_bytes());
    hasher.update(session_id.as_bytes());
    hasher.update(&nonce.to_be_bytes());
    hasher.update(&issued_at_unix.to_be_bytes());
    hasher.update(&expires_at_unix.to_be_bytes());
    hasher.update(payload_hash_hex.as_bytes());
    hasher.finalize().to_hex().to_string()
}

/// Returns a canonical replay-cache key for a session and nonce pair.
#[must_use]
fn replay_key(session_id: &str, nonce: u64) -> String {
    format!("{session_id}:{nonce}")
}

/// Returns the current UNIX timestamp in seconds.
///
/// In the unlikely event that system time is observed before the UNIX epoch,
/// the function returns zero instead of panicking.
#[must_use]
fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{digest_payload, P2PNetwork, ProtocolEnvelope, SessionTicket};
    use crate::config::{ExternalDomainKind, NetworkConfig, SecurityMode};
    use crate::error::NetworkError;
    use crate::gossip::peer::{NodeCertificate, Peer, PeerRole};
    use aoxcunity::messages::ConsensusMessage;
    use aoxcunity::vote::{Vote, VoteKind};

    fn test_certificate() -> NodeCertificate {
        NodeCertificate {
            subject: "node-1".to_string(),
            issuer: "AOXC-ROOT".to_string(),
            valid_from_unix: 1,
            valid_until_unix: u64::MAX,
            serial: "serial-1".to_string(),
            domain_attestation_hash: "attestation-hash-1".to_string(),
        }
    }

    fn test_peer() -> Peer {
        Peer::new(
            "node-1",
            "10.0.0.1:2727",
            "AOXC-MAINNET",
            ExternalDomainKind::Native,
            PeerRole::Validator,
            3,
            true,
            test_certificate(),
        )
    }

    fn test_vote() -> ConsensusMessage {
        ConsensusMessage::Vote(Vote {
            voter: [1u8; 32],
            block_hash: [2u8; 32],
            height: 1,
            round: 0,
            kind: VoteKind::Prepare,
        })
    }

    #[test]
    fn checked_constructor_rejects_invalid_config() {
        let mut config = NetworkConfig::default();
        config.max_outbound_peers = 0;

        let result = P2PNetwork::new_checked(config);
        assert!(matches!(result, Err(NetworkError::InvalidConfig(_))));
    }

    #[test]
    fn secure_broadcast_requires_active_session() {
        let mut net = P2PNetwork::new(NetworkConfig::default());
        net.register_peer(test_peer()).expect("peer should register");

        let err = net
            .broadcast_secure("node-1", test_vote())
            .expect_err("broadcast without session must fail");

        assert!(matches!(err, NetworkError::UnknownPeer(_)));
    }

    #[test]
    fn session_based_broadcast_is_accepted() {
        let mut net = P2PNetwork::new(NetworkConfig::default());

        net.register_peer(test_peer()).expect("peer should register");
        net.establish_session("node-1")
            .expect("session should be established");

        let envelope = net
            .broadcast_secure("node-1", test_vote())
            .expect("broadcast should be accepted");

        assert_eq!(envelope.chain_id, "AOXC-MAINNET");
        assert_eq!(envelope.protocol_serial, 2626);
        assert!(!envelope.payload_hash_hex.is_empty());
        assert!(!envelope.frame_hash_hex.is_empty());
        assert!(net.receive().is_some());
    }

    #[test]
    fn banned_peer_cannot_broadcast() {
        let mut net = P2PNetwork::new(NetworkConfig::default());

        net.register_peer(test_peer()).expect("peer should register");
        net.establish_session("node-1")
            .expect("session should be established");
        net.ban_peer("node-1");

        let err = net
            .broadcast_secure("node-1", test_vote())
            .expect_err("banned peer must not broadcast");

        assert!(matches!(err, NetworkError::PeerBanned(_)));
    }

    #[test]
    fn banned_peer_cannot_register_again_during_ban_window() {
        let mut net = P2PNetwork::new(NetworkConfig::default());

        net.register_peer(test_peer()).expect("peer should register");
        net.ban_peer("node-1");

        let err = net
            .register_peer(test_peer())
            .expect_err("banned peer must not re-register");

        assert!(matches!(err, NetworkError::PeerBanned(_)));
    }

    #[test]
    fn replay_cache_detects_duplicate_nonce_for_same_session() {
        let mut net = P2PNetwork::new(NetworkConfig::default());

        net.register_peer(test_peer()).expect("peer should register");
        let ticket = net
            .establish_session("node-1")
            .expect("session should be established");

        let envelope =
            ProtocolEnvelope::new("AOXC-MAINNET", 2626, &ticket, test_vote(), 100)
                .expect("envelope creation must succeed");

        let replay_key = format!("{}:{}", envelope.session_id, envelope.nonce);
        net.replay_cache.insert(replay_key.clone());
        net.replay_order.push_back(replay_key);

        let err = net
            .broadcast_secure("node-1", test_vote())
            .expect_err("duplicate session nonce must be rejected");

        assert!(matches!(err, NetworkError::ReplayDetected));
    }

    #[test]
    fn expired_session_is_rejected_in_secure_mode() {
        let mut net = P2PNetwork::new(NetworkConfig::default());

        net.register_peer(test_peer()).expect("peer should register");
        net.establish_session("node-1")
            .expect("session should be established");

        let session = net
            .sessions
            .get_mut("node-1")
            .expect("session should exist");
        session.expires_at_unix = 0;

        let err = net
            .broadcast_secure("node-1", test_vote())
            .expect_err("expired session must be rejected");

        assert!(matches!(err, NetworkError::HandshakeTimeout));
    }

    #[test]
    fn insecure_mode_allows_expired_session_broadcast() {
        let mut config = NetworkConfig::default();
        config.security_mode = SecurityMode::Insecure;

        let mut net = P2PNetwork::new(config);

        net.register_peer(test_peer()).expect("peer should register");
        net.establish_session("node-1")
            .expect("session should be established");

        let session = net
            .sessions
            .get_mut("node-1")
            .expect("session should exist");
        session.expires_at_unix = 0;

        assert!(net.broadcast_secure("node-1", test_vote()).is_ok());
    }

    #[test]
    fn register_peer_rejects_capacity_overflow() {
        let mut config = NetworkConfig::default();
        config.max_inbound_peers = 1;
        config.max_outbound_peers = 1;

        let mut net = P2PNetwork::new(config);

        net.register_peer(test_peer()).expect("first peer should register");

        let second_peer = Peer::new(
            "node-2",
            "10.0.0.2:2727",
            "AOXC-MAINNET",
            ExternalDomainKind::Native,
            PeerRole::Validator,
            3,
            true,
            NodeCertificate {
                subject: "node-2".to_string(),
                issuer: "AOXC-ROOT".to_string(),
                valid_from_unix: 1,
                valid_until_unix: u64::MAX,
                serial: "serial-2".to_string(),
                domain_attestation_hash: "attestation-hash-2".to_string(),
            },
        );

        net.register_peer(second_peer)
            .expect("second peer should register inside aggregate capacity");

        let third_peer = Peer::new(
            "node-3",
            "10.0.0.3:2727",
            "AOXC-MAINNET",
            ExternalDomainKind::Native,
            PeerRole::Validator,
            3,
            true,
            NodeCertificate {
                subject: "node-3".to_string(),
                issuer: "AOXC-ROOT".to_string(),
                valid_from_unix: 1,
                valid_until_unix: u64::MAX,
                serial: "serial-3".to_string(),
                domain_attestation_hash: "attestation-hash-3".to_string(),
            },
        );

        let err = net
            .register_peer(third_peer)
            .expect_err("aggregate capacity overflow must be rejected");

        assert!(matches!(err, NetworkError::PeerAdmissionDenied(_)));
    }

    #[test]
    fn oversize_frame_is_rejected() {
        let mut config = NetworkConfig::default();
        config.max_frame_bytes = 64;

        let mut net = P2PNetwork::new(config);

        net.register_peer(test_peer()).expect("peer should register");
        net.establish_session("node-1")
            .expect("session should be established");

        let err = net
            .broadcast_secure("node-1", test_vote())
            .expect_err("oversize frame must be rejected");

        assert!(matches!(err, NetworkError::FrameTooLarge));
    }

    #[test]
    fn payload_hash_is_deterministic() {
        let hash_a = digest_payload(&test_vote()).expect("hashing must succeed");
        let hash_b = digest_payload(&test_vote()).expect("hashing must succeed");
        assert_eq!(hash_a, hash_b);
    }

    #[test]
    fn metrics_are_updated_on_successful_broadcast_and_receive() {
        let mut net = P2PNetwork::new(NetworkConfig::default());

        net.register_peer(test_peer()).expect("peer should register");
        net.establish_session("node-1")
            .expect("session should be established");

        net.broadcast_secure("node-1", test_vote())
            .expect("broadcast should succeed");
        let _ = net.receive();

        let snapshot = net.metrics_snapshot();
        assert_eq!(snapshot.accepted_peers, 1);
        assert_eq!(snapshot.active_sessions, 1);
        assert_eq!(snapshot.frames_out, 1);
        assert_eq!(snapshot.frames_in, 1);
        assert_eq!(snapshot.gossip_messages, 1);
        assert!(snapshot.bytes_out > 0);
        assert!(snapshot.bytes_in > 0);
    }

    #[test]
    fn protocol_envelope_rejects_empty_chain_id() {
        let ticket = SessionTicket {
            peer_id: "node-1".to_string(),
            cert_fingerprint: "fp".to_string(),
            established_at_unix: 1,
            replay_window_nonce: 7,
            session_id: "session-1".to_string(),
            expires_at_unix: u64::MAX,
        };

        let err = ProtocolEnvelope::new("", 2626, &ticket, test_vote(), 1)
            .expect_err("empty chain id must be rejected");

        assert!(matches!(err, NetworkError::ProtocolMismatch(_)));
    }

    #[test]
    fn protocol_envelope_detects_payload_tampering() {
        let ticket = SessionTicket {
            peer_id: "node-1".to_string(),
            cert_fingerprint: "fp".to_string(),
            established_at_unix: 1,
            replay_window_nonce: 7,
            session_id: "session-1".to_string(),
            expires_at_unix: u64::MAX,
        };

        let mut envelope =
            ProtocolEnvelope::new("AOXC-MAINNET", 2626, &ticket, test_vote(), 1)
                .expect("envelope should be created");

        envelope.payload_hash_hex = "deadbeef".to_string();

        let err = envelope
            .verify_against("AOXC-MAINNET", 2626)
            .expect_err("tampered envelope must be rejected");

        assert!(matches!(err, NetworkError::ProtocolMismatch(_)));
    }
}
