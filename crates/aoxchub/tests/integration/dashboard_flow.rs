use std::collections::HashSet;

use aoxchub::app::layout::sidebar::SIDEBAR_MENU_ITEMS;

#[test]
fn sidebar_items_are_unique_and_non_empty() {
    let mut labels = HashSet::new();
    let mut hrefs = HashSet::new();

    for (label, href) in SIDEBAR_MENU_ITEMS {
        assert!(!label.trim().is_empty(), "label must not be empty");
        assert!(href.starts_with('#'), "href must be hash anchor: {href}");
        assert!(labels.insert(label), "duplicate label: {label}");
        assert!(hrefs.insert(href), "duplicate href: {href}");
    }
}
