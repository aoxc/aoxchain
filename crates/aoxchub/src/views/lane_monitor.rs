use dioxus::prelude::*;

use crate::components::glass::GlassSurface;
use crate::state::GlobalChainState;

#[component]
pub fn LaneMonitor() -> Element {
    let chain = use_context::<Signal<GlobalChainState>>();

    rsx! {
        div { class: "space-y-4",
            h2 { class: "text-2xl font-bold text-white", "Lane Monitor" }
            p { class: "text-slate-300", "Runtime bazlı yük dengelemesi ve saturasyon takibi." }

            div { class: "grid gap-4 md:grid-cols-2",
                for lane in chain.read().lanes.clone() {
                    GlassSurface { class: Some("p-4".to_string()),
                        h3 { class: "text-lg font-semibold text-white", "{lane.kind:?}" }
                        p { class: "text-slate-300", "Current TPS: {lane.tps}" }
                        p { class: "text-slate-300", "Load: {lane.load_percent}%" }
                        p { class: "text-slate-400 text-sm", "Last checkpoint: {lane.last_checkpoint}" }
                    }
                }
            }
        }
    }
}
