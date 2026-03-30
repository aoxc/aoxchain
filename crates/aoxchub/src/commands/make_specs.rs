use crate::commands::catalog::CommandSpec;
use crate::domain::{command::CommandId, command_group::CommandGroup, command_risk::CommandRisk};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec {
            id: CommandId("workspace_build"),
            label: "Build Workspace",
            group: CommandGroup::WorkspaceOperations,
            program: "make",
            args: &["build"],
            description: "Builds the AOXC workspace using the canonical Make target.",
            risk: CommandRisk::Stateful,
            requires_confirmation: true,
        },
    ]
}
