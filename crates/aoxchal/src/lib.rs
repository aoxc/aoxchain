pub mod cpu_opt;
pub mod mem_manager;

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

/// HAL (Hardware Abstraction Layer) işlemlerinde oluşabilecek hatalar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HalError {
    MemoryAllocationFailed(String),
    UnsupportedInstructionSet,
    SecureWipeFailed,
}

impl fmt::Display for HalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MemoryAllocationFailed(msg) => write!(f, "memory allocation failed: {msg}"),
            Self::UnsupportedInstructionSet => write!(f, "required CPU instruction set is not supported"),
            Self::SecureWipeFailed => write!(f, "failed to securely wipe memory"),
        }
    }
}

impl Error for HalError {}
