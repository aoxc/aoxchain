#[test]
fn catalog_includes_release_matrix_and_sqlite_memory_actions() {
    assert!(aoxchub::commands::find("make-build-release-matrix").is_some());
    assert!(aoxchub::commands::find("make-publish-release").is_some());
    assert!(aoxchub::commands::find("make-db-init-sqlite").is_some());
    assert!(aoxchub::commands::find("make-db-status-sqlite").is_some());
    assert!(aoxchub::commands::find("ui-mainnet").is_some());
    assert!(aoxchub::commands::find("ui-testnet").is_some());
}

#[test]
fn catalog_includes_core_operator_surface_commands() {
    assert!(aoxchub::commands::find("aoxc-chain-create").is_some());
    assert!(aoxchub::commands::find("aoxc-genesis-verify").is_some());
    assert!(aoxchub::commands::find("aoxc-node-status").is_some());
    assert!(aoxchub::commands::find("aoxc-network-verify").is_some());
    assert!(aoxchub::commands::find("aoxc-wallet-balance").is_some());
    assert!(aoxchub::commands::find("aoxc-stake-delegate").is_some());
    assert!(aoxchub::commands::find("aoxc-audit-export").is_some());
}
