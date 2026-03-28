#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StakingReadModel {
    pub validator_list: String,
    pub voting_power: String,
    pub active_state: String,
    pub stake_distribution: String,
    pub delegation: String,
    pub rewards: String,
    pub slash_history: String,
    pub join_exit_rotation: String,
    pub source: String,
}

pub async fn read_staking() -> StakingReadModel {
    StakingReadModel {
        validator_list: "authoritative staking index required".to_string(),
        voting_power: "authoritative staking index required".to_string(),
        active_state: "authoritative staking index required".to_string(),
        stake_distribution: "authoritative staking index required".to_string(),
        delegation: "authoritative staking index required".to_string(),
        rewards: "authoritative staking index required".to_string(),
        slash_history: "authoritative staking index required".to_string(),
        join_exit_rotation: "authoritative staking index required".to_string(),
        source: "staking_service <- staking control API".to_string(),
    }
}
