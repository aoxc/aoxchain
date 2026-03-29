use dioxus::prelude::*;

use crate::app::router::Route;

#[component]
pub fn AppRoot() -> Element {
    rsx! {
        document::Link {
            rel: "stylesheet",
            href: asset!("/assets/styles/global.css")
        }

        Router::<Route> {}
    }
}
