use dioxus::prelude::*;

#[derive(Clone, Copy)]
struct BinaryEntry {
    name: &'static str,
    location: &'static str,
    purpose: &'static str,
    profile: &'static str,
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
    let mut panel = use_signal(|| CommandPanel::Make);
    let mut query = use_signal(String::new);

    let binaries = [
        BinaryEntry {
            name: "aoxc",
            location: "~/.AOXCData/bin/aoxc",
            purpose: "Core chain runtime, validator lifecycle, and diagnostics",
            profile: "real / local-dev",
        },
        BinaryEntry {
            name: "aoxchub",
            location: "~/.AOXCData/bin/aoxchub",
            purpose: "Operator control-surface and desktop orchestration",
            profile: "default",
        },
        BinaryEntry {
            name: "aoxckit",
            location: "~/.AOXCData/bin/aoxckit",
            purpose: "Identity, key material, and governance utility tooling",
            profile: "default / real",
        },
    ];

    let paths = [
        DataPathEntry {
            path: "~/.AOXCData/home/default/ledger/db/main.redb",
            role: "Ledger state authority",
            status: "Mounted and checked at startup",
            criticality: "Critical",
        },
        DataPathEntry {
            path: "~/.AOXCData/home/default/runtime/db/index",
            role: "Runtime index and metadata journal",
            status: "Continuously synchronized",
            criticality: "High",
        },
        DataPathEntry {
            path: "~/.AOXCData/home/default/runtime/db/ipfs",
            role: "Content-addressed payload cache",
            status: "Pinned and integrity-tracked",
            criticality: "High",
        },
        DataPathEntry {
            path: "~/.AOXCData/logs/real-chain",
            role: "Chain and service operational logs",
            status: "Streaming to observability panel",
            criticality: "Medium",
        },
        DataPathEntry {
            path: "~/.AOXCData/home/default/keys/operator_key.json",
            role: "Operator signing authority",
            status: "Policy-bound secure access",
            criticality: "Critical",
        },
    ];

    let make_commands = [
        CommandEntry {
            command: "make build",
            target: "Workspace binaries",
            outcome: "Builds aoxc + aoxchub + aoxckit release artifacts",
        },
        CommandEntry {
            command: "make test",
            target: "Unit and integration coverage",
            outcome: "Runs deterministic checks before deployment promotion",
        },
        CommandEntry {
            command: "make run-hub",
            target: "AOXC Hub desktop surface",
            outcome: "Launches full operations interface",
        },
        CommandEntry {
            command: "make release-check",
            target: "Readiness gates",
            outcome: "Validates manifests, compatibility, and smoke flows",
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

    let active_commands = if panel() == CommandPanel::Make {
        &make_commands[..]
    } else {
        &cli_commands[..]
    };

    let query_lc = query().to_lowercase();
    let filtered_commands: Vec<&CommandEntry> = active_commands
        .iter()
        .filter(|entry| {
            query_lc.is_empty()
                || entry.command.to_lowercase().contains(&query_lc)
                || entry.target.to_lowercase().contains(&query_lc)
                || entry.outcome.to_lowercase().contains(&query_lc)
        })
        .collect();

    let total_assets = binaries.len() + paths.len();
    let critical_assets = paths
        .iter()
        .filter(|entry| entry.criticality == "Critical")
        .count();

    rsx! {
        section {
            id: "validators",
            class: "content-grid integration-grid",

            article {
                id: "integration-checklist",
                class: "panel glass",
                h2 { "AOXCData Binary Integration Matrix" }
                p {
                    class: "integration-subtitle",
                    "All critical executables are mapped to one operational surface with deterministic profile boundaries."
                }
                p {
                    class: "integration-tag",
                    "Assets: {total_assets} | Critical: {critical_assets}"
                }
                div {
                    class: "integration-cards",
                    for entry in binaries {
                        div {
                            class: "integration-card",
                            h3 { "{entry.name}" }
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
                            th { "Integration State" }
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
                div { class: "hero-actions",
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| panel.set(CommandPanel::Make),
                        "Make Commands"
                    }
                    button {
                        class: "btn btn-ghost",
                        onclick: move |_| panel.set(CommandPanel::Cli),
                        "CLI Commands"
                    }
                    input {
                        class: "wallet-input",
                        r#type: "search",
                        value: query,
                        placeholder: "Filter command / target / output",
                        oninput: move |evt| query.set(evt.value()),
                    }
                }

                table {
                    class: "hub-table integration-table",
                    thead {
                        tr {
                            th { "Command" }
                            th { "Target" }
                            th { "Result" }
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
