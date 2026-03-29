use dioxus::prelude::*;

#[component]
pub fn DashboardSection() -> Element {
    let kpis = [
        ("Network TPS", "42,781", "+18.4%"),
        ("Finality", "480ms", "-12.1%"),
        ("24h Volume", "$1.28B", "+9.7%"),
        ("Active Wallets", "892,114", "+6.2%"),
    ];

    let widgets = [
        ("Node Sync", "99.998%", "All regions healthy"),
        ("Smart Contract Calls", "18.2M/day", "EVM + WASM pipelines"),
        ("IBC Packets", "2.4M", "Cross-chain finalized"),
        ("Alert Queue", "03", "2 warning, 1 info"),
        ("Treasury Balance", "94.2M AOXC", "Policy constrained"),
        ("Governance Cycle", "Epoch 219", "Voting phase active"),
    ];

    rsx! {
        section {
            id: "dashboard",
            class: "metrics-grid",
            for (title, value, delta) in kpis {
                article {
                    class: "metric-card glass",
                    p { class: "metric-title", "{title}" }
                    p { class: "metric-value", "{value}" }
                    p { class: "metric-delta", "{delta}" }
                }
            }
        }

        section {
            class: "widget-grid",
            for (title, value, note) in widgets {
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
