//! Stateless transaction envelope validation.

use thiserror::Error;

use crate::tx::envelope::TxEnvelope;

#[derive(Debug, Clone, Copy)]
pub struct ValidationPolicy {
    pub expected_chain_id: u64,
    pub max_payload_bytes: usize,
}

impl ValidationPolicy {
    pub const fn standard(expected_chain_id: u64) -> Self {
        Self {
            expected_chain_id,
            max_payload_bytes: 64 * 1024,
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("chain id mismatch")]
    ChainIdMismatch,
    #[error("payload cannot be empty")]
    EmptyPayload,
    #[error("payload exceeds max bytes")]
    PayloadTooLarge,
    #[error("gas limit must be positive")]
    ZeroGasLimit,
    #[error("gas price must be positive")]
    ZeroGasPrice,
}

pub fn validate(tx: &TxEnvelope, policy: ValidationPolicy) -> Result<(), ValidationError> {
    if tx.chain_id != policy.expected_chain_id {
        return Err(ValidationError::ChainIdMismatch);
    }
    if tx.payload.is_empty() {
        return Err(ValidationError::EmptyPayload);
    }
    if tx.payload.len() > policy.max_payload_bytes {
        return Err(ValidationError::PayloadTooLarge);
    }
    if tx.fee_budget.gas_limit == 0 {
        return Err(ValidationError::ZeroGasLimit);
    }
    if tx.fee_budget.gas_price == 0 {
        return Err(ValidationError::ZeroGasPrice);
    }
    Ok(())
}
