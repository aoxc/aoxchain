use dioxus::prelude::*;

use crate::i18n::Language;
use crate::route::Route;
use crate::services::network_profile::resolve_profile;
use crate::services::rpc_client::RpcClient;
use crate::services::telemetry::latest_snapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NavKey {
    Overview,
    Consensus,
    ValidatorsStaking,
    ExecutionLanes,
    Explorer,
    WalletTreasury,
    NodesInfrastructure,
    TelemetryAudit,
    GovernanceControl,
    SettingsSecurity,
}

impl NavKey {
    #[inline]
    fn route(self) -> Route {
        match self {
            Self::Overview => Route::Overview {},
            Self::Consensus => Route::Consensus {},
            Self::ValidatorsStaking => Route::ValidatorsStaking {},
            Self::ExecutionLanes => Route::ExecutionLanes {},
            Self::Explorer => Route::Explorer {},
            Self::WalletTreasury => Route::WalletTreasury {},
            Self::NodesInfrastructure => Route::NodesInfrastructure {},
            Self::TelemetryAudit => Route::TelemetryAudit {},
            Self::GovernanceControl => Route::GovernanceControl {},
            Self::SettingsSecurity => Route::SettingsSecurity {},
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NavItem {
    label: &'static str,
    key: NavKey,
    badge: &'static str,
}

const NAV_ITEMS: [NavItem; 10] = [
    NavItem {
        label: "Overview",
        key: NavKey::Overview,
        badge: "Chain",
    },
    NavItem {
        label: "Consensus",
        key: NavKey::Consensus,
        badge: "Core",
    },
    NavItem {
        label: "Validators & Staking",
        key: NavKey::ValidatorsStaking,
        badge: "Security",
    },
    NavItem {
        label: "Execution Lanes",
        key: NavKey::ExecutionLanes,
        badge: "Runtime",
    },
    NavItem {
        label: "Explorer",
        key: NavKey::Explorer,
        badge: "Inspection",
    },
    NavItem {
        label: "Wallet & Treasury",
        key: NavKey::WalletTreasury,
        badge: "Custody",
    },
    NavItem {
        label: "Nodes & Infrastructure",
        key: NavKey::NodesInfrastructure,
        badge: "Ops",
    },
    NavItem {
        label: "Telemetry & Audit",
        key: NavKey::TelemetryAudit,
        badge: "Evidence",
    },
    NavItem {
        label: "Governance & Control",
        key: NavKey::GovernanceControl,
        badge: "Policy",
    },
    NavItem {
        label: "Settings & Security",
        key: NavKey::SettingsSecurity,
        badge: "Boundary",
    },
];

#[inline]
fn resolve_language() -> Language {
    match std::env::var("AOXCHUB_LANG").ok().as_deref() {
        Some("tr") | Some("TR") => Language::TR,
        _ => Language::EN,
    }
}

#[inline]
fn language_label(language: Language) -> &'static str {
    match language {
        Language::TR => "TR",
        Language::EN => "EN",
    }
}

#[component]
pub fn Header() -> Element {
    let language = resolve_language();
    let profile = resolve_profile();

    rsx! {
        header { class: "aox-header",
            div { class: "aox-header-copy",
                p { class: "aox-kicker", "AOXCHAIN ORBITAL CONSOLE" }
                h1 { class: "aox-title", "AOXCHUB Advanced Command Deck" }
                p { class: "aox-header-subtitle", "Real-time operations dashboard with RPC-backed telemetry, governance controls, and release automation surfaces." }
            }

            div { class: "aox-chip-row",
                span { class: "aox-chip", "Profile: {profile.title()}" }
                span { class: "aox-chip", "Language: {language_label(language)}" }
                span { class: "aox-chip aox-chip--good", "Transport: RPC Live" }
            }
        }
    }
}

#[component]
pub fn Sidebar() -> Element {
    rsx! {
        aside { class: "aox-sidebar",
            div { class: "aox-brand",
                p { class: "aox-kicker", "AOXCHAIN CONTROL PLANE" }
                h2 { "AOXCHUB Admin" }
                p { class: "aox-brand-subtitle", "Glassmorphism workspace for production orchestration across consensus, execution, explorer, and security domains." }
            }

            nav { class: "aox-nav",
                for item in NAV_ITEMS.into_iter() {
                    Link {
                        to: item.key.route(),
                        class: "aox-nav-link",
                        span { "{item.label}" }
                        strong { "{item.badge}" }
                    }
                }
            }

            div { class: "aox-security-box",
                p { class: "aox-kicker", "Security Baseline" }
                p {
                    "Signer-gated mutation model is preserved. Every critical action remains policy verified and auditable before execution."
                }
            }
        }
    }
}

#[component]
pub fn RightOperationsPanel() -> Element {
    let telemetry = use_resource(move || async move { latest_snapshot().await });

    let endpoint = RpcClient::endpoint();
    let (status, block, peers, chain_id) = match telemetry() {
        Some(snapshot) => (
            if snapshot.healthy {
                "Healthy"
            } else {
                "Degraded"
            }
            .to_string(),
            snapshot
                .latest_block
                .map(|value| format!("#{value}"))
                .unwrap_or_else(|| "N/A".to_string()),
            snapshot
                .peer_count
                .map(|value| value.to_string())
                .unwrap_or_else(|| "N/A".to_string()),
            snapshot.chain_id.unwrap_or_else(|| "N/A".to_string()),
        ),
        None => (
            "Loading".to_string(),
            "...".to_string(),
            "...".to_string(),
            "...".to_string(),
        ),
    };

    rsx! {
        aside { class: "aox-right-panel",
            section { class: "aox-right-card",
                p { class: "aox-kicker", "Live RPC Signal" }
                h3 { "Network transport" }
                ul {
                    li { "Endpoint: {endpoint}" }
                    li { "Health: {status}" }
                    li { "Head: {block}" }
                    li { "Peers: {peers}" }
                    li { "Chain ID: {chain_id}" }
                }
            }

            section { class: "aox-right-card",
                p { class: "aox-kicker", "Operator Toolchain" }
                h3 { "CLI + Make Surface" }
                ul {
                    li { "aoxc telemetry snapshot" }
                    li { "aoxc explorer block latest" }
                    li { "make test-mainnet-compat" }
                    li { "make telemetry-drill" }
                }
            }
        }
    }
}
