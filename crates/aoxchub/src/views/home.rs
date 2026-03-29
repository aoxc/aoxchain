use dioxus::prelude::*;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

use super::menus::{
    DashboardSection, DomainSections, OperationsSection, OverviewSection, WalletSetupSection,
};

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct ProfileSnapshot {
    profile: String,
    chain_id: String,
    rpc_addr: String,
    p2p_port: String,
    telemetry_port: String,
    validators_path: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct RpcProbe {
    profile: String,
    url: String,
    ok: bool,
    latency_ms: u64,
    note: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct HubSnapshot {
    generated_at: String,
    profiles: Vec<ProfileSnapshot>,
    probes: Vec<RpcProbe>,
}

#[component]
pub fn Home() -> Element {
    rsx! {
        div {
            class: "hub-page retro-shell",
            WalletSetupSection {}
            OverviewSection {}
            DashboardSection {}
            OperationsSection {}
            DomainSections {}
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

#[allow(dead_code)]
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
