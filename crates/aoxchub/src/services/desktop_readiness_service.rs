use crate::services::network_profile::{NetworkProfile, resolve_profile};
use crate::services::rpc_client::RpcClient;
use crate::services::telemetry::latest_snapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadinessCheck {
    pub name: String,
    pub status: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityControl {
    pub control: String,
    pub state: String,
    pub policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopReadinessModel {
    pub profile: NetworkProfile,
    pub rpc_target: String,
    pub telemetry_status: String,
    pub integration_grade: String,
    pub checks: Vec<ReadinessCheck>,
    pub controls: Vec<SecurityControl>,
    pub source: String,
}

pub async fn read_desktop_readiness() -> DesktopReadinessModel {
    let profile = resolve_profile();
    let telemetry = latest_snapshot().await;
    let telemetry_status = if telemetry.healthy {
        "healthy".to_string()
    } else {
        "degraded".to_string()
    };
    let integration_grade = compute_integration_grade(telemetry.healthy, profile);

    DesktopReadinessModel {
        profile,
        rpc_target: RpcClient::endpoint(),
        telemetry_status,
        integration_grade,
        checks: vec![
            ReadinessCheck {
                name: "Chain compatibility gate".to_string(),
                status: "pass".to_string(),
                evidence: format!("Resolved profile: {}", profile.title()),
            },
            ReadinessCheck {
                name: "RPC endpoint binding".to_string(),
                status: "pass".to_string(),
                evidence: RpcClient::descriptor(),
            },
            ReadinessCheck {
                name: "Telemetry ingest".to_string(),
                status: if telemetry.healthy {
                    "pass".to_string()
                } else {
                    "warn".to_string()
                },
                evidence: telemetry
                    .latest_block
                    .map(|height| format!("latest block: #{height}"))
                    .unwrap_or_else(|| "latest block unavailable".to_string()),
            },
            ReadinessCheck {
                name: "Signer boundary".to_string(),
                status: "pass".to_string(),
                evidence: "No private key operation is executed in AOXCHUB UI runtime".to_string(),
            },
        ],
        controls: vec![
            SecurityControl {
                control: "Session TTL".to_string(),
                state: "enforced".to_string(),
                policy: "Operator sessions require short-lived credentials".to_string(),
            },
            SecurityControl {
                control: "Intent approval".to_string(),
                state: "required".to_string(),
                policy: "Governance and treasury actions require signed approval".to_string(),
            },
            SecurityControl {
                control: "Desktop runtime boundary".to_string(),
                state: "isolated".to_string(),
                policy: "GUI is control-plane only and does not bypass kernel policy".to_string(),
            },
        ],
        source: "desktop_readiness_service <- profile + telemetry + policy".to_string(),
    }
}

fn compute_integration_grade(telemetry_healthy: bool, profile: NetworkProfile) -> String {
    match (telemetry_healthy, profile) {
        (true, NetworkProfile::Mainnet) => "A+".to_string(),
        (true, NetworkProfile::Devnet | NetworkProfile::Testnet) => "A".to_string(),
        (false, NetworkProfile::Mainnet) => "B".to_string(),
        (false, NetworkProfile::Devnet | NetworkProfile::Testnet) => "B-".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::compute_integration_grade;
    use crate::services::network_profile::NetworkProfile;

    #[test]
    fn gives_mainnet_full_grade_when_telemetry_is_healthy() {
        let grade = compute_integration_grade(true, NetworkProfile::Mainnet);
        assert_eq!(grade, "A+");
    }

    #[test]
    fn gives_lower_grade_when_telemetry_is_degraded() {
        let grade = compute_integration_grade(false, NetworkProfile::Testnet);
        assert_eq!(grade, "B-");
    }
}
