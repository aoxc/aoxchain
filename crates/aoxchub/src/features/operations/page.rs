use dioxus::prelude::*;

#[derive(Clone, Copy)]
struct BinaryEntry {
    name: &'static str,
    location: &'static str,
    purpose: &'static str,
    profile: &'static str,
    health: &'static str,
}

#[derive(Clone, Copy)]
struct DataPathEntry {
    path: &'static str,
    role: &'static str,
    status: &'static str,
    criticality: &'static str,
}

#[derive(Clone, Copy)]
struct CommandEntry {
    command: &'static str,
    target: &'static str,
    outcome: &'static str,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CommandPanel {
    Make,
    Cli,
}

#[component]
pub fn OperationsSection() -> Element {
    let mut selected_profile = use_signal(|| "default");
    let mut panel = use_signal(|| CommandPanel::Make);
    let mut query = use_signal(String::new);

    let binaries = [
        BinaryEntry {
            name: "aoxc",
            location: "~/.AOXCData/bin/aoxc",
            purpose: "Core chain runtime, validator lifecycle, and diagnostics",
            profile: "real / local-dev",
            health: "Ready",
        },
        BinaryEntry {
            name: "aoxchub",
            location: "~/.AOXCData/bin/aoxchub",
            purpose: "Operator command center and workflow orchestration",
            profile: "default / real",
            health: "Ready",
        },
        BinaryEntry {
            name: "aoxckit",
            location: "~/.AOXCData/bin/aoxckit",
            purpose: "Identity, key lifecycle, and governance toolchain",
            profile: "default / real",
            health: "Ready",
        },
    ];

    let paths = [
        DataPathEntry {
            path: "~/.AOXCData/home/default/ledger/db/main.redb",
            role: "Ledger state authority",
            status: "Mounted and checked",
            criticality: "Critical",
        },
        DataPathEntry {
            path: "~/.AOXCData/home/default/runtime/db/index",
            role: "Runtime index and metadata journal",
            status: "Synchronized",
            criticality: "High",
        },
        DataPathEntry {
            path: "~/.AOXCData/home/default/runtime/db/ipfs",
            role: "Content-addressed payload cache",
            status: "Pinned and tracked",
            criticality: "High",
        },
        DataPathEntry {
            path: "~/.AOXCData/logs/real-chain",
            role: "Operational logs",
            status: "Streaming enabled",
            criticality: "Medium",
        },
        DataPathEntry {
            path: "~/.AOXCData/home/default/keys/operator_key.json",
            role: "Operator signing authority",
            status: "Policy-bound access",
            criticality: "Critical",
        },
    ];

    let make_commands = [
        CommandEntry {
            command: "make build",
            target: "Workspace binaries",
            outcome: "Builds release artifacts for aoxc/aoxchub/aoxckit",
        },
        CommandEntry {
            command: "make test",
            target: "Quality gate",
            outcome: "Runs deterministic unit and integration checks",
        },
        CommandEntry {
            command: "make run-hub",
            target: "Desktop control plane",
            outcome: "Launches AOXC Hub operator interface",
        },
        CommandEntry {
            command: "make release-check",
            target: "Readiness controls",
            outcome: "Validates manifests and release compatibility",
        },
    ];

    let cli_commands = [
        CommandEntry {
            command: "aoxc node status --home ~/.AOXCData/home/default",
            target: "Node health",
            outcome: "Returns sync head, peers, and liveness",
        },
        CommandEntry {
            command: "aoxc ledger verify --db ~/.AOXCData/home/default/ledger/db/main.redb",
            target: "Ledger verification",
            outcome: "Checks state integrity and canonical root",
        },
        CommandEntry {
            command: "aoxckit key info --file ~/.AOXCData/keys/operator_key.json",
            target: "Key audit",
            outcome: "Displays key metadata without private payload",
        },
        CommandEntry {
            command: "aoxchub --profile real",
            target: "Operations UI",
            outcome: "Starts production-oriented control surface",
        },
    ];

    let query_lc = query().to_lowercase();
    let active_commands = if panel() == CommandPanel::Make {
        &make_commands[..]
    } else {
        &cli_commands[..]
    };

    let mut filtered_commands: Vec<CommandEntry> = Vec::new();
    for entry in active_commands {
        if query_lc.is_empty()
            || entry.command.to_lowercase().contains(&query_lc)
            || entry.target.to_lowercase().contains(&query_lc)
            || entry.outcome.to_lowercase().contains(&query_lc)
        {
            filtered_commands.push(*entry);
        }
    }

    let total_assets = binaries.len() + paths.len();
    let critical_assets = paths
        .iter()
        .filter(|entry| entry.criticality == "Critical")
        .count();

    rsx! {
        section {
            class: "content-grid integration-grid",

            article {
                class: "panel glass integration-hero",
                h2 { "AOXC Hub • Ultra Integration Console" }
                p {
                    class: "integration-subtitle",
                    "A single operational surface for binaries, AOXCData directories, make targets, and CLI workflows."
                }
                div {
                    class: "integration-kpi-grid",
                    div { class: "integration-kpi", p { "Assets" } strong { "{total_assets}" } }
                    div { class: "integration-kpi", p { "Critical Paths" } strong { "{critical_assets}" } }
                    div { class: "integration-kpi", p { "Active Panel" } strong { if panel() == CommandPanel::Make { "Make" } else { "CLI" } } }
                    div { class: "integration-kpi", p { "Profile" } strong { "{selected_profile}" } }
                }
                div {
                    class: "integration-profile-switcher",
                    button {
                        class: if selected_profile() == "default" { "toggle-btn active" } else { "toggle-btn" },
                        onclick: move |_| selected_profile.set("default"),
                        "default"
                    }
                    button {
                        class: if selected_profile() == "local-dev" { "toggle-btn active" } else { "toggle-btn" },
                        onclick: move |_| selected_profile.set("local-dev"),
                        "local-dev"
                    }
                    button {
                        class: if selected_profile() == "real" { "toggle-btn active" } else { "toggle-btn" },
                        onclick: move |_| selected_profile.set("real"),
                        "real"
                    }
                }
            }

            article {
                class: "panel glass",
                h2 { "Binary Integration Matrix" }
                div {
                    class: "integration-cards",
                    for entry in binaries {
                        div {
                            class: "integration-card",
                            div {
                                class: "integration-card-head",
                                h3 { "{entry.name}" }
                                span { class: "integration-health", "{entry.health}" }
                            }
                            p { class: "mono-line", "{entry.location}" }
                            p { "{entry.purpose}" }
                            p { class: "integration-tag", "Profile Scope: {entry.profile}" }
                        }
                    }
                }
            }

            article {
                class: "panel glass",
                h2 { "Directory and Storage Coverage" }
                table {
                    class: "hub-table integration-table",
                    thead {
                        tr {
                            th { "Path" }
                            th { "Role" }
                            th { "Status" }
                            th { "Criticality" }
                        }
                    }
                    tbody {
                        for entry in paths {
                            tr {
                                td { class: "mono-line", "{entry.path}" }
                                td { "{entry.role}" }
                                td { "{entry.status}" }
                                td { "{entry.criticality}" }
                            }
                        }
                    }
                }
            }

            article {
                class: "panel glass",
                h2 { "Command Center" }
                div {
                    class: "command-toolbar",
                    div {
                        class: "command-tabs",
                        button {
                            class: if panel() == CommandPanel::Make { "toggle-btn active" } else { "toggle-btn" },
                            onclick: move |_| panel.set(CommandPanel::Make),
                            "Make"
                        }
                        button {
                            class: if panel() == CommandPanel::Cli { "toggle-btn active" } else { "toggle-btn" },
                            onclick: move |_| panel.set(CommandPanel::Cli),
                            "CLI"
                        }
                    }
                    input {
                        class: "command-filter",
                        r#type: "text",
                        placeholder: "Filter command, domain, or outcome...",
                        value: query,
                        oninput: move |evt| query.set(evt.value()),
                    }
                }

                table {
                    class: "hub-table integration-table",
                    thead {
                        tr {
                            th { "Command" }
                            th { "Domain" }
                            th { "Expected Output" }
                        }
                    }
                    tbody {
                        for entry in filtered_commands {
                            tr {
                                td { class: "mono-line", "{entry.command}" }
                                td { "{entry.target}" }
                                td { "{entry.outcome}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
