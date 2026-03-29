use dioxus::prelude::*;

/// Defines the canonical routing contract for AOXC Hub.
///
/// The router is intentionally minimal at this stage and exposes a single
/// stable entry route. Additional routes should only be introduced once their
/// page modules and navigation surfaces are fully implemented and exported.
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Home {},
}

/// Renders the canonical landing page for the current AOXC Hub release.
///
/// The page is intentionally lean and production-safe. It establishes a stable
/// route target while the broader application shell continues to evolve.
#[component]
pub fn Home() -> Element {
    rsx! {
        div {
            class: "page page-home",

            header {
                class: "page-header",
                h1 { "AOXC Hub Control Center" }
                p { "Main operational interface is online." }
            }

            section {
                class: "page-section",
                h2 { "Status" }
                p { "Routing has been normalized through the centralized app router." }
            }
        }
    }
}
