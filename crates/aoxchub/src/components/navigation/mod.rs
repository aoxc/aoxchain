use dioxus::prelude::*;

use crate::i18n::Language;
use crate::route::Route;
use crate::services::network_profile::resolve_profile;

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
    let profile = resolve_profile();

    rsx! {
        header { class: "flex items-center justify-between border-b border-white/10 bg-[#060a13]/90 px-6 py-4 backdrop-blur-md",
            div {
                p { class: "text-xs uppercase tracking-[0.2em] text-blue-300", "AOXCHUB" }
                h1 { class: "text-lg font-semibold text-white", "Protocol + Operations + Audit" }
            }
            div { class: "flex items-center gap-3 text-xs",
                span { class: "rounded-full bg-emerald-500/20 px-3 py-1 text-emerald-300 border border-emerald-400/30", "Desktop Role: Control Plane" }
                span { class: "rounded-full bg-blue-500/20 px-3 py-1 text-blue-200 border border-blue-400/30", "Profile: {profile.title()}" }
                span { class: "rounded-full bg-violet-500/20 px-3 py-1 text-violet-200 border border-violet-400/30", "{language_label}" }
            }
        }
    }
}

#[component]
pub fn Sidebar() -> Element {
    rsx! {
        aside { class: "w-72 border-r border-white/10 bg-[#04070d]/95 p-6 backdrop-blur-xl overflow-y-auto",
            div { class: "mb-8",
                h2 { class: "text-xl font-black tracking-tight text-white", "AOXCHAIN" }
                p { class: "text-xs uppercase tracking-[0.2em] text-blue-300", "Desktop Control Plane" }
            }

            nav { class: "space-y-2",
                SidebarLink { to: Route::Overview {}, label: "Overview" }
                SidebarLink { to: Route::Consensus {}, label: "Consensus" }
                SidebarLink { to: Route::ValidatorsStaking {}, label: "Validators & Staking" }
                SidebarLink { to: Route::ExecutionLanes {}, label: "Execution Lanes" }
                SidebarLink { to: Route::Explorer {}, label: "Explorer" }
                SidebarLink { to: Route::WalletTreasury {}, label: "Wallet & Treasury" }
                SidebarLink { to: Route::NodesInfrastructure {}, label: "Nodes & Infrastructure" }
                SidebarLink { to: Route::TelemetryAudit {}, label: "Telemetry & Audit" }
                SidebarLink { to: Route::GovernanceControl {}, label: "Governance & Control" }
                SidebarLink { to: Route::SettingsSecurity {}, label: "Settings & Security" }
            }

            div { class: "mt-8 rounded-2xl border border-white/10 bg-white/5 p-4",
                p { class: "text-xs uppercase tracking-wide text-slate-400", "Boundary Rule" }
                p { class: "mt-1 text-sm text-slate-200", "UI never bypasses kernel authority." }
                p { class: "text-xs text-slate-400", "Read model + signed intent + audit log" }
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
