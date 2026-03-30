use crate::domain::{command::CommandId, command_group::CommandGroup, command_risk::CommandRisk};

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub id: CommandId,
    pub label: &'static str,
    pub group: CommandGroup,
    pub program: &'static str,
    pub args: &'static [&'static str],
    pub description: &'static str,
    pub risk: CommandRisk,
    pub requires_confirmation: bool,
}

pub fn catalog() -> Vec<CommandSpec> {
    Vec::new()
}
