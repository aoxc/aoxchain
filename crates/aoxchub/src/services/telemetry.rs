#[derive(Debug, Clone)]
pub struct TelemetrySnapshot {
    pub healthy: bool,
    pub source: &'static str,
}

pub fn latest_snapshot() -> TelemetrySnapshot {
    TelemetrySnapshot {
        healthy: true,
        source: "mock-local",
    }
}
