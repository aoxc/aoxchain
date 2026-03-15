use thiserror::Error;

/// Network subsystem errors.
#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("peer disconnected")]
    PeerDisconnected,
}
