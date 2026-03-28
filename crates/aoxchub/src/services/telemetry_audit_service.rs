use crate::services::intent_service::latest_audit_events;
use crate::services::telemetry::latest_snapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetryAuditReadModel {
    pub metrics: String,
    pub logs: String,
    pub health_checks: String,
    pub evidence_store: String,
    pub operator_action_history: String,
    pub alert_stream: String,
    pub anomaly_flags: String,
    pub incident_timeline: String,
    pub source: String,
}

pub async fn read_telemetry_audit() -> TelemetryAuditReadModel {
    let telemetry = latest_snapshot().await;
    let events = latest_audit_events();
    let event_count = events.len();

    TelemetryAuditReadModel {
        metrics: if telemetry.healthy {
            "healthy ingress".to_string()
        } else {
            "degraded ingress".to_string()
        },
        logs: "authoritative log sink required".to_string(),
        health_checks: if telemetry.healthy {
            "pass".to_string()
        } else {
            "fail".to_string()
        },
        evidence_store: "authoritative evidence store required".to_string(),
        operator_action_history: format!("{event_count} recent audit events"),
        alert_stream: "authoritative alert stream required".to_string(),
        anomaly_flags: "none detected from current transport probe".to_string(),
        incident_timeline: "authoritative incident API required".to_string(),
        source: "telemetry_audit_service <- telemetry + audit APIs".to_string(),
    }
}
