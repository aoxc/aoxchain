#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfrastructureReadModel {
    pub node_inventory: String,
    pub roles: String,
    pub region: String,
    pub latency: String,
    pub uptime: String,
    pub version: String,
    pub resource_usage: String,
    pub snapshot_status: String,
    pub sync_mode: String,
    pub failover_readiness: String,
    pub source: String,
}

pub async fn read_infrastructure() -> InfrastructureReadModel {
    InfrastructureReadModel {
        node_inventory: "authoritative node inventory required".to_string(),
        roles: "validator / archive / sentry".to_string(),
        region: "authoritative infra registry required".to_string(),
        latency: "authoritative telemetry required".to_string(),
        uptime: "authoritative telemetry required".to_string(),
        version: "authoritative node management API required".to_string(),
        resource_usage: "disk/memory/cpu feed required".to_string(),
        snapshot_status: "authoritative snapshot API required".to_string(),
        sync_mode: "full / snap / archive".to_string(),
        failover_readiness: "continuous checks required".to_string(),
        source: "infrastructure_service <- node control plane".to_string(),
    }
}
