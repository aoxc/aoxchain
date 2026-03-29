use dioxus::prelude::*;

use crate::{
    Route,
    features::{
        dashboard::page::DashboardSection, explorer::page::OverviewSection,
        operations::page::OperationsSection, settings::page::SettingsSection,
        wallet::page::WalletSetupSection,
    },
};

#[component]
pub fn Navbar() -> Element {
    let menu_items = [
        ("Home", Route::Home {}),
        ("Wallet", Route::Wallet {}),
        ("Explorer", Route::Explorer {}),
        ("Dashboard", Route::Dashboard {}),
        ("Operations", Route::Operations {}),
        ("Settings", Route::Settings {}),
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
                        span { class: "brand-text", "AOXC Hub Control Center" }
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
                        for (label, route) in menu_items {
                            Link { to: route, "{label}" }
                        }
                    }
                }
                main {
                    class: "main-content",
                    div { class: "hub-page", Outlet::<Route> {} }
                }
            }

            footer {
                class: "footer",
                p { "AOX Hub is synchronized with AOXC chain services, validators, bridge relays, and governance streams." }
            }
        }
    }
}

#[component]
pub fn HomePage() -> Element {
    rsx! {
        section {
            class: "hero glass",
            div {
                class: "hero-copy",
                p { class: "eyebrow", "AOXC Integrated Operator Console" }
                h1 { "Gerçek AOXCHub Arayüzü" }
                p {
                    class: "hero-sub",
                    "Bu arayüz demo tek sayfa değildir; Wallet, Explorer, Dashboard, Operations ve Settings sayfaları ayrı rotalarda aktif olarak çalışır."
                }
            }
            div {
                class: "hero-panel",
                h3 { "Platform Status" }
                ul {
                    li { span { "Routing" } strong { "Active" } }
                    li { span { "Feature pages" } strong { "Loaded" } }
                    li { span { "AOXC binary integration" } strong { "Ready" } }
                    li { span { "UI shell" } strong { "Stable" } }
                }
            }
        }
    }
}

#[component]
pub fn WalletPage() -> Element {
    rsx! { WalletSetupSection {} }
}

#[component]
pub fn ExplorerPage() -> Element {
    rsx! { OverviewSection {} }
}

#[component]
pub fn DashboardPage() -> Element {
    rsx! { DashboardSection {} }
}

#[component]
pub fn OperationsPage() -> Element {
    rsx! { OperationsSection {} }
}

#[component]
pub fn SettingsPage() -> Element {
    rsx! { SettingsSection {} }
}
