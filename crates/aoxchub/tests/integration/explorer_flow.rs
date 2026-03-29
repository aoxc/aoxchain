use aoxchub::app::layout::sidebar::SIDEBAR_MENU_ITEMS;

#[test]
fn explorer_related_anchors_exist() {
    let anchor_set: std::collections::HashSet<&str> =
        SIDEBAR_MENU_ITEMS.iter().map(|(_, href)| *href).collect();

    for expected in ["#bridge", "#governance", "#staking", "#ecosystem"] {
        assert!(
            anchor_set.contains(expected),
            "missing explorer/domain anchor: {expected}"
        );
    }
}
