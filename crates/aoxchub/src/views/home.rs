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
    let metrics = [
        ("Network TPS", "42,781", "+18.4%"),
        ("Finality", "480ms", "-12.1%"),
        ("24h Volume", "$1.28B", "+9.7%"),
        ("Active Wallets", "892,114", "+6.2%"),
    ];

    let validators = [
        ("Atlas One", "99.99%", "6.2M AOX", "Europe"),
        ("Cypher Labs", "99.97%", "5.8M AOX", "North America"),
        ("Delta Forge", "99.95%", "5.2M AOX", "Asia"),
        ("Boreal Node", "99.92%", "4.9M AOX", "South America"),
    ];

    let activity = [
        ("Bridge", "Ethereum → AOX", "$12.4M", "2m ago"),
        ("Swap", "AOX / USDC", "$4.8M", "5m ago"),
        ("Staking", "Validator delegation", "$7.1M", "9m ago"),
        ("Mint", "Identity credential", "$1.2M", "13m ago"),
    ];

    rsx! {
        main {
            class: "hub-page",
            section {
                id: "overview",
                class: "hero",
                div {
                    class: "hero-copy",
                    p { class: "eyebrow", "AOXChain Infrastructure Console" }
                    h1 { "Operate AOX Hub with full-network visibility." }
                    p {
                        class: "hero-sub",
                        "Production-grade monitoring, validator intelligence, and ecosystem telemetry in a unified command interface."
                    }
                    div {
                        class: "hero-actions",
                        button { class: "btn btn-primary", "Launch Console" }
                        button { class: "btn btn-ghost", "Open Docs" }
                    }
                }
                div {
                    class: "hero-panel glass",
                    h3 { "System Health" }
                    ul {
                        li { span { "Consensus" } strong { "Healthy" } }
                        li { span { "Bridge" } strong { "Synced" } }
                        li { span { "RPC" } strong { "47 regions online" } }
                    }
                }
            }

            section {
                class: "metrics-grid",
                for (title, value, delta) in metrics {
                    article {
                        class: "metric-card glass",
                        p { class: "metric-title", "{title}" }
                        p { class: "metric-value", "{value}" }
                        p { class: "metric-delta", "{delta}" }
                    }
                }
            }

            section {
                class: "content-grid",
                article {
                    id: "validators",
                    class: "panel glass",
                    h2 { "Top Validators" }
                    table {
                        class: "hub-table",
                        thead {
                            tr {
                                th { "Node" }
                                th { "Uptime" }
                                th { "Stake" }
                                th { "Region" }
                            }
                        }
                        tbody {
                            for (node, uptime, stake, region) in validators {
                                tr {
                                    td { "{node}" }
                                    td { "{uptime}" }
                                    td { "{stake}" }
                                    td { "{region}" }
                                }
                            }
                        }
                    }
                }

                article {
                    id: "activity",
                    class: "panel glass",
                    h2 { "Recent Activity" }
                    ul {
                        class: "activity-list",
                        for (kind, pair, amount, time) in activity {
                            li {
                                div {
                                    p { class: "activity-kind", "{kind}" }
                                    p { class: "activity-pair", "{pair}" }
                                }
                                div {
                                    p { class: "activity-amount", "{amount}" }
                                    p { class: "activity-time", "{time}" }
                                }
                            }
                        }
                    }
                }
            }

            section {
                id: "ecosystem",
                class: "ecosystem glass",
                h2 { "Ecosystem Overview" }
                p { "AOX Hub is fully integrated with staking, bridge operations, observability pipelines, and governance automation." }
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
