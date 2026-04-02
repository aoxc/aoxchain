//! Transaction-scoped execution context.

/// Canonical transaction metadata for kernel execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TxContext {
    pub tx_hash: [u8; 32],
    pub tx_index: u32,
    pub gas_limit: u64,
    pub readonly: bool,
    pub spec_version: u32,
    pub feature_bitmap: u64,
}

impl TxContext {
    pub const fn new(
        tx_hash: [u8; 32],
        tx_index: u32,
        gas_limit: u64,
        readonly: bool,
        spec_version: u32,
        feature_bitmap: u64,
    ) -> Self {
        Self {
            tx_hash,
            tx_index,
            gas_limit,
            readonly,
            spec_version,
            feature_bitmap,
        }
    }
}
