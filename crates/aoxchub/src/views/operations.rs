use dioxus::prelude::*;

use crate::components::glass::GlassSurface;
use crate::services::consensus_service::read_consensus;
use crate::services::execution_service::read_execution_lanes;
use crate::services::explorer_service::read_explorer;
use crate::services::governance_service::read_governance;
use crate::services::infrastructure_service::read_infrastructure;
use crate::services::intent_service::{governance_intents, latest_audit_events};
use crate::services::overview_service::read_overview;
use crate::services::settings_service::read_settings;
use crate::services::staking_service::read_staking;
use crate::services::telemetry_audit_service::read_telemetry_audit;
use crate::services::treasury_service::read_treasury;

#[component]
pub fn Overview() -> Element {
    let data = use_resource(move || async move { read_overview().await });

    rsx! {
        PageShell { title: "Overview", subtitle: "Ana operasyon ekranı: zincirin genel sağlığını tek bakışta gösterir." }
        {
            match data() {
                Some(model) => rsx! {
                    SourceLabel { source: model.source }
                    GridSection {
                        cards: vec![
                            ("Chain ID", model.chain_id, "authoritative".to_string()),
                            ("Network Profile", model.network_profile.title().to_string(), model.network_profile.source_label().to_string()),
                            ("Latest Finalized Block", model.latest_finalized_block, "rpc".to_string()),
                            ("Head Block", model.head_block, "rpc".to_string()),
                            ("Sync Status", model.sync_status, "telemetry".to_string()),
                            ("Peer Count", model.peer_count, "telemetry".to_string()),
                            ("Validator Count", model.validator_count, "consensus registry".to_string()),
                            ("Network Health", model.network_health, "telemetry".to_string()),
                            ("Alerts Summary", model.alerts_summary, "alert stream".to_string()),
                        ]
                    }
                },
                None => rsx! { LoadingBox {} },
            }
        }
    }
}

#[component]
pub fn Consensus() -> Element {
    let data = use_resource(move || async move { read_consensus().await });

    rsx! {
        PageShell { title: "Consensus", subtitle: "Gerçek konsensus görünürlüğü: epoch, round, proposer ve sertifika akışı." }
        {
            match data() {
                Some(model) => rsx! {
                    SourceLabel { source: model.source }
                    GridSection {
                        cards: vec![
                            ("Current Epoch", model.current_epoch, "consensus API".to_string()),
                            ("Current Height", model.current_height, "rpc".to_string()),
                            ("Current Round", model.current_round, "consensus API".to_string()),
                            ("Proposer", model.proposer, "consensus API".to_string()),
                            ("Quorum Threshold", model.quorum_threshold, "policy".to_string()),
                            ("Finalized Head", model.finalized_head, "rpc".to_string()),
                            ("Lock State", model.lock_state, "consensus API".to_string()),
                            ("Timeout Events", model.timeout_events, "consensus API".to_string()),
                            ("Equivocation Evidence", model.equivocation_evidence, "evidence store".to_string()),
                            ("Continuity Certificate", model.continuity_certificate, "certificate API".to_string()),
                            ("Legitimacy Certificate", model.legitimacy_certificate, "certificate API".to_string()),
                            ("Execution Certificate", model.execution_certificate, "certificate API".to_string()),
                        ]
                    }
                },
                None => rsx! { LoadingBox {} },
            }
        }
    }
}

#[component]
pub fn ValidatorsStaking() -> Element {
    let data = use_resource(move || async move { read_staking().await });

    rsx! {
        PageShell { title: "Validators & Staking", subtitle: "Ekonomik güvenlik yüzeyi: validator, stake, delegasyon, slash geçmişi." }
        {
            match data() {
                Some(model) => rsx! {
                    SourceLabel { source: model.source }
                    GridSection {
                        cards: vec![
                            ("Validator List", model.validator_list, "staking API".to_string()),
                            ("Voting Power", model.voting_power, "staking API".to_string()),
                            ("Active / Inactive / Jailed", model.active_state, "staking API".to_string()),
                            ("Stake Distribution", model.stake_distribution, "staking API".to_string()),
                            ("Delegation", model.delegation, "staking API".to_string()),
                            ("Rewards", model.rewards, "staking API".to_string()),
                            ("Slash History", model.slash_history, "staking API".to_string()),
                            ("Join / Exit / Rotation", model.join_exit_rotation, "validator lifecycle API".to_string()),
                        ]
                    }
                },
                None => rsx! { LoadingBox {} },
            }
        }
    }
}

#[component]
pub fn ExecutionLanes() -> Element {
    let data = use_resource(move || async move { read_execution_lanes().await });

    rsx! {
        PageShell { title: "Execution Lanes", subtitle: "Multi-VM merkezi ekran: EVM, WASM, Move ve Cardano-style lane görünürlüğü." }
        {
            match data() {
                Some(model) => rsx! {
                    SourceLabel { source: model.source }
                    div { class: "grid gap-4 md:grid-cols-2",
                        for lane in model.lanes {
                            GlassSurface { class: Some("p-4".to_string()),
                                h3 { class: "text-base font-semibold text-white", "{lane.lane}" }
                                div { class: "mt-3 space-y-2 text-sm",
                                    LineItem { label: "Lane TPS", value: lane.tps, source: "lane telemetry".to_string() }
                                    LineItem { label: "Gas Usage", value: lane.gas_usage, source: "lane telemetry".to_string() }
                                    LineItem { label: "Failure Rate", value: lane.failure_rate, source: "lane telemetry".to_string() }
                                    LineItem { label: "Checkpoint Continuity", value: lane.checkpoint_continuity, source: "lane telemetry".to_string() }
                                    LineItem { label: "Lane Commitment Root", value: lane.commitment_root, source: "lane telemetry".to_string() }
                                    LineItem { label: "Compatibility Level", value: lane.compatibility_level, source: "profile-aware".to_string() }
                                }
                            }
                        }
                    }
                },
                None => rsx! { LoadingBox {} },
            }
        }
    }
}

#[component]
pub fn Explorer() -> Element {
    let data = use_resource(move || async move { read_explorer().await });

    rsx! {
        PageShell { title: "Explorer", subtitle: "Zincir içi denetlenebilirlik: block/tx/receipt/event/state diff/finality referansı." }
        {
            match data() {
                Some(model) => rsx! {
                    SourceLabel { source: model.source }
                    GridSection {
                        cards: vec![
                            ("Block Explorer", model.block_explorer, "rpc/indexer".to_string()),
                            ("Tx Explorer", model.tx_explorer, "rpc/indexer".to_string()),
                            ("Receipt Viewer", model.receipt_viewer, "rpc/indexer".to_string()),
                            ("Event Viewer", model.event_viewer, "indexer".to_string()),
                            ("State Diff Summary", model.state_diff_summary, "state API".to_string()),
                            ("Contract/Account Query", model.contract_account_query, "rpc".to_string()),
                            ("Finality Proof Reference", model.finality_proof_reference, "proof API".to_string()),
                        ]
                    }
                },
                None => rsx! { LoadingBox {} },
            }
        }
    }
}

#[component]
pub fn WalletTreasury() -> Element {
    let data = use_resource(move || async move { read_treasury().await });

    rsx! {
        PageShell { title: "Wallet & Treasury", subtitle: "Kurumsal varlık yönetimi: multisig, custody, policy check ve dry-run transfer." }
        {
            match data() {
                Some(model) => rsx! {
                    SourceLabel { source: model.source }
                    GridSection {
                        cards: vec![
                            ("Treasury Balances", model.treasury_balances, "treasury API".to_string()),
                            ("Hot / Cold Separation", model.hot_cold_separation, "custody policy".to_string()),
                            ("Multisig Status", model.multisig_status, "signer service".to_string()),
                            ("Pending Transfers", model.pending_transfers, "treasury API".to_string()),
                            ("Dry-run Transfer", model.dry_run_transfer, "intent service".to_string()),
                            ("Treasury Policy Checks", model.policy_checks, "policy engine".to_string()),
                            ("Custody Posture", model.custody_posture, "security policy".to_string()),
                        ]
                    }
                },
                None => rsx! { LoadingBox {} },
            }
        }
    }
}

#[component]
pub fn NodesInfrastructure() -> Element {
    let data = use_resource(move || async move { read_infrastructure().await });

    rsx! {
        PageShell { title: "Nodes & Infrastructure", subtitle: "Node operasyon omurgası: envanter, rol, region, latency, kaynak ve failover." }
        {
            match data() {
                Some(model) => rsx! {
                    SourceLabel { source: model.source }
                    GridSection {
                        cards: vec![
                            ("Node Inventory", model.node_inventory, "node control plane".to_string()),
                            ("Role", model.roles, "node control plane".to_string()),
                            ("Region", model.region, "infra registry".to_string()),
                            ("Latency", model.latency, "telemetry".to_string()),
                            ("Uptime", model.uptime, "telemetry".to_string()),
                            ("Version", model.version, "node management".to_string()),
                            ("Disk / Memory / CPU", model.resource_usage, "host metrics".to_string()),
                            ("Snapshot Status", model.snapshot_status, "snapshot API".to_string()),
                            ("Sync Mode", model.sync_mode, "node control plane".to_string()),
                            ("Failover Readiness", model.failover_readiness, "readiness checks".to_string()),
                        ]
                    }
                },
                None => rsx! { LoadingBox {} },
            }
        }
    }
}

#[component]
pub fn TelemetryAudit() -> Element {
    let data = use_resource(move || async move { read_telemetry_audit().await });

    rsx! {
        PageShell { title: "Telemetry & Audit", subtitle: "Operator-grade gözlemlenebilirlik: metrikler, loglar, health, anomaly ve incident timeline." }
        {
            match data() {
                Some(model) => rsx! {
                    SourceLabel { source: model.source }
                    GridSection {
                        cards: vec![
                            ("Metrics", model.metrics, "telemetry".to_string()),
                            ("Logs", model.logs, "log sink".to_string()),
                            ("Health Checks", model.health_checks, "health probes".to_string()),
                            ("Evidence Store", model.evidence_store, "evidence API".to_string()),
                            ("Operator Action History", model.operator_action_history, "audit trail".to_string()),
                            ("Alert Stream", model.alert_stream, "alert API".to_string()),
                            ("Anomaly Flags", model.anomaly_flags, "anomaly engine".to_string()),
                            ("Incident Timeline", model.incident_timeline, "incident API".to_string()),
                        ]
                    }
                    AuditTrailPanel {}
                },
                None => rsx! { LoadingBox {} },
            }
        }
    }
}

#[component]
pub fn GovernanceControl() -> Element {
    let data = use_resource(move || async move { read_governance().await });

    rsx! {
        PageShell { title: "Governance & Control", subtitle: "Kernel boundary korunarak intent + approval + audited execution yaklaşımı." }
        {
            match data() {
                Some(model) => rsx! {
                    SourceLabel { source: model.source }
                    GridSection {
                        cards: vec![
                            ("Protocol Parameter Proposals", model.protocol_parameter_proposals, "governance API".to_string()),
                            ("Validator Management Intents", model.validator_management_intents, "intent service".to_string()),
                            ("Emergency Controls", model.emergency_controls, "governance API".to_string()),
                            ("Upgrade Intents", model.upgrade_intents, "intent queue".to_string()),
                            ("Signed Action Queue", model.signed_action_queue, "intent queue".to_string()),
                            ("Approval Workflow", model.approval_workflow, "governance workflow".to_string()),
                            ("Execution Status", model.execution_status, "execution auditor".to_string()),
                        ]
                    }
                    IntentPanel {}
                },
                None => rsx! { LoadingBox {} },
            }
        }
    }
}

#[component]
pub fn SettingsSecurity() -> Element {
    let data = use_resource(move || async move { read_settings().await });

    rsx! {
        PageShell { title: "Settings & Security", subtitle: "Profile-aware güvenli entegrasyon menüsü: endpoint, kimlik, signer ve rol yönetimi." }
        {
            match data() {
                Some(model) => rsx! {
                    SourceLabel { source: model.source }
                    GridSection {
                        cards: vec![
                            ("RPC Endpoint Profile", model.rpc_endpoint_profile, "profile manager".to_string()),
                            ("Mainnet / Devnet / Testnet", model.environment_selector, "profile manager".to_string()),
                            ("API Auth", model.api_auth, "auth service".to_string()),
                            ("Operator Identity", model.operator_identity, "identity service".to_string()),
                            ("Hardware Key / Signer", model.signer_integration, "signer integration".to_string()),
                            ("Session Policy", model.session_policy, "security policy".to_string()),
                            ("Access Roles", model.access_roles, "rbac".to_string()),
                            ("Desktop Security Posture", model.desktop_security_posture, "security control plane".to_string()),
                        ]
                    }
                },
                None => rsx! { LoadingBox {} },
            }
        }
    }
}

#[component]
pub fn NotFoundPage(segments: Vec<String>) -> Element {
    let path = if segments.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", segments.join("/"))
    };

    rsx! {
        PageShell { title: "Page Not Found", subtitle: "Geçersiz route." }
        GlassSurface { class: Some("p-5".to_string()),
            p { class: "text-sm text-slate-300", "Requested route: {path}" }
        }
    }
}

#[component]
fn PageShell(title: &'static str, subtitle: &'static str) -> Element {
    rsx! {
        div { class: "space-y-4",
            div {
                h2 { class: "text-2xl font-bold text-white", "{title}" }
                p { class: "mt-1 text-sm text-slate-300", "{subtitle}" }
            }
        }
    }
}

#[component]
fn SourceLabel(source: String) -> Element {
    rsx! {
        GlassSurface { class: Some("p-3".to_string()), intensity: Some("low"),
            p { class: "text-xs uppercase tracking-wide text-cyan-300", "Data Source: {source}" }
        }
    }
}

#[component]
fn GridSection(cards: Vec<(&'static str, String, String)>) -> Element {
    rsx! {
        div { class: "grid gap-3 md:grid-cols-2 xl:grid-cols-3",
            for (label, value, source) in cards {
                GlassSurface { class: Some("p-4".to_string()), intensity: Some("low"),
                    p { class: "text-xs uppercase tracking-wide text-slate-400", "{label}" }
                    p { class: "mt-2 text-sm font-medium text-white break-all", "{value}" }
                    p { class: "mt-1 text-xs text-slate-500", "source: {source}" }
                }
            }
        }
    }
}

#[component]
fn LineItem(label: &'static str, value: String, source: String) -> Element {
    rsx! {
        div { class: "rounded-lg border border-white/10 bg-white/5 px-3 py-2",
            p { class: "text-xs uppercase tracking-wide text-slate-400", "{label}" }
            p { class: "text-sm text-white", "{value}" }
            p { class: "text-[11px] text-slate-500", "source: {source}" }
        }
    }
}

#[component]
fn IntentPanel() -> Element {
    let intents = governance_intents();

    rsx! {
        GlassSurface { class: Some("p-4".to_string()),
            h3 { class: "text-sm font-semibold text-white", "Signed Intent Queue" }
            div { class: "mt-3 space-y-2",
                for intent in intents {
                    div { class: "rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm",
                        p { class: "font-medium text-white", "{intent.id}" }
                        p { class: "text-slate-300", "{intent.action}" }
                        p { class: "text-xs text-slate-400", "dry-run: {intent.dry_run_supported} | approval: {intent.approval_required} | source: {intent.source}" }
                    }
                }
            }
        }
    }
}

#[component]
fn AuditTrailPanel() -> Element {
    let events = latest_audit_events();

    rsx! {
        GlassSurface { class: Some("p-4".to_string()),
            h3 { class: "text-sm font-semibold text-white", "Operator Action History" }
            div { class: "mt-3 space-y-2",
                for event in events {
                    div { class: "rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm",
                        p { class: "text-white", "{event.actor}" }
                        p { class: "text-slate-300", "{event.action} -> {event.target}" }
                        p { class: "text-xs text-slate-400", "outcome: {event.outcome} | source: {event.source}" }
                    }
                }
            }
        }
    }
}

#[component]
fn LoadingBox() -> Element {
    rsx! {
        GlassSurface { class: Some("p-4".to_string()),
            p { class: "text-sm text-slate-400", "Authoritative data source loading..." }
        }
    }
}
