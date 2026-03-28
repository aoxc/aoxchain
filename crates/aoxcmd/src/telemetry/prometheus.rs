// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    data_home::{resolve_home, write_file},
    error::{AppError, ErrorCode},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Canonical AOXC metrics snapshot persisted for lightweight operator telemetry.
///
/// Design intent:
/// - Preserve a compact point-in-time operational snapshot.
/// - Keep the payload serializable and stable for diagnostics surfaces.
/// - Enforce semantic validation before persistence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MetricsSnapshot {
    pub node_height: u64,
    pub produced_blocks: u64,
    pub treasury_balance: u64,
    pub recorded_at: String,
}

impl MetricsSnapshot {
    /// Constructs a canonical metrics snapshot with an explicit timestamp.
    pub fn new(
        node_height: u64,
        produced_blocks: u64,
        treasury_balance: u64,
        recorded_at: String,
    ) -> Self {
        Self {
            node_height,
            produced_blocks,
            treasury_balance,
            recorded_at,
        }
    }

    /// Validates semantic integrity for a metrics snapshot.
    ///
    /// Validation policy:
    /// - `recorded_at` must be a valid RFC3339 timestamp.
    /// - `produced_blocks` must not exceed `node_height`.
    pub fn validate(&self) -> Result<(), String> {
        chrono::DateTime::parse_from_rfc3339(&self.recorded_at)
            .map_err(|_| "recorded_at must be a valid RFC3339 timestamp".to_string())?;

        if self.produced_blocks > self.node_height {
            return Err("produced_blocks cannot exceed node_height".to_string());
        }

        Ok(())
    }
}

/// Returns the canonical AOXC metrics snapshot path.
///
/// Canonical path policy:
/// - Metrics snapshots are persisted at:
///   `<AOXC_HOME>/telemetry/metrics.json`.
pub fn metrics_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("telemetry").join("metrics.json"))
}

/// Persists a validated metrics snapshot to the canonical telemetry path.
///
/// Validation policy:
/// - Semantic validation is enforced before serialization.
/// - Only validated telemetry payloads are written to disk.
pub fn persist_metrics(snapshot: &MetricsSnapshot) -> Result<(), AppError> {
    snapshot
        .validate()
        .map_err(|error| AppError::new(ErrorCode::AuditFailure, error))?;

    let path = metrics_path()?;
    let content = serde_json::to_string_pretty(snapshot).map_err(|error| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            format!("Failed to encode metrics snapshot for {}", path.display()),
            error,
        )
    })?;

    write_file(&path, &content)
}

/// Builds a point-in-time metrics snapshot using the current UTC timestamp.
pub fn now(node_height: u64, produced_blocks: u64, treasury_balance: u64) -> MetricsSnapshot {
    MetricsSnapshot::new(
        node_height,
        produced_blocks,
        treasury_balance,
        Utc::now().to_rfc3339(),
    )
}

#[cfg(test)]
mod tests {
    use super::{metrics_path, now, persist_metrics, MetricsSnapshot};
    use crate::{
        error::ErrorCode,
        test_support::{aoxc_home_test_lock, AoxcHomeGuard, TestHome},
    };

    fn with_test_home<T>(label: &str, test: impl FnOnce(&TestHome) -> T) -> T {
        let _lock = aoxc_home_test_lock();
        let home = TestHome::new(label);
        let _guard = AoxcHomeGuard::install(home.path());
        test(&home)
    }

    #[test]
    fn metrics_path_resolves_inside_active_test_home() {
        with_test_home("metrics-path", |home| {
            let path = metrics_path().expect("metrics path must resolve");

            assert_eq!(path, home.path().join("telemetry").join("metrics.json"));
        });
    }

    #[test]
    fn now_builds_valid_metrics_snapshot() {
        let snapshot = now(10, 10, 500);

        assert_eq!(snapshot.node_height, 10);
        assert_eq!(snapshot.produced_blocks, 10);
        assert_eq!(snapshot.treasury_balance, 500);
        assert!(snapshot.validate().is_ok());
    }

    #[test]
    fn validate_rejects_invalid_timestamp() {
        let snapshot = MetricsSnapshot::new(5, 5, 100, "not-a-timestamp".to_string());

        assert!(snapshot.validate().is_err());
    }

    #[test]
    fn validate_rejects_produced_blocks_above_height() {
        let snapshot = MetricsSnapshot::new(1, 2, 100, Utc::now().to_rfc3339());

        assert!(snapshot.validate().is_err());
    }

    #[test]
    fn persist_metrics_writes_valid_snapshot() {
        with_test_home("persist-metrics", |_home| {
            let snapshot = now(3, 3, 1000);

            persist_metrics(&snapshot).expect("metrics snapshot should persist");

            let path = metrics_path().expect("metrics path must resolve");
            assert!(path.is_file());
        });
    }

    #[test]
    fn persist_metrics_rejects_invalid_snapshot() {
        with_test_home("persist-metrics-invalid", |_home| {
            let snapshot = MetricsSnapshot::new(1, 9, 100, Utc::now().to_rfc3339());

            let error = persist_metrics(&snapshot).expect_err("invalid metrics snapshot must fail");

            assert_eq!(error.code(), ErrorCode::AuditFailure.as_str());
        });
    }
}
