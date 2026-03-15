use crate::keyforge::cli::RegistryCommand;

pub fn handle(_command: RegistryCommand) -> Result<(), String> {
    Err("REGISTRY_COMMAND_NOT_IMPLEMENTED".to_string())
}
