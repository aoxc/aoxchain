use crate::app::router::Route;
use dioxus::prelude::*;

/// Renders the persistent top navigation bar for the AOXC Hub shell.
///
/// This component is strictly responsible for layout-level navigation chrome.
/// It must not own route declarations or page business logic.
#[component]
pub fn HeaderBar() -> Element {
    rsx! {
        header {
            class: "topbar",
            div {
                class: "topbar-inner",
                Link {
                    class: "brand",
                    to: Route::Home {},
                    span { class: "brand-mark", "AOX" }
                    span { class: "brand-text", "AOX Hub Control Center" }
                }
                div {
                    class: "network-pill",
                    span { class: "network-dot" }
                    "Mainnet Connected"
                }
            }
        }
    }
}
