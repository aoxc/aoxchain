use crate::services::rpc_client::{MetricsSnapshot, RpcClient};

#[derive(Debug, Clone)]
pub struct TelemetrySnapshot {
    pub metrics: MetricsSnapshot,
}

pub async fn latest_snapshot(client: &RpcClient) -> Result<TelemetrySnapshot, String> {
    let metrics = client.fetch_metrics().await?;
    Ok(TelemetrySnapshot { metrics })
}
