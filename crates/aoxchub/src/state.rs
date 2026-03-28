use dioxus::prelude::*;

use crate::services::telemetry::TelemetrySnapshot;
use crate::types::{ConsensusNode, LaneStatus};

#[derive(Clone, PartialEq)]
pub struct GlobalChainState {
    pub height: Option<u64>,
    pub lanes: Vec<LaneStatus>,
    pub total_staked: Option<u128>,
    pub network_health: Option<f32>,
    pub active_nodes: Option<usize>,
    pub nodes: Vec<ConsensusNode>,
}

impl GlobalChainState {
    pub fn empty() -> Self {
        Self {
            height: None,
            lanes: Vec::new(),
            total_staked: None,
            network_health: None,
            active_nodes: None,
            nodes: Vec::new(),
        }
    }

    pub fn apply_telemetry(&mut self, snapshot: &TelemetrySnapshot) {
        self.height = snapshot.latest_block;
        self.active_nodes = snapshot.peer_count;
        self.network_health = Some(if snapshot.healthy { 100.0 } else { 0.0 });
    }
}

pub fn provide_global_state() -> Signal<GlobalChainState> {
    use_context_provider(|| Signal::new(GlobalChainState::empty()))
}
