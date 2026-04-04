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

