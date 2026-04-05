// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};

pub const AOXC_Q_RELEASE_LINE: &str = "AOXC-Q-v0.2.0";

/// Cryptographic transport profile declared during peer handshake.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportCryptoProfile {
    /// Classical transport cryptography only.
    ClassicalV1,
    /// Hybrid profile requiring classical and PQ material.
    HybridV2,
    /// PQ-preferred transport profile.
    PostQuantumV3,
}

impl TransportCryptoProfile {
    /// Returns true when the profile has post-quantum requirements.
    #[must_use]
    pub const fn requires_post_quantum(self) -> bool {
        matches!(self, Self::HybridV2 | Self::PostQuantumV3)
    }
}

/// Peer class for handshake policy and anti-amplification budgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeerClass {
    Validator,
    Sentry,
    Archive,
    Observer,
    Bootstrap,
}

/// Wire-safe handshake intent sent by a remote peer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandshakeIntent {
    pub peer_id: String,
    pub peer_class: PeerClass,
    pub release_line: String,
    pub transport_profile: TransportCryptoProfile,
    pub protocol_version: u16,
    pub max_frame_bytes: usize,
    pub compression_enabled: bool,
    pub retry_token_present: bool,
    pub pq_kem_present: bool,
}

/// Local policy constraints for handshake admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandshakePolicy {
    pub minimum_protocol_version: u16,
    pub required_release_line: String,
    pub required_profile: TransportCryptoProfile,
    pub max_frame_bytes: usize,
    pub allow_compression: bool,
    pub require_retry_token: bool,
    pub require_pq_kem: bool,
}

impl Default for HandshakePolicy {
    fn default() -> Self {
        Self {
            minimum_protocol_version: 1,
            required_release_line: "AOXC-Q-v0.2.0".to_string(),
            required_profile: TransportCryptoProfile::HybridV2,
            max_frame_bytes: 256 * 1024,
            allow_compression: false,
            require_retry_token: true,
            require_pq_kem: true,
        }
    }
}

/// Stable admission failure reasons for transport handshakes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandshakeRejectReason {
    EmptyPeerId,
    ReleaseLineMismatch,
    ProtocolVersionTooOld,
    ProfileDowngradeRejected,
    FrameBudgetExceeded,
    CompressionForbidden,
    RetryTokenMissing,
    PostQuantumKemMissing,
}

impl HandshakePolicy {
    /// Evaluates a remote handshake intent against local security policy.
    pub fn evaluate(&self, intent: &HandshakeIntent) -> Result<(), HandshakeRejectReason> {
        if intent.peer_id.trim().is_empty() {
            return Err(HandshakeRejectReason::EmptyPeerId);
        }

        if intent.release_line.trim() != self.required_release_line {
            return Err(HandshakeRejectReason::ReleaseLineMismatch);
        }

        if intent.protocol_version < self.minimum_protocol_version {
            return Err(HandshakeRejectReason::ProtocolVersionTooOld);
        }

        if intent.transport_profile != self.required_profile {
            return Err(HandshakeRejectReason::ProfileDowngradeRejected);
        }

        if intent.max_frame_bytes > self.max_frame_bytes {
            return Err(HandshakeRejectReason::FrameBudgetExceeded);
        }

        if !self.allow_compression && intent.compression_enabled {
            return Err(HandshakeRejectReason::CompressionForbidden);
        }

        if self.require_retry_token && !intent.retry_token_present {
            return Err(HandshakeRejectReason::RetryTokenMissing);
        }

        if self.require_pq_kem
            && self.required_profile.requires_post_quantum()
            && !intent.pq_kem_present
        {
            return Err(HandshakeRejectReason::PostQuantumKemMissing);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AOXC_Q_RELEASE_LINE, HandshakeIntent, HandshakePolicy, HandshakeRejectReason, PeerClass,
        TransportCryptoProfile,
    };

    fn base_intent() -> HandshakeIntent {
        HandshakeIntent {
            peer_id: "validator-a".to_string(),
            peer_class: PeerClass::Validator,
            release_line: "AOXC-Q-v0.2.0".to_string(),
            transport_profile: TransportCryptoProfile::HybridV2,
            protocol_version: 1,
            max_frame_bytes: 128 * 1024,
            compression_enabled: false,
            retry_token_present: true,
            pq_kem_present: true,
        }
    }

    #[test]
    fn accepts_hybrid_profile_with_pq_kem_and_retry_token() {
        let policy = HandshakePolicy::default();
        assert_eq!(policy.evaluate(&base_intent()), Ok(()));
    }

    #[test]
    fn rejects_profile_downgrade() {
        let policy = HandshakePolicy::default();
        let mut intent = base_intent();
        intent.transport_profile = TransportCryptoProfile::ClassicalV1;

        assert_eq!(
            policy.evaluate(&intent),
            Err(HandshakeRejectReason::ProfileDowngradeRejected)
        );
    }

    #[test]
    fn rejects_release_line_mismatch() {
        let policy = HandshakePolicy::default();
        let mut intent = base_intent();
        intent.release_line = "AOXC-Q-v0.1.9".to_string();

        assert_eq!(
            policy.evaluate(&intent),
            Err(HandshakeRejectReason::ReleaseLineMismatch)
        );
    }

    #[test]
    fn rejects_missing_retry_token() {
        let policy = HandshakePolicy::default();
        let mut intent = base_intent();
        intent.retry_token_present = false;

        assert_eq!(
            policy.evaluate(&intent),
            Err(HandshakeRejectReason::RetryTokenMissing)
        );
    }

    #[test]
    fn rejects_missing_pq_kem_when_required() {
        let policy = HandshakePolicy::default();
        let mut intent = base_intent();
        intent.pq_kem_present = false;

        assert_eq!(
            policy.evaluate(&intent),
            Err(HandshakeRejectReason::PostQuantumKemMissing)
        );
    }
}
