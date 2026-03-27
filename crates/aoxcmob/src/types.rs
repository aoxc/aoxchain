// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};

/// Supported client platform families.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DevicePlatform {
    Android,
    Ios,
    Desktop,
    Simulator,
    Unknown(String),
}

/// Stable device profile persisted by the secure store.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceProfile {
    pub device_id: String,
    pub device_label: String,
    pub platform: DevicePlatform,
    pub public_key_hex: String,
    pub public_key_fingerprint: String,
    pub hd_path: Option<String>,
    pub app_installation_id: String,
    pub created_at_epoch_secs: u64,
}

/// Chain health summary suitable for battery-conscious mobile status views.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChainHealth {
    pub chain_id: String,
    pub height: u64,
    pub peer_count: u32,
    pub error_rate: f64,
    pub healthy: bool,
}

/// Task categories appropriate for a light mobile client.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskKind {
    GovernanceVote,
    SecurityWitness,
    Heartbeat,
    GovernanceReview,
}

/// Lightweight task descriptor fetched from a relay or control-plane service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskDescriptor {
    pub task_id: String,
    pub kind: TaskKind,
    pub title: String,
    pub detail: String,
    pub reward_units: u64,
    pub expires_at_epoch_secs: u64,
    pub required_session: bool,
}

/// Mobile witness decision model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WitnessDecision {
    Approve,
    Reject,
    Abstain,
}

/// Canonical task receipt before signature wrapping.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskReceipt {
    pub task_id: String,
    pub decision: WitnessDecision,
    pub client_timestamp_epoch_secs: u64,
    pub device_id: String,
    pub session_id: String,
}

/// Signed task receipt emitted by the device.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedTaskReceipt {
    pub receipt: TaskReceipt,
    pub signature_hex: String,
    pub payload_hash_hex: String,
    pub public_key_hex: String,
}
