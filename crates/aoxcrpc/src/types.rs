// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

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
    pub security_posture: RpcSecurityPosture,

    pub uptime_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RpcSecurityPosture {
    pub level: String,
    pub score_band: String,
    pub blockers: Vec<String>,
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
    #[serde(default)]
    pub identity_tier: Option<String>,
    #[serde(default)]
    pub signer_algorithms: Vec<String>,
    #[serde(default)]
    pub remaining_budget_units: Option<u32>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuantumHashLevel {
    pub algorithm: String,
    pub security_bits_classical: u16,
    pub security_bits_quantum_estimated: u16,
    pub purpose: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuantumKeyLevel {
    pub primitive: String,
    pub security_bits_classical: u16,
    pub security_bits_quantum_estimated: u16,
    pub purpose: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuantumCryptoProfile {
    pub profile_version: String,
    pub assurance_target_percent: f64,
    pub hash_levels: Vec<QuantumHashLevel>,
    pub key_levels: Vec<QuantumKeyLevel>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuantumControl {
    pub control_id: String,
    pub objective: String,
    pub enforcement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuantumApiCapability {
    pub name: String,
    pub status: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuantumCliCapability {
    pub command: String,
    pub status: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuantumOpsPlaybook {
    pub release_gate: Vec<String>,
    pub runtime_controls: Vec<String>,
    pub incident_response: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuantumFullProfile {
    pub profile_version: String,
    pub posture: String,
    pub api_capabilities: Vec<QuantumApiCapability>,
    pub cli_capabilities: Vec<QuantumCliCapability>,
    pub controls: Vec<QuantumControl>,
    pub ops_playbook: QuantumOpsPlaybook,
}
