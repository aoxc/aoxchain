use dioxus::prelude::*;

mod features;
mod views;

use views::{
    DashboardPage, ExplorerPage, HomePage, Navbar, OperationsPage, SettingsPage, WalletPage,
};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Navbar)]
        #[route("/")]
        Home {},
        #[route("/wallet")]
        Wallet {},
        #[route("/explorer")]
        Explorer {},
        #[route("/dashboard")]
        Dashboard {},
        #[route("/operations")]
        Operations {},
        #[route("/settings")]
        Settings {},
}

const FAVICON: Asset = asset!("/assets/images/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styles/global.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}

#[component]
fn Home() -> Element {
    rsx! { HomePage {} }
}

#[component]
fn Wallet() -> Element {
    rsx! { WalletPage {} }
}

#[component]
fn Explorer() -> Element {
    rsx! { ExplorerPage {} }
}

#[component]
fn Dashboard() -> Element {
    rsx! { DashboardPage {} }
}

#[component]
fn Operations() -> Element {
    rsx! { OperationsPage {} }
}

#[component]
fn Settings() -> Element {
    rsx! { SettingsPage {} }
}
