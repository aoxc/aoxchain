use dioxus::prelude::*;

#[component]
pub fn FooterBar() -> Element {
    rsx! {
        footer {
            class: "footer",
            p { "AOX Hub is synchronized with AOXC chain services, validators, bridge relays, and governance streams." }
        }
    }
}
