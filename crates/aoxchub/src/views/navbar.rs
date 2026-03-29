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
                    span {
                        class: "brand-stack",
                        strong { "AOX Hub" }
                        small { "Integrated Runtime Surface" }
                    }
                }
                div {
                    class: "top-actions",
                    span { class: "status-dot" }
                    p { "System Ready" }
                    button { class: "btn btn-ghost", "Run Diagnostics" }
                }
            }
        }
        Outlet::<Route> {}
    }
}
