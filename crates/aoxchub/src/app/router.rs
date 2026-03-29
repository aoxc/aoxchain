use dioxus::prelude::*;

use crate::app::layout::app_layout::AppLayout;
use crate::features::dashboard::page::DashboardSection;
use crate::features::explorer::page::OverviewSection;
use crate::features::operations::page::OperationsSection;
use crate::features::settings::page::SettingsSection;
use crate::features::wallet::page::WalletSetupSection;

pub const INTEGRATION_CHECKLIST: [(&str, &str); 5] = [
    (
        "Network profile baseline",
        "Validate Dev / Testnet / Mainnet endpoints with deterministic RPC failover.",
    ),
    (
        "Wallet security enforcement",
        "Confirm session policy, signer isolation, and key material handling controls.",
    ),
    (
        "Operations readiness",
        "Execute validator and bridge command drills with auditable runbook evidence.",
    ),
    (
        "Governance alignment",
        "Verify proposal state transitions, voting telemetry, and treasury signaling.",
    ),
    (
        "Release observability",
        "Correlate health checks, log streams, and alert thresholds before rollout.",
    ),
];

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[layout(AppLayout)]
    #[route("/")]
    Landing {},
    #[route("/dashboard")]
    Dashboard {},
    #[route("/wallet")]
    Wallet {},
    #[route("/operations")]
    Operations {},
    #[route("/overview")]
    Overview {},
    #[route("/settings")]
    Settings {},
}

#[component]
fn Landing() -> Element {
    let _ = INTEGRATION_CHECKLIST;
    rsx! { OverviewSection {} }
}

#[component]
fn Dashboard() -> Element {
    rsx! { DashboardSection {} }
}

#[component]
fn Wallet() -> Element {
    rsx! { WalletSetupSection {} }
}

#[component]
fn Operations() -> Element {
    rsx! { OperationsSection {} }
}

#[component]
fn Overview() -> Element {
    rsx! { OverviewSection {} }
}

#[component]
fn Settings() -> Element {
    rsx! { SettingsSection {} }
}
