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
    pub consensus: ConsensusSnapshot,
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
            consensus: ConsensusSnapshot::default(),
        }
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now().to_rfc3339();
    }
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
    }
}
