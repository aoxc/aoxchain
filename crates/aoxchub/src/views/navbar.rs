use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Navbar() -> Element {
    let menu_items = [
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
        div {
            class: "app-frame",
            header {
                class: "topbar",
                div {
                    class: "topbar-inner",
                    Link {
                        class: "brand",
                        to: Route::Home {},
                        span { class: "brand-mark", "AOX" }
                        span { class: "brand-text", "AOX Hub Control Center" }
                    }
                    div {
                        class: "network-pill",
                        span { class: "network-dot" }
                        "Mainnet Connected"
                    }
                }
            }

            div {
                class: "app-layout",
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

                main {
                    class: "main-content",
                    Outlet::<Route> {}
                }
            }

            footer {
                class: "footer",
                p { "AOX Hub is synchronized with AOXC chain services, validators, bridge relays, and governance streams." }
            }
        }
    }
}
