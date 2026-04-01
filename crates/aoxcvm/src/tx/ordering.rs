//! Ordering helpers for mempool selection.

use crate::tx::envelope::TxEnvelope;

/// Higher score should be prioritized first.
pub fn priority_score(tx: &TxEnvelope) -> u128 {
    let payload = tx.payload.len().max(1) as u128;
    (tx.fee_budget.gas_price as u128) * 1_000_000 / payload
}
