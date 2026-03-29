use dioxus::prelude::*;

use crate::app::router::Route;

const ROUTE_MENU_ITEMS: [(&str, &str, Route); 5] = [
    ("◉", "Dashboard", Route::Home {}),
    ("◎", "Wallet", Route::Wallet {}),
    ("◌", "Operations", Route::Operations {}),
    ("◈", "Overview", Route::Overview {}),
    ("◍", "Settings", Route::Settings {}),
];

pub const SIDEBAR_MENU_ITEMS: [(&str, &str); 9] = [
    ("Integration Checklist", "#integration-checklist"),
    ("Wallet Setup", "#wallet-setup"),
    ("Overview", "#overview"),
    ("Dashboard Metrics", "#dashboard"),
    ("Validator Matrix", "#validators"),
    ("Bridge", "#bridge"),
    ("Governance", "#governance"),
    ("Staking", "#staking"),
    ("Ecosystem", "#ecosystem"),
];

#[component]
pub fn SidebarMenu() -> Element {
    rsx! {
        aside {
            class: "sidebar glass",

            div {
                class: "sidebar-brand",
                p { class: "sidebar-title", "AOXC Hub" }
                p { class: "sidebar-subtitle", "Operational command surface" }
            }

            p { class: "sidebar-label", "Route Navigation" }
            nav {
                class: "sidebar-nav",
                for (icon, label, route) in ROUTE_MENU_ITEMS {
                    Link {
                        class: "sidebar-link",
                        to: route,
                        span { class: "sidebar-link-icon", "{icon}" }
                        span { class: "sidebar-link-text", "{label}" }
                    }
                }
            }

            p { class: "sidebar-label", "Quick Anchors" }
            nav {
                class: "sidebar-nav",
                for (label, href) in SIDEBAR_MENU_ITEMS {
                    a {
                        class: "sidebar-link",
                        href: href,
                        span { class: "sidebar-link-icon", "•" }
                        span { class: "sidebar-link-text", "{label}" }
                    }
                }
            }

            section {
                class: "sidebar-banner",
                p { class: "sidebar-banner-title", "Mainnet Window" }
                p { class: "sidebar-banner-copy", "Bridge, governance, and validator controls are active." }
                button { class: "btn btn-primary sidebar-banner-btn", "Open Runtime Terminal" }
            }
        }
    }
}
