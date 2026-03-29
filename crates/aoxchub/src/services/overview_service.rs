use crate::services::network_profile::{NetworkProfile, resolve_profile};
use crate::services::rpc_client::RpcClient;
use crate::services::telemetry::latest_snapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverviewReadModel {
    pub chain_id: String,
    pub network_profile: NetworkProfile,
    pub latest_finalized_block: String,
    pub head_block: String,
    pub sync_status: String,
    pub peer_count: String,
    pub validator_count: String,
    pub network_health: String,
    pub alerts_summary: String,
    pub source: String,
}

pub async fn read_overview() -> OverviewReadModel {
    let telemetry = latest_snapshot().await;
    let network_profile = resolve_profile();
    let finalized = telemetry
        .latest_block
        .map(|v| format!("#{v}"))
        .unwrap_or_else(|| "unavailable".to_string());
    let peers = telemetry
        .peer_count
        .map(|v| v.to_string())
        .unwrap_or_else(|| "unavailable".to_string());

    OverviewReadModel {
        chain_id: telemetry
            .chain_id
            .clone()
            .unwrap_or_else(|| "unavailable".to_string()),
        network_profile,
        latest_finalized_block: finalized.clone(),
        head_block: finalized,
        sync_status: if telemetry.healthy {
            "synced".to_string()
        } else {
            "degraded".to_string()
        },
        peer_count: peers,
        validator_count: "authoritative validator registry required".to_string(),
        network_health: if telemetry.healthy {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        },
        alerts_summary: if telemetry.healthy {
            "no critical alerts".to_string()
        } else {
            "check telemetry transport".to_string()
        },
        source: format!("overview_service <- {}", RpcClient::descriptor()),
    }
}
