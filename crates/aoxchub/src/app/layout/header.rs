use dioxus::prelude::*;

use crate::app::navigation::scroll_to_anchor;
use crate::app::router::Route;

#[derive(Clone)]
enum CommandAction {
    Route(Route),
    Anchor(&'static str),
}

#[derive(Clone)]
struct CommandItem {
    category: &'static str,
    label: &'static str,
    description: &'static str,
    action: CommandAction,
}

fn command_items() -> Vec<CommandItem> {
    vec![
        CommandItem {
            category: "Navigate",
            label: "Landing Overview",
            description: "Return to the primary AOXC Hub overview",
            action: CommandAction::Route(Route::Landing {}),
        },
        CommandItem {
            category: "Navigate",
            label: "Dashboard",
            description: "Open real-time metrics and production indicators",
            action: CommandAction::Route(Route::Dashboard {}),
        },
        CommandItem {
            category: "Navigate",
            label: "Wallet",
            description: "Access wallet onboarding and transaction tools",
            action: CommandAction::Route(Route::Wallet {}),
        },
        CommandItem {
            category: "Navigate",
            label: "Operations",
            description: "Run operational controls and service checks",
            action: CommandAction::Route(Route::Operations {}),
        },
        CommandItem {
            category: "Navigate",
            label: "Chain Overview",
            description: "Inspect registry, explorer, and domain data",
            action: CommandAction::Route(Route::Overview {}),
        },
        CommandItem {
            category: "Navigate",
            label: "Settings",
            description: "Configure endpoint, theme, and profile preferences",
            action: CommandAction::Route(Route::Settings {}),
        },
        CommandItem {
            category: "Anchors",
            label: "Integration Checklist",
            description: "Jump to release and readiness checkpoint panel",
            action: CommandAction::Anchor("#integration-checklist"),
        },
        CommandItem {
            category: "Anchors",
            label: "Validator Matrix",
            description: "Jump directly to validator posture section",
            action: CommandAction::Anchor("#validators"),
        },
        CommandItem {
            category: "Anchors",
            label: "Governance",
            description: "Jump to governance and policy alignment section",
            action: CommandAction::Anchor("#governance"),
        },
    ]
}

#[component]
pub fn HeaderBar() -> Element {
    let mut palette_open = use_signal(|| false);
    let mut query = use_signal(String::new);
    let navigator = use_navigator();

    let normalized_query = query().trim().to_lowercase();
    let commands = command_items();
    let filtered: Vec<CommandItem> = commands
        .into_iter()
        .filter(|item| {
            if normalized_query.is_empty() {
                return true;
            }

            item.label.to_lowercase().contains(&normalized_query)
                || item.description.to_lowercase().contains(&normalized_query)
                || item.category.to_lowercase().contains(&normalized_query)
        })
        .collect();

    rsx! {
        header {
            class: "header glass",

            div {
                class: "header-brand",

                Link {
                    class: "header-home-link",
                    to: Route::Landing {},
                    "AOXC Hub Control Center"
                }
            }

            div {
                class: "header-actions",
                button {
                    class: "header-action-btn",
                    r#type: "button",
                    onclick: move |_| {
                        query.set(String::new());
                        palette_open.set(true);
                    },
                    "Ultra Command Center"
                }
            }
        }

        if palette_open() {
            div {
                class: "command-palette-overlay",
                onclick: move |_| palette_open.set(false),

                section {
                    class: "command-palette",
                    onclick: move |event| event.stop_propagation(),

                    header {
                        class: "command-palette-header",
                        p { class: "command-palette-title", "AOXC Hub · Ultra Command Center" }
                        button {
                            class: "command-close-btn",
                            r#type: "button",
                            onclick: move |_| palette_open.set(false),
                            "Close"
                        }
                    }

                    input {
                        class: "command-search-input",
                        r#type: "text",
                        placeholder: "Search route, section, or operational action...",
                        value: "{query}",
                        oninput: move |event| query.set(event.value()),
                    }

                    div {
                        class: "command-list",
                        if filtered.is_empty() {
                            div {
                                class: "command-empty-state",
                                "No command found for the current query."
                            }
                        } else {
                            for item in filtered {
                                button {
                                    class: "command-item",
                                    r#type: "button",
                                    onclick: {
                                        let action = item.action.clone();
                                        move |_| {
                                            match action.clone() {
                                                CommandAction::Route(route) => {
                                                    let _ = navigator.push(route);
                                                }
                                                CommandAction::Anchor(anchor) => {
                                                    scroll_to_anchor(anchor);
                                                }
                                            }
                                            palette_open.set(false);
                                        }
                                    },
                                    p { class: "command-item-label", "{item.label}" }
                                    p { class: "command-item-description", "{item.description}" }
                                    span { class: "command-item-category", "{item.category}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
