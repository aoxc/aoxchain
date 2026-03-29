use aoxchub::app::app_root::AppRoot;
use aoxchub::app::layout::sidebar::SIDEBAR_MENU_ITEMS;

#[test]
fn app_root_component_symbol_is_linkable() {
    let _entry: fn() -> dioxus::prelude::Element = AppRoot;
}

#[test]
fn sidebar_links_cover_primary_sections() {
    let anchors: Vec<&str> = SIDEBAR_MENU_ITEMS.iter().map(|(_, href)| *href).collect();
    for required in [
        "#integration-checklist",
        "#wallet-setup",
        "#overview",
        "#dashboard",
        "#validators",
    ] {
        assert!(anchors.contains(&required), "missing anchor: {required}");
    }
}
