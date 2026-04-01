//! Dry-run request/response types for execution simulation.

use serde::{Deserialize, Serialize};

use crate::tx::{envelope::TxEnvelope, hash::TxDigest};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DryRunRequest {
    pub tx: TxEnvelope,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DryRunResponse {
    pub accepted: bool,
    pub estimated_gas_used: u64,
}

impl DryRunRequest {
    pub fn tx_id(&self) -> TxDigest {
        TxDigest::from_envelope(&self.tx)
    }
}
