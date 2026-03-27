use dioxus::prelude::*;

use crate::components::glass::GlassSurface;
use crate::services::rpc_client::RpcClient;

#[component]
pub fn LaneMonitor() -> Element {
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
                h2 { class: "text-2xl font-bold text-white", "Lane Monitor (Realtime Metrics)" }
                button {
                    class: "rounded-lg border border-blue-400/40 bg-blue-500/15 px-3 py-1 text-sm text-blue-100",
                    onclick: move |_| *refresh_tick.write() += 1,
                    "Yenile"
                }
            }
            p { class: "text-slate-300", "Lane throughput metrikleri henüz ayrı endpointte yayınlanmıyorsa, RPC node global metrikleri gösterilir." }

            {match snapshot() {
                Some(Ok(metrics)) => rsx! {
                    div { class: "grid gap-4 md:grid-cols-2",
                        GlassSurface { class: Some("p-4".to_string()),
                            h3 { class: "font-semibold text-white", "Traffic" }
                            p { class: "text-sm text-slate-300", "requests_total: {metrics.requests_total}" }
                            p { class: "text-sm text-slate-300", "rejected_total: {metrics.rejected_total}" }
                        }
                        GlassSurface { class: Some("p-4".to_string()),
                            h3 { class: "font-semibold text-white", "Rate Limit" }
                            p { class: "text-sm text-slate-300", "rate_limited_total: {metrics.rate_limited_total}" }
                            p { class: "text-sm text-slate-300", "readiness_score: {metrics.readiness_score}" }
                        }
                    }
                },
                Some(Err(err)) => rsx! {
                    GlassSurface { class: Some("p-4 border-red-500/40".to_string()),
                        p { class: "text-sm text-red-200", "Lane monitor verisi çekilemedi: {err}" }
                    }
                },
                None => rsx! { GlassSurface { class: Some("p-4".to_string()), "Yükleniyor..." } },
            }}
        }
    }
}
