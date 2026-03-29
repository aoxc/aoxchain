use dioxus::prelude::*;

use crate::components::navigation::{Header, RightOperationsPanel, Sidebar};
use crate::route::Route;

#[component]
pub fn AdminLayout() -> Element {
    rsx! {
        div { class: "aox-shell",
            Sidebar {}

            div { class: "aox-main-column",
                Header {}

                main { class: "aox-workspace",
                    Outlet::<Route> {}
                }

                footer { class: "aox-footer",
                    span { "AOXCHUB • Production Desktop Control Surface" }
                    span { "Consensus kernel, custody, and signer boundaries remain isolated by design." }
                }
            }

            RightOperationsPanel {}
        }
    }
}
