use crate::commands::catalog::CommandSpec;
use crate::domain::{command::CommandId, command_group::CommandGroup, command_risk::CommandRisk};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec {
            id: CommandId("aoxc_doctor"),
            label: "Run AOXC Doctor",
            group: CommandGroup::Diagnostics,
            program: "aoxc",
            args: &["doctor"],
            description: "Runs the canonical AOXC diagnostic workflow.",
            risk: CommandRisk::ReadOnly,
            requires_confirmation: true,
        },
    ]
}
