//! Receipt and execution invariant checks for AOXCVM phase-1.

use crate::receipts::outcome::ReceiptStatus;
use crate::vm::machine::ExecutionResult;

/// Invariant verification errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvariantError {
    GasOverflow,
    NonSuccessReceipt,
}

/// Core invariants that must hold for accepted phase-1 execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvariantVerifier;

impl InvariantVerifier {
    /// Verifies post-conditions on successful deterministic execution.
    pub fn verify(result: &ExecutionResult, gas_limit: u64) -> Result<(), InvariantError> {
        if result.receipt.gas_used > gas_limit {
            return Err(InvariantError::GasOverflow);
        }
        if result.receipt.status != ReceiptStatus::Success {
            return Err(InvariantError::NonSuccessReceipt);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{InvariantError, InvariantVerifier};
    use crate::receipts::outcome::{ExecutionReceipt, ReceiptStatus};
    use crate::vm::machine::ExecutionResult;

    #[test]
    fn rejects_overflowed_gas() {
        let result = ExecutionResult {
            receipt: ExecutionReceipt {
                status: ReceiptStatus::Success,
                gas_used: 101,
                log: vec![],
                state_root: [0; 32],
            },
            stack: vec![],
        };
        assert_eq!(
            InvariantVerifier::verify(&result, 100),
            Err(InvariantError::GasOverflow)
        );
    }
}
