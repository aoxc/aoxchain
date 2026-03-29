use dioxus::prelude::*;

use crate::i18n::Language;
use crate::route::Route;
use crate::services::network_profile::resolve_profile;

#[derive(Clone, Copy, PartialEq, Eq)]
struct NavItem {
    label: &'static str,
    route: Route,
    badge: &'static str,
}

const NAV_ITEMS: [NavItem; 10] = [
    NavItem {
        label: "Overview",
        route: Route::Overview {},
        badge: "Chain",
    },
    NavItem {
        label: "Consensus",
        route: Route::Consensus {},
        badge: "Core",
    },
    NavItem {
        label: "Validators & Staking",
        route: Route::ValidatorsStaking {},
        badge: "Security",
    },
    NavItem {
        label: "Execution Lanes",
        route: Route::ExecutionLanes {},
        badge: "Runtime",
    },
    NavItem {
        label: "Explorer",
        route: Route::Explorer {},
        badge: "Inspection",
    },
    NavItem {
        label: "Wallet & Treasury",
        route: Route::WalletTreasury {},
        badge: "Custody",
    },
    NavItem {
        label: "Nodes & Infrastructure",
        route: Route::NodesInfrastructure {},
        badge: "Ops",
    },
    NavItem {
        label: "Telemetry & Audit",
        route: Route::TelemetryAudit {},
        badge: "Evidence",
    },
    NavItem {
        label: "Governance & Control",
        route: Route::GovernanceControl {},
        badge: "Policy",
    },
    NavItem {
        label: "Settings & Security",
        route: Route::SettingsSecurity {},
        badge: "Boundary",
    },
];

#[component]
pub fn Header() -> Element {
    let language = match std::env::var("AOXCHUB_LANG").ok().as_deref() {
        Some("tr") | Some("TR") => Language::TR,
        _ => Language::EN,
    };
    let language_label = match language {
        Language::TR => "TR",
        Language::EN => "EN",
    };
    let profile = resolve_profile();

    rsx! {
        header { class: "aox-header",
            div {
                p { class: "aox-kicker", "AOXC Integrated Control Surface" }
                h1 { class: "aox-title", "Production Chain Interface" }
            }
            div { class: "aox-chip-row",
                span { class: "aox-chip", "Profile: {profile.title()}" }
                span { class: "aox-chip", "Language: {language_label}" }
                span { class: "aox-chip aox-chip--good", "Boundary: Fail-Closed" }
            }
        }
    }
}

#[component]
pub fn Sidebar() -> Element {
    rsx! {
        aside { class: "aox-sidebar",
            div { class: "aox-brand",
                p { class: "aox-kicker", "AOXCHAIN" }
                h2 { "Unified User / Dev / Validator Plane" }
            }

            nav { class: "aox-nav",
                for item in NAV_ITEMS {
                    Link {
                        to: item.route,
                        class: "aox-nav-link",
                        span { "{item.label}" }
                        strong { "{item.badge}" }
                    }
                }
            }

            div { class: "aox-security-box",
                p { class: "aox-kicker", "Security Posture" }
                p { "Wallet approvals, governance intents, and node operations are constrained behind explicit policy boundaries." }
            }
        }
    }
}

#[component]
pub fn RightOperationsPanel() -> Element {
    rsx! {
        aside { class: "aox-right-panel",
            section { class: "aox-right-card",
                p { class: "aox-kicker", "Wallet Approval Queue" }
                h3 { "Pending signature review" }
                ul {
                    li { "Transfer Intent • Dry-run verified" }
                    li { "Treasury Policy Update • Waiting multisig" }
                    li { "Validator Rotation • Governance checkpoint" }
                }
            }

            section { class: "aox-right-card",
                p { class: "aox-kicker", "Node Operations" }
                h3 { "Live validator controls" }
                ul {
                    li { "Health probes: healthy" }
                    li { "Snapshot service: synchronized" }
                    li { "Upgrade channel: locked to signed manifests" }
                }
            }
        }
    }
}
