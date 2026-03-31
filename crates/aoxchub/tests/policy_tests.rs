use aoxchub::{environments::Environment, services::HubService};

#[tokio::test]
async fn mainnet_blocks_high_risk_packaging_actions() {
    let service = HubService::new();
    service.set_environment(Environment::Mainnet).await;

    assert!(!service.is_command_allowed(Environment::Mainnet, "make-publish-release"));
    assert!(!service.is_command_allowed(Environment::Mainnet, "make-build-release-matrix"));
}

#[tokio::test]
async fn mainnet_allows_explicitly_approved_high_risk_runtime_actions() {
    let service = HubService::new();
    service.set_environment(Environment::Mainnet).await;

    assert!(service.is_command_allowed(Environment::Mainnet, "mainnet-start"));
    assert!(service.is_command_allowed(Environment::Mainnet, "aoxc-node-start"));
    assert!(service.is_command_allowed(Environment::Mainnet, "aoxc-node-stop"));
}

#[tokio::test]
async fn unknown_commands_are_fail_closed() {
    let service = HubService::new();
    service.set_environment(Environment::Testnet).await;

    assert!(!service.is_command_allowed(Environment::Testnet, "not-a-command"));
}
