use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct ProfileSnapshot {
    profile: String,
    chain_id: String,
    rpc_addr: String,
    p2p_port: String,
    telemetry_port: String,
    validators_path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct RpcProbe {
    profile: String,
    url: String,
    ok: bool,
    latency_ms: u64,
    note: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct HubSnapshot {
    generated_at: String,
    profiles: Vec<ProfileSnapshot>,
    probes: Vec<RpcProbe>,
}

#[component]
pub fn Home() -> Element {
    let snapshot = use_resource(move || async move { hub_snapshot().await });

    rsx! {
        main {
            class: "hub-shell",
            aside {
                class: "sidebar glass",
                div {
                    class: "sidebar-brand",
                    div { class: "sidebar-logo", "AOX" }
                    div {
                        p { class: "sidebar-title", "AOX Hub Control" }
                        p { class: "sidebar-subtitle", "Chain-integrated operations" }
                    }
                }
                nav {
                    class: "side-nav",
                    a { href: "#overview", "Overview" }
                    a { href: "#profiles", "Profiles" }
                    a { href: "#rpc", "RPC Health" }
                    a { href: "#validators", "Validators" }
                    a { href: "#ops", "Operations" }
                }
                section {
                    class: "side-card",
                    p { class: "side-card-title", "CLI + API" }
                    ul {
                        li { code { "aoxcmd boot --profile mainnet" } }
                        li { code { "aoxcmd telemetry refresh" } }
                        li { code { "curl :28657/status" } }
                    }
                }
            }

            section {
                class: "workspace",
                section {
                    id: "overview",
                    class: "hero glass",
                    div {
                        p { class: "eyebrow", "AOXCHAIN COMMAND CENTER" }
                        h1 { "Fully integrated AOX Hub surface for chain, CLI, and RPC operations." }
                        p {
                            class: "hero-sub",
                            "Interface aligns with runtime profiles, validator operations, telemetry endpoints, and API observability."
                        }
                    }
                    div {
                        class: "hero-chart",
                        p { "Throughput Envelope" }
                        div { class: "chart-bars",
                            span { class: "bar b1" }
                            span { class: "bar b2" }
                            span { class: "bar b3" }
                            span { class: "bar b4" }
                            span { class: "bar b5" }
                            span { class: "bar b6" }
                        }
                    }
                }

                {
                    match snapshot() {
                        None => rsx! {
                            section { class: "panel glass", h2 { "Loading integrated hub snapshot..." } }
                        },
                        Some(Err(error)) => rsx! {
                            section {
                                class: "panel glass",
                                h2 { "Snapshot unavailable" }
                                p { "{error}" }
                            }
                        },
                        Some(Ok(data)) => rsx! {
                            section { class: "kpi-grid",
                                article { class: "kpi glass", p { "Profiles" } strong { "{data.profiles.len()}" } }
                                article { class: "kpi glass", p { "RPC Probes" } strong { "{data.probes.len()}" } }
                                article {
                                    class: "kpi glass",
                                    p { "Healthy Endpoints" }
                                    strong { "{data.probes.iter().filter(|probe| probe.ok).count()}" }
                                }
                                article { class: "kpi glass", p { "Snapshot (UTC)" } strong { "{data.generated_at}" } }
                            }

                            section {
                                id: "profiles",
                                class: "panel glass",
                                h2 { "Network Profiles" }
                                table { class: "hub-table",
                                    thead {
                                        tr {
                                            th { "Profile" }
                                            th { "Chain" }
                                            th { "RPC" }
                                            th { "P2P" }
                                            th { "Telemetry" }
                                            th { "Validators" }
                                        }
                                    }
                                    tbody {
                                        for profile in data.profiles {
                                            tr {
                                                td { "{profile.profile}" }
                                                td { "{profile.chain_id}" }
                                                td { "{profile.rpc_addr}" }
                                                td { "{profile.p2p_port}" }
                                                td { "{profile.telemetry_port}" }
                                                td { "{profile.validators_path}" }
                                            }
                                        }
                                    }
                                }
                            }

                            section {
                                id: "rpc",
                                class: "panel glass",
                                h2 { "RPC Endpoint Health" }
                                div { class: "probe-grid",
                                    for probe in data.probes {
                                        article { class: if probe.ok { "probe ok" } else { "probe fail" },
                                            p { class: "probe-title", "{probe.profile}" }
                                            p { class: "probe-url", "{probe.url}" }
                                            p { class: "probe-latency", "{probe.latency_ms} ms" }
                                            p { class: "probe-note", "{probe.note}" }
                                        }
                                    }
                                }
                            }
                        },
                    }
                }

                section {
                    id: "validators",
                    class: "panel glass",
                    h2 { "Validator Operations Checklist" }
                    ul { class: "ops-list",
                        li { "Genesis validator set loaded and non-empty." }
                        li { "RPC, P2P, and telemetry ports remain distinct per profile policy." }
                        li { "Metrics snapshots persisted under canonical telemetry path." }
                        li { "Governance serial and network profile family values remain coherent." }
                    }
                }

                section {
                    id: "ops",
                    class: "panel glass",
                    h2 { "Operator Commands" }
                    div { class: "cmd-grid",
                        code { "make up" }
                        code { "aoxcmd describe profile --name mainnet" }
                        code { "aoxcmd ops status --profile testnet" }
                        code { "aoxcmd telemetry refresh --profile localnet" }
                    }
                }

                footer {
                    class: "footer glass",
                    p { "AOX Hub integrates runtime profiles, RPC probes, and operator workflows into a single control plane." }
                }
            }
        }
    }
}

#[get("/api/hub-snapshot")]
async fn hub_snapshot() -> Result<HubSnapshot> {
    let generated_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string());

    let profiles = ["mainnet", "testnet", "validation", "localnet"];
    let mut profile_snapshots = Vec::with_capacity(profiles.len());

    for profile in profiles {
        let path = format!("configs/aoxhub/{profile}.toml");
        let content = std::fs::read_to_string(&path).unwrap_or_default();

        profile_snapshots.push(ProfileSnapshot {
            profile: profile.to_string(),
            chain_id: parse_toml_value(&content, "chain_id")
                .unwrap_or_else(|| "unknown".to_string()),
            rpc_addr: parse_toml_value(&content, "rpc_bind")
                .or_else(|| parse_toml_value(&content, "rpc_address"))
                .unwrap_or_else(|| "127.0.0.1".to_string()),
            p2p_port: parse_toml_value(&content, "p2p_port").unwrap_or_else(|| "n/a".to_string()),
            telemetry_port: parse_toml_value(&content, "prometheus_port")
                .unwrap_or_else(|| "n/a".to_string()),
            validators_path: parse_toml_value(&content, "validators")
                .unwrap_or_else(|| "n/a".to_string()),
        });
    }

    let mut probes = Vec::with_capacity(profile_snapshots.len());
    for profile in &profile_snapshots {
        let url = if profile.rpc_addr.contains(':') {
            format!("http://{}", profile.rpc_addr)
        } else {
            format!("http://{}:{}", profile.rpc_addr, 28657)
        };

        let start = std::time::Instant::now();
        let client = reqwest::Client::new();
        let outcome = client
            .get(format!("{url}/status"))
            .timeout(std::time::Duration::from_millis(500))
            .send()
            .await;

        let (ok, note) = match outcome {
            Ok(response) => {
                if response.status().is_success() {
                    (true, "status endpoint reachable".to_string())
                } else {
                    (false, format!("http {}", response.status().as_u16()))
                }
            }
            Err(error) => (false, error.to_string()),
        };

        probes.push(RpcProbe {
            profile: profile.profile.clone(),
            url,
            ok,
            latency_ms: start.elapsed().as_millis() as u64,
            note,
        });
    }

    Ok(HubSnapshot {
        generated_at,
        profiles: profile_snapshots,
        probes,
    })
}

fn parse_toml_value(content: &str, key: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || !trimmed.starts_with(key) {
            return None;
        }
        let (_, raw) = trimmed.split_once('=')?;
        let value = raw.trim().trim_matches('"').to_string();
        Some(value)
    })
}
