use aoxchub::app::router::INTEGRATION_CHECKLIST;

#[test]
fn checklist_mentions_environment_strategy() {
    let joined = INTEGRATION_CHECKLIST
        .iter()
        .map(|(title, detail)| format!("{title} {detail}"))
        .collect::<Vec<_>>()
        .join(" ");

    assert!(joined.contains("Dev"));
    assert!(joined.contains("Testnet"));
    assert!(joined.contains("Mainnet"));
    assert!(joined.contains("RPC"));
}
