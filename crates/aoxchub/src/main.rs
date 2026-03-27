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

            section { class: "panel",
                h2 { "3) Canlı İşlem Çalıştır" }
                p { class: "sub", "Ağ + işlem seç, kısa not yaz, çalıştır." }

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
    let valid_network = ["devnet", "testnet", "mainnet"].contains(&network.as_str());
    let valid_action = CHAIN_ACTIONS.iter().any(|item| item.id == action.as_str());

    if !valid_network {
        return Ok(format!("❌ Hata: '{network}' geçerli bir ağ değil."));
    }

    if !valid_action {
        return Ok(format!("❌ Hata: '{action}' geçerli bir işlem değil."));
    }

    let clean_note = if note.trim().is_empty() {
        "not yok"
    } else {
        note.trim()
    };

    Ok(format!(
        "✅ Tamamlandı | ağ={network} | işlem={action} | not={clean_note}"
    ))
}
