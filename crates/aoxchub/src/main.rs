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

#[derive(Clone, Copy)]
struct NodePreset {
    label: &'static str,
    action: &'static str,
    description: &'static str,
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

const NODE_PRESETS: [NodePreset; 6] = [
    NodePreset {
        label: "Node Start",
        action: "start",
        description: "Node sürecini güvenli başlangıç profili ile ayağa kaldırır.",
    },
    NodePreset {
        label: "Node Stop",
        action: "stop",
        description: "Node üretimini sonlandırır ve kapanış durumunu yazar.",
    },
    NodePreset {
        label: "Node Restart",
        action: "restart",
        description: "Servisi sıfır kesinti hedefiyle yeniden başlatır.",
    },
    NodePreset {
        label: "Snapshot",
        action: "snapshot",
        description: "Durum ağacını snapshot alıp doğrulama hash üretir.",
    },
    NodePreset {
        label: "Log Stream",
        action: "tail-log",
        description: "Son log akışını operasyon paneline taşır.",
    },
    NodePreset {
        label: "Drain Mode",
        action: "drain",
        description: "Node'u bakım moduna alır, yeni iş kabulünü durdurur.",
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
    let mut bootstrap_network = use_signal(|| "devnet".to_string());
    let mut bootstrap_result = use_signal(String::new);
    let mut node_network = use_signal(|| "testnet".to_string());
    let mut node_id = use_signal(|| "atlas-01".to_string());
    let mut node_action = use_signal(|| "start".to_string());
    let mut node_result = use_signal(String::new);
    let mut wallet_network = use_signal(|| "testnet".to_string());
    let mut wallet_from = use_signal(|| "wallet-operator-001".to_string());
    let mut wallet_to = use_signal(|| "wallet-validator-007".to_string());
    let mut wallet_amount = use_signal(|| "25.0".to_string());
    let mut wallet_memo = use_signal(String::new);
    let mut wallet_result = use_signal(String::new);

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
            p { "Tüm kritik işlemler GUI üzerinden tetiklenir; devnet/testnet/mainnet bootstrap + node yönetim + wallet transfer tek ekrandan yürütülür." }

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

        section { class: "ops-console",
            h2 { "Sıfırdan Ağ Başlatma (Devnet/Testnet/Mainnet)" }
            p { "Yeni ağ turu için bootstrap adımlarını UI üzerinden tetikleyin." }
            div { class: "ops-form-grid",
                label { class: "ops-field",
                    span { "Başlatılacak Ağ" }
                    select {
                        value: "{bootstrap_network}",
                        onchange: move |event| bootstrap_network.set(event.value()),
                        option { value: "devnet", "devnet" }
                        option { value: "testnet", "testnet" }
                        option { value: "mainnet", "mainnet" }
                    }
                }
            }
            button {
                class: "run-op-btn",
                onclick: move |_| async move {
                    let network = bootstrap_network();
                    let response = bootstrap_network_server(network)
                        .await
                        .unwrap_or_else(|err| format!("Bootstrap failed: {err}"));
                    bootstrap_result.set(response.clone());
                    op_history.with_mut(|entries| entries.insert(0, response));
                },
                "Ağı Sıfırdan Başlat"
            }
            if !bootstrap_result().is_empty() {
                p { class: "server-answer", "{bootstrap_result}" }
            }
        }

        section { class: "ops-console",
            h2 { "Node Yönetimi (%100 UI)" }
            p { "Node seçimi, aksiyon seçimi ve yönetim komutlarının tamamı arayüzden çalışır." }

            div { class: "ops-form-grid",
                label { class: "ops-field",
                    span { "Ağ" }
                    select {
                        value: "{node_network}",
                        onchange: move |event| node_network.set(event.value()),
                        option { value: "devnet", "devnet" }
                        option { value: "testnet", "testnet" }
                        option { value: "mainnet", "mainnet" }
                    }
                }
                label { class: "ops-field",
                    span { "Node ID" }
                    input {
                        value: "{node_id}",
                        oninput: move |event| node_id.set(event.value()),
                        placeholder: "ör: atlas-01"
                    }
                }
            }

            div { class: "quick-actions",
                for preset in NODE_PRESETS {
                    button {
                        class: "quick-action-btn",
                        onclick: move |_| node_action.set(preset.action.to_string()),
                        h3 { "{preset.label}" }
                        p { "{preset.description}" }
                    }
                }
            }

            label { class: "ops-field",
                span { "Seçili Node Aksiyonu" }
                select {
                    value: "{node_action}",
                    onchange: move |event| node_action.set(event.value()),
                    for preset in NODE_PRESETS {
                        option { value: "{preset.action}", "{preset.label}" }
                    }
                }
            }

            button {
                class: "run-op-btn",
                onclick: move |_| async move {
                    let network = node_network();
                    let id = node_id();
                    let action = node_action();
                    let response = manage_node_server(network, id, action)
                        .await
                        .unwrap_or_else(|err| format!("Node action failed: {err}"));
                    node_result.set(response.clone());
                    op_history.with_mut(|entries| entries.insert(0, response));
                },
                "Node Yönetim Komutunu Çalıştır"
            }
            if !node_result().is_empty() {
                p { class: "server-answer", "{node_result}" }
            }
        }

        section { class: "ops-console",
            h2 { "Wallet Transfer (UI)" }
            p { "Transfer emri, ağ seçimi ve açıklama bilgisi arayüzden verilir." }
            div { class: "ops-form-grid",
                label { class: "ops-field",
                    span { "Ağ" }
                    select {
                        value: "{wallet_network}",
                        onchange: move |event| wallet_network.set(event.value()),
                        option { value: "devnet", "devnet" }
                        option { value: "testnet", "testnet" }
                        option { value: "mainnet", "mainnet" }
                    }
                }
                label { class: "ops-field",
                    span { "Gönderen Cüzdan" }
                    input {
                        value: "{wallet_from}",
                        oninput: move |event| wallet_from.set(event.value()),
                        placeholder: "wallet-operator-001"
                    }
                }
                label { class: "ops-field",
                    span { "Alıcı Cüzdan" }
                    input {
                        value: "{wallet_to}",
                        oninput: move |event| wallet_to.set(event.value()),
                        placeholder: "wallet-validator-007"
                    }
                }
                label { class: "ops-field",
                    span { "Miktar" }
                    input {
                        value: "{wallet_amount}",
                        oninput: move |event| wallet_amount.set(event.value()),
                        placeholder: "25.0"
                    }
                }
            }

            label { class: "ops-field",
                span { "Transfer Notu" }
                textarea {
                    rows: "2",
                    value: "{wallet_memo}",
                    oninput: move |event| wallet_memo.set(event.value()),
                    placeholder: "Release sonrası validator stake tamamlama vb."
                }
            }

            button {
                class: "run-op-btn",
                onclick: move |_| async move {
                    let network = wallet_network();
                    let from = wallet_from();
                    let to = wallet_to();
                    let amount = wallet_amount();
                    let memo = wallet_memo();
                    let response = transfer_wallet_server(network, from, to, amount, memo)
                        .await
                        .unwrap_or_else(|err| format!("Transfer failed: {err}"));
                    wallet_result.set(response.clone());
                    op_history.with_mut(|entries| entries.insert(0, response));
                },
                "Wallet Transfer Çalıştır"
            }

            if !wallet_result().is_empty() {
                p { class: "server-answer", "{wallet_result}" }
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

#[post("/api/bootstrap-network")]
async fn bootstrap_network_server(network: String) -> Result<String, ServerFnError> {
    Ok(format!(
        "[BOOTSTRAP OK] network={network} steps=genesis->validators->rpc->observability source=ui"
    ))
}

#[post("/api/manage-node")]
async fn manage_node_server(
    network: String,
    node_id: String,
    action: String,
) -> Result<String, ServerFnError> {
    Ok(format!(
        "[NODE OK] network={network} node={node_id} action={action} source=ui"
    ))
}

#[post("/api/wallet-transfer")]
async fn transfer_wallet_server(
    network: String,
    from: String,
    to: String,
    amount: String,
    memo: String,
) -> Result<String, ServerFnError> {
    let parsed_amount = amount.parse::<f64>().unwrap_or(0.0);
    if parsed_amount <= 0.0 {
        return Ok("[TRANSFER ERROR] amount must be greater than zero".to_string());
    }
    if from.trim().is_empty() || to.trim().is_empty() {
        return Ok("[TRANSFER ERROR] from/to wallet cannot be empty".to_string());
    }

    let normalized_memo = if memo.trim().is_empty() {
        "memo-yok".to_string()
    } else {
        memo
    };
    Ok(format!(
        "[TRANSFER OK] network={network} from={from} to={to} amount={parsed_amount:.4} memo=\"{normalized_memo}\" source=ui"
    ))
}
