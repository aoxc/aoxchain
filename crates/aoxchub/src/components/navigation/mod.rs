use dioxus::prelude::*;

use crate::i18n::Language;
use crate::route::Route;
use crate::services::network_profile::resolve_profile;

/// Represents a stable, compile-safe identifier for a sidebar destination.
///
/// This enum intentionally stores only lightweight discriminants rather than
/// full `Route` values. The design avoids unnecessary trait constraints on
/// configuration records and keeps the navigation model resilient if `Route`
/// evolves to include non-`Copy` or otherwise richer state in the future.
///
/// Security and maintainability rationale:
/// - Prevents invalid `Copy` assumptions for route-bearing structures.
/// - Keeps static navigation metadata deterministic and side-effect free.
/// - Provides a single conversion boundary from UI intent to router state.
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
    /// Resolves the stable navigation key into the concrete router destination.
    ///
    /// This conversion is intentionally explicit to preserve auditability and
    /// reduce the probability of silent routing drift during future refactors.
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

/// Immutable navigation descriptor used by the control surface sidebar.
///
/// The structure is deliberately limited to `'static` metadata and a stable
/// route key. This allows the record to remain `Copy`, trivially analyzable,
/// and suitable for static initialization without imposing trait requirements
/// on higher-order routing types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NavItem {
    /// Human-readable navigation label rendered in the sidebar.
    label: &'static str,

    /// Stable internal destination identifier.
    key: NavKey,

    /// Short operational classification badge shown beside the label.
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

/// Resolves the effective interface language from process-level configuration.
///
/// The function applies a fail-safe default to English when the environment
/// variable is absent, malformed, or unsupported. This behavior is intentional
/// for operational predictability in production deployments.
#[inline]
fn resolve_language() -> Language {
    match std::env::var("AOXCHUB_LANG").ok().as_deref() {
        Some("tr") | Some("TR") => Language::TR,
        _ => Language::EN,
    }
}

/// Returns the compact language marker displayed in the header.
///
/// The function is separated from the component body to keep the render path
/// declarative and to centralize presentation mapping for future expansion.
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
                p { class: "aox-kicker", "AOXCHAIN OPERATIONS LEDGER" }
                h1 { class: "aox-title", "Defter Kalitesinde Operasyon Merkezi" }
                p { class: "aox-header-subtitle", "Zincir yönetimi, validator süreçleri ve güvenlik kontrolleri için rafine ve denetlenebilir masaüstü deneyimi." }
            }

            div { class: "aox-chip-row",
                span { class: "aox-chip", "Profile: {profile.title()}" }
                span { class: "aox-chip", "Language: {language_label(language)}" }
                span { class: "aox-chip aox-chip--good", "Audit Mode: Active" }
            }
        }
    }
}

#[component]
pub fn Sidebar() -> Element {
    rsx! {
        aside { class: "aox-sidebar",
            div { class: "aox-brand",
                p { class: "aox-kicker", "AOXCHAIN CONTROL LEDGER" }
                h2 { "AOXCHUB Notebook" }
                p { class: "aox-brand-subtitle", "Sadece arayüz değil; ritmi, katmanı ve operasyon ruhu olan üretim seviyesi kontrol defteri." }
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
                    "Wallet, governance ve node işlemleri; imza, politika ve denetim sınırları altında yürütülür."
                }
            }
        }
    }
}

#[component]
pub fn RightOperationsPanel() -> Element {
    rsx! {
        aside { class: "aox-right-panel",
            section { class: "aox-right-card",
                p { class: "aox-kicker", "Operator Queue" }
                h3 { "Critical approvals" }
                ul {
                    li { "Treasury transfer intent • dry-run complete" }
                    li { "Governance policy uplift • multisig waiting" }
                    li { "Validator rotation set • checkpoint ready" }
                }
            }

            section { class: "aox-right-card",
                p { class: "aox-kicker", "Runtime Status" }
                h3 { "Live network posture" }
                ul {
                    li { "Health probes: nominal" }
                    li { "Snapshot sync: in policy window" }
                    li { "Upgrade channel: signed manifest only" }
                }
            }
        }
    }
}
