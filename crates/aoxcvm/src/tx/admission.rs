//! Admission policy entry-point for mempool ingestion.

use crate::tx::{envelope::TxEnvelope, validation, validation::ValidationPolicy};

pub fn admit(tx: &TxEnvelope, policy: ValidationPolicy) -> Result<(), validation::ValidationError> {
    validation::validate(tx, policy)
}
