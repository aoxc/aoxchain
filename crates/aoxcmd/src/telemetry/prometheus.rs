use crate::{
    data_home::{resolve_home, write_file},
    error::{AppError, ErrorCode},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub node_height: u64,
    pub produced_blocks: u64,
    pub treasury_balance: u64,
    pub recorded_at: String,
}

pub fn metrics_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("telemetry").join("metrics.json"))
}

pub fn persist_metrics(snapshot: &MetricsSnapshot) -> Result<(), AppError> {
    let content = serde_json::to_string_pretty(snapshot)
        .map_err(|e| AppError::with_source(ErrorCode::OutputEncodingFailed, "Failed to encode metrics snapshot", e))?;
    write_file(&metrics_path()?, &content)
}

pub fn now(node_height: u64, produced_blocks: u64, treasury_balance: u64) -> MetricsSnapshot {
    MetricsSnapshot {
        node_height,
        produced_blocks,
        treasury_balance,
        recorded_at: Utc::now().to_rfc3339(),
    }
}
