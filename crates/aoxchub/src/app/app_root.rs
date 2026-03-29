use dioxus::prelude::*;

use crate::app::layout::header::HeaderBar;
use crate::app::router::Route;

#[component]
pub fn AppRoot() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/styles/global.css") }

        div {
            class: "theme-dark app-shell",
            HeaderBar {}

            main {
                class: "app-shell-content",
                Router::<Route> {}
            }
        }
    }
}
