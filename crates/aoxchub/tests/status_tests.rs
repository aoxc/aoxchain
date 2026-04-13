use aoxchub::{environments::Environment, services::HubService};

#[tokio::test]
async fn state_contains_dashboard_snapshot_fields() {
    let service = HubService::new();
    service.set_environment(Environment::Mainnet).await;

    let state = service.state().await;
    assert_eq!(state.environment, Environment::Mainnet);
    assert!(!state.dashboard.chain_name.is_empty());
    assert!(state.dashboard.current_height >= state.dashboard.finalized_height);
    assert!(state.dashboard.validator_count > 0);
    assert!(!state.dashboard.genesis_fingerprint.is_empty());
    assert_eq!(state.dashboard.quick_actions.len(), 4);
    if state.dashboard.selected_binary_id.is_some() {
        assert!(state.dashboard.selected_binary_path.is_some());
        assert_eq!(state.dashboard.selected_binary_allowed, Some(true));
    } else {
        assert_eq!(state.dashboard.selected_binary_path, None);
        assert_eq!(state.dashboard.selected_binary_allowed, None);
    }
}

#[tokio::test]
async fn dashboard_updates_between_environments() {
    let service = HubService::new();

    service.set_environment(Environment::Mainnet).await;
    let mainnet = service.state().await.dashboard;

    service.set_environment(Environment::Testnet).await;
    let testnet = service.state().await.dashboard;

    assert_ne!(mainnet.network_kind, testnet.network_kind);
    assert_ne!(mainnet.network_id, testnet.network_id);
}

#[tokio::test]
async fn dashboard_warns_when_selected_binary_is_disallowed_for_environment() {
    let service = HubService::new();
    service.set_environment(Environment::Testnet).await;

    service
        .add_custom_binary(String::from("/tmp/aoxc-custom"))
        .await
        .expect("custom binary should be accepted in testnet");

    let state = service.state().await;
    let custom = state
        .binaries
        .iter()
        .find(|candidate| candidate.path == "/tmp/aoxc-custom")
        .expect("custom binary candidate must exist");

    service
        .set_binary(custom.id.clone())
        .await
        .expect("custom binary should be selectable in testnet");

    service.set_environment(Environment::Mainnet).await;
    let mainnet_state = service.state().await;

    assert_eq!(mainnet_state.dashboard.selected_binary_allowed, Some(false));
    assert!(mainnet_state.dashboard.last_warnings.iter().any(|line| {
        line.contains("Selected binary source is not allowed in the active environment policy")
    }));
    assert_eq!(mainnet_state.dashboard.health_status, "restricted");
}

#[tokio::test]
async fn dashboard_is_degraded_without_selected_binary() {
    let service = HubService::new();
    service.set_environment(Environment::Testnet).await;
    service
        .add_custom_binary(String::from("/tmp/aoxc-custom-for-none"))
        .await
        .expect("custom binary should be accepted in testnet");

    let state = service.state().await;
    let custom = state
        .binaries
        .iter()
        .find(|candidate| candidate.path == "/tmp/aoxc-custom-for-none")
        .expect("custom binary candidate must exist");

    service
        .set_binary(custom.id.clone())
        .await
        .expect("custom binary should be selectable in testnet");

    *service.selected_binary_id.write().await = None;
    let state_without_selection = service.state().await;

    assert_eq!(state_without_selection.dashboard.selected_binary_id, None);
    assert_eq!(state_without_selection.dashboard.selected_binary_path, None);
    assert_eq!(
        state_without_selection.dashboard.selected_binary_allowed,
        None
    );
    assert_eq!(state_without_selection.dashboard.health_status, "degraded");
    assert!(
        state_without_selection
            .dashboard
            .last_warnings
            .iter()
            .any(|line| line.contains("No AOXC binary is currently selected"))
    );
}
