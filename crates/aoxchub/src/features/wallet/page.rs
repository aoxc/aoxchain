use std::time::{SystemTime, UNIX_EPOCH};

use dioxus::prelude::*;

fn build_demo_address(seed: u128) -> String {
    let mut body = format!("{seed:032x}");
    if body.len() < 38 {
        body = format!("{body:0<38}");
    }
    format!("aoxc1{}", &body[..38])
}

#[component]
pub fn WalletSetupSection() -> Element {
    let mut wallet_label = use_signal(|| String::from("operator-main"));
    let mut generated = use_signal(Vec::<String>::new);

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

    let policy_checks = [
        (
            "Session policy",
            "Desktop session key and signing scope validated.",
        ),
        (
            "Recovery integrity",
            "Backup checksum confirmed with operator verification.",
        ),
        (
            "Funding readiness",
            "Gas threshold satisfies governance and bridge workflows.",
        ),
        (
            "Audit log",
            "Address generation and usage trail exported to AOXCData logs.",
        ),
    ];

    let wallet_books = [
        (
            "Treasury Wallet",
            "aoxc1treasury...",
            "96.4M AOXC",
            "Multi-sig",
        ),
        (
            "Staking Rewards Vault",
            "aoxc1staking...",
            "12.8M AOXC",
            "Policy lock",
        ),
        (
            "NFT Royalty Sink",
            "aoxc1nftroy...",
            "2.7M AOXC",
            "Automated",
        ),
    ];

    let token_and_nft_controls = [
        (
            "Token mint policy",
            "Mint controls are bound to governance-approved binary hashes.",
        ),
        (
            "Treasury spending",
            "Budget releases require quorum proof and signature threshold checks.",
        ),
        (
            "NFT collection gate",
            "Collection creation enforces metadata schema and royalty floor.",
        ),
        (
            "Cross-chain wallet relay",
            "Outbound transfers validate bridge risk score before final signing.",
        ),
    ];

    rsx! {
        section {
            id: "wallet-setup",
            class: "panel glass",
            h2 { "Wallet and Treasury Administration" }
            p { class: "hero-sub", "The wallet workspace covers address generation, treasury custody, staking funding paths, token/NFT controls, and policy-bound signing." }

            article {
                class: "wallet-generator",
                h3 { "Quick Address Generator" }
                p {
                    class: "wallet-generator-note",
                    "This panel generates deterministic operator-format addresses for workflow rehearsal and policy verification."
                }
                div {
                    class: "wallet-generator-row",
                    input {
                        class: "wallet-input",
                        r#type: "text",
                        value: wallet_label,
                        placeholder: "Wallet label",
                        oninput: move |evt| wallet_label.set(evt.value()),
                    }
                    button {
                        class: "wallet-generate-btn",
                        onclick: move |_| {
                            let nanos = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .map(|duration| duration.as_nanos())
                                .unwrap_or(0);
                            let seed = nanos ^ (wallet_label().len() as u128 * 97_531);
                            let address = format!("{}  ({})", build_demo_address(seed), wallet_label());
                            generated.with_mut(|addresses| {
                                addresses.insert(0, address);
                                if addresses.len() > 5 {
                                    addresses.pop();
                                }
                            });
                        },
                        "Generate Address"
                    }
                }

                if generated().is_empty() {
                    p { class: "wallet-empty", "No address generated yet." }
                } else {
                    ul {
                        class: "wallet-address-list",
                        for address in generated() {
                            li { class: "mono-line", "{address}" }
                        }
                    }
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
                class: "panel glass",
                h3 { "Treasury and Staking Wallet Book" }
                table {
                    class: "hub-table integration-table",
                    thead {
                        tr {
                            th { "Wallet" }
                            th { "Address" }
                            th { "Balance" }
                            th { "Control" }
                        }
                    }
                    tbody {
                        for (name, address, balance, control) in wallet_books {
                            tr {
                                td { "{name}" }
                                td { class: "mono-line", "{address}" }
                                td { "{balance}" }
                                td { "{control}" }
                            }
                        }
                    }
                }
            }

            article {
                class: "wallet-policy panel glass",
                h3 { "Token and NFT Control Checklist" }
                ul {
                    class: "activity-list",
                    for (name, detail) in token_and_nft_controls {
                        li {
                            div {
                                p { class: "activity-kind", "{name}" }
                                p { class: "activity-pair", "{detail}" }
                            }
                            div {
                                p { class: "activity-amount", "Enforced" }
                                p { class: "activity-time", "Live" }
                            }
                        }
                    }
                }
            }

            article {
                class: "wallet-policy panel glass",
                h3 { "Operational Policy Validation" }
                ul {
                    class: "activity-list",
                    for (name, detail) in policy_checks {
                        li {
                            div {
                                p { class: "activity-kind", "{name}" }
                                p { class: "activity-pair", "{detail}" }
                            }
                            div {
                                p { class: "activity-amount", "Pass" }
                                p { class: "activity-time", "Required" }
                            }
                        }
                    }
                }
            }
        }
    }
}
