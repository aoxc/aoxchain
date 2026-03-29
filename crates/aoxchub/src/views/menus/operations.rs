use dioxus::prelude::*;

#[component]
pub fn OperationsSection() -> Element {
    let validators = [
        ("Atlas One", "99.99%", "6.2M AOXC", "Europe"),
        ("Cypher Labs", "99.97%", "5.8M AOXC", "North America"),
        ("Delta Forge", "99.95%", "5.2M AOXC", "Asia"),
        ("Boreal Node", "99.92%", "4.9M AOXC", "South America"),
    ];

    let activity = [
        ("Bridge", "Ethereum → AOXC", "$12.4M", "2m ago"),
        ("Swap", "AOXC / USDC", "$4.8M", "5m ago"),
        ("Staking", "Validator delegation", "$7.1M", "9m ago"),
        ("Mint", "Identity credential", "$1.2M", "13m ago"),
    ];

    rsx! {
        section {
            class: "content-grid",
            article {
                id: "validators",
                class: "panel glass",
                h2 { "Validator Matrix" }
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
                id: "rpc-monitor",
                class: "panel glass",
                h2 { "Operational Stream" }
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
    }
}
