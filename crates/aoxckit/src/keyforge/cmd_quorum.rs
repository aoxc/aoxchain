use crate::keyforge::cli::QuorumCommand;

pub fn handle(_command: QuorumCommand) -> Result<(), String> {
    Err("QUORUM_COMMAND_NOT_IMPLEMENTED".to_string())
}
