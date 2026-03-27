use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LaneKind {
    Evm,
    Move,
    Wasm,
    Cardano,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LaneStatus {
    pub kind: LaneKind,
    pub tps: f32,
    pub load_percent: u8,
    pub is_active: bool,
    pub last_checkpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConsensusNode {
    pub id: String,
    pub stake_weight: u64,
    pub latency_ms: u32,
    pub region: String,
    pub online: bool,
}

#[cfg(test)]
mod tests {
    use super::{ConsensusNode, LaneKind, LaneStatus};

    #[test]
    fn lane_kind_serializes_in_camel_case_variant_name() {
        let encoded = serde_json::to_string(&LaneKind::Wasm).expect("serialize");
        assert_eq!(encoded, "\"Wasm\"");
    }

    #[test]
    fn lane_status_round_trip_preserves_fields() {
        let lane = LaneStatus {
            kind: LaneKind::Move,
            tps: 1200.5,
            load_percent: 65,
            is_active: true,
            last_checkpoint: "0xabc".to_string(),
        };

        let encoded = serde_json::to_string(&lane).expect("serialize");
        let decoded: LaneStatus = serde_json::from_str(&encoded).expect("deserialize");

        assert_eq!(decoded.kind, LaneKind::Move);
        assert_eq!(decoded.tps, 1200.5);
        assert_eq!(decoded.load_percent, 65);
        assert!(decoded.is_active);
        assert_eq!(decoded.last_checkpoint, "0xabc");
    }

    #[test]
    fn consensus_node_online_flag_is_preserved() {
        let node = ConsensusNode {
            id: "atlas-01".to_string(),
            stake_weight: 100,
            latency_ms: 20,
            region: "eu-central".to_string(),
            online: false,
        };

        let encoded = serde_json::to_string(&node).expect("serialize");
        let decoded: ConsensusNode = serde_json::from_str(&encoded).expect("deserialize");
        assert!(!decoded.online);
    }
}
