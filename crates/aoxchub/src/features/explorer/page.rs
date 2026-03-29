use dioxus::prelude::*;

#[component]
pub fn OverviewSection() -> Element {
    let capabilities = [
        (
            "Chain Management",
            "Protocol lifecycle, release channels, governance voting, and deterministic upgrade evidence.",
        ),
        (
            "Node Fleet Management",
            "Validator onboarding, RPC region coverage, failover health, and binary profile alignment.",
        ),
        (
            "Treasury and Staking",
            "Treasury flows, emission controls, staking APY surfaces, and delegation policy checks.",
        ),
        (
            "Token and NFT Operations",
            "Mint policy boundaries, collection analytics, metadata integrity, and transfer control windows.",
        ),
    ];

    let bridge_matrix = [
        ("Ethereum", "Online", "Finality lag: 1.2 blocks"),
        ("Cardano", "Online", "Mithril snapshot validated"),
        ("Sui", "Online", "Light-client proof healthy"),
        ("Cosmos IBC", "Online", "Packet success: 99.998%"),
    ];

    let governance_pipeline = [
        (
            "Proposal intake",
            "Policy and quorum validation before on-chain registration.",
        ),
        (
            "Risk simulation",
            "Economic and liveness simulation performed against testnet mirror.",
        ),
        (
            "Voting execution",
            "Operator wallet signing with session policy and audit trail.",
        ),
        (
            "Enactment",
            "Timed activation with emergency rollback and post-upgrade scorecard.",
        ),
    ];

    let staking_products = [
        ("Native Validator Staking", "6.9% APR", "Mainnet"),
        ("Liquid Staking Token", "5.8% APR", "Mainnet"),
        ("Institutional Vault", "7.1% APR", "Permissioned"),
    ];

    let ecosystem_widgets = [
        ("Treasury NAV", "$184.3M", "Policy-restricted spend windows"),
        ("Token Holders", "1,284,220", "24h net growth +0.8%"),
        ("NFT Collections", "412", "Royalty enforcement active"),
        ("DAO Integrations", "67", "All relays healthy"),
    ];

    rsx! {
        section {
            id: "overview",
            class: "hero glass",
            div {
                class: "hero-copy",
                p { class: "eyebrow", "AOXC Hub • Full Administrative Surface" }
                h1 { "Welcome to the complete chain command center" }
                p {
                    class: "hero-sub",
                    "This landing experience centralizes chain administration, node operations, treasury controls, staking, token/NFT management, and wallet orchestration in one interactive desktop workflow."
                }
                div {
                    class: "hero-actions",
                    Link { class: "btn btn-primary", to: crate::app::router::Route::Dashboard {}, "Open Live Dashboard" }
                    Link { class: "btn btn-ghost", to: crate::app::router::Route::Operations {}, "Open Binary Command Center" }
                }
            }
            div {
                class: "hero-panel",
                h3 { "Mainnet readiness snapshot" }
                ul {
                    li { span { "Consensus" } strong { "Healthy" } }
                    li { span { "Node Fleet" } strong { "147 / 147 online" } }
                    li { span { "Treasury" } strong { "Policy-compliant" } }
                    li { span { "Command Surface" } strong { "Interactive" } }
                }
            }
        }

        section { class: "widget-grid",
            for (title, detail) in capabilities {
                article {
                    class: "widget-card glass",
                    p { class: "widget-title", "{title}" }
                    p { class: "widget-note", "{detail}" }
                }
            }
        }

        section {
            id: "bridge",
            class: "panel glass",
            h2 { "Cross-Chain Bridge Control" }
            table {
                class: "hub-table integration-table",
                thead { tr { th { "Bridge" } th { "Status" } th { "Operational Note" } } }
                tbody {
                    for (network, status, note) in bridge_matrix {
                        tr {
                            td { "{network}" }
                            td { "{status}" }
                            td { "{note}" }
                        }
                    }
                }
            }
        }

        section {
            id: "governance",
            class: "panel glass",
            h2 { "Governance Execution Pipeline" }
            ul {
                class: "activity-list",
                for (step, detail) in governance_pipeline {
                    li {
                        div {
                            p { class: "activity-kind", "{step}" }
                            p { class: "activity-pair", "{detail}" }
                        }
                        div {
                            p { class: "activity-amount", "Active" }
                            p { class: "activity-time", "Tracked" }
                        }
                    }
                }
            }
        }

        section {
            id: "staking",
            class: "panel glass",
            h2 { "Staking and Yield Surfaces" }
            table {
                class: "hub-table integration-table",
                thead { tr { th { "Product" } th { "Yield" } th { "Profile" } } }
                tbody {
                    for (product, apr, profile) in staking_products {
                        tr {
                            td { "{product}" }
                            td { "{apr}" }
                            td { "{profile}" }
                        }
                    }
                }
            }
        }

        section {
            id: "ecosystem",
            class: "widget-grid",
            for (title, value, note) in ecosystem_widgets {
                article {
                    class: "widget-card glass",
                    p { class: "widget-title", "{title}" }
                    h3 { "{value}" }
                    p { class: "widget-note", "{note}" }
                }
            }
        }
    }
}
