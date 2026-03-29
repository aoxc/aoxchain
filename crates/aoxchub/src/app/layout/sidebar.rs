use dioxus::prelude::*;

pub const SIDEBAR_MENU_ITEMS: [(&str, &str); 10] = [
    ("Integration Checklist", "#integration-checklist"),
    ("Wallet Setup", "#wallet-setup"),
    ("Overview", "#overview"),
    ("Dashboard", "#dashboard"),
    ("Validators", "#validators"),
    ("RPC Monitor", "#rpc-monitor"),
    ("Bridge", "#bridge"),
    ("Governance", "#governance"),
    ("Staking", "#staking"),
    ("Ecosystem", "#ecosystem"),
];

#[component]
pub fn SidebarMenu() -> Element {
    let menu_items = [
        ("Integration Checklist", "#integration-checklist"),
        ("Wallet Setup", "#wallet-setup"),
        ("Overview", "#overview"),
        ("Dashboard", "#dashboard"),
        ("Validators", "#validators"),
        ("RPC Monitor", "#rpc-monitor"),
        ("Bridge", "#bridge"),
        ("Governance", "#governance"),
        ("Staking", "#staking"),
        ("Ecosystem", "#ecosystem"),
    ];

    rsx! {
        aside {
            class: "sidebar glass",
            p { class: "sidebar-label", "Navigation" }
            nav {
                class: "sidebar-nav",
                for (label, href) in SIDEBAR_MENU_ITEMS {
                    a { href: href, "{label}" }
                }
            }
        }
    }
}
