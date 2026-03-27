use dioxus::prelude::*;

use crate::components::glass::GlassSurface;
use crate::services::rpc_client::RpcClient;

#[component]
pub fn ConsensusMap() -> Element {
    let client = use_context::<RpcClient>();
    let refresh_tick = use_context::<Signal<u64>>();

    let data = use_resource(move || {
        let client = client.clone();
        let trigger = refresh_tick();
        async move {
            let _ = trigger;
            client.fetch_dashboard().await
        }
    });

    rsx! {
        div { class: "space-y-4",
            div { class: "flex items-center justify-between",
                h2 { class: "text-2xl font-bold text-white", "Consensus Map" }
                button {
                    class: "rounded-lg border border-blue-400/40 bg-blue-500/15 px-3 py-1 text-sm text-blue-100",
                    onclick: move |_| *refresh_tick.write() += 1,
                    "Yenile"
                }
            }
            p { class: "text-slate-300", "Chain kimliği, blok yüksekliği ve health durumuyla konsensüs hazır olma kontrolü." }

            {match data() {
                Some(Ok(snapshot)) => rsx! {
                    GlassSurface { class: Some("p-5".to_string()),
                        ul { class: "space-y-2 text-sm text-slate-200",
                            li { "configured chain_id: {snapshot.health.chain_id}" }
                            li { "evm chainId: {snapshot.chain_id_hex}" }
                            li { "height: {snapshot.block_height}" }
                            li { "status: {snapshot.health.status}" }
                            li { "readiness_score: {snapshot.health.readiness_score}" }
                        }
                    }
                },
                Some(Err(err)) => rsx! {
                    GlassSurface { class: Some("p-5 border-red-500/40".to_string()),
                        p { class: "text-sm text-red-200", "Consensus verisi alınamadı: {err}" }
                    }
                },
                None => rsx! { GlassSurface { class: Some("p-5".to_string()), "Yükleniyor..." } },
            }}
        }
    }
}
