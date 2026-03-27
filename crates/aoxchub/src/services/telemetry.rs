use crate::services::rpc_client::{MetricsSnapshot, RpcClient};

#[derive(Debug, Clone)]
pub struct TelemetrySnapshot {
    pub metrics: MetricsSnapshot,
}

pub async fn latest_snapshot(client: &RpcClient) -> Result<TelemetrySnapshot, String> {
    let metrics = client.fetch_metrics().await?;
    Ok(TelemetrySnapshot { metrics })
}

#[cfg(test)]
mod tests {
    use super::TelemetrySnapshot;
    use crate::services::rpc_client::MetricsSnapshot;

    #[test]
    fn telemetry_snapshot_holds_metrics_values() {
        let snapshot = TelemetrySnapshot {
            metrics: MetricsSnapshot {
                requests_total: 10.0,
                rejected_total: 1.0,
                rate_limited_total: 2.0,
                readiness_score: 90.0,
            },
        };

        assert_eq!(snapshot.metrics.requests_total, 10.0);
        assert_eq!(snapshot.metrics.rejected_total, 1.0);
        assert_eq!(snapshot.metrics.rate_limited_total, 2.0);
        assert_eq!(snapshot.metrics.readiness_score, 90.0);
    }
}
