use dioxus::prelude::*;

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

    let hardening = [
        ("Transport", "TLS pin set, certificate rollover monitored."),
        (
            "Secrets",
            "Operator key scope is redacted in desktop telemetry.",
        ),
        (
            "Automation",
            "Release gate requires build + integration + smoke pass.",
        ),
    ];

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
                    for (name, value) in hardening {
                        li {
                            div {
                                p { class: "activity-kind", "{name}" }
                                p { class: "activity-pair", "{value}" }
                            }
                            div {
                                p { class: "activity-amount", "Enforced" }
                                p { class: "activity-time", "Continuous" }
                            }
                        }
                    }
                }
            }
        }
    }
}
