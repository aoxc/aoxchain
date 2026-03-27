use dioxus::prelude::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    Home {},
}

#[derive(Clone, Copy)]
struct BeginnerStep {
    id: &'static str,
    title: &'static str,
    explain: &'static str,
}

#[derive(Clone, Copy)]
struct ChainAction {
    id: &'static str,
    title: &'static str,
    explain: &'static str,
    example: &'static str,
}

#[derive(Clone, Copy)]
struct ChainCapability {
    title: &'static str,
    scope: &'static str,
    slos: &'static str,
    fallback: &'static str,
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");

const BEGINNER_STEPS: [BeginnerStep; 5] = [
    BeginnerStep {
        id: "1",
        title: "Programı Aç",
        explain: "AOXHub düğmesine bas. Ekran açılınca hazırız.",
    },
    BeginnerStep {
        id: "2",
        title: "Ağı Seç",
        explain: "Devnet test için, Mainnet gerçek kullanım için.",
    },
    BeginnerStep {
        id: "3",
        title: "İşlem Türü Seç",
        explain: "Para gönder, stake yap, oy ver gibi işlemi seç.",
    },
    BeginnerStep {
        id: "4",
        title: "Not Yaz",
        explain: "Neden yaptığını kısa yaz: örn. 'test transferi'.",
    },
    BeginnerStep {
        id: "5",
        title: "Çalıştır ve Kontrol Et",
        explain: "Çalıştır'a bas. Sonuç yeşilse tamam, kırmızıysa düzelt.",
    },
];

const CHAIN_ACTIONS: [ChainAction; 7] = [
    ChainAction {
        id: "transfer",
        title: "Transfer",
        explain: "Bir cüzdandan diğerine coin gönderir.",
        example: "Örnek: 1 AOXC gönder",
    },
    ChainAction {
        id: "stake",
        title: "Stake",
        explain: "Coin kilitleyip ağ güvenliğine katkı sağlar.",
        example: "Örnek: 50 AOXC stake et",
    },
    ChainAction {
        id: "unstake",
        title: "Unstake",
        explain: "Stake edilen coinleri geri açar.",
        example: "Örnek: 10 AOXC unstake et",
    },
    ChainAction {
        id: "vote",
        title: "Yönetişim Oyu",
        explain: "Tekliflere evet/hayır oyu verir.",
        example: "Örnek: Proposal #21 için evet",
    },
    ChainAction {
        id: "contract-call",
        title: "Sözleşme Çağrısı",
        explain: "Akıllı sözleşmede bir fonksiyon çalıştırır.",
        example: "Örnek: mint(1)",
    },
    ChainAction {
        id: "bridge",
        title: "Köprü (Bridge)",
        explain: "Varlığı bir ağdan diğerine taşır.",
        example: "Örnek: Testnet -> Mainnet",
    },
    ChainAction {
        id: "multisig",
        title: "Multisig",
        explain: "Birden fazla imza ile güvenli onay yapar.",
        example: "Örnek: 2/3 imza ile onay",
    },
];

const CHAIN_CAPABILITIES: [ChainCapability; 6] = [
    ChainCapability {
        title: "Transfer + Fee Estimation",
        scope: "Native coin transfer, fee simulation, nonce protection",
        slos: "P95 submission < 900ms",
        fallback: "CLI fallback: aoxc tx transfer --safe-mode",
    },
    ChainCapability {
        title: "Staking + Validator Delegation",
        scope: "Stake, unstake, reward query, validator health snapshot",
        slos: "P95 action ack < 1200ms",
        fallback: "CLI fallback: aoxc staking rebalance",
    },
    ChainCapability {
        title: "Bridge Dispatch",
        scope: "Source lock + destination proof checkpoint",
        slos: "Proof finality window monitored every 20s",
        fallback: "CLI fallback: aoxc bridge relay --proof",
    },
    ChainCapability {
        title: "Governance Voting",
        scope: "Proposal inspect, vote cast, tally monitor",
        slos: "Vote ack < 2 blocks",
        fallback: "CLI fallback: aoxc gov vote --broadcast",
    },
    ChainCapability {
        title: "Contract Invoke",
        scope: "ABI call builder, dry-run, signed execution",
        slos: "Preflight simulation success target > 99%",
        fallback: "CLI fallback: aoxc contract call --simulate-first",
    },
    ChainCapability {
        title: "Treasury + Multisig",
        scope: "Policy-aware approval lanes, signer quorum checks",
        slos: "Quorum verification in < 600ms",
        fallback: "CLI fallback: aoxc treasury multisig --strict",
    },
];

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

#[component]
fn Home() -> Element {
    let mut selected_network = use_signal(|| "devnet".to_string());
    let mut selected_action = use_signal(|| CHAIN_ACTIONS[0].id.to_string());
    let mut operator_note = use_signal(String::new);
    let mut latest_result = use_signal(String::new);
    let mut op_history = use_signal(Vec::<String>::new);
    let success_count = op_history()
        .iter()
        .filter(|entry| entry.starts_with("[OK]"))
        .count();
    let error_count = op_history()
        .iter()
        .filter(|entry| entry.starts_with("[ERR]"))
        .count();

    let success_count = op_history()
        .iter()
        .filter(|entry| entry.starts_with("✅"))
        .count();
    let error_count = op_history()
        .iter()
        .filter(|entry| entry.starts_with("❌"))
        .count();

    rsx! {
        main { class: "page",
            section { class: "hero",
                img { class: "hero-image", src: HEADER_SVG, alt: "AOXHub" }
                h1 { "AOXHub: Herkes İçin Çok Basit Zincir Ekranı" }
                p { "Web, mobil ve masaüstü aynı ekrandan çalışır. 7 yaşındaki biri bile adımları izleyerek işlem yapabilir." }
                div { class: "badge-row",
                    span { class: "badge", "Web ✅" }
                    span { class: "badge", "Mobil ✅" }
                    span { class: "badge", "Desktop ✅" }
                }
            }

            section { class: "panel",
                h2 { "1) Sıfırdan Başlangıç (Binary'den)" }
                p { class: "sub", "Sadece sırayla git. Her adım kısa ve açık." }
                div { class: "step-grid",
                    for step in BEGINNER_STEPS {
                        article { class: "step-card",
                            div { class: "step-id", "{step.id}" }
                            h3 { "{step.title}" }
                            p { "{step.explain}" }
                        }
                    }
                }
            }

            section { class: "panel",
                h2 { "2) Tüm Zincir İşlemleri" }
                p { class: "sub", "Bu kutular ne işe yaradığını çocuk diliyle anlatır." }
                div { class: "action-grid",
                    for action in CHAIN_ACTIONS {
                        article { class: "action-card",
                            h3 { "{action.title}" }
                            p { "{action.explain}" }
                            p { class: "example", "{action.example}" }
                        }
                    }
                }
            }

        section { class: "grid-section",
            h2 { "Ultra Premium Zincir Yetenek Matrisi" }
            p { class: "section-note", "Web + mobile + desktop yüzeylerinin ortak işlem kabiliyetleri ve SLO hedefleri." }
            div { class: "card-grid",
                for capability in CHAIN_CAPABILITIES {
                    article { class: "card",
                        h3 { "{capability.title}" }
                        p { "Kapsam: {capability.scope}" }
                        p { "SLO: {capability.slos}" }
                        code { "{capability.fallback}" }
                    }
                }
            }
        }

        section { class: "echo-box",
            h2 { "ServerFn doğrulama" }
            p { "UI'dan girilen veri server function ile geri döner; web/desktop/mobile davranışı tek kaynakta tutulur." }
            input {
                value: "{echo_input}",
                placeholder: "Komut ya da payload yazın...",
                oninput: move |event| {
                    echo_input.set(event.value());
                }
            }
            button {
                onclick: move |_| async move {
                    let input = echo_input();
                    let result = echo_server(input).await.unwrap_or_else(|_| "echo failed".to_string());
                    server_echo.set(result);
                },
                "Echo çalıştır"
            }
            if !server_echo().is_empty() {
                p { class: "server-answer", "Server yanıtı: {server_echo}" }
            }
        }

                div { class: "form-grid",
                    label {
                        span { "Ağ" }
                        select {
                            value: "{selected_network}",
                            onchange: move |event| selected_network.set(event.value()),
                            option { value: "devnet", "devnet (öğrenme)" }
                            option { value: "testnet", "testnet (deneme)" }
                            option { value: "mainnet", "mainnet (gerçek)" }
                        }
                    }

                    label {
                        span { "İşlem" }
                        select {
                            value: "{selected_action}",
                            onchange: move |event| selected_action.set(event.value()),
                            for action in CHAIN_ACTIONS {
                                option { value: "{action.id}", "{action.title}" }
                            }
                        }
                    }
                }

                label {
                    span { "Kısa Not" }
                    textarea {
                        rows: "3",
                        value: "{operator_note}",
                        placeholder: "Örn: test transferi",
                        oninput: move |event| operator_note.set(event.value()),
                    }
                }

                button {
                    class: "run-btn",
                    onclick: move |_| async move {
                        let network = selected_network();
                        let action = selected_action();
                        let note = operator_note();

                        let result = run_operation_server(network, action, note)
                            .await
                            .unwrap_or_else(|e| format!("❌ Sistem hatası: {e}"));

                        latest_result.set(result.clone());
                        op_history.with_mut(|history| history.insert(0, result));
                    },
                    "İşlemi Çalıştır"
                }

                div { class: "stats-row",
                    p { "Toplam: {op_history().len()}" }
                    p { "Başarılı: {success_count}" }
                    p { "Hata: {error_count}" }
                }

                if !latest_result().is_empty() {
                    p { class: "result", "Sonuç: {latest_result}" }
                }

                if !op_history().is_empty() {
                    div { class: "history",
                        h3 { "Geçmiş" }
                        ul {
                            for item in op_history() {
                                li { "{item}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[post("/api/run-operation")]
async fn run_operation_server(
    network: String,
    action: String,
    note: String,
) -> Result<String, ServerFnError> {
    let is_known_network = ["devnet", "testnet", "mainnet"]
        .iter()
        .any(|known| known == &network.as_str());
    let is_known_action = OPERATION_PRESETS
        .iter()
        .any(|preset| preset.action == action.as_str());
    if !is_known_network || !is_known_action {
        return Ok(format!(
            "[ERR] network={network} action={action} reason=\"unsupported preset or network\" source=ui"
        ));
    }
    let normalized_note = if note.trim().is_empty() {
        "not yok".to_string()
    } else {
        note.trim()
    };

    Ok(format!(
        "✅ Tamamlandı | ağ={network} | işlem={action} | not={clean_note}"
    ))
}
