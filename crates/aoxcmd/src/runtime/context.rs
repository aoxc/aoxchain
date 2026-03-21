use crate::{
    config::settings::Settings,
    node::state::NodeState,
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeContext {
    pub settings: Settings,
    pub node_state: Option<NodeState>,
}
