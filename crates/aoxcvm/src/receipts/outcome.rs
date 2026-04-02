//! Execution outcome and receipt types for AOXCVM phase-1.

use crate::state::JournaledState;
use sha2::{Digest, Sha256};

/// Transaction execution status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiptStatus {
    Success,
    Reverted,
    Failed,
}

/// Deterministic receipt emitted by phase-1 executor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionReceipt {
    pub status: ReceiptStatus,
    pub gas_used: u64,
    pub log: Vec<String>,
    pub state_root: [u8; 32],
}

impl ExecutionReceipt {
    /// Constructs a receipt using the deterministic state root hash.
    pub fn from_state(
        status: ReceiptStatus,
        gas_used: u64,
        log: Vec<String>,
        state: &JournaledState,
    ) -> Self {
        let state_root = hash_state_root(state);
        Self {
            status,
            gas_used,
            log,
            state_root,
        }
    }
}

/// Hashes canonical state bytes using SHA-256 for verifier comparison.
pub fn hash_state_root(state: &JournaledState) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(state.canonical_bytes());
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::{ExecutionReceipt, ReceiptStatus};
    use crate::state::JournaledState;

    #[test]
    fn receipt_state_root_is_stable() {
        let mut state = JournaledState::default();
        state.put(b"k".to_vec(), b"v".to_vec());
        let a = ExecutionReceipt::from_state(ReceiptStatus::Success, 10, vec![], &state);
        let b = ExecutionReceipt::from_state(ReceiptStatus::Success, 10, vec![], &state);
        assert_eq!(a.state_root, b.state_root);
    }
}
