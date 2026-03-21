use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct UnityStatus {
    pub consensus_mode: &'static str,
    pub quorum_profile: &'static str,
}

pub fn unity_status() -> UnityStatus {
    UnityStatus {
        consensus_mode: "deterministic-local",
        quorum_profile: "single-operator-bootstrap",
    }
}
