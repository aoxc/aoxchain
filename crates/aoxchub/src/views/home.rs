use dioxus::prelude::*;
use serde_json::Value;

use crate::components::glass::GlassSurface;
use crate::services::rpc_client::{ChainSnapshot, RpcClient};

#[component]
pub fn Home() -> Element {
    let client = use_context::<RpcClient>();
    let refresh_tick = use_context::<Signal<u64>>();

    let dashboard = use_resource(move || {
        let client = client.clone();
        let trigger = refresh_tick();
        async move {
            let _ = trigger;
            client.fetch_dashboard().await
        }
    });

    rsx! {
        div { class: "space-y-6",
            div { class: "flex items-center justify-between",
                h2 { class: "text-2xl font-bold text-white", "AOXCHUB Live Overview" }
                button {
                    class: "rounded-lg border border-blue-400/40 bg-blue-500/15 px-3 py-1 text-sm text-blue-100 hover:bg-blue-500/25",
                    onclick: move |_| *refresh_tick.write() += 1,
                    "Yenile"
                }
            }
            p { class: "text-sm text-slate-300", "Veriler doğrudan RPC/health/metrics endpointlerinden okunur." }

            {match dashboard() {
                Some(Ok(snapshot)) => rsx! { DashboardCards { snapshot: snapshot.clone() } },
                Some(Err(err)) => rsx! {
                    GlassSurface { class: Some("p-5 border-red-500/40".to_string()),
                        h3 { class: "text-lg font-semibold text-red-300", "Canlı veri alınamadı" }
                        p { class: "mt-2 text-sm text-red-100", "{err}" }
                        p { class: "mt-2 text-xs text-slate-400", "AOXHUB_API_BASE / AOXHUB_RPC_BASE ortam değişkenlerini node adresinize göre ayarlayın." }
                    }
                },
                None => rsx! {
                    GlassSurface { class: Some("p-5".to_string()), "Yükleniyor..." }
                }
            }}
        }
    }
}

#[component]
fn DashboardCards(snapshot: ChainSnapshot) -> Element {
    rsx! {
        div { class: "grid gap-4 md:grid-cols-2 xl:grid-cols-4",
            MetricCard { title: "Chain ID (RPC)", value: snapshot.chain_id_hex, hint: snapshot.health.chain_id }
            MetricCard { title: "Current Block", value: format!("#{}", snapshot.block_height), hint: format!("uptime {}s", snapshot.health.uptime_secs) }
            MetricCard { title: "Readiness", value: format!("{}", snapshot.health.readiness_score), hint: snapshot.health.status }
            MetricCard { title: "Requests Total", value: format!("{:.0}", snapshot.metrics.requests_total), hint: "aox_rpc_requests_total".to_string() }
        }

        GlassSurface { class: Some("p-5 md:col-span-2 xl:col-span-4".to_string()),
            h3 { class: "mb-3 text-lg font-semibold text-white", "RPC Sağlık Uyarıları" }
            if snapshot.health.warnings.is_empty() {
                p { class: "text-sm text-emerald-300", "Uyarı yok" }
            } else {
                ul { class: "list-disc space-y-1 pl-5 text-sm text-amber-200",
                    for warning in snapshot.health.warnings {
                        li { "{warning}" }
                    }
                }
            }
            if !snapshot.health.errors.is_empty() {
                h4 { class: "mt-4 text-sm font-semibold text-red-300", "Hatalar" }
                ul { class: "list-disc space-y-1 pl-5 text-sm text-red-200",
                    for err in snapshot.health.errors {
                        li { "{err}" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn Wallet() -> Element {
    let client = use_context::<RpcClient>();

    let mut tx_hash = use_signal(String::new);
    let mut receipt_result = use_signal(|| None::<String>);

    let mut to_addr = use_signal(String::new);
    let mut call_data = use_signal(String::new);
    let mut call_result = use_signal(|| None::<String>);
    let mut gas_result = use_signal(|| None::<String>);

    rsx! {
        div { class: "space-y-6",
            h2 { class: "text-2xl font-bold text-white", "Wallet & Chain Operations" }
            p { class: "text-slate-300", "EVM uyumlu gerçek RPC çağrıları: receipt, eth_call, eth_estimateGas." }

            GlassSurface { class: Some("p-5 space-y-3".to_string()),
                h3 { class: "text-lg font-semibold text-white", "Tx Receipt Sorgula" }
                input {
                    class: "w-full rounded-lg border border-white/15 bg-black/30 p-2 text-sm text-white",
                    placeholder: "0x... tx hash",
                    value: tx_hash(),
                    oninput: move |e| tx_hash.set(e.value()),
                }
                button {
                    class: "rounded-lg border border-blue-400/40 bg-blue-500/15 px-3 py-1 text-sm text-blue-100",
                    onclick: {
                        let client = client.clone();
                        let hash = tx_hash();
                        move |_| {
                            spawn(async move {
                                let out = client
                                    .eth_get_transaction_receipt(&hash)
                                    .await
                                    .map(|v| serde_json::to_string_pretty(&v).unwrap_or_else(|_| v.to_string()))
                                    .unwrap_or_else(|e| e);
                                receipt_result.set(Some(out));
                            });
                        }
                    },
                    "Sorgula"
                }
                if let Some(result) = receipt_result() {
                    pre { class: "overflow-x-auto rounded-lg bg-black/30 p-3 text-xs text-slate-200", "{result}" }
                }
            }

            GlassSurface { class: Some("p-5 space-y-3".to_string()),
                h3 { class: "text-lg font-semibold text-white", "eth_call / eth_estimateGas" }
                input {
                    class: "w-full rounded-lg border border-white/15 bg-black/30 p-2 text-sm text-white",
                    placeholder: "to address (0x...)",
                    value: to_addr(),
                    oninput: move |e| to_addr.set(e.value()),
                }
                textarea {
                    class: "w-full rounded-lg border border-white/15 bg-black/30 p-2 text-sm text-white",
                    rows: 5,
                    placeholder: "0x function selector + encoded params",
                    value: call_data(),
                    oninput: move |e| call_data.set(e.value()),
                }
                div { class: "flex gap-2",
                    button {
                        class: "rounded-lg border border-blue-400/40 bg-blue-500/15 px-3 py-1 text-sm text-blue-100",
                        onclick: {
                            let client = client.clone();
                            let to = to_addr();
                            let data = call_data();
                            move |_| {
                                spawn(async move {
                                    let out = client.eth_call(&to, &data).await.unwrap_or_else(|e| e);
                                    call_result.set(Some(out));
                                });
                            }
                        },
                        "eth_call"
                    }
                    button {
                        class: "rounded-lg border border-purple-400/40 bg-purple-500/15 px-3 py-1 text-sm text-purple-100",
                        onclick: {
                            let client = client.clone();
                            let to = to_addr();
                            let data = call_data();
                            move |_| {
                                spawn(async move {
                                    let out = client.eth_estimate_gas(&to, &data).await.unwrap_or_else(|e| e);
                                    gas_result.set(Some(out));
                                });
                            }
                        },
                        "eth_estimateGas"
                    }
                }
                if let Some(result) = call_result() {
                    p { class: "text-xs text-slate-200", "eth_call: {result}" }
                }
                if let Some(result) = gas_result() {
                    p { class: "text-xs text-slate-200", "estimateGas: {result}" }
                }
            }
        }
    }
}

#[component]
pub fn Nodes() -> Element {
    let client = use_context::<RpcClient>();
    let refresh_tick = use_context::<Signal<u64>>();

    let snapshot = use_resource(move || {
        let client = client.clone();
        let trigger = refresh_tick();
        async move {
            let _ = trigger;
            client.fetch_metrics().await
        }
    });

    rsx! {
        div { class: "space-y-4",
            div { class: "flex items-center justify-between",
                h2 { class: "text-2xl font-bold text-white", "Node / RPC Runtime" }
                button {
                    class: "rounded-lg border border-blue-400/40 bg-blue-500/15 px-3 py-1 text-sm text-blue-100",
                    onclick: move |_| *refresh_tick.write() += 1,
                    "Yenile"
                }
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
