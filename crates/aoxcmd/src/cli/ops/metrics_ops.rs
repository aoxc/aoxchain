use super::*;

pub fn cmd_metrics(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct MetricsView {
        node_height: u64,
        produced_blocks: u64,
        treasury_balance: u64,
        recorded_at: String,
        source: &'static str,
    }

    let state = lifecycle::load_state()?;
    let ledger = ledger::load().unwrap_or_default();
    let metrics_path = crate::telemetry::prometheus::metrics_path()?;
    if metrics_path.exists() {
        let raw = fs::read_to_string(&metrics_path).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!(
                    "Failed to read metrics snapshot from {}",
                    metrics_path.display()
                ),
                error,
            )
        })?;
        let snapshot: crate::telemetry::prometheus::MetricsSnapshot = serde_json::from_str(&raw)
            .map_err(|error| {
                AppError::with_source(
                    ErrorCode::OutputEncodingFailed,
                    format!(
                        "Failed to parse metrics snapshot from {}",
                        metrics_path.display()
                    ),
                    error,
                )
            })?;
        let response = MetricsView {
            node_height: snapshot.node_height,
            produced_blocks: snapshot.produced_blocks,
            treasury_balance: snapshot.treasury_balance,
            recorded_at: snapshot.recorded_at,
            source: "telemetry-snapshot",
        };
        return emit_serialized(&response, output_format(args));
    }

    let response = MetricsView {
        node_height: state.current_height,
        produced_blocks: state.produced_blocks,
        treasury_balance: ledger.treasury_balance,
        recorded_at: Utc::now().to_rfc3339(),
        source: "derived-live",
    };
    emit_serialized(&response, output_format(args))
}
