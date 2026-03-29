use dioxus::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
struct SecurityDrill {
    status: String,
    scenarios: Vec<String>,
    requirements: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct TelemetrySnapshot {
    status: String,
    alerts_required: Vec<String>,
}

fn load_security_drill() -> SecurityDrill {
    serde_json::from_str(include_str!(
        "../../../../../artifacts/network-production-closure/security-drill.json"
    ))
    .unwrap_or_else(|_| SecurityDrill {
        status: String::from("unknown"),
        scenarios: Vec::new(),
        requirements: Vec::new(),
    })
}

fn load_telemetry_snapshot() -> TelemetrySnapshot {
    serde_json::from_str(include_str!(
        "../../../../../artifacts/network-production-closure/telemetry-snapshot.json"
    ))
    .unwrap_or_else(|_| TelemetrySnapshot {
        status: String::from("unknown"),
        alerts_required: Vec::new(),
    })
}

#[component]
pub fn SettingsSection() -> Element {
    let endpoints = [
        ("Primary RPC", "https://rpc.mainnet.aoxc.example"),
        ("Fallback RPC", "https://rpc.backup.aoxc.example"),
        ("Indexer", "https://indexer.aoxc.example"),
    ];

    let preferences = [
        ("Theme", "Retro Dark"),
        ("Language", "Turkish + English"),
        ("Auto Refresh", "Every 5 seconds"),
        ("Notification Mode", "Desktop + In-app"),
    ];

    let security_drill = load_security_drill();
    let telemetry = load_telemetry_snapshot();
    let drill_status = security_drill.status.clone();
    let drill_scenarios = security_drill.scenarios.clone();
    let drill_requirements = security_drill.requirements.clone();
    let telemetry_status = telemetry.status.clone();
    let telemetry_alerts = telemetry.alerts_required.clone();

    rsx! {
        section {
            class: "content-grid",
            article {
                class: "panel glass",
                h2 { "Endpoint Configuration" }
                ul {
                    class: "activity-list",
                    for (name, value) in endpoints {
                        li {
                            div {
                                p { class: "activity-kind", "{name}" }
                                p { class: "activity-pair", "{value}" }
                            }
                            div {
                                p { class: "activity-amount", "Connected" }
                                p { class: "activity-time", "Validated" }
                            }
                        }
                    }
                }
            }

            article {
                class: "panel glass",
                h2 { "Operator Preferences" }
                ul {
                    class: "activity-list",
                    for (name, value) in preferences {
                        li {
                            div {
                                p { class: "activity-kind", "{name}" }
                                p { class: "activity-pair", "{value}" }
                            }
                            div {
                                p { class: "activity-amount", "Applied" }
                                p { class: "activity-time", "Live" }
                            }
                        }
                    }
                }
            }

            article {
                class: "panel glass",
                h2 { "Security Hardening" }
                ul {
                    class: "activity-list",
                    for scenario in drill_scenarios {
                        li {
                            div {
                                p { class: "activity-kind", "{scenario}" }
                                p { class: "activity-pair", "Requirement baseline captured in production-closure drill." }
                            }
                            div {
                                p { class: "activity-amount", "{drill_status}" }
                                p { class: "activity-time", "Scenario" }
                            }
                        }
                    }
                }
            }

            article {
                class: "panel glass",
                h2 { "Telemetry Alert Requirements" }
                p { class: "hero-sub", "Snapshot status: {telemetry_status}" }
                ul {
                    class: "hero-panel-list",
                    for alert in telemetry_alerts {
                        li { class: "hero-sub", "{alert}" }
                    }
                }
                p { class: "hero-sub", "Security drill requirements" }
                ul {
                    class: "hero-panel-list",
                    for item in drill_requirements {
                        li { class: "hero-sub", "{item}" }
                    }
                }
            }
        }
    }
}
