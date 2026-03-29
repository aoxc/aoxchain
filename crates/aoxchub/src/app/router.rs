use dioxus::prelude::*;

use crate::app::layout::app_layout::AppLayout;
use crate::features::dashboard::page::DashboardSection;
use crate::features::explorer::page::OverviewSection;
use crate::features::operations::page::OperationsSection;
use crate::features::settings::page::SettingsSection;
use crate::features::wallet::page::WalletSetupSection;

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
