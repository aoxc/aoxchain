use dioxus::prelude::*;

use crate::components::navigation::{Header, Sidebar};
use crate::route::Route;

#[component]
pub fn AdminLayout() -> Element {
    rsx! {
        div { class: "flex h-screen overflow-hidden bg-[#03050a] text-slate-100",
            Sidebar {}
            div { class: "flex min-w-0 flex-1 flex-col",
                Header {}
                main { class: "min-h-0 flex-1 overflow-y-auto p-6 md:p-8",
                    Outlet::<Route> {}
                }
            }
        }
    }
}
