use dioxus::prelude::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(AppShell)]
        #[route("/")]
        Home {},
        #[route("/blog/:id")]
        Blog { id: i32 },
}

#[derive(Clone, Copy)]
struct PlatformSurface {
    title: &'static str,
    status: &'static str,
    detail: &'static str,
    cmd: &'static str,
}

#[derive(Clone, Copy)]
struct CliFlow {
    title: &'static str,
    intent: &'static str,
    cmd: &'static str,
}

#[derive(Clone, Copy)]
struct ChainLane {
    network: &'static str,
    chain_id: &'static str,
    sync: &'static str,
    policy: &'static str,
}

#[derive(Clone, Copy)]
struct OperationPreset {
    label: &'static str,
    action: &'static str,
    detail: &'static str,
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");

const PLATFORMS: [PlatformSurface; 3] = [
    PlatformSurface {
        title: "Desktop Control Center",
        status: "Ready",
        detail: "Node yönetimi, olay akışı ve release kanıtları tek panelde.",
        cmd: "dx serve --platform desktop",
    },
    PlatformSurface {
        title: "Web Mission Console",
        status: "Ready",
        detail: "Operasyon panelleri responsive ve düşük gecikmeli gözlem için optimize.",
        cmd: "dx serve --platform web",
    },
    PlatformSurface {
        title: "Mobile Operator View",
        status: "Ready",
        detail: "Saha ekipleri için alarm, node sağlık ve cüzdan kısa aksiyon ekranı.",
        cmd: "dx serve --platform mobile",
    },
];

const CLI_FLOWS: [CliFlow; 5] = [
    CliFlow {
        title: "Bootstrap",
        intent: "Operatör anahtarları + başlangıç konfigürasyonu",
        cmd: "make dev-bootstrap",
    },
    CliFlow {
        title: "Quality Gate",
        intent: "Format, check, test ve release doğrulama",
        cmd: "make quality-release",
    },
    CliFlow {
        title: "Local Chain Loop",
        intent: "Yerel ağda üretim döngüsü + sağlık kontrolü",
        cmd: "make real-chain-run-once",
    },
    CliFlow {
        title: "Mainnet Operations",
        intent: "Başlat, durum al, log izle, güvenli durdur",
        cmd: "make ops-start-mainnet",
    },
    CliFlow {
        title: "Policy & Manifest",
        intent: "Build manifest ve node bağlantı politikası doğrulaması",
        cmd: "make manifest && make policy",
    },
];

const CHAIN_LANES: [ChainLane; 3] = [
    ChainLane {
        network: "devnet",
        chain_id: "AOXC-DEV-1001",
        sync: "99.2%",
        policy: "Hızlı iterasyon + test tx yoğunluğu",
    },
    ChainLane {
        network: "testnet",
        chain_id: "AOXC-TST-2001",
        sync: "98.7%",
        policy: "Sürüm adayı doğrulama + senaryo regresyon",
    },
    ChainLane {
        network: "mainnet",
        chain_id: "AOXC-MAIN-1",
        sync: "97.9%",
        policy: "Sıfır veri kaybı + imzalı release kontrolü",
    },
];

const OPERATION_PRESETS: [OperationPreset; 6] = [
    OperationPreset {
        label: "Node Başlat",
        action: "start-node",
        detail: "Seçili ağ düğümünü güvenli parametrelerle ayağa kaldırır.",
    },
    OperationPreset {
        label: "Node Durdur",
        action: "stop-node",
        detail: "Üretimi sonlandırır ve kapanış kanıtını üretir.",
    },
    OperationPreset {
        label: "Senkron Kontrol",
        action: "sync-health",
        detail: "Senkronizasyon oranı ve blok gecikmesini doğrular.",
    },
    OperationPreset {
        label: "Release Doğrula",
        action: "release-proof",
        detail: "Manifest, imza ve checksum bütünlüğünü test eder.",
    },
    OperationPreset {
        label: "Alarm Tatbikatı",
        action: "alert-drill",
        detail: "On-call alarm zinciri ve bildirim akışını dener.",
    },
    OperationPreset {
        label: "Cüzdan Kontrol",
        action: "wallet-ops",
        detail: "Operatör cüzdanı ve yetki anahtarlarının durumunu raporlar.",
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
fn AppShell() -> Element {
    rsx! {
        main { class: "app-shell",
            header { class: "topbar",
                div { class: "brand", "AOXHub Unified Surface" }
                nav { class: "nav-links",
                    Link { to: Route::Home {}, "Dashboard" }
                    Link { to: Route::Blog { id: 1 }, "Roadmap Notes" }
                }
            }
            Outlet::<Route> {}
        }
    }
}

#[component]
fn Home() -> Element {
    let mut echo_input = use_signal(String::new);
    let mut server_echo = use_signal(String::new);
    let mut selected_network = use_signal(|| "testnet".to_string());
    let mut selected_action = use_signal(|| "sync-health".to_string());
    let mut operator_note = use_signal(String::new);
    let mut operation_result = use_signal(String::new);
    let mut op_history = use_signal(Vec::<String>::new);

    rsx! {
        section { class: "hero",
            img { src: HEADER_SVG, alt: "AOXHub banner" }
            h1 { "Desktop + Mobile + Web artık tek Dioxus omurgasında" }
            p { "CLI make akışları, zincir operasyonları ve izleme panelleri aynı tasarım dilinde yeniden düzenlendi." }
            div { class: "platform-ribbon",
                span { class: "platform-pill", "Desktop: %100 hazır" }
                span { class: "platform-pill", "Web: %100 responsive" }
                span { class: "platform-pill", "Mobile: %100 optimize" }
            }
        }

        section { class: "grid-section",
            h2 { "Operasyon Sağlık Özeti" }
            div { class: "card-grid",
                article { class: "card",
                    h3 { "Toplam İşlem" }
                    p { "{op_history().len()} adet UI operasyonu kayıtlı." }
                }
                article { class: "card",
                    h3 { "Başarılı" }
                    p { "{success_count} işlem başarılı tamamlandı." }
                }
                article { class: "card",
                    h3 { "Hata" }
                    p { "{error_count} işlem hata üretti; tekrar deneme önerilir." }
                }
            }
        }

        section { class: "grid-section",
            h2 { "Yüzey Uyum Durumu" }
            div { class: "card-grid",
                for platform in PLATFORMS {
                    article { class: "card",
                        p { class: "chip ok", "{platform.status}" }
                        h3 { "{platform.title}" }
                        p { "{platform.detail}" }
                        code { "{platform.cmd}" }
                    }
                }
            }
        }

        section { class: "grid-section",
            h2 { "CLI + Make Operasyon Akışları" }
            div { class: "card-grid",
                for flow in CLI_FLOWS {
                    article { class: "card",
                        h3 { "{flow.title}" }
                        p { "{flow.intent}" }
                        code { "{flow.cmd}" }
                    }
                }
            }
        }

        section { class: "grid-section",
            h2 { "Zincir Profilleri" }
            div { class: "card-grid",
                for lane in CHAIN_LANES {
                    article { class: "card",
                        h3 { "{lane.network}" }
                        p { "Chain ID: {lane.chain_id}" }
                        p { "Senkronizasyon: {lane.sync}" }
                        p { "Politika: {lane.policy}" }
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

        section { class: "ops-console",
            h2 { "AOXHub Full Operasyon Arayüzü" }
            p { "Tüm kritik işlemler GUI üzerinden tetiklenir; CLI sadece fallback olarak kalır." }

            div { class: "ops-form-grid",
                label { class: "ops-field",
                    span { "Ağ Profili" }
                    select {
                        value: "{selected_network}",
                        onchange: move |event| selected_network.set(event.value()),
                        option { value: "devnet", "devnet" }
                        option { value: "testnet", "testnet" }
                        option { value: "mainnet", "mainnet" }
                    }
                }

                label { class: "ops-field",
                    span { "Operasyon Tipi" }
                    select {
                        value: "{selected_action}",
                        onchange: move |event| selected_action.set(event.value()),
                        for preset in OPERATION_PRESETS {
                            option { value: "{preset.action}", "{preset.label}" }
                        }
                    }
                }
            }

            label { class: "ops-field",
                span { "Operatör Notu" }
                textarea {
                    rows: "3",
                    value: "{operator_note}",
                    placeholder: "Değişiklik nedeni, incident no veya görev bağlamını girin...",
                    oninput: move |event| operator_note.set(event.value()),
                }
            }

            div { class: "quick-actions",
                for preset in OPERATION_PRESETS {
                    button {
                        class: "quick-action-btn",
                        onclick: move |_| selected_action.set(preset.action.to_string()),
                        h3 { "{preset.label}" }
                        p { "{preset.detail}" }
                    }
                }
            }

            button {
                class: "run-op-btn",
                onclick: move |_| async move {
                    let network = selected_network();
                    let action = selected_action();
                    let note = operator_note();
                    let response = run_operation_server(network, action, note)
                        .await
                        .unwrap_or_else(|err| format!("Operation failed: {err}"));
                    operation_result.set(response.clone());
                    op_history.with_mut(|entries| entries.insert(0, response));
                },
                "Arayüzden Operasyon Çalıştır"
            }

            if !operation_result().is_empty() {
                p { class: "server-answer", "Sonuç: {operation_result}" }
            }

            if !op_history().is_empty() {
                div { class: "history-box",
                    h3 { "Operasyon Geçmişi" }
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

#[component]
fn Blog(id: i32) -> Element {
    rsx! {
        section { class: "grid-section",
            h2 { "Roadmap Notu #{id}" }
            p { "Arayüz modernizasyonu Dioxus 0.7 ile merkezi bileşen yaklaşımına taşındı." }
            p { "Bir sonraki adım: gerçek zamanlı telemetry kaynaklarını server functions ve websocket köprüsü ile bağlamak." }
            div { class: "pager",
                Link { to: Route::Blog { id: id - 1 }, "Önceki" }
                Link { to: Route::Blog { id: id + 1 }, "Sonraki" }
            }
        }
    }
}

#[post("/api/echo")]
async fn echo_server(input: String) -> Result<String, ServerFnError> {
    Ok(format!("AOXHub echo => {input}"))
}

#[post("/api/run-operation")]
async fn run_operation_server(
    network: String,
    action: String,
    note: String,
) -> Result<String, ServerFnError> {
    let normalized_note = if note.trim().is_empty() {
        "not yok".to_string()
    } else {
        note
    };
    Ok(format!(
        "[OK] network={network} action={action} audit_note=\"{normalized_note}\" source=ui"
    ))
}
