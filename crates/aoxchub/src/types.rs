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
