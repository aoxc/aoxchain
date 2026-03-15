use serde::{Deserialize, Serialize};

/// Generic health response returned by RPC endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub chain_id: String,
    pub genesis_hash: Option<String>,
    pub tls_enabled: bool,
    pub mtls_enabled: bool,
    pub tls_cert_sha256: Option<String>,
    pub readiness_score: u8,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub recommendations: Vec<String>,
    pub uptime_secs: u64,
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
    pub user_hint: Option<String>,
}
