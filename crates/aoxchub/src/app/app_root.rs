use dioxus::prelude::*;

use crate::app::layout::footer::FooterBar;
use crate::app::layout::header::HeaderBar;
use crate::app::layout::sidebar::SidebarMenu;
use crate::app::router::Route;

#[component]
pub fn AppRoot() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/styles/global.css") }

        div {
            class: "theme-dark app-frame",
            HeaderBar {}

            div {
                class: "app-layout",
                SidebarMenu {}

                main {
                    class: "main-content",
                    Router::<Route> {}
                }
            }

            FooterBar {}
        }
    }
}
