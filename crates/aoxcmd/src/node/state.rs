// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    pub initialized: bool,
    pub running: bool,
    pub current_height: u64,
    pub produced_blocks: u64,
    pub last_tx: String,
    pub updated_at: String,
    #[serde(default)]
    pub key_material: KeyMaterialSnapshot,
    #[serde(default)]
    pub consensus: ConsensusSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyMaterialSnapshot {
    pub bundle_fingerprint: String,
    pub operational_state: String,
    pub consensus_public_key_hex: String,
    pub transport_public_key_hex: String,
}

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
            network_id: 2626,
            last_parent_hash_hex: hex::encode([0u8; 32]),
            last_block_hash_hex: hex::encode([0u8; 32]),
            last_proposer_hex: hex::encode([0u8; 32]),
            last_round: 0,
            last_timestamp_unix: 0,
            last_message_kind: "bootstrap".to_string(),
            last_section_count: 0,
        }
    }
}

impl NodeState {
    pub fn bootstrap() -> Self {
        Self {
            initialized: true,
            running: false,
            current_height: 0,
            produced_blocks: 0,
            last_tx: "none".to_string(),
            updated_at: Utc::now().to_rfc3339(),
            key_material: KeyMaterialSnapshot::default(),
            consensus: ConsensusSnapshot::default(),
        }
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now().to_rfc3339();
    }

    pub fn validate(&self) -> Result<(), String> {
        chrono::DateTime::parse_from_rfc3339(&self.updated_at)
            .map_err(|_| "updated_at must be a valid RFC3339 timestamp".to_string())?;
        if self.running && !self.initialized {
            return Err("running node state must also be initialized".to_string());
        }
        if self.produced_blocks > self.current_height {
            return Err("produced_blocks cannot exceed current_height".to_string());
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

#[cfg(test)]
mod tests {
    use super::NodeState;

    #[test]
    fn bootstrap_initializes_consensus_snapshot() {
        let state = NodeState::bootstrap();

        assert_eq!(state.consensus.network_id, 2626);
        assert_eq!(state.consensus.last_message_kind, "bootstrap");
        assert_eq!(state.consensus.last_section_count, 0);
        assert!(state.key_material.bundle_fingerprint.is_empty());
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
        assert_eq!(state.consensus.last_block_hash_hex, hex::encode([0u8; 32]));
        assert_eq!(state.consensus.last_message_kind, "bootstrap");
        assert!(state.key_material.consensus_public_key_hex.is_empty());
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
}
