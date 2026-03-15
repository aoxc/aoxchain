use crate::context::TxContext;

/// Lightweight host transaction wrapper used by schedulers or mempools.
#[derive(Debug, Clone)]
pub struct HostedTransaction {
    pub tx: TxContext,
    pub received_at_unix: u64,
}

impl HostedTransaction {
    /// Creates a new hosted transaction wrapper.
    pub fn new(tx: TxContext, received_at_unix: u64) -> Self {
        Self { tx, received_at_unix }
    }
}
