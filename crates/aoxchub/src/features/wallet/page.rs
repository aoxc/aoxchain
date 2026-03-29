use dioxus::prelude::*;

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

    rsx! {
        section {
            id: "wallet-setup",
            class: "panel glass",
            h2 { "Wallet Onboarding" }
            p { class: "hero-sub", "İlk menü cüzdan oluşturma ile başlar: adres üretimi, yedekleme, fonlama ve politika bağlama adımları tek akışta yönetilir." }
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
        }
    }
}
