use dioxus::prelude::*;

use crate::route::Route;
use crate::services::telemetry::latest_snapshot;
use crate::state::provide_global_state;

pub fn App() -> Element {
    let mut chain = provide_global_state();

    let telemetry = use_resource(move || async move { latest_snapshot().await });
    use_effect(move || {
        if let Some(snapshot) = telemetry() {
            chain.with_mut(|state| state.apply_telemetry(&snapshot));
        }
    });

    rsx! {
        document::Stylesheet { href: asset!("/assets/main.css") }
        Router::<Route> {}
    }
}
