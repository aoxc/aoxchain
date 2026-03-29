use dioxus::prelude::*;

use crate::app::layout::app_layout::AppLayout;
use crate::features::dashboard::page::DashboardSection;
use crate::features::explorer::page::OverviewSection;
use crate::features::operations::page::OperationsSection;
use crate::features::settings::page::SettingsSection;
use crate::features::wallet::page::WalletSetupSection;

pub const INTEGRATION_CHECKLIST: [(&str, &str); 5] = [
    (
        "Network profile isolation",
        "Route and endpoint state preserve strict Dev / Testnet / Mainnet boundaries.",
    ),
    (
        "RPC compatibility handshake",
        "Startup checks validate RPC version, chain id, and genesis hash alignment.",
    ),
    (
        "Wallet security",
        "Signing operations remain gated by desktop session policy and key-scope rules.",
    ),
    (
        "Observability readiness",
        "Health panels publish telemetry, logs, and validator risk summaries for operators.",
    ),
    (
        "Release gate",
        "Build checks and smoke validation must pass before Mainnet promotion.",
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
    let _integration_checklist = INTEGRATION_CHECKLIST;
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
