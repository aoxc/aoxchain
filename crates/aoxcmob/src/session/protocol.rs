// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::types::DeviceProfile;
use serde::{Deserialize, Serialize};

/// Challenge returned by the relay before a mobile session is opened.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionChallenge {
    pub challenge_id: String,
    pub relay_nonce: String,
    pub issued_at_epoch_secs: u64,
    pub expires_at_epoch_secs: u64,
    pub audience: String,
    pub session_ttl_secs: u64,
    pub relay_signature_hex: Option<String>,
}

/// Signed device response to a relay-issued session challenge.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionEnvelope {
    pub challenge_id: String,
    pub relay_nonce: String,
    pub device_id: String,
    pub app_id: String,
    pub chain_id: String,
    pub client_nonce: u64,
    pub client_timestamp_epoch_secs: u64,
    pub public_key_hex: String,
    pub payload_hash_hex: String,
    pub signature_hex: String,
}

/// Relay-issued session permit. This is intentionally short-lived.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionPermit {
    pub session_id: String,
    pub device_id: String,
    pub issued_at_epoch_secs: u64,
    pub expires_at_epoch_secs: u64,
    pub relay_signature_hint: String,
    pub relay_signature_hex: Option<String>,
}

/// Combined result returned after a successful session open operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionContext {
    pub profile: DeviceProfile,
    pub permit: SessionPermit,
}

/// Canonical session signing payload used to create the envelope signature.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionSigningPayload {
    pub challenge_id: String,
    pub relay_nonce: String,
    pub device_id: String,
    pub app_id: String,
    pub chain_id: String,
    pub client_nonce: u64,
    pub client_timestamp_epoch_secs: u64,
    pub public_key_hex: String,
}

/// Canonical relay challenge payload expected to be signed by the relay identity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelayChallengeSigningPayload {
    pub challenge_id: String,
    pub relay_nonce: String,
    pub issued_at_epoch_secs: u64,
    pub expires_at_epoch_secs: u64,
    pub audience: String,
    pub session_ttl_secs: u64,
}

/// Canonical relay permit payload expected to be signed by the relay identity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelayPermitSigningPayload {
    pub session_id: String,
    pub device_id: String,
    pub issued_at_epoch_secs: u64,
    pub expires_at_epoch_secs: u64,
    pub relay_signature_hint: String,
}
