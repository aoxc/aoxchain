// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::error::RpcError;
use crate::middleware::zkp_validator::ZkpValidator;
use crate::types::{TxSubmissionRequest, TxSubmissionResult};

#[derive(Debug, Clone, Default)]
pub struct TxSubmissionService {
    pub zkp_validator: ZkpValidator,
}

impl TxSubmissionService {
    pub fn submit(&self, request: TxSubmissionRequest) -> Result<TxSubmissionResult, RpcError> {
        if request.actor_id.trim().is_empty() || request.tx_payload.is_empty() {
            return Err(RpcError::InvalidRequest);
        }

        self.zkp_validator.validate(&request.zkp_proof)?;

        Ok(TxSubmissionResult {
            tx_id: format!("tx-{}", hex_fragment(&request.tx_payload)),
            accepted: true,
        })
    }
}

fn hex_fragment(payload: &[u8]) -> String {
    payload
        .iter()
        .take(6)
        .map(|byte| format!("{:02x}", byte))
        .collect()
}
