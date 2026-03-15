use crate::keys::loader::KeyLoaderError;
use crate::node::state::NodeInitError;

use std::fmt;

#[derive(Debug)]
pub enum AppError {
    InvalidArgument(String),
    KeyBootstrapFailed(KeyLoaderError),
    NodeBootstrapFailed(NodeInitError),
    Io(String),
    Runtime(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument(msg) => write!(f, "invalid argument: {msg}"),
            Self::KeyBootstrapFailed(error) => write!(f, "key bootstrap failed: {error}"),
            Self::NodeBootstrapFailed(error) => write!(f, "node bootstrap failed: {error}"),
            Self::Io(msg) => write!(f, "io failure: {msg}"),
            Self::Runtime(msg) => write!(f, "runtime failure: {msg}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<KeyLoaderError> for AppError {
    fn from(value: KeyLoaderError) -> Self {
        Self::KeyBootstrapFailed(value)
    }
}

impl From<NodeInitError> for AppError {
    fn from(value: NodeInitError) -> Self {
        Self::NodeBootstrapFailed(value)
    }
}
