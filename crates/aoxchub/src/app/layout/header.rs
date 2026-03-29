use crate::Route;
use dioxus::prelude::*;

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
