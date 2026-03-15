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

/// Canonical machine-readable error payload for RPC HTTP/WS surfaces.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RpcErrorResponse {
    pub code: &'static str,
    pub message: String,
    pub retry_after_ms: Option<u64>,
    pub request_id: Option<String>,
}
