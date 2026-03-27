use dioxus::prelude::*;

use crate::types::{ConsensusNode, LaneKind, LaneStatus};

#[derive(Clone, PartialEq)]
pub struct GlobalChainState {
    pub height: u64,
    pub lanes: Vec<LaneStatus>,
    pub total_staked: u128,
    pub network_health: f32,
    pub active_nodes: usize,
    pub nodes: Vec<ConsensusNode>,
}

impl GlobalChainState {
    pub fn seeded() -> Self {
        Self {
            height: 1_450_982,
            lanes: vec![
                LaneStatus {
                    kind: LaneKind::Evm,
                    tps: 850.5,
                    load_percent: 34,
                    is_active: true,
                    last_checkpoint: "0x7a...a1".into(),
                },
                LaneStatus {
                    kind: LaneKind::Move,
                    tps: 1_240.2,
                    load_percent: 67,
                    is_active: true,
                    last_checkpoint: "0x11...f4".into(),
                },
                LaneStatus {
                    kind: LaneKind::Wasm,
                    tps: 320.1,
                    load_percent: 42,
                    is_active: true,
                    last_checkpoint: "0x54...9b".into(),
                },
                LaneStatus {
                    kind: LaneKind::Cardano,
                    tps: 140.4,
                    load_percent: 22,
                    is_active: false,
                    last_checkpoint: "syncing".into(),
                },
            ],
            total_staked: 12_500_000_000,
            network_health: 99.98,
            active_nodes: 57,
            nodes: vec![
                ConsensusNode {
                    id: "atlas-01".into(),
                    stake_weight: 320_000,
                    latency_ms: 21,
                    region: "eu-central".into(),
                    online: true,
                },
                ConsensusNode {
                    id: "ember-02".into(),
                    stake_weight: 284_000,
                    latency_ms: 28,
                    region: "us-east".into(),
                    online: true,
                },
                ConsensusNode {
                    id: "delta-03".into(),
                    stake_weight: 261_000,
                    latency_ms: 35,
                    region: "ap-south".into(),
                    online: true,
                },
                ConsensusNode {
                    id: "cypher-04".into(),
                    stake_weight: 240_000,
                    latency_ms: 90,
                    region: "sa-east".into(),
                    online: false,
                },
            ],
        }
    }
}

pub fn provide_global_state() {
    use_context_provider(|| Signal::new(GlobalChainState::seeded()));
}
