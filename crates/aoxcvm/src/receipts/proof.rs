//! Simple verifier proof object for phase-1 execution receipts.

use crate::receipts::commitment::ReceiptCommitment;
use crate::receipts::outcome::ExecutionReceipt;

/// Proof bundle produced by a deterministic verifier run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiptProof {
    pub commitment: ReceiptCommitment,
    pub replay_count: u8,
}

impl ReceiptProof {
    /// Builds a proof from a receipt and verifier replay count.
    pub fn new(receipt: &ExecutionReceipt, replay_count: u8) -> Self {
        Self {
            commitment: ReceiptCommitment::from_receipt(receipt),
            replay_count,
        }
    }

    /// Checks whether another receipt matches this proof commitment.
    pub fn verify_receipt(&self, receipt: &ExecutionReceipt) -> bool {
        ReceiptCommitment::from_receipt(receipt) == self.commitment
    }
}

#[cfg(test)]
mod tests {
    use super::ReceiptProof;
    use crate::receipts::outcome::{ExecutionReceipt, ReceiptStatus};

    #[test]
    fn proof_verifies_same_receipt() {
        let receipt = ExecutionReceipt {
            status: ReceiptStatus::Success,
            gas_used: 10,
            log: vec!["ok".into()],
            state_root: [3; 32],
        };
        let proof = ReceiptProof::new(&receipt, 2);
        assert!(proof.verify_receipt(&receipt));
    }
}
