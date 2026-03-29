use aoxchub::app::router::{INTEGRATION_CHECKLIST, Route};

#[test]
fn home_route_is_root_path() {
    assert_eq!(Route::Home {}.to_string(), "/");
}

#[test]
fn integration_checklist_has_core_items() {
    assert_eq!(INTEGRATION_CHECKLIST.len(), 5);
    assert!(
        INTEGRATION_CHECKLIST
            .iter()
            .any(|(title, _)| title.contains("Network profile"))
    );
    assert!(
        INTEGRATION_CHECKLIST
            .iter()
            .any(|(_, detail)| detail.contains("Dev / Testnet / Mainnet"))
    );
}
