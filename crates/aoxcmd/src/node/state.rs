// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use chrono::Utc;
use serde::{Deserialize, Serialize};

const DEFAULT_NETWORK_ID: u32 = 2626;
const DEFAULT_LAST_MESSAGE_KIND: &str = "bootstrap";
const DEFAULT_LAST_TX: &str = "none";

/// Canonical in-memory runtime state persisted by the AOXC operator plane.
///
/// Design objectives:
/// - Preserve backward-compatible deserialization for legacy payloads that may
///   omit recently introduced fields.
/// - Keep the runtime state self-validating so persistence and load paths can
///   enforce semantic integrity at the boundary.
/// - Separate operator key snapshot state from consensus snapshot state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    pub initialized: bool,
    pub running: bool,
    pub current_height: u64,
    pub produced_blocks: u64,
    #[serde(default = "default_last_tx")]
    pub last_tx: String,
    pub updated_at: String,
    #[serde(default)]
    pub key_material: KeyMaterialSnapshot,
    #[serde(default)]
    pub consensus: ConsensusSnapshot,
}

/// Snapshot of the active operator key material as observed by runtime flows.
///
/// Compatibility policy:
/// - All fields default to empty strings so legacy state payloads remain
///   deserializable even if key metadata did not exist at the time they were
///   written.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyMaterialSnapshot {
    pub bundle_fingerprint: String,
    pub operational_state: String,
    pub consensus_public_key_hex: String,
    pub transport_public_key_hex: String,
}

/// Snapshot of the most recent consensus-visible runtime facts.
///
/// Compatibility policy:
/// - Defaults preserve deterministic bootstrap semantics for legacy payloads
///   that omitted consensus detail fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusSnapshot {
    pub network_id: u32,
    pub last_parent_hash_hex: String,
    pub last_block_hash_hex: String,
    pub last_proposer_hex: String,
    pub last_round: u64,
    pub last_timestamp_unix: u64,
    pub last_message_kind: String,
    pub last_section_count: usize,
}

impl Default for ConsensusSnapshot {
    fn default() -> Self {
        Self {
            network_id: DEFAULT_NETWORK_ID,
            last_parent_hash_hex: zero_hash_hex(),
            last_block_hash_hex: zero_hash_hex(),
            last_proposer_hex: zero_hash_hex(),
            last_round: 0,
            last_timestamp_unix: 0,
            last_message_kind: DEFAULT_LAST_MESSAGE_KIND.to_string(),
            last_section_count: 0,
        }
    }
}

impl NodeState {
    /// Returns the canonical bootstrap runtime state.
    ///
    /// Bootstrap invariants:
    /// - Runtime begins initialized but not running.
    /// - Height and produced block count begin at zero.
    /// - Consensus snapshot reflects the canonical bootstrap baseline.
    pub fn bootstrap() -> Self {
        Self {
            initialized: true,
            running: false,
            current_height: 0,
            produced_blocks: 0,
            last_tx: default_last_tx(),
            updated_at: Utc::now().to_rfc3339(),
            key_material: KeyMaterialSnapshot::default(),
            consensus: ConsensusSnapshot::default(),
        }
    }

    /// Refreshes the operator-facing update timestamp.
    pub fn touch(&mut self) {
        self.updated_at = Utc::now().to_rfc3339();
    }

    /// Validates semantic integrity for persisted runtime state.
    ///
    /// Validation policy:
    /// - `updated_at` must be a valid RFC3339 timestamp.
    /// - A running node must also be initialized.
    /// - `produced_blocks` must never exceed `current_height`.
    /// - Consensus hash fields must always remain canonical 32-byte hex strings.
    /// - Consensus metadata fields that are semantically required must not be blank.
    /// - Key material public keys, when present, must remain canonical 32-byte hex strings.
    pub fn validate(&self) -> Result<(), String> {
        chrono::DateTime::parse_from_rfc3339(&self.updated_at)
            .map_err(|_| "updated_at must be a valid RFC3339 timestamp".to_string())?;

        if self.running && !self.initialized {
            return Err("running node state must also be initialized".to_string());
        }

        if self.produced_blocks > self.current_height {
            return Err("produced_blocks cannot exceed current_height".to_string());
        }

        if self.last_tx.trim().is_empty() {
            return Err("last_tx must not be blank".to_string());
        }

        if self.consensus.network_id == 0 {
            return Err("consensus.network_id must be non-zero".to_string());
        }

        if self.consensus.last_message_kind.trim().is_empty() {
            return Err("consensus.last_message_kind must not be blank".to_string());
        }

        validate_hex_field(
            &self.consensus.last_parent_hash_hex,
            32,
            "consensus.last_parent_hash_hex",
        )?;
        validate_hex_field(
            &self.consensus.last_block_hash_hex,
            32,
            "consensus.last_block_hash_hex",
        )?;
        validate_hex_field(
            &self.consensus.last_proposer_hex,
            32,
            "consensus.last_proposer_hex",
        )?;

        validate_optional_hex_field(
            &self.key_material.consensus_public_key_hex,
            32,
            "key_material.consensus_public_key_hex",
        )?;
        validate_optional_hex_field(
            &self.key_material.transport_public_key_hex,
            32,
            "key_material.transport_public_key_hex",
        )?;

        Ok(())
    }
}

fn validate_hex_field(value: &str, bytes: usize, field: &str) -> Result<(), String> {
    let decoded = hex::decode(value).map_err(|_| format!("{field} must be valid hex"))?;
    if decoded.len() != bytes {
        return Err(format!("{field} must decode to {bytes} bytes"));
    }
    Ok(())
}

fn validate_optional_hex_field(value: &str, bytes: usize, field: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Ok(());
    }
    validate_hex_field(value, bytes, field)
}

fn zero_hash_hex() -> String {
    hex::encode([0u8; 32])
}

fn default_last_tx() -> String {
    DEFAULT_LAST_TX.to_string()
}

#[cfg(test)]
mod tests {
    use super::{NodeState, default_last_tx, zero_hash_hex};

    #[test]
    fn bootstrap_initializes_consensus_snapshot() {
        let state = NodeState::bootstrap();

        assert_eq!(state.consensus.network_id, 2626);
        assert_eq!(state.consensus.last_message_kind, "bootstrap");
        assert_eq!(state.consensus.last_section_count, 0);
        assert!(state.key_material.bundle_fingerprint.is_empty());
        assert_eq!(state.last_tx, default_last_tx());
    }

    #[test]
    fn legacy_state_payload_deserializes_with_default_consensus_snapshot() {
        let raw = r#"{
            "initialized": true,
            "running": false,
            "current_height": 7,
            "produced_blocks": 7,
            "last_tx": "legacy",
            "updated_at": "2026-01-01T00:00:00Z"
        }"#;

        let state: NodeState = serde_json::from_str(raw).expect("legacy state should deserialize");

        assert_eq!(state.current_height, 7);
        assert_eq!(state.consensus.last_block_hash_hex, zero_hash_hex());
        assert_eq!(state.consensus.last_message_kind, "bootstrap");
        assert!(state.key_material.consensus_public_key_hex.is_empty());
    }

    #[test]
    fn legacy_state_payload_deserializes_with_default_last_tx_when_missing() {
        let raw = r#"{
            "initialized": true,
            "running": false,
            "current_height": 2,
            "produced_blocks": 2,
            "updated_at": "2026-01-01T00:00:00Z"
        }"#;

        let state: NodeState = serde_json::from_str(raw).expect("legacy state should deserialize");

        assert_eq!(state.last_tx, "none");
    }

    #[test]
    fn validate_rejects_non_initialized_running_state() {
        let mut state = NodeState::bootstrap();
        state.initialized = false;
        state.running = true;

        assert!(state.validate().is_err());
    }

    #[test]
    fn validate_rejects_invalid_consensus_hashes() {
        let mut state = NodeState::bootstrap();
        state.consensus.last_block_hash_hex = "xyz".to_string();

        assert!(state.validate().is_err());
    }

    #[test]
    fn validate_rejects_blank_last_tx() {
        let mut state = NodeState::bootstrap();
        state.last_tx = "   ".to_string();

        assert!(state.validate().is_err());
    }

    #[test]
    fn validate_rejects_zero_network_id() {
        let mut state = NodeState::bootstrap();
        state.consensus.network_id = 0;

        assert!(state.validate().is_err());
    }

    #[test]
    fn validate_rejects_blank_last_message_kind() {
        let mut state = NodeState::bootstrap();
        state.consensus.last_message_kind = "".to_string();

        assert!(state.validate().is_err());
    }

    #[test]
    fn validate_rejects_invalid_optional_key_hex_when_present() {
        let mut state = NodeState::bootstrap();
        state.key_material.consensus_public_key_hex = "aa".to_string();

        assert!(state.validate().is_err());
    }

    #[test]
    fn validate_accepts_empty_optional_key_hex_fields() {
        let state = NodeState::bootstrap();

        assert!(state.validate().is_ok());
    }
}
