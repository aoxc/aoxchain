//! Receipt commitments for AOXCVM phase-1 verification.

use crate::receipts::outcome::{ExecutionReceipt, ReceiptStatus};
use sha2::{Digest, Sha256};

/// Canonical commitment computed from receipt fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReceiptCommitment {
    pub digest: [u8; 32],
}

impl ReceiptCommitment {
    /// Computes a deterministic digest for an execution receipt.
    pub fn from_receipt(receipt: &ExecutionReceipt) -> Self {
        let mut hasher = Sha256::new();
        hasher.update([status_code(receipt.status)]);
        hasher.update(receipt.gas_used.to_le_bytes());
        hasher.update(receipt.state_root);
        hasher.update((receipt.log.len() as u64).to_le_bytes());
        for line in &receipt.log {
            let bytes = line.as_bytes();
            hasher.update((bytes.len() as u64).to_le_bytes());
            hasher.update(bytes);
        }
        Self {
            digest: hasher.finalize().into(),
        }
    }
}

const fn status_code(status: ReceiptStatus) -> u8 {
    match status {
        ReceiptStatus::Success => 0,
        ReceiptStatus::Reverted => 1,
        ReceiptStatus::Failed => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::ReceiptCommitment;
    use crate::receipts::outcome::{ExecutionReceipt, ReceiptStatus};

    #[test]
    fn receipt_commitment_is_stable() {
        let receipt = ExecutionReceipt {
            status: ReceiptStatus::Success,
            gas_used: 42,
            log: vec!["x".into(), "y".into()],
            state_root: [7; 32],
        };
        let a = ReceiptCommitment::from_receipt(&receipt);
        let b = ReceiptCommitment::from_receipt(&receipt);
        assert_eq!(a, b);
    }
}
