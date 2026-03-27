// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{config::settings::Settings, node::state::NodeState};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeContext {
    pub settings: Settings,
    pub node_state: Option<NodeState>,
}
