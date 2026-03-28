use dioxus::prelude::*;

use crate::components::glass::GlassSurface;
use crate::state::GlobalChainState;

#[component]
pub fn LaneMonitor() -> Element {
    let chain = use_context::<Signal<GlobalChainState>>();
    let lanes = chain.read().lanes.clone();

    rsx! {
        div { class: "space-y-4",
            h2 { class: "text-2xl font-bold text-white", "Lane Monitor" }
            p { class: "text-slate-300", "Runtime bazlı yük dengelemesi ve saturasyon takibi." }

            if lanes.is_empty() {
                GlassSurface { class: Some("p-4".to_string()),
                    p {
                        class: "rounded-xl border border-dashed border-white/10 bg-white/5 px-4 py-6 text-sm text-slate-400",
                        "Gerçek lane telemetrisi bağlı olmadığı için lane verisi gösterilemiyor."
                    }
                }
            } else {
                div { class: "grid gap-4 md:grid-cols-2",
                    for lane in lanes {
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
}
