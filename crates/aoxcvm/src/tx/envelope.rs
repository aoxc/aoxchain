//! Transaction envelope.

use serde::{Deserialize, Serialize};

use crate::tx::{fee::FeeBudget, kind::TxKind, payload::TxPayload};

/// Canonical transaction envelope used by hash/validation/admission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxEnvelope {
    pub chain_id: u64,
    pub nonce: u64,
    pub kind: TxKind,
    pub fee_budget: FeeBudget,
    pub payload: TxPayload,
}

impl TxEnvelope {
    pub const fn new(
        chain_id: u64,
        nonce: u64,
        kind: TxKind,
        fee_budget: FeeBudget,
        payload: TxPayload,
    ) -> Self {
        Self {
            chain_id,
            nonce,
            kind,
            fee_budget,
            payload,
        }
    }
}
