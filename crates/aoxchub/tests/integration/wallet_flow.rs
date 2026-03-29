use aoxchub::app::router::INTEGRATION_CHECKLIST;

#[test]
fn checklist_includes_wallet_security_item() {
    assert!(INTEGRATION_CHECKLIST.iter().any(|(title, detail)| {
        title.contains("Wallet security") && detail.contains("session policy")
    }));
}
