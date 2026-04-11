// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use thiserror::Error;

/// Stable network subsystem errors with operator-facing symbolic codes.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum NetworkError {
    #[error("peer disconnected")]
    PeerDisconnected,
    #[error("peer already registered: {0}")]
    PeerAlreadyRegistered(String),
    #[error("unknown peer: {0}")]
    UnknownPeer(String),
    #[error("certificate validation failed: {0}")]
    CertificateValidationFailed(String),
    #[error("invalid security mode transition")]
    InvalidSecurityMode,
    #[error("network configuration invalid: {0}")]
    InvalidConfig(String),
    #[error("frame exceeds configured limit")]
    FrameTooLarge,
    #[error("transport handshake timed out")]
    HandshakeTimeout,
    #[error("replay detected")]
    ReplayDetected,
    #[error("peer admission denied: {0}")]
    PeerAdmissionDenied(String),
    #[error("peer banned: {0}")]
    PeerBanned(String),
    #[error("protocol mismatch: {0}")]
    ProtocolMismatch(String),
    #[error("interoperability policy denied external domain")]
    InteropDenied,
    #[error("sync request invalid: {0}")]
    InvalidSyncRequest(String),
    #[error("transport unavailable: {0}")]
    TransportUnavailable(String),
    #[error("serialization failure: {0}")]
    Serialization(String),
    #[error("quantum policy violation: {0}")]
    QuantumPolicyViolation(String),
}

impl NetworkError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::PeerDisconnected => "AOXCNET_PEER_DISCONNECTED",
            Self::PeerAlreadyRegistered(_) => "AOXCNET_PEER_ALREADY_REGISTERED",
            Self::UnknownPeer(_) => "AOXCNET_UNKNOWN_PEER",
            Self::CertificateValidationFailed(_) => "AOXCNET_CERT_VALIDATION_FAILED",
            Self::InvalidSecurityMode => "AOXCNET_INVALID_SECURITY_MODE",
            Self::InvalidConfig(_) => "AOXCNET_INVALID_CONFIG",
            Self::FrameTooLarge => "AOXCNET_FRAME_TOO_LARGE",
            Self::HandshakeTimeout => "AOXCNET_HANDSHAKE_TIMEOUT",
            Self::ReplayDetected => "AOXCNET_REPLAY_DETECTED",
            Self::PeerAdmissionDenied(_) => "AOXCNET_PEER_ADMISSION_DENIED",
            Self::PeerBanned(_) => "AOXCNET_PEER_BANNED",
            Self::ProtocolMismatch(_) => "AOXCNET_PROTOCOL_MISMATCH",
            Self::InteropDenied => "AOXCNET_INTEROP_DENIED",
            Self::InvalidSyncRequest(_) => "AOXCNET_INVALID_SYNC_REQUEST",
            Self::TransportUnavailable(_) => "AOXCNET_TRANSPORT_UNAVAILABLE",
            Self::Serialization(_) => "AOXCNET_SERIALIZATION",
            Self::QuantumPolicyViolation(_) => "AOXCNET_QUANTUM_POLICY_VIOLATION",
        }
    }
}
