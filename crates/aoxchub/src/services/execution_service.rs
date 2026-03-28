#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionLaneModel {
    pub lane: &'static str,
    pub tps: String,
    pub gas_usage: String,
    pub failure_rate: String,
    pub checkpoint_continuity: String,
    pub commitment_root: String,
    pub compatibility_level: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionReadModel {
    pub lanes: Vec<ExecutionLaneModel>,
    pub source: String,
}

pub async fn read_execution_lanes() -> ExecutionReadModel {
    let placeholder = |lane: &'static str| ExecutionLaneModel {
        lane,
        tps: "authoritative lane telemetry required".to_string(),
        gas_usage: "authoritative lane telemetry required".to_string(),
        failure_rate: "authoritative lane telemetry required".to_string(),
        checkpoint_continuity: "authoritative lane telemetry required".to_string(),
        commitment_root: "authoritative lane telemetry required".to_string(),
        compatibility_level: "profile-aware compatibility pending".to_string(),
    };

    ExecutionReadModel {
        lanes: vec![
            placeholder("EVM lane"),
            placeholder("WASM lane"),
            placeholder("Move lane"),
            placeholder("Cardano-style lane"),
        ],
        source: "execution_service <- lane runtime APIs".to_string(),
    }
}
