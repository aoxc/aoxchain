use dioxus::prelude::*;

use crate::components::glass::GlassSurface;
use crate::state::GlobalChainState;

#[component]
pub fn Home() -> Element {
    let chain = use_context::<Signal<GlobalChainState>>();
    let total_tps: f32 = chain.read().lanes.iter().map(|lane| lane.tps).sum();

    rsx! {
        div { class: "space-y-6",
            h2 { class: "text-2xl font-bold text-white", "AOXCHUB Overview" }

            div { class: "grid gap-4 md:grid-cols-2 xl:grid-cols-4",
                MetricCard { title: "Current Block", value: format!("#{}", chain.read().height), hint: "L1 finalized" }
                MetricCard { title: "Aggregate TPS", value: format!("{total_tps:.1}"), hint: "Cross-runtime" }
                MetricCard { title: "Network Health", value: format!("{:.2}%", chain.read().network_health), hint: "Consensus signal" }
                MetricCard { title: "Total Staked", value: format!("{} AOXC", chain.read().total_staked), hint: "Secured in vault" }
            }
            p { class: "text-sm text-slate-300", "Veriler doğrudan RPC/health/metrics endpointlerinden okunur." }

            GlassSurface { class: Some("p-5".to_string()), intensity: Some("low"),
                h3 { class: "mb-4 text-lg font-semibold text-white", "Execution Lanes" }
                div { class: "space-y-3",
                    for lane in chain.read().lanes.clone() {
                        LaneRow { lane: lane }
                    }
                },
                None => rsx! {
                    GlassSurface { class: Some("p-5".to_string()), "Yükleniyor..." }
                }
            }
        }
    }
}

#[component]
pub fn Wallet() -> Element {
    rsx! {
        div { class: "space-y-4",
            h2 { class: "text-2xl font-bold text-white", "Wallet & Treasury" }
            p { class: "text-slate-300", "Multi-sig treasury, validator rewards, and operational budget visibility." }
            GlassSurface { class: Some("p-5".to_string()),
                ul { class: "list-disc space-y-2 pl-5 text-slate-200",
                    li { "Treasury balance: 143,920,000 AOXC" }
                    li { "Staking reward payout window: every 6 hours" }
                    li { "Hot wallet exposure: 4.2% of total funds" }
                }
            }
        }
    }
}

#[component]
pub fn Nodes() -> Element {
    let chain = use_context::<Signal<GlobalChainState>>();

    rsx! {
        div { class: "space-y-4",
            h2 { class: "text-2xl font-bold text-white", "Validator Nodes" }
            p { class: "text-slate-300", "Active nodes: {chain.read().active_nodes}" }
            GlassSurface { class: Some("p-5".to_string()),
                table { class: "w-full text-left text-sm",
                    thead { class: "text-slate-400",
                        tr {
                            th { class: "pb-2", "Node" }
                            th { class: "pb-2", "Region" }
                            th { class: "pb-2", "Latency" }
                            th { class: "pb-2", "Status" }
                        }
                    }
                    tbody {
                        for node in chain.read().nodes.clone() {
                            tr { class: "border-t border-white/10",
                                td { class: "py-2", "{node.id}" }
                                td { class: "py-2", "{node.region}" }
                                td { class: "py-2", "{node.latency_ms} ms" }
                                td { class: "py-2", if node.online { "online" } else { "offline" } }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn MetricCard(title: &'static str, value: String, hint: &'static str) -> Element {
    rsx! {
        GlassSurface { class: Some("p-4".to_string()), intensity: Some("low"),
            p { class: "text-xs uppercase tracking-wide text-slate-400", "{title}" }
            p { class: "mt-2 text-xl font-semibold text-white", "{value}" }
            p { class: "mt-1 text-xs text-slate-400", "{hint}" }
        }
    }
}

#[component]
fn LaneRow(lane: crate::types::LaneStatus) -> Element {
    let width = format!("{}%", lane.load_percent);
    let state = if lane.is_active { "active" } else { "idle" };

    rsx! {
        div { class: "rounded-xl border border-white/10 bg-white/5 p-4",
            div { class: "flex items-center justify-between",
                p { class: "font-semibold text-white", "{lane.kind:?}" }
                p { class: "text-sm text-slate-300", "{lane.tps} TPS" }
            }
            p { class: "mt-1 text-xs text-slate-400", "Checkpoint: {lane.last_checkpoint} • {state}" }
            div { class: "mt-3 h-2 rounded-full bg-slate-800",
                div { class: "h-full rounded-full bg-blue-500", style: "width: {width}" }
            }
            p { class: "text-slate-300", "Prometheus metrics üzerinden node davranışı izlenir." }

            {match snapshot() {
                Some(Ok(metrics)) => rsx! {
                    GlassSurface { class: Some("p-5".to_string()),
                        ul { class: "space-y-2 text-sm text-slate-200",
                            li { "requests_total: {metrics.requests_total}" }
                            li { "rejected_total: {metrics.rejected_total}" }
                            li { "rate_limited_total: {metrics.rate_limited_total}" }
                            li { "health_readiness_score: {metrics.readiness_score}" }
                        }
                    }
                },
                Some(Err(err)) => rsx! {
                    GlassSurface { class: Some("p-5 border-red-500/40".to_string()),
                        p { class: "text-red-200 text-sm", "Metrics okunamadı: {err}" }
                    }
                },
                None => rsx! { GlassSurface { class: Some("p-5".to_string()), "Yükleniyor..." } },
            }}
        }
    }
}

#[component]
fn MetricCard(title: &'static str, value: String, hint: String) -> Element {
    rsx! {
        GlassSurface { class: Some("p-4".to_string()), intensity: Some("low"),
            p { class: "text-xs uppercase tracking-wide text-slate-400", "{title}" }
            p { class: "mt-2 text-xl font-semibold text-white break-all", "{value}" }
            p { class: "mt-1 text-xs text-slate-400", "{hint}" }
        }
    }
}

fn _pretty_json(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}
