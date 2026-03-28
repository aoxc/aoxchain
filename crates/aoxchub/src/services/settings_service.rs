use crate::services::network_profile::resolve_profile;
use crate::services::rpc_client::RpcClient;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsReadModel {
    pub rpc_endpoint_profile: String,
    pub environment_selector: String,
    pub api_auth: String,
    pub operator_identity: String,
    pub signer_integration: String,
    pub session_policy: String,
    pub access_roles: String,
    pub desktop_security_posture: String,
    pub source: String,
}

pub async fn read_settings() -> SettingsReadModel {
    let profile = resolve_profile();

    SettingsReadModel {
        rpc_endpoint_profile: RpcClient::endpoint(),
        environment_selector: profile.title().to_string(),
        api_auth: "token or mTLS required".to_string(),
        operator_identity: "operator/root@aoxhub".to_string(),
        signer_integration: "hardware signer supported".to_string(),
        session_policy: "short-lived session enforced".to_string(),
        access_roles: "viewer / operator / approver".to_string(),
        desktop_security_posture: "kernel boundary protected (UI control-plane only)".to_string(),
        source: "settings_service <- profile + auth APIs".to_string(),
    }
}
