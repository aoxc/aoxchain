use dioxus::prelude::*;

mod views;
use views::{HubPage, Navbar};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
        #[route("/")]
        Home {},
}

const FAVICON: Asset = asset!("/assets/images/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styles/global.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}

#[component]
fn Home() -> Element {
    rsx! { HubPage {} }
}
