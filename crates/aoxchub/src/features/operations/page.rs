use dioxus::prelude::*;

struct BinaryEntry {
    name: &'static str,
    location: &'static str,
    purpose: &'static str,
    profile: &'static str,
}

struct DataPathEntry {
    path: &'static str,
    role: &'static str,
    status: &'static str,
}

struct CommandEntry {
    command: &'static str,
    target: &'static str,
    outcome: &'static str,
}

#[component]
pub fn OperationsSection() -> Element {
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
        },
        DataPathEntry {
            path: "~/.AOXCData/home/default/runtime/db/index",
            role: "Runtime index and metadata journal",
            status: "Continuously synchronized",
        },
        DataPathEntry {
            path: "~/.AOXCData/home/default/runtime/db/ipfs",
            role: "Content-addressed payload cache",
            status: "Pinned and integrity-tracked",
        },
        DataPathEntry {
            path: "~/.AOXCData/logs/real-chain",
            role: "Chain and service operational logs",
            status: "Streaming to observability panel",
        },
        DataPathEntry {
            path: "~/.AOXCData/home/default/keys/operator_key.json",
            role: "Operator signing authority",
            status: "Policy-bound secure access",
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
            outcome: "Returns sync head, peer set, and liveness",
        },
        CommandEntry {
            command: "aoxc ledger verify --db ~/.AOXCData/home/default/ledger/db/main.redb",
            target: "Ledger verification",
            outcome: "Checks state integrity and canonical root",
        },
        CommandEntry {
            command: "aoxckit key info --file ~/.AOXCData/keys/operator_key.json",
            target: "Operator key audit",
            outcome: "Prints key metadata without exposing private material",
        },
        CommandEntry {
            command: "aoxchub --profile real",
            target: "Operational UI",
            outcome: "Starts real-network control surface with dashboard panels",
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
                class: "panel glass",
                h2 { "AOXCData Binary Integration Matrix" }
                p {
                    class: "integration-subtitle",
                    "All critical executables are mapped to one operational surface with deterministic profile boundaries."
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
                        }
                    }
                    tbody {
                        for entry in paths {
                            tr {
                                td { class: "mono-line", "{entry.path}" }
                                td { "{entry.role}" }
                                td { "{entry.status}" }
                            }
                        }
                    }
                }
            }

            article {
                class: "panel glass",
                h2 { "Make Command Control Plane" }
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
                        for entry in make_commands {
                            tr {
                                td { class: "mono-line", "{entry.command}" }
                                td { "{entry.target}" }
                                td { "{entry.outcome}" }
                            }
                        }
                    }
                }
            }

            article {
                class: "panel glass",
                h2 { "CLI Command Surface" }
                table {
                    class: "hub-table integration-table",
                    thead {
                        tr {
                            th { "Command" }
                            th { "Operational Domain" }
                            th { "Expected Output" }
                        }
                    }
                    tbody {
                        for entry in cli_commands {
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
