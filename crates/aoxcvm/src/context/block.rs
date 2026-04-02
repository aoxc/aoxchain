//! Block-scoped execution context.

/// Canonical block identity and time metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockContext {
    pub epoch: u64,
    pub height: u64,
    pub timestamp_ms: u64,
    pub block_hash: [u8; 32],
}

impl BlockContext {
    pub const fn new(epoch: u64, height: u64, timestamp_ms: u64, block_hash: [u8; 32]) -> Self {
        Self {
            epoch,
            height,
            timestamp_ms,
            block_hash,
        }
    }
}
