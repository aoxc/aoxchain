use dioxus::prelude::*;

use crate::app::router::Route;

pub const SIDEBAR_MENU_ITEMS: [(&str, Route); 5] = [
    ("Dashboard", Route::Dashboard {}),
    ("Wallet", Route::Wallet {}),
    ("Operations", Route::Operations {}),
    ("Overview", Route::Overview {}),
    ("Settings", Route::Settings {}),
];

#[component]
pub fn SidebarMenu() -> Element {
    rsx! {
        aside {
            class: "sidebar glass",

            p {
                class: "sidebar-label",
                "Navigation"
            }

            nav {
                class: "sidebar-nav",

                for (label, route) in SIDEBAR_MENU_ITEMS {
                    Link {
                        class: "sidebar-link",
                        to: route,
                        "{label}"
                    }
                }
            }
        }
    }
}
