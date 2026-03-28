use dioxus::prelude::*;

use crate::i18n::Language;
use crate::route::Route;

#[component]
pub fn Header() -> Element {
    let language = match std::env::var("AOXCHUB_LANG").ok().as_deref() {
        Some("tr") | Some("TR") => Language::TR,
        _ => Language::EN,
    };
    let language_label = match language {
        Language::TR => "Language: TR",
        Language::EN => "Language: EN",
    };

    rsx! {
        header { class: "flex items-center justify-between border-b border-white/10 bg-[#060a13]/90 px-6 py-4 backdrop-blur-md",
            div {
                p { class: "text-xs uppercase tracking-[0.2em] text-blue-300", "AOXCHUB" }
                h1 { class: "text-lg font-semibold text-white", "Mission Control" }
            }
            div { class: "flex items-center gap-3 text-sm",
                span { class: "rounded-full bg-emerald-500/20 px-3 py-1 text-emerald-300 border border-emerald-400/30", "Network: Healthy" }
                span { class: "rounded-full bg-blue-500/20 px-3 py-1 text-blue-200 border border-blue-400/30", "Mode: Mainnet Preview" }
                span { class: "rounded-full bg-violet-500/20 px-3 py-1 text-violet-200 border border-violet-400/30", "{language_label}" }
            }
        }
    }
}

#[component]
pub fn Sidebar() -> Element {
    rsx! {
        aside { class: "w-72 border-r border-white/10 bg-[#04070d]/95 p-6 backdrop-blur-xl",
            div { class: "mb-8",
                h2 { class: "text-xl font-black tracking-tight text-white", "AOXCHAIN" }
                p { class: "text-xs uppercase tracking-[0.2em] text-blue-300", "Operations Hub" }
            }

            nav { class: "space-y-2",
                SidebarLink { to: Route::Home {}, label: "Overview" }
                SidebarLink { to: Route::LaneMonitor {}, label: "Lane Monitor" }
                SidebarLink { to: Route::ConsensusMap {}, label: "Consensus Map" }
                SidebarLink { to: Route::ZkpAudit {}, label: "ZKP Audit" }
                SidebarLink { to: Route::Explorer {}, label: "Explorer" }
                SidebarLink { to: Route::Wallet {}, label: "Wallet" }
                SidebarLink { to: Route::Staking {}, label: "Staking" }
                SidebarLink { to: Route::Nodes {}, label: "Nodes" }
            }

            div { class: "mt-8 rounded-2xl border border-white/10 bg-white/5 p-4",
                p { class: "text-xs uppercase tracking-wide text-slate-400", "Session" }
                p { class: "mt-1 text-sm text-slate-200", "Operator: root@aoxhub" }
                p { class: "text-xs text-slate-400", "Access: Level-3" }
            }
        }
    }
}

#[component]
fn SidebarLink(to: Route, label: &'static str) -> Element {
    rsx! {
        Link {
            to,
            class: "block rounded-xl border border-transparent px-3 py-2 text-sm text-slate-200 transition hover:border-blue-400/40 hover:bg-blue-500/10 hover:text-white",
            "{label}"
        }
    }
}
