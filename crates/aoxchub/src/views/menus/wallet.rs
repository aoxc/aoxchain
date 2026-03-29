use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WalletRecord {
    label: String,
    address: String,
    created_at: String,
}

#[component]
pub fn WalletSetupSection() -> Element {
    let wallet_steps = [
        (
            "1",
            "Create Address",
            "Generate a new AOXC wallet and bind an operator label.",
        ),
        (
            "2",
            "Backup Secret",
            "Export the recovery phrase to an offline vault before any transfer.",
        ),
        (
            "3",
            "Fund Wallet",
            "Bridge or faucet initial AOXC for gas, staking, and governance actions.",
        ),
        (
            "4",
            "Policy Bind",
            "Attach signature policy and session controls for desktop operations.",
        ),
    ];

    let mut wallet_label = use_signal(String::new);
    let mut refresh_tick = use_signal(|| 0_u64);
    let wallets = use_resource(move || async move {
        let _tick = refresh_tick();
        wallet_list().await.unwrap_or_default()
    });

    rsx! {
        section {
            id: "wallet-setup",
            class: "panel glass",
            h2 { "Wallet Onboarding" }
            p { class: "hero-sub", "İlk menü cüzdan oluşturma ile başlar: adres üretimi, yedekleme, fonlama ve politika bağlama adımları tek akışta yönetilir." }
            div {
                class: "wallet-create-form",
                input {
                    class: "wallet-input",
                    r#type: "text",
                    value: wallet_label(),
                    placeholder: "Wallet label (ör: ops-main)",
                    oninput: move |e| wallet_label.set(e.value()),
                }
                button {
                    class: "btn btn-primary",
                    onclick: move |_| {
                        let label = wallet_label();
                        if label.trim().is_empty() {
                            return;
                        }
                        spawn(async move {
                            let _ = wallet_create(label).await;
                            wallet_label.set(String::new());
                            *refresh_tick.write() += 1;
                        });
                    },
                    "Create Wallet Address"
                }
            }
            div {
                class: "wallet-steps",
                for (index, title, detail) in wallet_steps {
                    article {
                        class: "wallet-step",
                        span { class: "wallet-step-index", "{index}" }
                        div {
                            p { class: "wallet-step-title", "{title}" }
                            p { class: "wallet-step-detail", "{detail}" }
                        }
                    }
                }
            }
            article {
                class: "panel wallet-list-panel",
                h3 { "Generated Wallets" }
                p { class: "hero-sub", "Storage path: ~/.AOXCData/desktop/testnet/wallets/registry.json" }
                ul {
                    class: "wallet-list",
                    if let Some(entries) = wallets() {
                        if entries.is_empty() {
                            li { "Henüz wallet oluşturulmadı." }
                        } else {
                            for entry in entries {
                                li {
                                    p { class: "wallet-step-title", "{entry.label}" }
                                    code { "{entry.address}" }
                                }
                            }
                        }
                    } else {
                        li { "Wallet list loading..." }
                    }
                }
            }
        }
    }
}

#[post("/api/wallet/create")]
async fn wallet_create(label: String) -> Result<WalletRecord> {
    let mut records = load_wallets();
    let created_at = unix_timestamp();
    let address = build_address(&label, &created_at);

    let record = WalletRecord {
        label,
        address,
        created_at,
    };
    records.push(record.clone());
    persist_wallets(&records)?;
    Ok(record)
}

#[get("/api/wallet/list")]
async fn wallet_list() -> Result<Vec<WalletRecord>> {
    Ok(load_wallets())
}

#[allow(dead_code)]
fn wallet_registry_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    std::path::PathBuf::from(home)
        .join(".AOXCData")
        .join("desktop")
        .join("testnet")
        .join("wallets")
        .join("registry.json")
}

#[allow(dead_code)]
fn persist_wallets(records: &[WalletRecord]) -> Result<()> {
    let path = wallet_registry_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let payload = serde_json::to_string_pretty(records)
        .map_err(|error| std::io::Error::other(error.to_string()))?;
    std::fs::write(path, payload)?;
    Ok(())
}

#[allow(dead_code)]
fn load_wallets() -> Vec<WalletRecord> {
    let path = wallet_registry_path();
    let content = std::fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str::<Vec<WalletRecord>>(&content).unwrap_or_default()
}

#[allow(dead_code)]
fn unix_timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|v| v.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

#[allow(dead_code)]
fn build_address(label: &str, created_at: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    label.hash(&mut hasher);
    created_at.hash(&mut hasher);
    format!("aoxc1{:016x}", hasher.finish())
}
