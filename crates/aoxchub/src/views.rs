use dioxus::prelude::*;

use crate::Route;

#[component]
pub fn Navbar() -> Element {
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
                        a { href: "#wallet-setup", "Wallet Setup" }
                        a { href: "#overview", "Overview" }
                        a { href: "#dashboard", "Dashboard" }
                        a { href: "#validators", "Validators" }
                        a { href: "#rpc-monitor", "RPC Monitor" }
                        a { href: "#bridge", "Bridge" }
                        a { href: "#governance", "Governance" }
                        a { href: "#staking", "Staking" }
                        a { href: "#ecosystem", "Ecosystem" }
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

#[component]
pub fn HubPage() -> Element {
    let commands = [
        (
            "Node lifecycle",
            "aoxc node start --profile localnet",
            "Service control panel",
        ),
        (
            "Health probes",
            "aoxc ops health --json",
            "RPC monitor + validator panel",
        ),
        (
            "Wallet sync",
            "aoxc wallet sync --all",
            "Wallet setup + balances",
        ),
        (
            "Bridge monitor",
            "aoxc bridge status --watch",
            "Bridge menu live events",
        ),
        (
            "Governance queue",
            "aoxc gov proposals --open",
            "Governance section",
        ),
        (
            "Staking matrix",
            "aoxc stake validators --rank",
            "Staking section",
        ),
    ];

    rsx! {
        div {
            class: "hub-page",

            section {
                id: "wallet-setup",
                class: "panel glass",
                h2 { "Wallet Onboarding" }
                p { class: "hero-sub", "Cüzdan kurulum, yedekleme, fonlama ve politika bağlama adımları tek panelde tamamlanır." }
            }

            section {
                id: "overview",
                class: "hero glass",
                div {
                    class: "hero-copy",
                    p { class: "eyebrow", "AOXC Real Network Operations" }
                    h1 { "AOXCHub: Integrated Control Surface" }
                    p { class: "hero-sub", "Tüm menüler zincir operasyonları, binary komutları ve servis telemetrisi ile aynı arayüzde entegredir." }
                }
                div {
                    class: "hero-panel",
                    h3 { "Live Network Status" }
                    ul {
                        li { span { "Consensus" } strong { "Healthy" } }
                        li { span { "Bridge Relays" } strong { "Synchronized" } }
                        li { span { "RPC Regions" } strong { "47 Online" } }
                        li { span { "Governance" } strong { "Running" } }
                    }
                }
            }

            section {
                id: "dashboard",
                class: "metrics-grid",
                article { class: "metric-card glass", p { class: "metric-title", "Network TPS" } p { class: "metric-value", "42,781" } p { class: "metric-delta", "+18.4%" } }
                article { class: "metric-card glass", p { class: "metric-title", "Finality" } p { class: "metric-value", "480ms" } p { class: "metric-delta", "-12.1%" } }
                article { class: "metric-card glass", p { class: "metric-title", "24h Volume" } p { class: "metric-value", "$1.28B" } p { class: "metric-delta", "+9.7%" } }
                article { class: "metric-card glass", p { class: "metric-title", "Active Wallets" } p { class: "metric-value", "892,114" } p { class: "metric-delta", "+6.2%" } }
            }

            section {
                class: "content-grid",
                article {
                    id: "validators",
                    class: "panel glass",
                    h2 { "Validator Matrix" }
                    table {
                        class: "hub-table",
                        thead { tr { th { "Node" } th { "Uptime" } th { "Stake" } th { "Region" } } }
                        tbody {
                            tr { td { "Atlas One" } td { "99.99%" } td { "6.2M AOXC" } td { "Europe" } }
                            tr { td { "Cypher Labs" } td { "99.97%" } td { "5.8M AOXC" } td { "North America" } }
                            tr { td { "Delta Forge" } td { "99.95%" } td { "5.2M AOXC" } td { "Asia" } }
                        }
                    }
                }

                article {
                    id: "rpc-monitor",
                    class: "panel glass",
                    h2 { "Binary Command Integration" }
                    ul { class: "activity-list",
                        for (scope, command, target) in commands {
                            li {
                                div {
                                    p { class: "activity-kind", "{scope}" }
                                    p { class: "activity-pair", "{command}" }
                                }
                                div {
                                    p { class: "activity-amount", "Integrated" }
                                    p { class: "activity-time", "{target}" }
                                }
                            }
                        }
                    }
                }
            }

            section { id: "bridge", class: "panel glass", h2 { "Bridge" } p { "Cross-chain relay health, queue depth and transfer confirmations are visible here." } }
            section { id: "governance", class: "panel glass", h2 { "Governance" } p { "Proposal lifecycle, voting windows and execution readiness are tracked end-to-end." } }
            section { id: "staking", class: "panel glass", h2 { "Staking" } p { "Delegation flows, validator scoring and reward snapshots are centralized in this menu." } }
            section { id: "ecosystem", class: "panel glass", h2 { "Ecosystem" } p { "Ecosystem services, developer APIs and operational dashboards are linked from one surface." } }
        }
    }
}
