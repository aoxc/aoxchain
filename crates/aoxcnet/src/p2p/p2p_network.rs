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

        if self.is_banned(&peer.id) {
            self.metrics.rejected_peers = self.metrics.rejected_peers.saturating_add(1);
            return Err(NetworkError::PeerBanned(peer.id));
        }

        if self.peers.contains_key(&peer.id) {
            self.metrics.rejected_peers = self.metrics.rejected_peers.saturating_add(1);
            return Err(NetworkError::PeerAlreadyRegistered(peer.id));
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
        if self.config.requires_mutual_auth()
            && self
                .sessions
                .get(peer_id)
                .is_some_and(|ticket| ticket.expires_at_unix >= now)
        {
            self.metrics.failed_handshakes = self.metrics.failed_handshakes.saturating_add(1);
            return Err(NetworkError::PeerAdmissionDenied(
                "active mutually authenticated session already exists".to_string(),
            ));
        }

        let transport_profile = required_transport_profile(self.config.security_mode);
        let handshake_policy = handshake_policy_for_mode(&self.config, transport_profile);
        let handshake_intent = HandshakeIntent {
            peer_id: peer.id.clone(),
            peer_class: peer_class_for_role(peer.role),
            release_line: AOXC_Q_RELEASE_LINE.to_string(),
            transport_profile,
            protocol_version: 1,
            max_frame_bytes: self.config.max_frame_bytes,
            compression_enabled: false,
            retry_token_present: true,
            pq_kem_present: transport_profile.requires_post_quantum(),
        };

        if let Err(reason) = handshake_policy.evaluate(&handshake_intent) {
            self.metrics.failed_handshakes = self.metrics.failed_handshakes.saturating_add(1);
            return Err(NetworkError::PeerAdmissionDenied(format!(
                "handshake admission denied: {}",
                describe_handshake_reject(reason)
            )));
        }

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

            let mut envelope_ticket = ticket.clone();
            if security_mode == SecurityMode::Insecure && now > envelope_ticket.expires_at_unix {
                envelope_ticket.expires_at_unix = now;
            }

            let envelope = ProtocolEnvelope::new(
                &canonical_chain_id,
                canonical_protocol_serial,
                &envelope_ticket,
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
        self.metrics.bytes_out = self.metrics.bytes_out.saturating_add(encoded.len() as u64);
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
            self.metrics.bytes_in = self.metrics.bytes_in.saturating_add(encoded.len() as u64);
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
#[allow(clippy::too_many_arguments)]
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

#[must_use]
fn required_transport_profile(mode: SecurityMode) -> TransportCryptoProfile {
    match mode {
        SecurityMode::Insecure => TransportCryptoProfile::ClassicalV1,
        SecurityMode::MutualAuth => TransportCryptoProfile::HybridV2,
        SecurityMode::AuditStrict => TransportCryptoProfile::PostQuantumV3,
    }
}

#[must_use]
fn peer_class_for_role(role: crate::gossip::peer::PeerRole) -> PeerClass {
    match role {
        crate::gossip::peer::PeerRole::Validator => PeerClass::Validator,
        crate::gossip::peer::PeerRole::Relay => PeerClass::Sentry,
        crate::gossip::peer::PeerRole::Observer => PeerClass::Observer,
        crate::gossip::peer::PeerRole::Bridge => PeerClass::Archive,
    }
}

#[must_use]
fn handshake_policy_for_mode(
    config: &NetworkConfig,
    required_profile: TransportCryptoProfile,
) -> HandshakePolicy {
    HandshakePolicy {
        minimum_protocol_version: 1,
        required_release_line: AOXC_Q_RELEASE_LINE.to_string(),
        required_profile,
        max_frame_bytes: config.max_frame_bytes,
        allow_compression: false,
        require_retry_token: config.security_mode != SecurityMode::Insecure,
        require_pq_kem: required_profile.requires_post_quantum(),
    }
}

#[must_use]
fn describe_handshake_reject(reason: HandshakeRejectReason) -> &'static str {
    match reason {
        HandshakeRejectReason::EmptyPeerId => "empty peer identifier",
        HandshakeRejectReason::ReleaseLineMismatch => "release line mismatch",
        HandshakeRejectReason::ProtocolVersionTooOld => "protocol version too old",
        HandshakeRejectReason::ProfileDowngradeRejected => "transport profile downgrade rejected",
        HandshakeRejectReason::FrameBudgetExceeded => "declared frame budget exceeds policy",
        HandshakeRejectReason::CompressionForbidden => "compression is forbidden",
        HandshakeRejectReason::RetryTokenMissing => "retry token missing",
        HandshakeRejectReason::PostQuantumKemMissing => "post-quantum KEM missing",
    }
}
