use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Navbar() -> Element {
    rsx! {
        header {
            class: "topbar",
            div {
                class: "topbar-inner",
                Link {
                    class: "brand",
                    to: Route::Home {},
                    span { class: "brand-mark", "AOX" }
                    span { class: "brand-text", "AOX Hub" }
                }
                nav {
                    class: "top-links",
                    a { href: "#overview", "Overview" }
                    a { href: "#validators", "Validators" }
                    a { href: "#activity", "Activity" }
                    a { href: "#ecosystem", "Ecosystem" }
                }
            }
        }
        Outlet::<Route> {}
    }
}
