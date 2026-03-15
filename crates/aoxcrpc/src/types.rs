use serde::{Deserialize, Serialize};

/// Generic health response returned by RPC endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStatus {
    pub chain_id: String,
    pub height: u64,
    pub syncing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxSubmissionRequest {
    pub actor_id: String,
    pub tx_payload: Vec<u8>,
    pub zkp_proof: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxSubmissionResult {
    pub tx_id: String,
    pub accepted: bool,
}
