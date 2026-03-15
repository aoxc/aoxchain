use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockConfirmedEvent {
    pub block_hash: String,
    pub height: u64,
}
