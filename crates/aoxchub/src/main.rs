use dioxus::prelude::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    Home {},
}

#[derive(Clone, Copy)]
struct EnvironmentCard {
    id: &'static str,
    title: &'static str,
    rpc: &'static str,
    finality: &'static str,
    status: &'static str,
}

#[derive(Clone, Copy)]
struct ActionMenu {
    id: &'static str,
    title: &'static str,
    desc: &'static str,
}

#[derive(Clone, Copy)]
struct QuickMetric {
    label: &'static str,
    value: &'static str,
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

const ENVIRONMENTS: [EnvironmentCard; 3] = [
    EnvironmentCard {
        id: "mainnet",
        title: "Mainnet",
        rpc: "rpc.mainnet.aoxc",
        finality: "~2 blok",
        status: "Production",
    },
    EnvironmentCard {
        id: "testnet",
        title: "Testnet",
        rpc: "rpc.testnet.aoxc",
        finality: "~3 blok",
        status: "Staging",
    },
    EnvironmentCard {
        id: "devnet",
        title: "Devnet",
        rpc: "rpc.devnet.aoxc",
        finality: "anlık",
        status: "Developer",
    },
];

const ACTIONS: [ActionMenu; 8] = [
    ActionMenu {
        id: "transfer",
        title: "Transfer",
        desc: "Native token gönderimi",
    },
    ActionMenu {
        id: "stake",
        title: "Stake",
        desc: "Validator delegasyonu",
    },
    ActionMenu {
        id: "unstake",
        title: "Unstake",
        desc: "Stake çözme",
    },
    ActionMenu {
        id: "governance",
        title: "Governance",
        desc: "Oylama ve teklif",
    },
    ActionMenu {
        id: "contract",
        title: "Contract",
        desc: "Sözleşme çağrısı",
    },
    ActionMenu {
        id: "bridge",
        title: "Bridge",
        desc: "Ağlar arası varlık geçişi",
    },
    ActionMenu {
        id: "multisig",
        title: "Multisig",
        desc: "Çoklu imza onayı",
    },
    ActionMenu {
        id: "treasury",
        title: "Treasury",
        desc: "Fon yönetimi",
    },
];

const METRICS: [QuickMetric; 4] = [
    QuickMetric {
        label: "TPS",
        value: "1,248",
    },
    QuickMetric {
        label: "Finality",
        value: "1.8s",
    },
    QuickMetric {
        label: "Node Health",
        value: "99.98%",
    },
    QuickMetric {
        label: "Sync",
        value: "Hızlı",
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
    let mut selected_env = use_signal(|| "testnet".to_string());
    let mut selected_action = use_signal(|| ACTIONS[0].id.to_string());
    let mut note = use_signal(String::new);
    let mut latest_result = use_signal(String::new);
    let mut history = use_signal(Vec::<String>::new);

    let ok_count = history().iter().filter(|v| v.starts_with("✅")).count();
    let err_count = history().iter().filter(|v| v.starts_with("❌")).count();

    rsx! {
        main { class: "layout-shell",
            aside { class: "glass left-sidebar",
                h1 { class: "brand", "AOXHub Control" }
                p { class: "muted", "Mainnet / Testnet / Devnet tek arayüz" }

                nav { class: "menu-list",
                    for action in ACTIONS {
                        button {
                            class: if selected_action() == action.id { "menu-btn active" } else { "menu-btn" },
                            onclick: {
                                let action_id = action.id.to_string();
                                move |_| selected_action.set(action_id.clone())
                            },
                            span { class: "menu-title", "{action.title}" }
                            span { class: "menu-desc", "{action.desc}" }
                        }
                    }
                }
            }

            section { class: "glass content",
                header { class: "content-head",
                    div {
                        h2 { "Tüm Zincir Yönetim Paneli" }
                        p { class: "muted", "Desktop + Web + Mobil uyumlu yeni nesil arayüz" }
                    }
                    div { class: "metric-row",
                        for metric in METRICS {
                            article { class: "metric-pill",
                                p { class: "metric-label", "{metric.label}" }
                                p { class: "metric-value", "{metric.value}" }
                            }
                        }
                    }
                }

                section { class: "env-grid",
                    for env in ENVIRONMENTS {
                        button {
                            class: if selected_env() == env.id { "env-card selected" } else { "env-card" },
                            onclick: {
                                let env_id = env.id.to_string();
                                move |_| selected_env.set(env_id.clone())
                            },
                            h3 { "{env.title}" }
                            p { "RPC: {env.rpc}" }
                            p { "Finality: {env.finality}" }
                            p { class: "env-status", "{env.status}" }
                        }
                    }
                }

                section { class: "control-panel",
                    h3 { "İşlem Konsolu" }
                    p { class: "muted", "Seçilen ağ ve menüye göre işlem tetiklenir." }

                    label { class: "field",
                        span { "Operasyon Notu" }
                        textarea {
                            value: "{note}",
                            rows: "4",
                            placeholder: "Örn: testnet bridge dry-run",
                            oninput: move |event| note.set(event.value()),
                        }
                    }

                    button {
                        class: "primary-btn",
                        onclick: move |_| async move {
                            let env = selected_env();
                            let action = selected_action();
                            let operator_note = note();

                            let result = run_operation_server(env, action, operator_note)
                                .await
                                .unwrap_or_else(|err| format!("❌ sunucu hatası: {err}"));

                            latest_result.set(result.clone());
                            history.with_mut(|items| items.insert(0, result));
                        },
                        "İşlemi Çalıştır"
                    }

                    if !latest_result().is_empty() {
                        p { class: "latest", "Sonuç: {latest_result}" }
                    }
                }
            }

            aside { class: "glass right-sidebar",
                h3 { "Operasyon Durumu" }
                div { class: "status-grid",
                    article { class: "status-card",
                        p { "Toplam" }
                        strong { "{history().len()}" }
                    }
                    article { class: "status-card ok",
                        p { "Başarılı" }
                        strong { "{ok_count}" }
                    }
                    article { class: "status-card err",
                        p { "Hata" }
                        strong { "{err_count}" }
                    }
                }

                h4 { "Son İşlemler" }
                ul { class: "history-list",
                    if history().is_empty() {
                        li { "Henüz işlem yok." }
                    } else {
                        for entry in history().iter().take(8) {
                            li { "{entry}" }
                        }
                    }
                }
            }

            footer { class: "glass footer",
                div { class: "footer-col",
                    h4 { "Ağ Uyum" }
                    p { "Mainnet, Testnet, Devnet profilleri tek ekran yönetimi." }
                }
                div { class: "footer-col",
                    h4 { "Güvenlik" }
                    p { "İşlemler önce doğrulanır, sonra yayınlanır." }
                }
                div { class: "footer-col",
                    h4 { "Platform" }
                    p { "Responsive: desktop, tablet, mobil web." }
                }
            }
        }
    }
}

#[post("/api/run-operation")]
async fn run_operation_server(
    environment: String,
    action: String,
    note: String,
) -> Result<String, ServerFnError> {
    let env_valid = ENVIRONMENTS
        .iter()
        .any(|env| env.id == environment.as_str());
    let action_valid = ACTIONS.iter().any(|item| item.id == action.as_str());

    if !env_valid || !action_valid {
        return Ok(format!(
            "❌ doğrulama başarısız | env={environment} | action={action}"
        ));
    }

    let safe_note = if note.trim().is_empty() {
        "notesiz".to_string()
    } else {
        note.trim().to_string()
    };

    Ok(format!(
        "✅ işlendi | env={environment} | action={action} | note={safe_note}"
    ))
}
