use dioxus::prelude::*;

use crate::app::router::Route;

#[component]
pub fn HeaderBar() -> Element {
    rsx! {
        header {
            class: "header glass",

            div {
                class: "header-brand",

                Link {
                    class: "header-home-link",
                    to: Route::Home {},
                    "AOXC Hub Control Center"
                }
            }

            div {
                class: "header-actions",
                button {
                    class: "header-action-btn",
                    r#type: "button",
                    "Launch Mainnet Console"
                }
            }
        }
    }
}
