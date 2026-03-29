use dioxus::prelude::*;

#[component]
pub fn OverviewSection() -> Element {
    rsx! {
        section {
            id: "overview",
            class: "hero glass",
            div {
                class: "hero-copy",
                p { class: "eyebrow", "AOXC Real Network Operations" }
                h1 { "AOXCHub: Retro Ops Desktop" }
                p {
                    class: "hero-sub",
                    "Header, sidebar, dashboard widget sistemi ve tüm operasyon menüleriyle AOXC zincir servislerine gerçek ağ uyumlu erişim sunar."
                }
                div {
                    class: "hero-actions",
                    button { class: "btn btn-primary", "Launch Mainnet Console" }
                    button { class: "btn btn-ghost", "Open Integration Guide" }
                }
            }
            div {
                class: "hero-panel",
                h3 { "Live Network Status" }
                ul {
                    li { span { "Consensus" } strong { "Healthy" } }
                    li { span { "Bridge Relays" } strong { "Synchronized" } }
                    li { span { "RPC Regions" } strong { "47 Online" } }
                    li { span { "Governance Engine" } strong { "Running" } }
                }
            }
        }
    }
}
