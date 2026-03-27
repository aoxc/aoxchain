use dioxus::prelude::*;

use crate::components::glass::GlassSurface;
use crate::state::GlobalChainState;

#[component]
pub fn ConsensusMap() -> Element {
    let chain = use_context::<Signal<GlobalChainState>>();

    rsx! {
        div { class: "space-y-4",
            h2 { class: "text-2xl font-bold text-white", "Consensus Map" }
            p { class: "text-slate-300", "Region bazlı doğrulayıcı dağılımı ve gecikme gözlemi." }
            GlassSurface { class: Some("p-5".to_string()),
                ul { class: "space-y-2 text-slate-200",
                    for node in chain.read().nodes.clone() {
                        li { class: "rounded-lg border border-white/10 bg-white/5 px-3 py-2",
                            "{node.id} • {node.region} • {node.latency_ms}ms • {if node.online { "online" } else { "offline" }}"
                        }
                    }
                }
            }
        }
    }
}
