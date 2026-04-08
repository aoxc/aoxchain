// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

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
