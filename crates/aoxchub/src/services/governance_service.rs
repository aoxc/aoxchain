use crate::services::intent_service::governance_intents;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceReadModel {
    pub protocol_parameter_proposals: String,
    pub validator_management_intents: String,
    pub emergency_controls: String,
    pub upgrade_intents: String,
    pub signed_action_queue: String,
    pub approval_workflow: String,
    pub execution_status: String,
    pub source: String,
}

pub async fn read_governance() -> GovernanceReadModel {
    let intents = governance_intents();

    GovernanceReadModel {
        protocol_parameter_proposals: "authoritative governance API required".to_string(),
        validator_management_intents: "intent-only mode enabled".to_string(),
        emergency_controls: "guard-railed".to_string(),
        upgrade_intents: format!("{} intents queued", intents.len()),
        signed_action_queue: "signed intents only".to_string(),
        approval_workflow: "two-step approval required".to_string(),
        execution_status: "awaiting approvals".to_string(),
        source: "governance_service <- governance control API".to_string(),
    }
}
