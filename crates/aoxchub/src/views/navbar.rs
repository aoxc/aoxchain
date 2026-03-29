use crate::Route;
use dioxus::prelude::*;

use super::layout::{FooterBar, HeaderBar, SidebarMenu};

#[component]
pub fn Navbar() -> Element {
    rsx! {
        div {
            class: "app-frame",
            HeaderBar {}
            div {
                class: "app-layout",
                SidebarMenu {}
                main {
                    class: "main-content",
                    Outlet::<Route> {}
                }
            }
            FooterBar {}
        }
    }
}
