use dioxus::prelude::*;
use serde::Deserialize;

use crate::app::router::Route;

#[derive(Debug, Deserialize, Clone)]
struct RolloutSnapshot {
    status: String,
    surfaces: Vec<String>,
    requirements: Vec<String>,
}

fn load_rollout_snapshot() -> RolloutSnapshot {
    serde_json::from_str(include_str!(
        "../../../../../artifacts/network-production-closure/aoxhub-rollout.json"
    ))
    .unwrap_or_else(|_| RolloutSnapshot {
        status: String::from("unknown"),
        surfaces: Vec::new(),
        requirements: Vec::new(),
    })
}

#[component]
pub fn DashboardSection() -> Element {
    let kpis = [
        ("Network TPS", "42,781", "+18.4%"),
        ("Finality", "480ms", "-12.1%"),
        ("24h Volume", "$1.28B", "+9.7%"),
        ("Active Wallets", "892,114", "+6.2%"),
    ];

    let spotlight = [
        (
            "Cross-chain relay expansion",
            "New relays for Ethereum, Cardano, and Sui are synchronized with policy-safe execution windows.",
            "Bridge Health: 99.997%",
        ),
        (
            "Governance voting week",
            "Epoch 219 proposal review is active. Operators can sign, audit, and publish voting evidence.",
            "Live Proposals: 14",
        ),
        (
            "Validator quality matrix",
            "Regional validator telemetry indicates stable performance and no unresolved consensus faults.",
            "Incidents: 0 critical",
        ),
    ];

    let widgets = [
        ("Node Sync", "99.998%", "All regions healthy"),
        ("Smart Contract Calls", "18.2M/day", "EVM + WASM pipelines"),
        ("IBC Packets", "2.4M", "Cross-chain finalized"),
        ("Alert Queue", "03", "2 warning, 1 info"),
        ("Treasury Balance", "94.2M AOXC", "Policy constrained"),
        ("Governance Cycle", "Epoch 219", "Voting phase active"),
    ];

    let release_tracks = [
        (
            "Mainnet",
            "Release Candidate 2026.03.29",
            "All readiness gates passed with full deterministic coverage.",
        ),
        (
            "Testnet",
            "Canary Build 2026.03.29",
            "Chaos profile enabled with latency and gossip stress controls.",
        ),
        (
            "Dev",
            "Nightly Build 2026.03.29",
            "Protocol experiments and schema migration rehearsals are active.",
        ),
    ];

    rsx! {
        section {
            class: "hero glass dashboard-hero",
            div {
                class: "hero-copy",
                p { class: "eyebrow", "AOXC Control Tower" }
                h1 { "Cam gibi net, tam kapsamlı operasyon arayüzü" }
                p {
                    class: "hero-sub",
                    "Dashboard, banner, widget, menü ve kritik zincir akışlarını tek panelde toplayan genişletilmiş AOXC Hub deneyimi."
                }
                div {
                    class: "hero-actions",
                    Link { class: "btn btn-primary", to: Route::Operations {}, "Open Operations Center" }
                    Link { class: "btn btn-ghost", to: Route::Wallet {}, "Manage Wallet Flows" }
                }
            }
            div {
                class: "hero-panel",
                h3 { "Today at a glance" }
                ul {
                    li { span { "Consensus" } strong { "Stable" } }
                    li { span { "Bridge Relays" } strong { "Synchronized" } }
                    li { span { "Governance" } strong { "Voting Active" } }
                    li { span { "Alert Status" } strong { "3 Non-Critical" } }
                }
            }
        }

        section {
            class: "spotlight-slider",
            for (title, body, tag) in spotlight {
                article {
                    class: "spotlight-card glass",
                    p { class: "spotlight-tag", "{tag}" }
                    h3 { "{title}" }
                    p { class: "hero-sub", "{body}" }
                }
            }
        }

        section {
            id: "dashboard",
            class: "metrics-grid",
            for (title, value, delta) in kpis {
                article {
                    class: "metric-card glass",
                    p { class: "metric-title", "{title}" }
                    p { class: "metric-value", "{value}" }
                    p { class: "metric-delta", "{delta}" }
                }
            }
        }

        section {
            class: "widget-grid",
            for (title, value, note) in widgets {
                article {
                    class: "widget-card glass",
                    p { class: "widget-title", "{title}" }
                    h3 { "{value}" }
                    p { class: "widget-note", "{note}" }
                }
            }
        }

        section {
            class: "release-grid",
            for (track, version, detail) in release_tracks {
                article {
                    class: "release-card glass",
                    p { class: "widget-title", "{track}" }
                    h3 { "{version}" }
                    p { class: "widget-note", "{detail}" }
                }
            }
        }

        section {
            class: "menu-cards",
            Link {
                class: "menu-card glass",
                to: Route::Dashboard {},
                h3 { "Dashboard" }
                p { "Real-time KPI view, spotlight cards, and operator summaries." }
            }
            Link {
                class: "menu-card glass",
                to: Route::Wallet {},
                h3 { "Wallet" }
                p { "Address lifecycle, funding flow, and policy binding actions." }
            }
            Link {
                class: "menu-card glass",
                to: Route::Operations {},
                h3 { "Operations" }
                p { "Validator matrix, runtime health, and stream diagnostics." }
            }
            Link {
                class: "menu-card glass",
                to: Route::Overview {},
                h3 { "Overview" }
                p { "Network-level architecture and ecosystem health snapshot." }
            }
            Link {
                class: "menu-card glass",
                to: Route::Settings {},
                h3 { "Settings" }
                p { "Endpoint controls, language preferences, and automation setup." }
            }
        }
    }
}
