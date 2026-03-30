use thiserror::Error;

#[derive(Debug, Error)]
pub enum HubError {
    #[error("I/O failure: {0}")]
    Io(#[from] std::io::Error),
    #[error("command not found: {0}")]
    UnknownCommand(String),
    #[error("security policy violation: {0}")]
    Security(String),
}
