use dioxus::prelude::*;

#[component]
pub fn SidebarMenu() -> Element {
    let menu_items = [
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
                for (label, href) in menu_items {
                    a { href: href, "{label}" }
                }
            }
        }
    }
}
