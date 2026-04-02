//! Ergonomic transaction envelope builder.

use thiserror::Error;

use crate::tx::{
    envelope::TxEnvelope, fee::FeeBudget, kind::TxKind, payload::TxPayload, validation,
    validation::ValidationPolicy,
};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BuildError {
    #[error("missing transaction kind")]
    MissingKind,
    #[error("missing payload")]
    MissingPayload,
    #[error(transparent)]
    Validation(#[from] validation::ValidationError),
}

#[derive(Debug, Clone)]
pub struct TxBuilder {
    chain_id: u64,
    nonce: u64,
    fee_budget: FeeBudget,
    kind: Option<TxKind>,
    payload: Option<TxPayload>,
}

impl TxBuilder {
    pub const fn new(chain_id: u64, nonce: u64, fee_budget: FeeBudget) -> Self {
        Self {
            chain_id,
            nonce,
            fee_budget,
            kind: None,
            payload: None,
        }
    }

    pub fn kind(mut self, kind: TxKind) -> Self {
        self.kind = Some(kind);
        self
    }

    pub fn payload(mut self, payload: TxPayload) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn build(self) -> Result<TxEnvelope, BuildError> {
        let kind = self.kind.ok_or(BuildError::MissingKind)?;
        let payload = self.payload.ok_or(BuildError::MissingPayload)?;
        let tx = TxEnvelope::new(self.chain_id, self.nonce, kind, self.fee_budget, payload);
        validation::validate(&tx, ValidationPolicy::standard(self.chain_id))?;
        Ok(tx)
    }
}
