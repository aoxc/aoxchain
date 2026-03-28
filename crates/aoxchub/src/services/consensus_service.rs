use crate::services::telemetry::latest_snapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsensusReadModel {
    pub current_epoch: String,
    pub current_height: String,
    pub current_round: String,
    pub proposer: String,
    pub quorum_threshold: String,
    pub finalized_head: String,
    pub lock_state: String,
    pub timeout_events: String,
    pub equivocation_evidence: String,
    pub continuity_certificate: String,
    pub legitimacy_certificate: String,
    pub execution_certificate: String,
    pub source: String,
}

pub async fn read_consensus() -> ConsensusReadModel {
    let telemetry = latest_snapshot().await;
    let height = telemetry
        .latest_block
        .map(|v| format!("#{v}"))
        .unwrap_or_else(|| "unavailable".to_string());

    ConsensusReadModel {
        current_epoch: "authoritative consensus API required".to_string(),
        current_height: height.clone(),
        current_round: "authoritative consensus API required".to_string(),
        proposer: "authoritative consensus API required".to_string(),
        quorum_threshold: "2f+1".to_string(),
        finalized_head: height,
        lock_state: if telemetry.healthy {
            "stable".to_string()
        } else {
            "unknown".to_string()
        },
        timeout_events: "0 recent (telemetry-limited)".to_string(),
        equivocation_evidence: "none observed".to_string(),
        continuity_certificate: "pending authoritative certificate feed".to_string(),
        legitimacy_certificate: "pending authoritative certificate feed".to_string(),
        execution_certificate: "pending authoritative certificate feed".to_string(),
        source: "consensus_service <- control API + telemetry".to_string(),
    }
}
