use dioxus::prelude::*;

use crate::route::Route;
use crate::state::provide_global_state;

pub fn App() -> Element {
    provide_global_state();

    rsx! {
        document::Stylesheet { href: asset!("/assets/main.css") }
        Router::<Route> {}
    }
}
