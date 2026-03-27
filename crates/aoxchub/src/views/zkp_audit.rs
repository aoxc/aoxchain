use dioxus::prelude::*;

use crate::components::glass::GlassSurface;

#[component]
pub fn ZkpAudit() -> Element {
    let checks = vec![
        ("Batch #44921", "verified", "14ms"),
        ("Batch #44922", "verified", "17ms"),
        ("Batch #44923", "pending", "--"),
        ("Batch #44924", "verified", "11ms"),
    ];

    rsx! {
        div { class: "space-y-4",
            h2 { class: "text-2xl font-bold text-white", "ZKP Audit Stream" }
            p { class: "text-slate-300", "Proof doğrulama boru hattı ve son durum olayları." }
            GlassSurface { class: Some("p-5".to_string()),
                div { class: "space-y-2",
                    for (batch, status, latency) in checks {
                        div { class: "rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm text-slate-200",
                            "{batch} • {status} • latency: {latency}"
                        }
                    }
                }
            }
        }
    }
}
