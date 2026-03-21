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
        }
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now().to_rfc3339();
    }
}
