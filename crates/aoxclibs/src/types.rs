use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum LibError {
    #[error("encoding/decoding operation failed: {0}")]
    EncodingError(String),

    #[error("input validation failed: {0}")]
    ValidationError(String),

    #[error("time computation error: {0}")]
    TimeError(String),
}
