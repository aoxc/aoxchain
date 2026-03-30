#[test]
fn catalog_includes_release_matrix_and_sqlite_memory_actions() {
    assert!(aoxchub::commands::find("make-build-release-matrix").is_some());
    assert!(aoxchub::commands::find("make-publish-release").is_some());
    assert!(aoxchub::commands::find("make-db-init-sqlite").is_some());
    assert!(aoxchub::commands::find("make-db-status-sqlite").is_some());
    assert!(aoxchub::commands::find("ui-mainnet").is_some());
    assert!(aoxchub::commands::find("ui-testnet").is_some());
}
