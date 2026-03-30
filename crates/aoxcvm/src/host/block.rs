// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::context::BlockContext;
use crate::host::receipt::ExecutionReceipt;

/// Finalized block output assembled by the host.
#[derive(Debug, Clone)]
pub struct FinalizedBlock {
    pub context: BlockContext,
    pub receipts: Vec<ExecutionReceipt>,
}

impl FinalizedBlock {
    /// Creates an empty finalized block container.
    pub fn new(context: BlockContext) -> Self {
        Self {
            context,
            receipts: Vec::new(),
        }
    }

    /// Appends a receipt into the finalized block output.
    pub fn push_receipt(&mut self, receipt: ExecutionReceipt) {
        self.receipts.push(receipt);
    }
}
