use dioxus::prelude::*;

use crate::components::glass::GlassSurface;
use crate::services::telemetry::latest_snapshot;
use crate::state::GlobalChainState;

#[derive(Clone, Copy, PartialEq)]
enum NetworkProfile {
    Mainnet,
    Devnet,
    Testnet,
}

impl NetworkProfile {
    fn title(self) -> &'static str {
        match self {
            Self::Mainnet => "Mainnet",
            Self::Devnet => "Devnet",
            Self::Testnet => "Testnet",
        }
    }

    fn rpc_endpoint(self) -> &'static str {
        match self {
            Self::Mainnet => "https://rpc.mainnet.aoxchain.io",
            Self::Devnet => "https://rpc.devnet.aoxchain.io",
            Self::Testnet => "https://rpc.testnet.aoxchain.io",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct CommandSpec {
    command: &'static str,
    purpose: &'static str,
    status: &'static str,
}

const MAKE_COMMANDS: [CommandSpec; 6] = [
    CommandSpec {
        command: "make build",
        purpose: "Tüm binary hedeflerini derler",
        status: "stable",
    },
    CommandSpec {
        command: "make test",
        purpose: "Çekirdek test süitini çalıştırır",
        status: "stable",
    },
    CommandSpec {
        command: "make test-mainnet-compat",
        purpose: "Mainnet binary uyumu kontrolü",
        status: "stable",
    },
    CommandSpec {
        command: "make test-devnet-compat",
        purpose: "Devnet binary uyumu kontrolü",
        status: "stable",
    },
    CommandSpec {
        command: "make telemetry-drill",
        purpose: "Telemetry pipeline doğrulaması",
        status: "stable",
    },
    CommandSpec {
        command: "make desktop-release",
        purpose: "Desktop paketi üretimi",
        status: "preview",
    },
];

const AOXC_CLI_COMMANDS: [CommandSpec; 6] = [
    CommandSpec {
        command: "aoxc wallet status",
        purpose: "Wallet health ve bakiye kontrolü",
        status: "stable",
    },
    CommandSpec {
        command: "aoxc wallet transfer --dry-run",
        purpose: "Transfer senaryosunu güvenli simüle eder",
        status: "stable",
    },
    CommandSpec {
        command: "aoxc explorer block latest",
        purpose: "En son blok özetini getirir",
        status: "stable",
    },
    CommandSpec {
        command: "aoxc explorer tx <hash>",
        purpose: "Tekil işlem inceleme",
        status: "stable",
    },
    CommandSpec {
        command: "aoxc telemetry snapshot",
        purpose: "Anlık telemetry metriklerini döker",
        status: "stable",
    },
    CommandSpec {
        command: "aoxc telemetry stream",
        purpose: "Canlı telemetry olay akışı",
        status: "preview",
    },
];

#[component]
pub fn Home() -> Element {
    let chain = use_context::<Signal<GlobalChainState>>();
    let mut profile = use_signal(|| NetworkProfile::Mainnet);
    let total_tps: f32 = chain.read().lanes.iter().map(|lane| lane.tps).sum();
    let telemetry = latest_snapshot();

    rsx! {
        div { class: "space-y-6",
            h2 { class: "text-2xl font-bold text-white", "AOXCHUB Desktop Control Plane" }
            p { class: "text-sm text-slate-300", "Mainnet/Devnet/Testnet arasında hızlı geçiş, wallet, explorer ve telemetry tek panelde." }

            div { class: "flex flex-wrap gap-2",
                for item in [NetworkProfile::Mainnet, NetworkProfile::Devnet, NetworkProfile::Testnet] {
                    button {
                        class: if profile() == item {
                            "rounded-xl border border-blue-400 bg-blue-500/20 px-3 py-2 text-sm text-white transition"
                        } else {
                            "rounded-xl border border-white/15 bg-white/5 px-3 py-2 text-sm text-slate-300 transition hover:border-blue-400/40"
                        },
                        onclick: move |_| profile.set(item),
                        "{item.title()}"
                    }
                }
            }

            div { class: "grid gap-4 md:grid-cols-2 xl:grid-cols-4",
                MetricCard { title: "Current Block", value: format!("#{}", chain.read().height), hint: "L1 finalized".to_string() }
                MetricCard { title: "Aggregate TPS", value: format!("{total_tps:.1}"), hint: "Cross-runtime".to_string() }
                MetricCard { title: "Network Health", value: format!("{:.2}%", chain.read().network_health), hint: "Consensus signal".to_string() }
                MetricCard { title: "RPC Endpoint", value: profile().rpc_endpoint().to_string(), hint: format!("{} profile", profile().title()) }
            }

            CompatibilityPanel {}
            TelemetryPanel { telemetry_source: telemetry.source, telemetry_ok: telemetry.healthy }
            WalletPanel {}
            ExplorerPanel { chain: chain.read().clone() }
            CommandPanel { title: "Make Komutları", commands: MAKE_COMMANDS.to_vec() }
            CommandPanel { title: "AOXC CLI Komutları", commands: AOXC_CLI_COMMANDS.to_vec() }
        }
    }
}

#[component]
pub fn Wallet() -> Element {
    rsx! {
        div { class: "space-y-4",
            h2 { class: "text-2xl font-bold text-white", "Wallet & Treasury" }
            WalletPanel {}
        }
    }
}

#[component]
pub fn Nodes() -> Element {
    let chain = use_context::<Signal<GlobalChainState>>();

    rsx! {
        div { class: "space-y-4",
            h2 { class: "text-2xl font-bold text-white", "Validator Nodes" }
            p { class: "text-slate-300", "Active nodes: {chain.read().active_nodes}" }
            GlassSurface { class: Some("p-5".to_string()),
                table { class: "w-full text-left text-sm",
                    thead { class: "text-slate-400",
                        tr {
                            th { class: "pb-2", "Node" }
                            th { class: "pb-2", "Region" }
                            th { class: "pb-2", "Latency" }
                            th { class: "pb-2", "Status" }
                        }
                    }
                    tbody {
                        for node in chain.read().nodes.clone() {
                            tr { class: "border-t border-white/10",
                                td { class: "py-2", "{node.id}" }
                                td { class: "py-2", "{node.region}" }
                                td { class: "py-2", "{node.latency_ms} ms" }
                                td { class: "py-2", if node.online { "online" } else { "offline" } }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    let path = if segments.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", segments.join("/"))
    };

    rsx! {
        GlassSurface { class: Some("p-8".to_string()),
            h2 { class: "text-2xl font-bold text-white", "Sayfa bulunamadı" }
            p { class: "mt-2 text-slate-300", "İstenen rota: {path}" }
            p { class: "mt-1 text-slate-400", "Yan menüden geçerli bir modül seçebilirsiniz." }
        }
    }
}

#[component]
fn CompatibilityPanel() -> Element {
    let matrix = [
        (
            "desktop-linux-x86_64",
            "mainnet ✅",
            "devnet ✅",
            "testnet ✅",
        ),
        (
            "desktop-macos-arm64",
            "mainnet ✅",
            "devnet ✅",
            "testnet ✅",
        ),
        (
            "desktop-windows-x86_64",
            "mainnet ✅",
            "devnet ✅",
            "testnet ⚠ review",
        ),
    ];

    rsx! {
        GlassSurface { class: Some("p-5".to_string()),
            h3 { class: "text-lg font-semibold text-white", "Binary Uyumluluk Matrisi" }
            p { class: "mt-2 text-sm text-slate-300", "Mainnet/Devnet/Testnet hedefleri için release profile durumu." }
            div { class: "mt-4 space-y-2",
                for (target, main, dev, test) in matrix {
                    div { class: "rounded-xl border border-white/10 bg-white/5 px-4 py-3 text-sm text-slate-200",
                        "{target} • {main} • {dev} • {test}"
                    }
                }
            }
        }
    }
}

#[component]
fn TelemetryPanel(telemetry_source: &'static str, telemetry_ok: bool) -> Element {
    let status = if telemetry_ok { "healthy" } else { "degraded" };

    rsx! {
        GlassSurface { class: Some("p-5".to_string()),
            h3 { class: "text-lg font-semibold text-white", "Telemetry" }
            p { class: "mt-2 text-sm text-slate-300", "Kaynak: {telemetry_source} • Durum: {status}" }
            div { class: "mt-4 grid gap-3 md:grid-cols-3",
                MetricCard { title: "Ingest/sec", value: "12,490".to_string(), hint: "events".to_string() }
                MetricCard { title: "Error rate", value: "0.03%".to_string(), hint: "last 15m".to_string() }
                MetricCard { title: "Trace backlog", value: "3".to_string(), hint: "pending".to_string() }
            }
        }
    }
}

#[component]
fn WalletPanel() -> Element {
    let accounts = [
        ("Treasury", "143,920,000 AOXC", "multisig: 4/7"),
        ("Hot Wallet", "6,051,200 AOXC", "ops limit: 250,000 AOXC"),
        ("Bridge Reserve", "24,410,000 AOXC", "locked"),
    ];

    rsx! {
        GlassSurface { class: Some("p-5".to_string()),
            h3 { class: "text-lg font-semibold text-white", "Wallet Operasyonları" }
            div { class: "mt-4 space-y-2",
                for (name, balance, policy) in accounts {
                    div { class: "rounded-xl border border-white/10 bg-white/5 px-4 py-3",
                        p { class: "font-medium text-white", "{name}" }
                        p { class: "text-sm text-slate-300", "{balance}" }
                        p { class: "text-xs text-slate-400", "{policy}" }
                    }
                }
            }
        }
    }
}

#[component]
fn ExplorerPanel(chain: GlobalChainState) -> Element {
    rsx! {
        GlassSurface { class: Some("p-5".to_string()),
            h3 { class: "text-lg font-semibold text-white", "Explorer" }
            p { class: "mt-2 text-sm text-slate-300", "Son blok ve validator durumları." }
            div { class: "mt-4 space-y-2",
                for lane in chain.lanes {
                    div { class: "rounded-xl border border-white/10 bg-white/5 px-4 py-3 text-sm text-slate-200",
                        "{lane.kind:?} lane • {lane.tps} TPS • checkpoint {lane.last_checkpoint}"
                    }
                }
            }
        }
    }
}

#[component]
fn CommandPanel(title: &'static str, commands: Vec<CommandSpec>) -> Element {
    rsx! {
        GlassSurface { class: Some("p-5".to_string()),
            h3 { class: "text-lg font-semibold text-white", "{title}" }
            div { class: "mt-4 space-y-2",
                for cmd in commands {
                    div { class: "rounded-xl border border-white/10 bg-white/5 px-4 py-3",
                        div { class: "flex items-center justify-between gap-3",
                            code { class: "text-xs text-blue-200", "{cmd.command}" }
                            span {
                                class: if cmd.status == "stable" {
                                    "rounded-full border border-emerald-400/40 bg-emerald-500/20 px-2 py-0.5 text-[10px] uppercase tracking-wide text-emerald-200"
                                } else {
                                    "rounded-full border border-amber-400/40 bg-amber-500/20 px-2 py-0.5 text-[10px] uppercase tracking-wide text-amber-200"
                                },
                                "{cmd.status}"
                            }
                        }
                        p { class: "mt-2 text-sm text-slate-300", "{cmd.purpose}" }
                    }
                }
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
