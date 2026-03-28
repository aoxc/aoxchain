use dioxus::prelude::*;

use crate::components::glass::GlassSurface;
use crate::services::telemetry::latest_snapshot;

#[component]
pub fn ZkpAudit() -> Element {
    let telemetry = use_resource(move || async move { latest_snapshot().await });
    let snapshot = telemetry();

    rsx! {
        div { class: "space-y-4",
            h2 { class: "text-2xl font-bold text-white", "ZKP Audit Stream" }
            p { class: "text-slate-300", "Proof doğrulama boru hattı ve son durum olayları." }
            GlassSurface { class: Some("p-5".to_string()),
                div { class: "space-y-2",
                    if let Some(snapshot) = snapshot {
                        div { class: "rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm text-slate-200",
                            "RPC source: {snapshot.source}"
                        }
                        div { class: "rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm text-slate-200",
                            if let Some(block) = snapshot.latest_block {
                                "Latest finalized block: #{block}"
                            } else {
                                "Latest finalized block: unavailable"
                            }
                        }
                        div { class: "rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm text-slate-200",
                            if snapshot.healthy {
                                "Verification transport status: healthy"
                            } else {
                                "Verification transport status: degraded"
                            }
                        }
                    } else {
                        div { class: "rounded-lg border border-dashed border-white/10 bg-white/5 px-3 py-2 text-sm text-slate-400",
                            "Gerçek ZKP audit kaynağı yükleniyor..."
                        }
                    }
                }
            }
        }
    }
}
