use crate::services::telemetry::latest_snapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplorerReadModel {
    pub block_explorer: String,
    pub tx_explorer: String,
    pub receipt_viewer: String,
    pub event_viewer: String,
    pub state_diff_summary: String,
    pub contract_account_query: String,
    pub finality_proof_reference: String,
    pub source: String,
}

pub async fn read_explorer() -> ExplorerReadModel {
    let telemetry = latest_snapshot().await;
    let latest_block = telemetry
        .latest_block
        .map(|b| format!("latest block #{b}"))
        .unwrap_or_else(|| "latest block unavailable".to_string());

    ExplorerReadModel {
        block_explorer: latest_block,
        tx_explorer: "ready (hash query required)".to_string(),
        receipt_viewer: "ready (receipt hash required)".to_string(),
        event_viewer: "ready (event filters required)".to_string(),
        state_diff_summary: "authoritative state-diff API required".to_string(),
        contract_account_query: "ready (address query required)".to_string(),
        finality_proof_reference: "authoritative proof endpoint required".to_string(),
        source: "explorer_service <- rpc + indexer".to_string(),
    }
}
