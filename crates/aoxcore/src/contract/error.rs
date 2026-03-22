use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ContractRegistryError {
    #[error("contract already registered: {0}")]
    AlreadyRegistered(String),
    #[error("contract not found: {0}")]
    NotFound(String),
    #[error("invalid state transition: {0}")]
    InvalidTransition(String),
}
