use dioxus::prelude::*;

use crate::features::dashboard::page::DashboardSection;
use crate::features::explorer::domains::DomainSections;
use crate::features::explorer::page::OverviewSection;
use crate::features::operations::page::OperationsSection;
use crate::features::settings::page::SettingsSection;
use crate::features::wallet::page::WalletSetupSection;

/// Defines the canonical routing contract for AOXC Hub.
///
/// The router is intentionally minimal at this stage and exposes a single
/// stable entry route. Additional routes should only be introduced once their
/// page modules and navigation surfaces are fully implemented and exported.
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Home {},
}

/// Renders the canonical landing page for the current AOXC Hub release.
///
/// The page is intentionally lean and production-safe. It establishes a stable
/// route target while the broader application shell continues to evolve.
#[component]
pub fn Home() -> Element {
    let integration_checklist = [
        (
            "Network profile selector",
            "Dev / Testnet / Mainnet context toggles and isolation rules.",
        ),
        (
            "RPC capability handshake",
            "Version, chain-id, genesis hash, and method support validation.",
        ),
        (
            "Wallet security baseline",
            "Seed backup flow, session policy, and signing boundaries.",
        ),
        (
            "Observability hooks",
            "Structured logs, health telemetry, and operator-visible diagnostics.",
        ),
        (
            "Release gate",
            "Build checks, smoke tests, and deployment readiness evidence.",
        ),
    ];

    rsx! {
        div {
            class: "hub-page",

            header {
                class: "hero glass",
                h1 { "AOXC Hub Control Center" }
                p { class: "hero-sub", "Main operational interface is online. UI integration skeleton is now connected end-to-end." }
            }

            section {
                id: "integration-checklist",
                class: "panel glass",
                h2 { "System Integration Checklist" }
                p { class: "hero-sub", "Core checklist is embedded in the interface so content details can be expanded incrementally without leaving empty screens." }
                ul {
                    class: "activity-list",
                    for (item, detail) in integration_checklist {
                        li {
                            div {
                                p { class: "activity-kind", "{item}" }
                                p { class: "activity-pair", "{detail}" }
                            }
                            div {
                                p { class: "activity-amount", "Pending Detail" }
                                p { class: "activity-time", "Ready for content input" }
                            }
                        }
                    }
                }
            }

            WalletSetupSection {}
            OverviewSection {}
            DashboardSection {}
            OperationsSection {}
            SettingsSection {}
            DomainSections {}
        }
    }
}
