use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiBuilderError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("dataset error: {0}")]
    Dataset(String),
    #[error("training error: {0}")]
    Training(String),
    #[error("registry error: {0}")]
    Registry(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}
