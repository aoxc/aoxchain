use dioxus::prelude::*;

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
