use dioxus::prelude::*;

use crate::app::router::Route;

const CORE_MENU_ITEMS: [(&str, &str, Route); 3] = [
    ("◉", "Dashboard", Route::Dashboard {}),
    ("◎", "Wallet", Route::Wallet {}),
    ("◌", "Operations", Route::Operations {}),
];

const CONTROL_MENU_ITEMS: [(&str, &str, Route); 2] = [
    ("◈", "Overview", Route::Overview {}),
    ("◍", "Settings", Route::Settings {}),
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

            p { class: "sidebar-label", "Core Navigation" }
            nav {
                class: "sidebar-nav",
                for (icon, label, route) in CORE_MENU_ITEMS {
                    Link {
                        class: "sidebar-link",
                        to: route,
                        span { class: "sidebar-link-icon", "{icon}" }
                        span { class: "sidebar-link-text", "{label}" }
                    }
                }
            }

            p { class: "sidebar-label", "Control Panels" }
            nav {
                class: "sidebar-nav",
                for (icon, label, route) in CONTROL_MENU_ITEMS {
                    Link {
                        class: "sidebar-link",
                        to: route,
                        span { class: "sidebar-link-icon", "{icon}" }
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
