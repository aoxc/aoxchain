use dioxus::prelude::*;

use crate::components::glass::GlassSurface;
use crate::state::GlobalChainState;

#[derive(Clone, PartialEq)]
struct ConsensusNodeViewModel {
    id: String,
    region: String,
    latency_label: String,
    status_label: &'static str,
    status_class: &'static str,
    row_class: &'static str,
}

#[component]
pub fn ConsensusMap() -> Element {
    // The component consumes the globally provided chain state.
    // This access pattern assumes the root application has already injected
    // `Signal<GlobalChainState>` through the Dioxus context system.
    let chain = use_context::<Signal<GlobalChainState>>();

    // Materialize a stable snapshot for the current render pass.
    // This prevents repeated reads from the signal inside the RSX tree and
    // improves clarity of the rendering pipeline.
    let snapshot = chain.read().clone();

    // Transform raw consensus node data into a UI-specific projection.
    // This keeps presentation logic localized and avoids embedding repeated
    // conditional formatting rules directly inside the markup.
    let nodes: Vec<ConsensusNodeViewModel> = snapshot
        .nodes
        .into_iter()
        .map(|node| {
            let (status_label, status_class, row_class) = if node.online {
                (
                    "Online",
                    "border-emerald-400/30 bg-emerald-500/10 text-emerald-200",
                    "rounded-xl border border-white/10 bg-white/5 px-4 py-3 transition",
                )
            } else {
                (
                    "Offline",
                    "border-rose-400/30 bg-rose-500/10 text-rose-200",
                    "rounded-xl border border-rose-500/20 bg-rose-500/5 px-4 py-3 transition",
                )
            };

            ConsensusNodeViewModel {
                id: node.id,
                region: node.region,
                latency_label: format!("{} ms", node.latency_ms),
                status_label,
                status_class,
                row_class,
            }
        })
        .collect();

    let total_nodes = nodes.len();
    let online_nodes = nodes
        .iter()
        .filter(|node| node.status_label == "Online")
        .count();
    let offline_nodes = total_nodes.saturating_sub(online_nodes);

    rsx! {
        div { class: "space-y-5",
            div { class: "space-y-2",
                h2 { class: "text-2xl font-bold text-white", "Consensus Map" }
                p {
                    class: "text-sm text-slate-300",
                    "Operational visibility for validator distribution, regional placement, and network latency posture."
                }
            }

            GlassSurface { class: Some("p-5".to_string()),
                div { class: "mb-5 grid gap-3 md:grid-cols-3",
                    SummaryCard {
                        title: "Total Validators",
                        value: total_nodes.to_string(),
                        hint: "Nodes currently registered in the consensus snapshot"
                    }
                    SummaryCard {
                        title: "Online Validators",
                        value: online_nodes.to_string(),
                        hint: "Validators responding normally at render time"
                    }
                    SummaryCard {
                        title: "Offline Validators",
                        value: offline_nodes.to_string(),
                        hint: "Validators requiring operational review or recovery"
                    }
                }

                if nodes.is_empty() {
                    div { class: "rounded-xl border border-dashed border-white/10 bg-white/5 px-4 py-6 text-sm text-slate-400",
                        "No consensus node data is currently available."
                    }
                } else {
                    div { class: "space-y-3",
                        for node in nodes {
                            div { class: "{node.row_class}",
                                div { class: "flex flex-col gap-3 md:flex-row md:items-center md:justify-between",
                                    div { class: "space-y-1",
                                        p { class: "text-sm font-semibold text-white", "{node.id}" }
                                        p { class: "text-xs uppercase tracking-wide text-slate-400", "{node.region}" }
                                    }

                                    div { class: "flex flex-wrap items-center gap-2",
                                        span {
                                            class: "rounded-full border border-blue-400/20 bg-blue-500/10 px-3 py-1 text-xs font-medium text-blue-200",
                                            "{node.latency_label}"
                                        }
                                        span {
                                            class: "rounded-full px-3 py-1 text-xs font-medium border {node.status_class}",
                                            "{node.status_label}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SummaryCard(title: &'static str, value: String, hint: &'static str) -> Element {
    rsx! {
        div { class: "rounded-xl border border-white/10 bg-white/5 px-4 py-4",
            p { class: "text-xs uppercase tracking-wide text-slate-400", "{title}" }
            p { class: "mt-2 text-xl font-semibold text-white", "{value}" }
            p { class: "mt-1 text-xs text-slate-500", "{hint}" }
        }
    }
}
