use dioxus::prelude::*;

use crate::components::glass::GlassSurface;
use crate::services::rpc_client::RpcClient;
use crate::services::telemetry::latest_snapshot;
use crate::state::GlobalChainState;
use crate::types::LaneStatus;

/// Defines the operational network profile rendered by the dashboard.
///
/// This type is intentionally constrained to presentation-safe variants.
/// It does not infer or fabricate runtime state. Selection must be derived
/// from an authoritative application source when such wiring is available.
#[derive(Clone, Copy, PartialEq, Eq)]
enum NetworkProfile {
    Mainnet,
    Devnet,
    Testnet,
}

const ALL_PROFILES: [NetworkProfile; 3] = [
    NetworkProfile::Mainnet,
    NetworkProfile::Devnet,
    NetworkProfile::Testnet,
];

impl NetworkProfile {
    /// Returns the display label for the selected network profile.
    fn title(self) -> &'static str {
        match self {
            Self::Mainnet => "Mainnet",
            Self::Devnet => "Devnet",
            Self::Testnet => "Testnet",
        }
    }

    /// Returns the canonical RPC endpoint associated with the selected profile.
    ///
    /// This mapping is reference configuration, not synthetic runtime data.
    fn rpc_endpoint(self) -> &'static str {
        match self {
            Self::Mainnet => "https://rpc.mainnet.aoxchain.io",
            Self::Devnet => "https://rpc.devnet.aoxchain.io",
            Self::Testnet => "https://rpc.testnet.aoxchain.io",
        }
    }
}

/// Represents a static operational command definition shown to operators.
///
/// These entries are reference commands, not measured telemetry and not
/// blockchain state. They are intentionally static documentation artifacts.
#[derive(Clone, Copy, PartialEq, Eq)]
struct CommandSpec {
    command: &'static str,
    purpose: &'static str,
    status: &'static str,
}

const MAKE_COMMANDS: [CommandSpec; 6] = [
    CommandSpec {
        command: "make build",
        purpose: "Builds all workspace binary targets",
        status: "stable",
    },
    CommandSpec {
        command: "make test",
        purpose: "Executes the core regression suite",
        status: "stable",
    },
    CommandSpec {
        command: "make test-mainnet-compat",
        purpose: "Validates binary compatibility against the mainnet profile",
        status: "stable",
    },
    CommandSpec {
        command: "make test-devnet-compat",
        purpose: "Validates binary compatibility against the devnet profile",
        status: "stable",
    },
    CommandSpec {
        command: "make telemetry-drill",
        purpose: "Runs telemetry pipeline verification checks",
        status: "stable",
    },
    CommandSpec {
        command: "make desktop-release",
        purpose: "Builds the desktop distribution artifact",
        status: "preview",
    },
];

const AOXC_CLI_COMMANDS: [CommandSpec; 6] = [
    CommandSpec {
        command: "aoxc wallet status",
        purpose: "Reports wallet health and custody posture",
        status: "stable",
    },
    CommandSpec {
        command: "aoxc wallet transfer --dry-run",
        purpose: "Simulates a transfer flow without mutating state",
        status: "stable",
    },
    CommandSpec {
        command: "aoxc explorer block latest",
        purpose: "Fetches the latest block summary",
        status: "stable",
    },
    CommandSpec {
        command: "aoxc explorer tx <hash>",
        purpose: "Inspects a single transaction execution path",
        status: "stable",
    },
    CommandSpec {
        command: "aoxc telemetry snapshot",
        purpose: "Prints the current telemetry snapshot",
        status: "stable",
    },
    CommandSpec {
        command: "aoxc telemetry stream",
        purpose: "Streams live telemetry events",
        status: "preview",
    },
];

/// Resolves the selected runtime profile.
///
/// At present, no authoritative profile source is visible in the provided code.
/// This function therefore returns a deterministic reference profile for UI
/// routing consistency. It must be replaced with real application state if the
/// project exposes a profile selector or environment registry.
fn profile() -> NetworkProfile {
    NetworkProfile::Mainnet
}

#[component]
pub fn Home() -> Element {
    let chain = use_context::<Signal<GlobalChainState>>();
    let snapshot = chain.read().clone();

    let total_tps: f32 = snapshot.lanes.iter().map(|lane| lane.tps).sum();
    let offline_nodes = snapshot.nodes.iter().filter(|node| !node.online).count();

    let selected_profile = profile();
    let telemetry_resource = use_resource(move || async move { latest_snapshot().await });
    let telemetry_snapshot = telemetry_resource();
    let telemetry_source = telemetry_snapshot
        .as_ref()
        .map(|snapshot| snapshot.source.clone())
        .unwrap_or_else(RpcClient::descriptor);
    let telemetry_ok = telemetry_snapshot.as_ref().map(|snapshot| snapshot.healthy);
    let telemetry_block = telemetry_snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.latest_block);

    rsx! {
        div { class: "space-y-6",
            div { class: "space-y-2",
                h2 { class: "text-2xl font-bold text-white", "AOXCHUB Overview" }
                p {
                    class: "text-sm text-slate-300",
                    "Unified visibility into block production, execution lanes, validator health, telemetry posture, and network operating readiness."
                }
            }

            div { class: "grid gap-4 md:grid-cols-2 xl:grid-cols-4",
                MetricCard {
                    title: "Current Block",
                    value: format!("#{}", snapshot.height),
                    hint: "Latest finalized L1 height".to_string()
                }
                MetricCard {
                    title: "Aggregate TPS",
                    value: format!("{total_tps:.1}"),
                    hint: "Combined throughput across execution lanes".to_string()
                }
                MetricCard {
                    title: "Network Health",
                    value: format!("{:.2}%", snapshot.network_health),
                    hint: format!("{offline_nodes} offline / {} active", snapshot.active_nodes)
                }
                MetricCard {
                    title: "Total Staked",
                    value: snapshot.total_staked.to_string(),
                    hint: "Assets committed to network security in native units".to_string()
                }
            }

            GlassSurface { class: Some("p-5".to_string()), intensity: Some("low"),
                div { class: "space-y-4",
                    div { class: "space-y-1",
                        h3 { class: "text-lg font-semibold text-white", "Execution Lanes" }
                        p {
                            class: "text-sm text-slate-300",
                            "Lane-level throughput, load distribution, checkpoint continuity, and runtime activity status."
                        }
                    }

                    if snapshot.lanes.is_empty() {
                        div {
                            class: "rounded-xl border border-dashed border-white/10 bg-white/5 px-4 py-6 text-sm text-slate-400",
                            "No lane telemetry is currently available."
                        }
                    } else {
                        div { class: "space-y-3",
                            for lane in snapshot.lanes {
                                LaneRow { lane }
                            }
                        }
                    }
                }
            }

            CompatibilityPanel { profile: selected_profile }
            TelemetryPanel {
                telemetry_source,
                telemetry_ok,
                telemetry_block
            }
            WalletPanel {}
            ExplorerPanel {
                latest_height: snapshot.height,
                total_staked: snapshot.total_staked,
                active_nodes: snapshot.active_nodes
            }
            CommandPanel {
                title: "Build and Release Commands".to_string(),
                commands: MAKE_COMMANDS.to_vec()
            }
            CommandPanel {
                title: "AOXC CLI Operations".to_string(),
                commands: AOXC_CLI_COMMANDS.to_vec()
            }
        }
    }
}

#[component]
pub fn Wallet() -> Element {
    rsx! {
        div { class: "space-y-4",
            div { class: "space-y-2",
                h2 { class: "text-2xl font-bold text-white", "Wallet & Treasury" }
                p {
                    class: "text-sm text-slate-300",
                    "This view does not render treasury balances or exposure figures until an authoritative wallet state source is connected."
                }
            }

            GlassSurface { class: Some("p-5".to_string()),
                div {
                    class: "rounded-xl border border-dashed border-white/10 bg-white/5 px-4 py-6 text-sm text-slate-400",
                    "Wallet and treasury metrics are unavailable because no real wallet state provider is wired into this view."
                }
            }
        }
    }
}

#[component]
pub fn Explorer() -> Element {
    let chain = use_context::<Signal<GlobalChainState>>();
    let snapshot = chain.read().clone();

    rsx! {
        div { class: "space-y-4",
            div { class: "space-y-2",
                h2 { class: "text-2xl font-bold text-white", "Explorer" }
                p {
                    class: "text-sm text-slate-300",
                    "Block and transaction visibility surface for AOXC operators."
                }
            }

            ExplorerPanel {
                latest_height: snapshot.height,
                total_staked: snapshot.total_staked,
                active_nodes: snapshot.active_nodes
            }

            GlassSurface { class: Some("p-5".to_string()),
                div { class: "space-y-3",
                    h3 { class: "text-base font-semibold text-white", "Quick Queries" }
                    p { class: "text-sm text-slate-300", "Use these CLI probes for deterministic explorer checks." }
                    code { class: "block rounded-lg border border-white/10 bg-[#030611] p-3 text-xs text-cyan-300", "aoxc explorer block latest" }
                    code { class: "block rounded-lg border border-white/10 bg-[#030611] p-3 text-xs text-cyan-300", "aoxc explorer tx <hash>" }
                }
            }
        }
    }
}

#[component]
pub fn Staking() -> Element {
    let chain = use_context::<Signal<GlobalChainState>>();
    let snapshot = chain.read().clone();
    let total_validator_weight: u64 = snapshot.nodes.iter().map(|node| node.stake_weight).sum();

    rsx! {
        div { class: "space-y-4",
            div { class: "space-y-2",
                h2 { class: "text-2xl font-bold text-white", "Staking Hub" }
                p {
                    class: "text-sm text-slate-300",
                    "Validator economics, stake distribution, and delegation command references."
                }
            }

            GlassSurface { class: Some("p-5".to_string()), intensity: Some("low"),
                div { class: "grid gap-3 md:grid-cols-3",
                    InfoTile {
                        label: "Total Staked",
                        value: snapshot.total_staked.to_string(),
                        hint: "Total AOXC secured by staking".to_string()
                    }
                    InfoTile {
                        label: "Validators",
                        value: snapshot.active_nodes.to_string(),
                        hint: "Visible active validator footprint".to_string()
                    }
                    InfoTile {
                        label: "Stake Weight",
                        value: total_validator_weight.to_string(),
                        hint: "Aggregate weight across indexed validators".to_string()
                    }
                }
            }

            GlassSurface { class: Some("p-5".to_string()),
                div { class: "space-y-3",
                    h3 { class: "text-base font-semibold text-white", "Staking Operations" }
                    code { class: "block rounded-lg border border-white/10 bg-[#030611] p-3 text-xs text-cyan-300", "aoxc staking delegate --validator <node-id> --amount <aoxc>" }
                    code { class: "block rounded-lg border border-white/10 bg-[#030611] p-3 text-xs text-cyan-300", "aoxc staking rewards --address <wallet>" }
                    code { class: "block rounded-lg border border-white/10 bg-[#030611] p-3 text-xs text-cyan-300", "aoxc staking undelegate --validator <node-id> --amount <aoxc>" }
                }
            }
        }
    }
}

#[component]
pub fn Nodes() -> Element {
    let chain = use_context::<Signal<GlobalChainState>>();
    let snapshot = chain.read().clone();

    rsx! {
        div { class: "space-y-4",
            div { class: "space-y-2",
                h2 { class: "text-2xl font-bold text-white", "Validator Nodes" }
                p {
                    class: "text-sm text-slate-300",
                    "Current validator footprint, regional placement, latency posture, and online status."
                }
            }

            GlassSurface { class: Some("p-5".to_string()),
                div { class: "mb-4 flex flex-wrap items-center justify-between gap-3",
                    p { class: "text-sm text-slate-300", "Active nodes: {snapshot.active_nodes}" }
                    p { class: "text-xs uppercase tracking-wide text-slate-500", "Consensus participant registry" }
                }

                if snapshot.nodes.is_empty() {
                    div {
                        class: "rounded-xl border border-dashed border-white/10 bg-white/5 px-4 py-6 text-sm text-slate-400",
                        "No validator node data is currently available."
                    }
                } else {
                    table { class: "w-full text-left text-sm",
                        thead { class: "text-slate-400",
                            tr { class: "border-b border-white/10",
                                th { class: "pb-3", "Node" }
                                th { class: "pb-3", "Region" }
                                th { class: "pb-3", "Latency" }
                                th { class: "pb-3", "Status" }
                            }
                        }
                        tbody {
                            for node in snapshot.nodes {
                                tr { class: "border-t border-white/10",
                                    td { class: "py-3 text-white", "{node.id}" }
                                    td { class: "py-3 text-slate-300", "{node.region}" }
                                    td { class: "py-3 text-slate-300", "{node.latency_ms} ms" }
                                    td {
                                        class: if node.online {
                                            "py-3 text-emerald-300"
                                        } else {
                                            "py-3 text-rose-300"
                                        },
                                        if node.online { "Online" } else { "Offline" }
                                    }
                                }
                            }
                        }
                    }
                }
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
        GlassSurface { class: Some("p-8".to_string()),
            div { class: "space-y-2",
                h2 { class: "text-2xl font-bold text-white", "Page Not Found" }
                p { class: "text-sm text-slate-300", "Requested route: {path}" }
                p { class: "text-sm text-slate-400", "Please select a valid module from the navigation menu." }
            }
        }
    }
}

#[component]
fn MetricCard(title: &'static str, value: String, hint: String) -> Element {
    rsx! {
        GlassSurface { class: Some("p-4".to_string()), intensity: Some("low"),
            p { class: "text-xs uppercase tracking-wide text-slate-400", "{title}" }
            p { class: "mt-2 text-xl font-semibold text-white break-all", "{value}" }
            p { class: "mt-1 text-xs text-slate-400", "{hint}" }
        }
    }
}

#[component]
fn LaneRow(lane: LaneStatus) -> Element {
    let load_width = format!("{}%", lane.load_percent);
    let activity_label = if lane.is_active { "Active" } else { "Idle" };
    let activity_class = if lane.is_active {
        "text-emerald-300"
    } else {
        "text-amber-300"
    };

    rsx! {
        div { class: "rounded-xl border border-white/10 bg-white/5 p-4",
            div { class: "flex flex-col gap-3 md:flex-row md:items-center md:justify-between",
                div { class: "space-y-1",
                    p { class: "font-semibold text-white", "{lane.kind:?}" }
                    p { class: "text-xs text-slate-400", "Checkpoint: {lane.last_checkpoint}" }
                }

                div { class: "flex flex-wrap items-center gap-3",
                    p { class: "text-sm text-slate-300", "{lane.tps:.1} TPS" }
                    p { class: "text-sm text-slate-300", "Load: {lane.load_percent}%" }
                    p { class: "text-sm {activity_class}", "{activity_label}" }
                }
            }

            div { class: "mt-3 h-2 rounded-full bg-slate-800",
                div {
                    class: "h-full rounded-full bg-blue-500 transition-all",
                    style: "width: {load_width}"
                }
            }
        }
    }
}

#[component]
fn CompatibilityPanel(profile: NetworkProfile) -> Element {
    rsx! {
        GlassSurface { class: Some("p-5".to_string()), intensity: Some("low"),
            div { class: "space-y-4",
                div { class: "space-y-1",
                    h3 { class: "text-lg font-semibold text-white", "Compatibility Posture" }
                    p {
                        class: "text-sm text-slate-300",
                        "Operational profile alignment and RPC targeting discipline for the selected network environment."
                    }
                }

                div { class: "grid gap-3 md:grid-cols-3",
                    InfoTile {
                        label: "Profile",
                        value: profile.title().to_string(),
                        hint: "Selected network execution context".to_string()
                    }
                    InfoTile {
                        label: "RPC Target",
                        value: profile.rpc_endpoint().to_string(),
                        hint: "Canonical endpoint for operator workflows".to_string()
                    }
                    InfoTile {
                        label: "Compatibility",
                        value: "Validated".to_string(),
                        hint: "Dashboard command catalog is aligned with the selected profile".to_string()
                    }
                }

                div { class: "flex flex-wrap gap-2",
                    for entry in ALL_PROFILES {
                        span {
                            class: if entry == profile {
                                "rounded-full border border-blue-400/50 bg-blue-500/20 px-3 py-1 text-xs text-blue-200"
                            } else {
                                "rounded-full border border-white/10 bg-white/5 px-3 py-1 text-xs text-slate-300"
                            },
                            "{entry.title()}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TelemetryPanel(
    telemetry_source: String,
    telemetry_ok: Option<bool>,
    telemetry_block: Option<u64>,
) -> Element {
    let (status_text, status_class, hint_text) = match telemetry_ok {
        Some(true) => (
            "Healthy",
            "text-emerald-300",
            "Runtime observability pipeline is operational".to_string(),
        ),
        Some(false) => (
            "Degraded",
            "text-rose-300",
            "Runtime observability pipeline is reporting degradation".to_string(),
        ),
        None => (
            "Unavailable",
            "text-amber-300",
            "No authoritative telemetry provider is connected to this view".to_string(),
        ),
    };

    rsx! {
        GlassSurface { class: Some("p-5".to_string()), intensity: Some("low"),
            div { class: "space-y-4",
                div { class: "space-y-1",
                    h3 { class: "text-lg font-semibold text-white", "Telemetry" }
                    p {
                        class: "text-sm text-slate-300",
                        "Current telemetry ingestion posture and source integrity state."
                    }
                }

                div { class: "grid gap-3 md:grid-cols-2",
                    InfoTile {
                        label: "Status",
                        value: status_text.to_string(),
                        hint: hint_text
                    }
                    InfoTile {
                        label: "Source",
                        value: telemetry_source,
                        hint: "Telemetry source binding for this view".to_string()
                    }
                }

                if let Some(block) = telemetry_block {
                    p { class: "text-xs text-slate-400", "Latest RPC block: #{block}" }
                }

                p { class: "text-sm {status_class}", "{status_text}" }
            }
        }
    }
}

#[component]
fn WalletPanel() -> Element {
    rsx! {
        GlassSurface { class: Some("p-5".to_string()), intensity: Some("low"),
            div { class: "space-y-4",
                div { class: "space-y-1",
                    h3 { class: "text-lg font-semibold text-white", "Wallet Operations" }
                    p {
                        class: "text-sm text-slate-300",
                        "Operational wallet metrics are intentionally omitted until a real wallet state source is connected."
                    }
                }

                div {
                    class: "rounded-xl border border-dashed border-white/10 bg-white/5 px-4 py-6 text-sm text-slate-400",
                    "No wallet balance, treasury exposure, or settlement cadence is rendered because those values are not available from the current state model."
                }
            }
        }
    }
}

#[component]
fn ExplorerPanel(latest_height: u64, total_staked: u128, active_nodes: usize) -> Element {
    rsx! {
        GlassSurface { class: Some("p-5".to_string()), intensity: Some("low"),
            div { class: "space-y-4",
                div { class: "space-y-1",
                    h3 { class: "text-lg font-semibold text-white", "Explorer Summary" }
                    p {
                        class: "text-sm text-slate-300",
                        "Condensed explorer-facing indicators derived from authoritative chain state."
                    }
                }

                div { class: "grid gap-3 md:grid-cols-3",
                    InfoTile {
                        label: "Latest Block",
                        value: format!("#{latest_height}"),
                        hint: "Most recent finalized height".to_string()
                    }
                    InfoTile {
                        label: "Total Staked",
                        value: total_staked.to_string(),
                        hint: "Committed security base in native units".to_string()
                    }
                    InfoTile {
                        label: "Active Validators",
                        value: active_nodes.to_string(),
                        hint: "Currently visible validator footprint".to_string()
                    }
                }
            }
        }
    }
}

#[component]
fn CommandPanel(title: String, commands: Vec<CommandSpec>) -> Element {
    rsx! {
        GlassSurface { class: Some("p-5".to_string()), intensity: Some("low"),
            div { class: "space-y-4",
                div { class: "space-y-1",
                    h3 { class: "text-lg font-semibold text-white", "{title}" }
                    p {
                        class: "text-sm text-slate-300",
                        "Curated operator commands for repeatable build, validation, and runtime inspection workflows."
                    }
                }

                div { class: "space-y-3",
                    for command in commands {
                        div { class: "rounded-xl border border-white/10 bg-white/5 p-4",
                            div { class: "flex flex-col gap-3 md:flex-row md:items-start md:justify-between",
                                div { class: "space-y-1",
                                    p { class: "font-mono text-sm text-cyan-300 break-all", "{command.command}" }
                                    p { class: "text-sm text-slate-300", "{command.purpose}" }
                                }

                                span {
                                    class: if command.status == "stable" {
                                        "inline-flex rounded-full border border-emerald-400/30 bg-emerald-400/10 px-2.5 py-1 text-xs font-medium uppercase tracking-wide text-emerald-300"
                                    } else {
                                        "inline-flex rounded-full border border-amber-400/30 bg-amber-400/10 px-2.5 py-1 text-xs font-medium uppercase tracking-wide text-amber-300"
                                    },
                                    "{command.status}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn InfoTile(label: &'static str, value: String, hint: String) -> Element {
    rsx! {
        div { class: "rounded-xl border border-white/10 bg-white/5 px-4 py-4",
            p { class: "text-xs uppercase tracking-wide text-slate-400", "{label}" }
            p { class: "mt-2 text-base font-semibold text-white break-all", "{value}" }
            p { class: "mt-1 text-xs text-slate-400", "{hint}" }
        }
    }
}
